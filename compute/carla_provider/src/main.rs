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
    Arc,
};
use std::time::Duration;

use carla::client::{Client, Vehicle};
use kuksa_rust_sdk::kuksa::common::ClientTraitV2;
use kuksa_rust_sdk::kuksa::val::v2::KuksaClientV2;
use kuksa_rust_sdk::v2_proto;
use tokio::{sync::Mutex, time::sleep};

const CLIENT_TIME_MS: u64 = 5_000;
const CARLA_HOST: &str = "192.168.43.249";
const CARLA_PORT: u16 = 2000;
const TM_PORT: u16 = 8000;
const KUKSA_HOST: &str = "http://192.168.43.241:55555";
// const KUKSA_HOST: &str = "http://localhost:55555";

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

    // Spawn a vehicle and enable autopilot bound to our TM port
    if let Some(spawn) = carla_world.map().recommended_spawn_points().get(0) {
        if let Some(bp) = carla_world
            .blueprint_library()
            .find("vehicle.mercedes.coupe_2020")
        {
            let actor = carla_world.spawn_actor(&bp, &spawn)?;
            let vehicle: Vehicle = actor
                .try_into()
                .expect("Spawned actor is not a vehicle (check blueprint)");
            vehicle.set_autopilot_opt(true, TM_PORT);
            log::info!("Vehicle spawned and autopilot enabled on TM port {}", TM_PORT);
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

            carla_weather.wetness = cnt as f32; // range 0..20 (CARLA accepts 0..100)
            log::info!("Wetness: {}", carla_weather.wetness);

            // Apply weather immediately (never await in the tick path)
            carla_world.set_weather(&carla_weather);

            // Offload KUKSA publish so the tick loop never blocks on I/O
            let val = carla_weather.wetness;
            let v2 = Arc::clone(&v2_client);
            tokio::spawn(async move {
                let mut client = v2.lock().await;
                if let Err(e) = client
                    .publish_value(
                        "Vehicle.Exterior.Humidity".to_owned(),
                        v2_proto::Value {
                            typed_value: Some(v2_proto::value::TypedValue::Float(val)),
                        },
                    )
                    .await
                {
                    log::warn!("publish_value failed: {e}");
                }
            });
        }

        // Light CPU throttle (simulation time advances only via tick())
        sleep(Duration::from_millis(50)).await;

        // Sync advance: world then Traffic Manager
        carla_world.tick();
        let _ = carla_tm.synchronous_tick();
    }

    // Cleanup: restore async world settings
    let mut s = carla_world.settings();
    s.synchronous_mode = false;
    s.fixed_delta_seconds = None;
    carla_world.apply_settings(&s, Duration::from_millis(CLIENT_TIME_MS));

    log::info!("Shutdown complete.");
    Ok(())
}
