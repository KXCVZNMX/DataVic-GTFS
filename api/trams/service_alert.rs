use crate::utils::fetcher::fetch_redis;
use crate::utils::types::{ApiError, AppState, protobuf_response};
use axum::extract::State;
use axum::response::Response;

const SA_CACHE_KEY: &str = "gtfs:trams:service-alerts";

pub async fn service_alert(State(state): State<AppState>) -> Result<Response, ApiError> {
    // let feed = fetch_redis(&state).await?;

    todo!()
}
