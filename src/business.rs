use std::{ops::Deref, path::PathBuf, sync::Arc};

use axum::http::StatusCode;
use doc_search::{
    Document, EmptyWordFilter, Index, MemoryStorage, OptionType, QueryOption, SimpleTokenizer,
};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Row};
use tokio::sync::RwLock;
use tracing::{debug, error};

use crate::{Category, Collection, CustError, FileStorage, ID, Item, Name, Price, Result};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbCollection {
    pub id: Option<ID>,
    pub name: Name,
}

impl From<DbCollection> for Collection {
    fn from(db: DbCategory) -> Self {
        Self {
            id: db.id,
            name: db.name,
            thumbnail: None,
        }
    }
}

impl From<Collection> for DbCollection {
    fn from(db: Collection) -> Self {
        Self {
            id: db.id,
            name: db.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbCategory {
    pub id: Option<ID>,
    pub name: Name,
    pub parent_category: Option<ID>,
}

impl From<DbCategory> for Category {
    fn from(db: DbCategory) -> Self {
        Self {
            id: db.id,
            name: db.name,
            parent_category: db.parent_category,
            thumbnail: None,
        }
    }
}

impl From<Category> for DbCategory {
    fn from(db: Category) -> Self {
        Self {
            id: db.id,
            name: db.name,
            parent_category: db.parent_category,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbItem {
    pub id: Option<ID>,
    pub name: Name,
    pub description: Option<String>,
    pub category_id: Option<ID>,
    pub price: Option<Price>,
}

impl From<DbItem> for Item {
    fn from(db: DbItem) -> Self {
        Self {
            id: db.id,
            name: db.name,
            description: db.description,
            category_id: db.category_id,
            price: db.price,
            thumbnail: None,
            fullsize: None,
        }
    }
}

impl From<Item> for DbItem {
    fn from(db: Item) -> Self {
        Self {
            id: db.id,
            name: db.name,
            description: db.description,
            category_id: db.category_id,
            price: db.price,
        }
    }
}

pub struct BusinessRules {
    conn: sqlx::SqlitePool,
    category_files: FileStorage<Category>,
    item_files: FileStorage<Item>,
    collection_files: FileStorage<Collection>,
    index: RwLock<Index<i64, MemoryStorage<i64>, PathBuf>>,
    tokenizer: SimpleTokenizer,
    filter: EmptyWordFilter,
}

impl BusinessRules {
    pub async fn new(
        index: Index<i64, MemoryStorage<i64>, PathBuf>,
        tokenizer: SimpleTokenizer,
        filter: EmptyWordFilter,
    ) -> Self {
        let index = RwLock::new(index);
        let conn = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite:db.sqlite")
            .await
            .unwrap();

        Self {
            conn,
            category_files: FileStorage::new(PathBuf::from("./categories")),
            item_files: FileStorage::new(PathBuf::from("./items")),
            collection_files: FileStorage::new(PathBuf::from("./collections")),
            index,
            tokenizer,
            filter,
        }
    }

    pub async fn init(&self) {
        // NOTE: with the new storage engine, the loading on startup is not needed, since the index
        // is kept in a different storage
    }

    pub async fn init_db(&self) {
        let db = &self.conn;
        db.execute(
            r#"
        CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            category_id INTEGER,
            price REAL,
            FOREIGN KEY (category_id) REFERENCES categories(id)
        );
        "#,
        )
            .await
            .unwrap();

        db.execute(
            r#"
        CREATE TABLE IF NOT EXISTS categories (
           id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            parent_category INTEGER,
            FOREIGN KEY (parent_category) REFERENCES categories(id)
        );
        "#,
        )
            .await
            .unwrap();

        db.execute(
            r#"
        CREATE UNIQUE INDEX IF NOT EXISTS category_name ON categories(name);
        "#,
        )
            .await
            .unwrap();

        db.execute(
            r#"
        CREATE TABLE IF NOT EXISTS collections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        );
        "#,
        )
            .await
            .unwrap();

        db.execute(
            r#"
        CREATE UNIQUE INDEX IF NOT EXISTS collections_name ON collections(name);
        "#,
        )
            .await
            .unwrap();

        db.execute(
            r#"
        CREATE TABLE IF NOT EXISTS collection_items (
            collection_id INTEGER,
            item_id INTEGER,
            PRIMARY KEY (collection_id, item_id),
            FOREIGN KEY (collection_id) REFERENCES collections(id),
            FOREIGN KEY (item_id) REFERENCES items(id)
        );
        "#,
        )
            .await
            .unwrap();
    }

    pub async fn add_item(&self, mut item: Item) -> Result<Item> {
        debug!("Adding item: {:?}", item);
        let mut tx = self.conn.begin().await?;

        sqlx::query("INSERT INTO items (name, description, category_id, price, image) VALUES (?, ?, ?, ?, ?)")
            .bind(item.name.clone())
            .bind(item.description.clone())
            .bind(item.category_id)
            .bind(item.price)
            .execute(&mut *tx)
            .await?;

        let last_inserted = sqlx::query("SELECT last_insert_rowid() as id")
            .fetch_one(&mut *tx)
            .await?;

        let id: ID = last_inserted.get("id");
        item.id = Some(id);

        self.item_files.store(&item).await?;

        tx.commit().await?;

        let data = match &item.description {
            Some(desc) => format!("{} {}", item.name, desc),
            None => format!("{}", item.name),
        };

        let document = Document::new(id as i64, data, &self.filter, &self.tokenizer);

        let mut index = self.index.write().await;
        index.insert_document(document).await?;

        Ok(item)
    }

    pub async fn get_item(&self, id: ID) -> Result<Item> {
        let mut item: Item = sqlx::query_as::<_, DbItem>("SELECT * FROM items WHERE id = ?")
            .bind(id)
            .fetch_one(&self.conn)
            .await?
            .into();

        let result = self.item_files.read(&mut item).await;
        if result.is_err() {
            error!("{}", result.err().unwrap());
        }

        Ok(item)
    }

    fn find_score_for_item(&self, id: ID, query_res: &Vec<(f64, &Document<i64>)>) -> Option<f64> {
        query_res.iter().find_map(|(x, v)| {
            if *v.get_id() as i32 == id {
                Some(x.clone())
            } else {
                None
            }
        })
    }

    pub async fn find_items(&self, name: Name) -> Result<Vec<Item>> {
        debug!("Searching for: {:?}", name);
        let index = self.index.read().await;
        let mut result = index
            .query(
                name.as_str(),
                &self.tokenizer,
                &self.filter,
                Some(QueryOption::new().add(OptionType::TfIdf).build()),
            )
            .await?.collect();

        if result.is_empty() {
            return Err(CustError::new(
                "no items for search query".to_string(),
                StatusCode::NOT_FOUND,
            ));
        }
        debug!("Search result: {:?}", result);

        result.sort_by(|(x, _), (y, _)| x.total_cmp(y));

        let ids: Vec<Arc<i64>> = result.iter().map(|(_x, v)| v.get_id()).collect();
        let params = format!("?{}", ", ?".repeat(ids.len() - 1));
        let query_str = format!("SELECT * FROM items WHERE id IN ({})", params);

        let query = sqlx::query_as::<_, DbItem>(&query_str);
        let query = ids
            .into_iter()
            .fold(query, |query, id| query.bind(id.deref().clone()));

        let items = query.fetch_all(&self.conn).await?;
        let items: Vec<(Option<f64>, Item)> = items
            .into_iter()
            .map(|item| {
                (
                    self.find_score_for_item(item.id.unwrap(), &result),
                    item.into(),
                )
            })
            .collect();

        let mut items: Vec<_> = items.into_iter().filter(|x| x.0.is_some()).collect();
        for (_, item) in &mut items {
            let result = self.item_files.read(item).await;
            if result.is_err() {
                error!("{}", result.err().unwrap());
            }
        }

        items.sort_by(|x, y| x.0.unwrap().total_cmp(&y.0.unwrap()));

        Ok(items.into_iter().map(|x| x.1).rev().collect())
    }

    pub async fn get_all_items(&self) -> Result<Vec<Item>> {
        let mut items: Vec<Item> = sqlx::query_as::<_, DbItem>("SELECT * FROM items")
            .fetch_all(&self.conn)
            .await?
            .into_iter()
            .map(Into::into)
            .collect();

        for item in &mut items {
            let result = self.item_files.read(item).await;
            if result.is_err() {
                error!("{}", result.err().unwrap());
            }
        }

        Ok(items)
    }

    pub async fn delete_item(&self, id: ID) -> Result<Item> {
        let mut tx = self.conn.begin().await?;

        let item: Item = sqlx::query_as::<_, DbItem>("SELECT * from items WHERE id = ?")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?
            .into();

        sqlx::query("DELETE FROM items WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // NOTE: This is to release the future faster
        {
            let mut index = self.index.write().await;
            let _ = index.remove_document(Arc::new(id as i64)).await?;
        }

        tx.commit().await?;

        // TODO: delete all connection before
        Ok(item)
    }

    pub async fn new_category(&self, mut category: Category) -> Result<Category> {
        debug!("adding new category: {:?}", category);
        category.id = None;
        let mut tx = self.conn.begin().await?;

        let tmp_cat: Option<Category> =
            sqlx::query_as::<_, DbCategory>("SELECT * FROM categories WHERE name = ?")
                .bind(category.name.clone())
                .bind(category.parent_category)
                .fetch_optional(&mut *tx)
                .await?
                .map(Into::into);
        debug!("tmp_cat: {:?}", tmp_cat);

        if tmp_cat.is_some() {
            return Err(CustError::new(
                "category already exists".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }

        sqlx::query("INSERT INTO categories (name, parent_category) VALUES (?, ?)")
            .bind(category.name.clone())
            .bind(category.parent_category)
            .execute(&mut *tx)
            .await?;

        let last_inserted = sqlx::query("SELECT last_insert_rowid() as id")
            .fetch_one(&mut *tx)
            .await?;

        let id: ID = last_inserted.get("id");
        category.id = Some(id);
        self.category_files.store(&category).await?;

        tx.commit().await?;

        debug!("added new category: {:?}", category);
        Ok(category)
    }

    pub async fn get_all_categories(&self) -> Result<Vec<Category>> {
        let mut categories: Vec<Category> =
            sqlx::query_as::<_, DbCategory>("SELECT * FROM categories")
                .fetch_all(&self.conn)
                .await?
                .into_iter()
                .map(|c| c.into())
                .collect();

        for category in &mut categories {
            let result = self.category_files.read(category).await;
            if result.is_err() {
                error!("{}", result.err().unwrap());
            }
        }

        Ok(categories)
    }

    pub async fn new_collection(&self, coll: Collection) -> Result<Collection> {
        let mut tx = self.conn.begin().await?;

        sqlx::query("INSERT INTO COLLECTIONS (name) VALUES (?)")
            .bind(coll.name.clone())
            .execute(&mut *tx)
            .await?;

        let last_inserted = sqlx::query("SELECT last_insert_rowid() as id")
            .fetch_one(&mut *tx)
            .await?;

        let id: ID = last_inserted.get("id");

        let mut collection = coll;
        collection.id = Some(id);

        self.collection_files.store(&collection).await?;

        tx.commit().await?;

        Ok(collection)
    }

    pub async fn get_all_collections(&self) -> Result<Vec<Collection>> {
        let mut list = sqlx::query_as::<_, Collection>("SELECT * from collections")
            .fetch_all(&self.conn)
            .await?;

        for c in &mut list {
            let result = self.collection_files.read(c).await;
            if result.is_err() {
                error!("{}", result.err().unwrap());
            }
        }

        Ok(list)
    }

    pub async fn get_collection(&self, id: ID) -> Result<Collection> {
        let mut collection = sqlx::query_as::<_, Collection>("SELECT * FROM collections WHERE id = ?")
            .bind(id)
            .fetch_one(&self.conn)
            .await?;

        let result = self.collection_files.read(&mut collection).await;
        if result.is_err() {
            error!("{}", result.err().unwrap());
        }

        Ok(collection)
    }

    pub async fn add_item_to_collection(&self, item_id: ID, collection_id: ID) -> Result<()> {
        let mut tx = self.conn.begin().await?;

        let _item = self.get_item(item_id).await?;
        let _colletion = self.get_collection(collection_id).await?;

        sqlx::query("INSERT INTO collection_items VALUES (?, ?)")
            .bind(item_id)
            .bind(collection_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn remove_item_from_collection(&self, item_id: ID, collection_id: ID) -> Result<()> {
        let mut tx = self.conn.begin().await?;

        // TODO: find a way to use tx here
        let _item = self.get_item(item_id).await?;
        let _collection = self.get_collection(collection_id).await?;

        sqlx::query("DELETE FROM collection_items WHERE item_id = ? AND collection_id = ?")
            .bind(item_id)
            .bind(collection_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}
