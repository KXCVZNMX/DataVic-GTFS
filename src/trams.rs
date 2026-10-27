use crate::trams::service_alert::service_alert;
use crate::trams::{trip_update::trip_updates, vehicle_position::vehicle_position};
use axum::Router;
use axum::routing::get;
use crate::trams::stop_arrivals::stop_arrivals;
use crate::utils::types::AppState;

mod service_alert;
mod trip_update;
mod vehicle_position;
mod stop_arrivals;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/trams/vehicle_position", get(vehicle_position))
        .route("/api/trams/trip_update", get(trip_updates))
        .route("/api/trams/service_alert", get(service_alert))
        .route("/api/trams/stop_arrivals", get(stop_arrivals))
        .with_state(state)
}
