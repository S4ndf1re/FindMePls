use std::sync::Arc;
use axum::extract::Path;
use axum::{extract::State, Json};

use crate::{BusinessRules, Category, Collection, CollectionItem, Item, Name, Result, ID};

#[axum_macros::debug_handler]
pub async fn add_item(
    State(state): State<Arc<BusinessRules>>,
    Json(item): Json<Item>,
) -> Result<Json<Item>> {
    Ok(Json(state.add_item(item).await?))
}

#[axum_macros::debug_handler]
pub async fn get_all_items(State(state): State<Arc<BusinessRules>>) -> Result<Json<Vec<Item>>> {
    Ok(Json(state.get_all_items().await?))
}

#[axum_macros::debug_handler]
pub async fn get_item(
    State(state): State<Arc<BusinessRules>>,
    Path(id): Path<ID>,
) -> Result<Json<Item>> {
    Ok(Json(state.get_item(id).await?))
}

#[axum_macros::debug_handler]
pub async fn find_items(
    State(state): State<Arc<BusinessRules>>,
    Path(name): Path<Name>,
) -> Result<Json<Vec<Item>>> {
    Ok(Json(state.find_items(name).await?))
}

#[axum_macros::debug_handler]
pub async fn delete_item(
    State(state): State<Arc<BusinessRules>>,
    Path(id): Path<ID>,
) -> Result<Json<Item>> {
    Ok(Json(state.delete_item(id).await?))
}

#[axum_macros::debug_handler]
pub async fn new_category(
    State(state): State<Arc<BusinessRules>>,
    Json(category): Json<Category>,
) -> Result<Json<Category>> {
    Ok(Json(state.new_category(category).await?))
}

#[axum_macros::debug_handler]
pub async fn get_all_categories(State(state): State<Arc<BusinessRules>>) -> Result<Json<Vec<Category>>> {
    Ok(Json(state.get_all_categories().await?))
}

#[axum_macros::debug_handler]
pub async fn new_collection(Json(_collection): Json<Collection>) -> Result<Json<Collection>> {
    todo!()
}

#[axum_macros::debug_handler]
pub async fn add_item_to_collection(
    Path((_collection_id, _item_id)): Path<(ID, ID)>,
) -> Result<Json<CollectionItem>> {
    todo!()
}

#[axum_macros::debug_handler]
pub async fn get_items_in_collection(Path(_collection_id): Path<ID>) -> Result<Json<Vec<Item>>> {
    todo!()
}

#[axum_macros::debug_handler]
pub async fn remove_item_from_collection(
    Path((_collection_id, _item_id)): Path<(ID, ID)>,
) -> Result<Json<CollectionItem>> {
    todo!()
}
