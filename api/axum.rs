mod trams;
mod utils;

use crate::utils::types::AppState;
use axum::response::IntoResponse;
use axum::{Json, Router, routing::get};
use redis::aio::ConnectionManager;
use serde_json::json;
use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

async fn health() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let redis_url = std::env::var("REDIS_URL").expect("Redis URL not set");
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis = ConnectionManager::new(redis_client).await?;

    let state = AppState {
        redis,
    };

    let router = Router::new()
        .route("/api/health", get(health))
        .merge(trams::router(state));

    // let router = Router::new()
    //     .merge(routes::health::router())
    //     .merge(routes::users::router());

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);
    vercel_runtime::run(app).await
}
