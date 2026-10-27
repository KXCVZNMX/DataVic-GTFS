use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::extract::Query;
use axum::Json;
use serde::Deserialize;
use crate::utils::fetcher::fetch_redis;
use crate::utils::parser::parse_trip_updates;
use crate::utils::types::{ApiError, AppState, Trip};

const TU_CACHE_KEY: &str = "gtfs:realtime:tram:trip_updates";

#[derive(Deserialize)]
pub struct TramQuery {
    stop_id: String
}

pub async fn stop_arrivals(State(mut state): State<AppState>, Query(q): Query<TramQuery>) -> Result<Response, ApiError> {
    let feed = fetch_redis(&mut state, TU_CACHE_KEY).await?;
    let v: Vec<Trip> = parse_trip_updates(&feed)
        .into_iter()
        .flat_map(|trip_update| trip_update.trips)
        .filter(|trip| {
            trip.stop_id.as_deref() == Some(q.stop_id.as_str())
        })
        .collect();

    Ok(Json(v).into_response())
}