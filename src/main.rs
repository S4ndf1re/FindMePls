use std::sync::Arc;

use axum::Router;
use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use doc_search::EmptyWordFilter;
use doc_search::Index;
use doc_search::MemoryStorage;
use doc_search::SimpleTokenizer;
use futures::join;
use tonic::transport::Server;
use tracing::Level;
use tracing::log::info;

pub use business::*;
pub use error::*;
pub use files::*;
pub use grpc_service::*;
pub use routes::*;
pub use types::*;

pub mod grpc_service;

pub mod files;

pub mod types;

pub mod business;

pub mod routes;

pub mod error;

mod util;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    info!("Starting up");


    let tokenizer = SimpleTokenizer::new();
    let filter = EmptyWordFilter {};
    let storage = MemoryStorage::new("storage.json");

    // TODO: add qdrant
    let index = Index::new(None, storage);

    let state = BusinessRules::new(index, tokenizer, filter).await;

    state.init_db().await;
    state.init().await;

    // build our application with a single route
    let app = Router::new()
        .route("/item/search/:name", get(find_items)) // search for items by name (this can
        // containt any query string and will even
        // handle some fuzziness)
        .route("/item", post(add_item)) // create a new item
        .route("/item", get(get_all_items)) // gel all items
        .route("/item/:id", get(get_item)) // get a specific item
        .route("/item/:id", delete(delete_item)); // delete an item

    let app = app
        .route("/category", post(new_category)) // create a new category
        .route("/category", get(get_all_categories)); // get all categories

    let app = app
        .route("/collection", post(new_collection)) // create a new collection
        .route(
            // add an item to a collection
            "/collection/:collection_id/:item_id",
            post(add_item_to_collection),
        )
        .route(
            // get all items in a collection
            "/collection/:collection_id/items",
            get(get_items_in_collection),
        )
        .route(
            // delete an item from a collection
            "/collection/:collection_id/:item_id",
            delete(remove_item_from_collection),
        );

    let rules = Arc::new(state);
    let app = app.with_state(Arc::clone(&rules));

    let web_future = tokio::spawn(async {
        // run it with hyper on localhost:3000
        axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let grpc_future = tokio::spawn(async {
        let addr = "0.0.0.0:50051".parse().unwrap();
        let find_me_pls_grpc = FindMePlsService::new(rules);
        Server::builder()
            .add_service(FindMePlsServer::new(find_me_pls_grpc))
            .serve(addr)
            .await
            .unwrap();
    });

    let (web_res, grpc_res) = join!(web_future, grpc_future);
    web_res.unwrap();
    grpc_res.unwrap();
}
