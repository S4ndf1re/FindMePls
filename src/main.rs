use axum::extract::Path;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde::Serialize;

type ID = i32;
type Name = String;
type Price = f32;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Collection {
    id: ID,
    name: Name,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CollectionItem {
    colelction_id: ID,
    item_id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Category {
    id: ID,
    name: Name,
    parent_category: Option<ID>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    id: ID,
    name: Name,
    category_id: Option<ID>,
    price: Option<Price>,
    image: Vec<u8>,
}

async fn index() -> &'static str {
    "Hello, World!"
}

#[axum_macros::debug_handler]
async fn add_item(Json(_item): Json<Item>) -> Json<Item> {
    todo!()
}

#[axum_macros::debug_handler]
async fn get_item(Path(_id): Path<ID>) -> Json<Item> {
    todo!()
}

#[axum_macros::debug_handler]
async fn find_items(Path(_name): Path<Name>) -> Json<Vec<Item>> {
    todo!()
}

#[axum_macros::debug_handler]
async fn delete_item(Path(_id): Path<ID>) -> Json<Item> {
    todo!()
}

#[axum_macros::debug_handler]
async fn new_category(Json(_category): Json<Category>) -> Json<Category> {
    todo!()
}

#[axum_macros::debug_handler]
async fn get_all_categories() -> Json<Vec<Category>> {
    todo!()
}

#[axum_macros::debug_handler]
async fn new_collection(Json(_collection): Json<Collection>) -> Json<Collection> {
    todo!()
}

#[axum_macros::debug_handler]
async fn add_item_to_collection(
    Path((_collection_id, _item_id)): Path<(ID, ID)>,
) -> Json<CollectionItem> {
    todo!()
}

#[axum_macros::debug_handler]
async fn get_items_in_collection(Path(_collection_id): Path<ID>) -> Json<Vec<Item>> {
    todo!()
}

#[axum_macros::debug_handler]
async fn remove_item_from_collection(
    Path((_collection_id, _item_id)): Path<(ID, ID)>,
) -> Json<CollectionItem> {
    todo!()
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/item/:name", get(find_items))
        .route("/item", post(add_item))
        .route("/item/:id", get(get_item))
        .route("/item/:id", delete(delete_item));

    let app = app
        .route("/category", post(new_category))
        .route("/category", get(get_all_categories));

    let app = app
        .route("/collection", post(new_collection))
        .route(
            "/collection/:collection_id/:item_id",
            post(add_item_to_collection),
        )
        .route(
            "/collection/:collection_id/items",
            get(get_items_in_collection),
        )
        .route(
            "/collection/:collection_id/:item_id",
            delete(remove_item_from_collection),
        );

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
