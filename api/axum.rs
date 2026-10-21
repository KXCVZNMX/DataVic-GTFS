use axum::response::IntoResponse;
use axum::{Json, Router, routing::get};
use axum::http::{HeaderMap, Uri};
use axum_governor::{nz, GovernorConfigBuilder, PeerIp, Quota, GovernorLayer};
use redis::aio::ConnectionManager;
use serde_json::json;
use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;
use pt_backend::trams;
use pt_backend::utils::types::AppState;

async fn health(headers: HeaderMap) -> Json<serde_json::Value> {
    let ip = headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    Json(json!({ "ip": ip }))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let redis_url = std::env::var("REDIS_URL").expect("Redis URL not set");
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis = ConnectionManager::new(redis_client).await?;

    let state = AppState {
        redis,
    };

    let cfg = GovernorConfigBuilder::default()
        .with_extractor(PeerIp::default())
        .expect_connect_info()
        .quota_default(Quota::requests_per_minute(nz!(24u32)))
        .finish()?;

    let router = Router::new()
        .route("/api/health", get(health))
        .merge(trams::router(state))
        .fallback(|uri: Uri| async move { format!("saw: {uri}") })
        .layer(GovernorLayer::new(cfg));

    // let router = Router::new()
    //     .merge(routes::health::router())
    //     .merge(routes::users::router());

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);
    vercel_runtime::run(app).await
}
