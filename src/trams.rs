use crate::trams::service_alert::service_alert;
use crate::trams::{trip_update::trip_updates, vehicle_position::vehicle_position};
use axum::Router;
use axum::routing::get;
use crate::utils::types::AppState;

mod service_alert;
mod trip_update;
mod vehicle_position;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/trams/vehicle_position", get(vehicle_position))
        .route("/api/trams/trip_update", get(trip_updates))
        .route("/api/trams/service_alert", get(service_alert))
        .with_state(state)
}
