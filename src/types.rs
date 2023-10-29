use std::borrow::Cow;

use axum::http::StatusCode;
use base64::Engine;
use serde::Deserialize;
use serde::Serialize;

use crate::CustError;
use crate::find_me_pls;
use crate::Result;
use crate::Storeable;

pub type ID = i32;
pub type Name = String;
pub type Price = f32;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Collection {
    pub id: Option<ID>,
    pub name: Name,
    pub thumbnail: Option<String>,
}

impl From<find_me_pls::Collection> for Collection {
    fn from(collection: find_me_pls::Collection) -> Self {
        Self {
            id: collection.id,
            name: collection.name,
            thumbnail: collection.thumbnail
                .map(|t| base64::engine::general_purpose::STANDARD.encode(t)),
        }
    }
}

impl From<Collection> for find_me_pls::Collection {
    fn from(collection: Collection) -> Self {
        let thumbnail = match collection.thumbnail {
            Some(thumbnail) => match base64::engine::general_purpose::STANDARD.decode(thumbnail) {
                Ok(thumbnail) => Some(thumbnail),
                _ => None,
            },
            None => None,
        };
        Self {
            id: collection.id,
            name: collection.name,
            thumbnail,
        }
    }
}

impl Storeable for Collection {
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

impl From<find_me_pls::Category> for Category {
    fn from(category: find_me_pls::Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
            parent_category: category.parent_category,
            thumbnail: category
                .thumbnail
                .map(|t| base64::engine::general_purpose::STANDARD.encode(t)),
        }
    }
}

impl From<Category> for find_me_pls::Category {
    fn from(category: Category) -> Self {
        let thumbnail = match category.thumbnail {
            Some(thumbnail) => match base64::engine::general_purpose::STANDARD.decode(thumbnail) {
                Ok(thumbnail) => Some(thumbnail),
                _ => None,
            },
            None => None,
        };

        Self {
            id: category.id,
            name: category.name,
            parent_category: category.parent_category,
            thumbnail,
        }
    }
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

impl From<find_me_pls::Item> for Item {
    fn from(item: find_me_pls::Item) -> Self {
        Self {
            id: item.id,
            name: item.name,
            description: item.description,
            category_id: item.category_id,
            price: item.price,
            thumbnail: item
                .thumbnail
                .map(|t| base64::engine::general_purpose::STANDARD.encode(t)),
            fullsize: item
                .fullsize
                .map(|t| base64::engine::general_purpose::STANDARD.encode(t)),
        }
    }
}

impl From<Item> for find_me_pls::Item {
    fn from(item: Item) -> Self {
        let thumbnail = match item.thumbnail {
            Some(thumbnail) => match base64::engine::general_purpose::STANDARD.decode(thumbnail) {
                Ok(thumbnail) => Some(thumbnail),
                _ => None,
            },
            None => None,
        };

        let fullsize = match item.fullsize {
            Some(fullsize) => match base64::engine::general_purpose::STANDARD.decode(fullsize) {
                Ok(fullsize) => Some(fullsize),
                _ => None,
            },
            None => None,
        };

        Self {
            id: item.id,
            name: item.name,
            description: item.description,
            category_id: item.category_id,
            price: item.price,
            thumbnail,
            fullsize,
        }
    }
}

impl Storeable for Item {
    fn as_bytes<'a>(&'a self) -> Result<Cow<'a, Vec<u8>>> {
        let mut data = vec![];

        let mut thumbnail_data = match &self.thumbnail {
            Some(thumbnail) => base64::engine::general_purpose::STANDARD.decode(thumbnail)?,
            None => vec![],
        };

        data.extend_from_slice(&thumbnail_data.len().to_le_bytes());
        data.append(&mut thumbnail_data);

        let mut image_data = match &self.fullsize {
            Some(fullsize) => base64::engine::general_purpose::STANDARD.decode(fullsize)?,
            None => vec![],
        };

        data.extend_from_slice(&image_data.len().to_le_bytes());
        data.append(&mut image_data);

        Ok(Cow::Owned(data))
    }

    fn change_from_bytes(&mut self, bytes: &[u8]) {
        // read first 4 to 8 (depending on 64 or 32 system) as the size for the thumbnail data
        // stream
        let mut size_bytes: [u8; (usize::BITS / 8) as usize] = [0; (usize::BITS / 8) as usize];
        size_bytes.copy_from_slice(&bytes[0..(usize::BITS / 8) as usize]);
        let size = usize::from_le_bytes(size_bytes);

        // remove first size bytes as they are only used for the rest size
        let rest = &bytes[((usize::BITS / 8) as usize)..];

        // read thumbnail data
        let data = &rest[0..size];
        let rest = &rest[size..];

        let thumbnail = base64::engine::general_purpose::STANDARD.encode(data);
        self.thumbnail = Some(thumbnail);

        // read image size
        let mut size_bytes: [u8; (usize::BITS / 8) as usize] = [0; (usize::BITS / 8) as usize];
        size_bytes.copy_from_slice(&rest[0..(usize::BITS / 8) as usize]);
        let size = usize::from_le_bytes(size_bytes);

        // define rest without image size
        let rest = &rest[((usize::BITS / 8) as usize)..];

        // read image data
        let data = &rest[0..size];
        let rest = &rest[size..];

        let image = base64::engine::general_purpose::STANDARD.encode(data);
        self.fullsize = Some(image);

        assert!(rest.len() == 0);
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

#[cfg(test)]
mod test_image_to_file {
    use crate::{Item, Storeable};

    #[test]
    fn serialize_and_deserialize() {
        let item = Item {
            id: None,
            name: "".to_owned(),
            description: None,
            category_id: None,
            price: None,
            thumbnail: Some("YXNkZg==".to_owned()),
            fullsize: Some("ZmRhcw==".to_owned()),
        };
        let data = item.as_bytes();
        assert!(data.is_ok());

        let data = data.unwrap();
        let mut item2 = item.clone();
        item2.thumbnail = None;
        item2.fullsize = None;

        item2.change_from_bytes(data.as_ref());
        assert!(item.thumbnail == item2.thumbnail);
        assert!(item.fullsize == item2.fullsize);
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
