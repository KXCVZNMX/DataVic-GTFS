use crate::utils::types::gtfs_realtime::FeedMessage;
use crate::utils::types::{StopTimeEvent, TripUpdate, Trip, VehiclePosition};

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

pub fn parse_trip_updates(feed: &FeedMessage) -> Vec<TripUpdate> {
    let mut tus: Vec<TripUpdate> = Vec::new();

    for entity in &feed.entity {
        let mut trips = Vec::new();

        if let Some(update) = &entity.trip_update {
            let trip_id = update.trip.trip_id.as_deref();

            for stop_update in &update.stop_time_update {
                trips.push( Trip {
                    trip_id: if let Some(tid) = trip_id { Some(tid.to_owned()) } else { None },
                    stop_id: stop_update.stop_id.clone(),
                    arrival: if let Some(arr) = stop_update.arrival { Some(StopTimeEvent::from(arr)) } else { None },
                    departure: if let Some(dep) = stop_update.departure { Some(StopTimeEvent::from(dep)) } else { None },
                });
            }
        }

        tus.push(TripUpdate {
            trips
        })
    }

    tus
}