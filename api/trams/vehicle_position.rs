use crate::utils::fetcher::fetch_redis;
use crate::utils::types::{ApiError, AppState, protobuf_response};
use crate::utils::parser::parse_vehicle_positions;
use axum::extract::State;
use axum::Json;
use axum::response::{IntoResponse, Response};

const VP_CACHE_KEY: &str = "gtfs:realtime:tram:vehicle_positions";

pub async fn vehicle_position(State(mut state): State<AppState>) -> Result<Response, ApiError> {
    let feed = fetch_redis(&mut state, VP_CACHE_KEY).await?;
    let v = parse_vehicle_positions(&feed);

    Ok(Json(v).into_response())
}
