use crate::utils::fetcher::fetch_redis;
use crate::utils::types::{ApiError, AppState};
use axum::extract::State;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::utils::parser::parse_trip_updates;

const TU_CACHE_KEY: &str = "gtfs:realtime:tram:trip_updates";

pub async fn trip_updates(State(mut state): State<AppState>) -> Result<Response, ApiError> {
    let feed = fetch_redis(&mut state, TU_CACHE_KEY).await?;
    let v = parse_trip_updates(&feed);

    Ok(Json(v).into_response())
}
