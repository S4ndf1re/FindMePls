use std::borrow::Cow;
use serde::Serialize;
use serde::Deserialize;

pub type ID = i32;
pub type Name = String;
pub type Price = f32;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Collection {
    pub id: Option<ID>,
    pub name: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CollectionItem {
    pub collection_id: ID,
    pub item_id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: Option<ID>,
    pub name: Name,
    pub parent_category: Option<ID>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: Option<ID>,
    pub name: Name,
    pub description: Option<String>,
    pub category_id: Option<ID>,
    pub price: Option<Price>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemSearch {
    pub id: ID,
    pub name: Name,
    pub description: Option<String>
}

pub fn tokenizer(s: &str) -> Vec<Cow<str>> {
     s.split(' ').map(Cow::from).collect::<Vec<_>>()
}

pub fn title_extract(d: &ItemSearch) -> Vec<&str> {
    vec![d.name.as_str()]
}

pub fn description_extract(d: &ItemSearch) -> Vec<&str> {
    match d.description {
        Some(ref s) => vec![s.as_str()],
        None => vec![],
    }
}
