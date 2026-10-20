use crate::trams::fetcher::fetch_redis;
use crate::types::{ApiError, AppState, protobuf_response};
use axum::extract::State;
use axum::response::Response;
use prost::Message;

const SERVICE_ALERT_URL: &str = "https://api.opendata.transport.vic.gov.au/opendata/public-transport/gtfs/realtime/v1/tram/service-alerts";
const SA_CACHE_KEY: &str = "gtfs:trams:service-alerts";

pub async fn service_alert(State(state): State<AppState>) -> Result<Response, ApiError> {
    // let feed = fetch_redis(&state).await?;

    todo!()
}
