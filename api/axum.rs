use axum::{Router, http::Uri, middleware};
use pt_backend::rate_limit::rate_limit;
use pt_backend::trams;
use pt_backend::utils::types::AppState;
use redis::aio::ConnectionManager;
use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let redis_url = std::env::var("REDIS_URL").expect("Redis URL not set");
    let redis_client = redis::Client::open(redis_url.as_str())?;
    let redis = ConnectionManager::new(redis_client).await?;

    let state = AppState { redis };

    let router = Router::new()
        .merge(trams::router(state.clone()))
        .fallback(|uri: Uri| async move { format!("saw: {uri}") })
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit));

    // let router = Router::new()
    //     .merge(routes::health::router())
    //     .merge(routes::users::router());

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);
    vercel_runtime::run(app).await
}
