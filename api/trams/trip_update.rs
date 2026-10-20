use crate::utils::fetcher::fetch_redis;
use crate::utils::types::{ApiError, AppState, protobuf_response};
use axum::extract::State;
use axum::response::Response;

const TRIP_UPDATES_URL: &str = "https://api.opendata.transport.vic.gov.au/opendata/public-transport/gtfs/realtime/v1/tram/trip-updates";
const TU_CACHE_KEY: &str = "gtfs:trams:trip-updates";

pub async fn trip_updates(State(state): State<AppState>) -> Result<Response, ApiError> {
    // let feed = fetch_redis(&state).await?;

    todo!()
}
