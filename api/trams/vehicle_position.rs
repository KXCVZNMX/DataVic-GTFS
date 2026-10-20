use crate::utils::fetcher::fetch_redis;
use crate::utils::types::{ApiError, AppState, protobuf_response};
use axum::extract::State;
use axum::response::Response;

const VEHICLE_POSITION_URL: &str = "https://api.opendata.transport.vic.gov.au/opendata/public-transport/gtfs/realtime/v1/tram/vehicle-positions";
const VP_CACHE_KEY: &str = "gtfs:trams:vehicle-position";

pub async fn vehicle_position(State(state): State<AppState>) -> Result<Response, ApiError> {
    // let feed = fetch_redis(&state).await?;

    todo!()
}
