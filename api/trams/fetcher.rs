use redis::AsyncCommands;
use base64::{engine::general_purpose::STANDARD, Engine};
use crate::types::{gtfs_realtime::FeedMessage, ApiError, AppState, Snapshot, decode_feed_message};

pub async fn fetch_redis(state: &mut AppState, key: &str) -> Result<FeedMessage, ApiError> {
    let resp: Option<String> = state
        .redis
        .get(&key)
        .await?;

    let snapshot: Snapshot = resp
        .ok_or(ApiError::Cache)
        .and_then(|v| {
            serde_json::from_str(&v).map_err(ApiError::from)
        })?;

    let pb_data = STANDARD.decode(&snapshot.body_base64)?;

    decode_feed_message(&pb_data)
}