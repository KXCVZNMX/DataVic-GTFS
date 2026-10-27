use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use prost::Message;
use redis::aio::ConnectionManager;
use redis::RedisError;
use serde::{Deserialize, Serialize};
use crate::utils::types::gtfs_realtime::TranslatedString;

#[derive(Clone)]
pub struct AppState {
    pub redis: ConnectionManager,
}

#[derive(Debug)]
pub enum ApiError {
    Cache,
    Upstream,
    InvalidProtobuf,
    RateLimited { retry_after_seconds: u64 },
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub(crate) fetched_at_unix_ms: u128,
    pub(crate) content_type: String,
    pub(crate) body_base64: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::Cache => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Upstream => StatusCode::BAD_GATEWAY,
            ApiError::InvalidProtobuf => StatusCode::BAD_GATEWAY,
            ApiError::RateLimited {
                retry_after_seconds,
            } => {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [(header::RETRY_AFTER, retry_after_seconds.to_string())],
                    "DataVic fetch rate limit reached; retry later",
                )
                    .into_response();
            }
        };

        (status, "Failed to retrieve data").into_response()
    }
}

impl From<RedisError> for ApiError {
    fn from(_: RedisError) -> Self {
        ApiError::Cache
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(_: serde_json::Error) -> Self {
        ApiError::Cache
    }
}

impl From<base64::DecodeError> for ApiError {
    fn from(_: base64::DecodeError) -> Self {
        ApiError::InvalidProtobuf
    }
}

pub fn protobuf_response(bytes: Vec<u8>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/x-protobuf")
        .body(axum::body::Body::from(bytes))
        .expect("valid response")
}

pub mod gtfs_realtime {
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

pub fn decode_feed_message(bytes: &[u8]) -> Result<gtfs_realtime::FeedMessage, ApiError> {
    gtfs_realtime::FeedMessage::decode(bytes).map_err(|_| ApiError::InvalidProtobuf)
}

#[derive(Serialize)]
pub struct VehiclePosition {
    pub(crate) entity_id: String,
    pub(crate) vehicle_id: Option<String>,
    pub(crate) trip_id: Option<TripInformation>,
    pub(crate) p_lat: f32,
    pub(crate) p_lng: f32,
    pub(crate) p_speed: Option<f32>,
    pub(crate) time: Option<u64>
}

#[derive(Serialize)]
pub struct StopTimeEvent {
    pub(crate) delay: Option<i32>,
    pub(crate) time: Option<i64>,
}

impl From<gtfs_realtime::trip_update::StopTimeEvent> for StopTimeEvent {
    fn from(value: gtfs_realtime::trip_update::StopTimeEvent) -> Self {
        StopTimeEvent {
            delay: value.delay,
            time: value.time,
        }
    }
}

#[derive(Serialize)]
pub struct TripInformation {
    pub(crate) route_type: String,
    pub(crate) route_id: String,
    pub(crate) timetable_type: String,
}

#[derive(Serialize)]
pub struct Trip {
    pub(crate) trip_info: Option<TripInformation>,
    pub(crate) trip_id: Option<String>,
    pub(crate) stop_id: Option<String>,
    pub(crate) arrival: Option<StopTimeEvent>,
    pub(crate) departure: Option<StopTimeEvent>
}

#[derive(Serialize)]
pub struct TripUpdate {
    pub(crate) trips: Vec<Trip>,
}

#[derive(Serialize)]
pub struct ServiceAlert {
    pub(crate) effect: Option<i32>,
    pub(crate) header: Option<String>
}