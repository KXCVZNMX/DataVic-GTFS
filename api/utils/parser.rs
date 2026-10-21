use crate::utils::types::gtfs_realtime::FeedMessage;
use crate::utils::types::VehiclePosition;

pub fn parse_vehicle_positions(feed: &FeedMessage) -> Vec<VehiclePosition> {
    let mut v = Vec::new();
    for entity in &feed.entity {
        // Ignore deleted entities from differential feeds.
        if entity.is_deleted.unwrap_or(false) {
            continue;
        }

        let Some(vehicle) = &entity.vehicle else {
            continue;
        };

        let Some(position) = &vehicle.position else {
            continue;
        };

        let vehicle_id = vehicle
            .vehicle
            .as_ref()
            .and_then(|v| v.id.as_deref());

        let trip_id = vehicle
            .trip
            .as_ref()
            .and_then(|trip| trip.trip_id.as_deref());

        v.push( VehiclePosition {
                entity_id: entity.id.clone(),
                vehicle_id: if let Some(vid) = vehicle_id { Some(vid.to_owned()) } else { None },
                trip_id: if let Some(tid) = trip_id { Some(tid.to_owned()) } else { None },
                p_lat: position.latitude,
                p_lng: position.longitude,
                p_speed: position.speed,
                time: vehicle.timestamp,
            }
        )
    }

    v
}