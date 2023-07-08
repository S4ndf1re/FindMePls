pub mod types;
use std::sync::Arc;
use tracing::Level;
use tracing::log::info;
pub use types::*;

pub mod business;
pub use business::*;

pub mod routes;
pub use routes::*;

pub mod error;
pub use error::*;

use axum::routing::delete;
use axum::routing::get;
use axum::routing::post;
use axum::Router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
    info!("Starting up");
    let state = BusinessRules::new().await;

    state.init_db().await;
    state.init().await;

    // build our application with a single route
    let app = Router::new()
        .route("/item/search/:name", get(find_items))
        .route("/item", post(add_item))
        .route("/item", get(get_all_items))
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

    let app = app.with_state(Arc::new(state));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
