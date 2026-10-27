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
    let v = parse_trip_updates(&feed)
        .into_iter()
        .map(|trip_update| {
            let mut tv = Vec::new();
            let _ = trip_update.trips.into_iter().map(|trips| {
                if trips.stop_id == Some(q.stop_id.clone()) {
                    tv.push(trips)
                }
            });

            tv
        })
        .collect::<Vec<Vec<Trip>>>();

    Ok(Json(v).into_response())
}