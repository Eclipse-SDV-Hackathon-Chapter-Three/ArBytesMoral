// Copyright (c) 2025 Elektrobit Automotive GmbH
//
// This program and the accompanying materials are made available under the
// terms of the Apache License, Version 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc
};
use std::sync::Mutex as StdMutex;
use std::time::Duration;

use carla::client::{ActorBase, Sensor, Client, Vehicle};
use carla::rpc::AttachmentType;
use kuksa_rust_sdk::kuksa::common::ClientTraitV2;
use kuksa_rust_sdk::kuksa::val::v2::KuksaClientV2;
use kuksa_rust_sdk::v2_proto;
use nalgebra::{Vector3, Isometry3, Translation3, UnitQuaternion};
use tokio::{sync::Mutex, time::sleep};

const CLIENT_TIME_MS: u64 = 5_000;
const CARLA_HOST: &str = "192.168.43.249";
const CARLA_PORT: u16 = 2000;
const TM_PORT: u16 = 8000;
const KUKSA_HOST: &str = "http://192.168.43.241:55555";
// const KUKSA_HOST: &str = "http://localhost:55555";

// Shared GNSS state (updated by sensor listener, read by tick loop)
#[derive(Default, Clone)]
struct GnssState {
    lat: f64,
    lon: f64,
    alt: f64,
}

// Velocity to speed_kmh conversion
fn velocity_to_speed_kmh(v: Vector3<f32>) -> f32 {
    let speed_ms = v.norm();
    speed_ms * 3.6
}

// KUKSA single-signal publisher as float (call this once per signal)
async fn publish_float_signal(client: Arc<Mutex<KuksaClientV2>>, path: &str, value: f32) {
    let mut c = client.lock().await;
    if let Err(e) = c
        .publish_value(
            path.to_owned(),
            v2_proto::Value {
                typed_value: Some(v2_proto::value::TypedValue::Float(value)),
            },
        )
        .await
    {
        log::warn!("Publish {path} failed: {e}");
    }
}

// KUKSA single-signal publisher as double (call this once per signal)
async fn publish_double_signal(client: Arc<Mutex<KuksaClientV2>>, path: &str, value: f64) {
    let mut c = client.lock().await;
    if let Err(e) = c
        .publish_value(
            path.to_owned(),
            v2_proto::Value {
                typed_value: Some(v2_proto::value::TypedValue::Double(value)),
            },
        )
        .await
    {
        log::warn!("Publish {path} failed: {e}");
    }
}

// Main
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    pretty_env_logger::init();

    // KUKSA client wrapped so we can share it from spawned tasks without blocking the tick loop
    let v2_client = Arc::new(Mutex::new(KuksaClientV2::from_host(KUKSA_HOST)));

    // Ctrl+C → graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    {
        let r = running.clone();
        ctrlc::set_handler(move || {
            log::warn!("Cancelled by user. Bye!");
            r.store(false, Ordering::SeqCst);
        })?;
    }

    // Connect to CARLA
    log::info!("Connecting to the CARLA server at {}:{}…", CARLA_HOST, CARLA_PORT);
    let mut carla_client = Client::connect(CARLA_HOST, CARLA_PORT, None);
    carla_client.set_timeout(Duration::from_millis(CLIENT_TIME_MS));

    // Traffic Manager (keep in outer scope so we can tick it in the loop)
    let mut carla_tm = carla_client.instance_tm(Some(TM_PORT));
    carla_tm.set_synchronous_mode(true);
    carla_tm.set_global_distance_to_leading_vehicle(2.5);

    // World in synchronous mode (50 ms per frame)
    let mut carla_world = carla_client.world();
    let mut carla_settings = carla_world.settings();
    carla_settings.synchronous_mode = true;
    carla_settings.fixed_delta_seconds = Some(0.05);
    carla_world.apply_settings(&carla_settings, Duration::from_millis(CLIENT_TIME_MS));

    log::info!(
        "World settings: synchronous={}, fixed_delta_seconds={:?}",
        carla_settings.synchronous_mode,
        carla_settings.fixed_delta_seconds
    );

    // Map & blueprints
    let carla_map = carla_world.map();
    let bp_lib = carla_world.blueprint_library();

    // Vehicle and GNSS handle
    let mut carla_vehicle: Option<Vehicle> = None;
    let gnss_state = Arc::new(StdMutex::new(GnssState::default()));
    let mut gnss_keepalive: Option<Sensor> = None;

    // Spawn a vehicle and enable autopilot bound to our TM port
    if let Some(spawn) = carla_map.recommended_spawn_points().get(0) {
        if let Some(veh_bp) = bp_lib.find("vehicle.mercedes.coupe_2020")
        {
            let actor = carla_world.spawn_actor(&veh_bp, &spawn)?;
            let vehicle: Vehicle = actor
                .try_into()
                .expect("Spawned actor is not a vehicle (check blueprint)");

            // Attach GNSS as child sensor and listen to data
            if let Some(mut gnss_bp) = bp_lib.find("sensor.other.gnss") {
                // (optional) align sensor rate with your world tick (0.05s = 20 Hz)
                let _ = gnss_bp.set_attribute("sensor_tick", "0.05");

                // relative pose on the roof
                let rel_tf: Isometry3<f32> = Isometry3::from_parts(
                    Translation3::new(0.0, 0.0, 1.8),
                    UnitQuaternion::identity(),
                );

                // Attach the sensor to the vehicle using World::spawn_actor_opt
                let gnss_actor = carla_world.spawn_actor_opt(
                    &gnss_bp,
                    &rel_tf,
                    Some(&vehicle),          // parent reference
                    AttachmentType::Rigid,   // keep it rigidly attached
                )?;

                // Turn the Actor into a Sensor and start listening
                let gnss_sensor: Sensor = gnss_actor.try_into()
                    .expect("GNSS actor is not a Sensor");
                gnss_keepalive = Some(gnss_sensor);

                // Register listener on the kept handle (no unwrap)
                if let Some(sensor) = gnss_keepalive.as_ref() {
                    let gnss_state_clone = Arc::clone(&gnss_state);
                    sensor.listen(move |data: carla::sensor::SensorData| {
                        if let Ok(m) = carla::sensor::data::GnssMeasurement::try_from(data) {
                            if let Ok(mut s) = gnss_state_clone.lock() {
                                s.lat = m.latitude();
                                s.lon = m.longitude();
                                s.alt = m.geo_location().altitude;
                            }
                        }
                    });
                }
            } else {
                log::warn!("GNSS blueprint not found");
            }

            vehicle.set_autopilot_opt(true, TM_PORT);
            log::info!("Vehicle spawned and autopilot enabled on TM port {}", TM_PORT);

            carla_vehicle = Some(vehicle);
        } else {
            log::error!("Vehicle blueprint not found");
        }
    } else {
        log::error!("No recommended spawn points available");
    }

    // Weather animation: wetness goes up/down between 0..20
    let mut carla_weather = carla_world.weather();
    let mut direction_up = true;
    let mut cnt: i32 = 0;
    let mut tick: u64 = 0;

    // Main loop
    while running.load(Ordering::SeqCst) {
        tick += 1;

        // Every 20 ticks ≈ 1 second (0.05 s * 20)
        if tick % 20 == 0 {
            if direction_up {
                cnt += 1;
                if cnt >= 20 {
                    direction_up = false;
                }
            } else {
                cnt -= 1;
                if cnt <= 0 {
                    direction_up = true;
                    cnt = 0;
                }
            }

            // Get current weather conditions from CARLA
            carla_weather = carla_world.weather();

            // Defaults if no vehicle is available
            let mut speed_kmh: f32 = 0.0;
            let (mut lat, mut lon, mut alt) = (0.0f32, 0.0f32, 0.0f32);

            // If we have a vehicle, read speed
            if let Some(ref vehicle) = carla_vehicle {
                speed_kmh = velocity_to_speed_kmh(vehicle.velocity());
            }

            // Read latest GNSS values from shared state
            if let Ok(s) = gnss_state.lock() {
                lat = s.lat as f32;
                lon = s.lon as f32;
                alt = s.alt as f32;
            }

            log::info!(
                "Speed: {:.1} km/h | lat: {:.6}, lon: {:.6}, alt: {:.1} m",
                speed_kmh,
                lat,
                lon,
                alt
            );

            // Update & apply weather
            carla_weather.wetness = cnt as f32; // 0..20 (CARLA accepts 0..100)
            log::info!("Wetness: {} %", carla_weather.wetness);
            carla_world.set_weather(&carla_weather);

            // Capture scalars for async publish
            let wet = carla_weather.wetness;
            let client = Arc::clone(&v2_client);

            // Offload 5 publishes using the single helper (non-blocking)
            tokio::spawn(async move {
                publish_float_signal(Arc::clone(&client), "Vehicle.Exterior.Humidity", wet).await;
                publish_float_signal(Arc::clone(&client), "Vehicle.Speed", speed_kmh).await;
                publish_double_signal(
                    Arc::clone(&client),
                    "Vehicle.CurrentLocation.Latitude",
                    lat as f64,
                )
                .await;
                publish_double_signal(
                    Arc::clone(&client),
                    "Vehicle.CurrentLocation.Longitude",
                    lon as f64,
                )
                .await;
                publish_double_signal(
                    Arc::clone(&client),
                    "Vehicle.CurrentLocation.Altitude",
                    alt as f64,
                )
                .await;
            });
        }

        // CPU throttle (simulation time advances only via tick())
        sleep(Duration::from_millis(50)).await;

        // Sync advance: world then Traffic Manager
        carla_world.tick();
        let _ = carla_tm.synchronous_tick();
    }

    // Cleanup sensor
    if let Some(sensor) = gnss_keepalive.take() {
        // Stop delivering callbacks
        sensor.stop();
    }

    // Restore async world settings
    let mut s = carla_world.settings();
    s.synchronous_mode = false;
    s.fixed_delta_seconds = None;
    carla_world.apply_settings(&s, Duration::from_millis(CLIENT_TIME_MS));

    log::info!("Shutdown complete.");
    Ok(())
}
