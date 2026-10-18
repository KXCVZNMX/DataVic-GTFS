use axum::{
    Router,
    routing::{get, post},
};
use tower::ServiceBuilder;
use vercel_runtime::Error;
use vercel_runtime::axum::VercelLayer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let router = Router::new();

    // let router = Router::new()
    //     .route("/", get(home))
    //     .route("/stream", get(stream_example))
    //     .route("/users", post(create_user))
    //     .fallback(fallback);

    let app = ServiceBuilder::new()
        .layer(VercelLayer::new())
        .service(router);
    vercel_runtime::run(app).await
}