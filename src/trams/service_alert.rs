use crate::utils::fetcher::fetch_redis;
use crate::utils::types::{ApiError, AppState, protobuf_response};
use axum::extract::State;
use axum::Json;
use axum::response::{IntoResponse, Response};
use crate::utils::parser::parse_service_alert;

const SA_CACHE_KEY: &str = "gtfs:realtime:tram:service_alerts";

pub async fn service_alert(State(mut state): State<AppState>) -> Result<Response, ApiError> {
    let feed = fetch_redis(&mut state, SA_CACHE_KEY).await?;
    let v = parse_service_alert(&feed);

    Ok(Json(v).into_response())
}
