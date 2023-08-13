use axum::http::StatusCode;
use base64::Engine;
use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;

use crate::CustError;
use crate::Result;
use crate::Storeable;

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
    pub thumbnail: Option<String>,
}

impl Storeable for Category {
    fn as_bytes<'a>(&'a self) -> Result<Cow<'a, Vec<u8>>> {
        Ok(match &self.thumbnail {
            Some(thumbnail) => {
                Cow::Owned(base64::engine::general_purpose::STANDARD.decode(thumbnail)?)
            }
            None => Cow::Owned(vec![]),
        })
    }

    fn change_from_bytes(&mut self, bytes: &[u8]) {
        let thumbnail = base64::engine::general_purpose::STANDARD.encode(bytes);
        self.thumbnail = Some(thumbnail);
    }

    fn filename<'a>(&'a self) -> Result<Cow<'a, str>> {
        match self.id {
            Some(id) => Ok(Cow::Owned(format!("{}.dat", id))),
            None => Err(CustError::new(
                "No valid id, therefore no existing filename".to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: Option<ID>,
    pub name: Name,
    pub description: Option<String>,
    pub category_id: Option<ID>,
    pub price: Option<Price>,
    pub thumbnail: Option<String>,
    pub fullsize: Option<String>,
}

impl Storeable for Item {
    fn as_bytes<'a>(&'a self) -> Result<Cow<'a, Vec<u8>>> {
        let mut data = vec![];


        let thumbnail_data = match &self.thumbnail {
            Some(thumbnail) => {
                base64::engine::general_purpose::STANDARD.decode(thumbnail)?
            }
            None => vec![],
        };

        // data.con thumbnail_data.len().to_le_bytes();

        Ok(Cow::Owned(data))
    }

    fn change_from_bytes(&mut self, bytes: &[u8]) {
        let thumbnail = base64::engine::general_purpose::STANDARD.encode(bytes);
        self.thumbnail = Some(thumbnail);
    }

    fn filename<'a>(&'a self) -> Result<Cow<'a, str>> {
        match self.id {
            Some(id) => Ok(Cow::Owned(format!("{}.dat", id))),
            None => Err(CustError::new(
                "No valid id, therefore no existing filename".to_owned(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemSearch {
    pub id: ID,
    pub name: Name,
    pub description: Option<String>,
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
