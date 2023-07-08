use axum::http::StatusCode;
use probly_search::{score::{bm25, zero_to_one}, Index, QueryResult};
use sqlx::{Executor, Row};
use tokio::sync::RwLock;
use tracing::{debug, field::debug};

use crate::{
    description_extract, title_extract, tokenizer, Category, CustError, Item, ItemSearch, Name,
    Result, ID,
};
#[derive(Debug)]
pub struct BusinessRules {
    conn: sqlx::SqlitePool,
    index: RwLock<Index<ID>>,
}

impl BusinessRules {
    pub async fn new() -> Self {
        let index = RwLock::new(Index::new(2));
        let conn = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite:db.sqlite")
            .await
            .unwrap();

        Self { conn, index }
    }

    pub async fn init(&self) {
        let mut index = self.index.write().await;
        sqlx::query_as::<_, ItemSearch>("SELECT id, name, description FROM items")
            .fetch_all(&self.conn)
            .await
            .unwrap()
            .into_iter()
            .for_each(|item| {
                index.add_document(
                    &[title_extract, description_extract],
                    tokenizer,
                    item.id,
                    &item,
                )
            });
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

        tx.commit().await?;

        item.id = Some(id);

        let item_search = ItemSearch {
            id,
            name: item.name.clone(),
            description: item.description.clone(),
        };

        let mut index = self.index.write().await;
        index.add_document(
            &[title_extract, description_extract],
            tokenizer,
            id,
            &item_search,
        );
        debug!("Item added: {:?}", item);
        Ok(item)
    }

    pub async fn get_item(&self, id: ID) -> Result<Item> {
        let item = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = ?")
            .bind(id)
            .fetch_one(&self.conn)
            .await?;

        Ok(item)
    }

    fn find_score_for_item(&self, id: ID, query_res: &Vec<QueryResult<ID>>) -> Option<f64> {
        query_res
            .iter()
            .find_map(|x| if x.key == id { Some(x.score) } else { None })
    }

    pub async fn find_items(&self, name: Name) -> Result<Vec<Item>> {
        debug!("Searching for: {:?}", name);
        let mut result =
            self.index
                .read()
                .await
                .query(name.as_str(), &mut bm25::new(), tokenizer, &[1.0, 0.5]);
        if result.is_empty() {
            return Err(CustError::new("no items for search query".to_string(), StatusCode::NOT_FOUND));
        }
        debug!("Search result: {:?}", result);

        result.sort_by(|x, y| x.score.total_cmp(&y.score));
        let ids: Vec<ID> = result.iter().map(|x| x.key).collect();
        let params = format!("?{}", ", ?".repeat(ids.len() - 1));
        let query_str = format!("SELECT * FROM items WHERE id IN ({})", params);

        let query = sqlx::query_as::<_, Item>(&query_str);
        let query = ids.into_iter().fold(query, |query, id| query.bind(id));

        let items = query.fetch_all(&self.conn).await?;
        let items: Vec<(Option<f64>, Item)> = items
            .into_iter()
            .map(|item| (self.find_score_for_item(item.id.unwrap(), &result), item))
            .collect();
        let mut items: Vec<_> = items.into_iter().filter(|x| x.0.is_some()).collect();
        items.sort_by(|x, y| x.0.unwrap().total_cmp(&y.0.unwrap()));

        Ok(items.into_iter().map(|x| x.1).rev().collect())
    }

    pub async fn get_all_items(&self) -> Result<Vec<Item>> {
        Ok(sqlx::query_as::<_, Item>("SELECT * FROM items")
            .fetch_all(&self.conn)
            .await?)
    }

    pub async fn delete_item(&self, id: ID) -> Result<Item> {
        let mut tx = self.conn.begin().await?;

        let item = sqlx::query_as::<_, Item>("SELECT * from items WHERE id = ?")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

        sqlx::query("DELETE FROM items WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        self.index.write().await.remove_document(id);

        tx.commit().await?;

        // TODO: delete all connection before
        Ok(item)
    }

    pub async fn new_category(&self, mut category: Category) -> Result<Category> {
        debug!("adding new category: {:?}", category);
        category.id = None;
        let mut tx = self.conn.begin().await?;

        let tmp_cat = sqlx::query_as::<_, Category>("SELECT * FROM categories WHERE name = ?")
            .bind(category.name.clone())
            .bind(category.parent_category)
            .fetch_optional(&mut *tx)
            .await?;
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

        tx.commit().await?;

        category.id = Some(id);

        debug!("added new category: {:?}", category);
        Ok(category)
    }

    pub async fn get_all_categories(&self) -> Result<Vec<Category>> {
        Ok(sqlx::query_as::<_, Category>("SELECT * FROM categories")
            .fetch_all(&self.conn)
            .await?)
    }
}
