use crate::utils::fetcher::fetch_redis;
use crate::utils::types::{ApiError, AppState, protobuf_response};
use axum::extract::State;
use axum::response::Response;

const TU_CACHE_KEY: &str = "gtfs:realtime:tram:trip_updates";

pub async fn trip_updates(State(mut state): State<AppState>) -> Result<Response, ApiError> {
    let feed = fetch_redis(&mut state, TU_CACHE_KEY).await?;

    todo!()
}
