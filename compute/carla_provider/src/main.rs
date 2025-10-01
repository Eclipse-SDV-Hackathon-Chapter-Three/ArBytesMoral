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

use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use kuksa_rust_sdk::kuksa::common::ClientTraitV2;
use kuksa_rust_sdk::kuksa::val::v2::KuksaClientV2;
use kuksa_rust_sdk::v2_proto;
use carla::client::Client;
use carla::client::Vehicle;
use tokio::time::sleep;
use tokio::sync::Mutex;
use log;

const CLIENT_TIME_MS: u64 = 5_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    // Assumption:
    // - Started after Carla and Kuksa Data broker
    // --> This is achieved with ankaios configuration

    // Initiate logging
    pretty_env_logger::init();

    // Initialize Kuksa Client
//    let host = "http://localhost:55555";
    let host = "http://192.168.43.241:55555";
    //let mut v2_client: KuksaClientV2 = KuksaClientV2::from_host(host);
    let v2_raw: KuksaClientV2 = KuksaClientV2::from_host(host);
    let v2_client = Arc::new(Mutex::new(v2_raw));


    // Stop the program gracefully on Ctrl-C
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    ctrlc::set_handler(move || {
        log::warn!("Cancelled by user. Bye!");
        running_clone.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Connect to the Carla Server
    log::info!("Connecting to the Carla Server at {}:{}...", "192.168.43.249", 2000);
    let mut carla_client = Client::connect("192.168.43.249", 2000, None);
    carla_client.set_timeout(Duration::from_millis(CLIENT_TIME_MS));

    let carla_tm_port = 8000u16;
    let mut carla_tm = carla_client.instance_tm(Some(carla_tm_port));
    carla_tm.set_synchronous_mode(true);

    // Configure Carla's World
    let mut carla_world = carla_client.world();
    let mut carla_settings = carla_world.settings();

    carla_settings.synchronous_mode = true;
    carla_settings.fixed_delta_seconds = Some(0.05);
    carla_world.apply_settings(&carla_settings, Duration::from_millis(CLIENT_TIME_MS));

    log::info!(
        "World Settings: Synchronous mode: {}, Fixed delta seconds: {:?}",
         carla_settings.synchronous_mode, carla_settings.fixed_delta_seconds
     );

    let carla_map = carla_world.map();
    let carla_spawns = carla_map.recommended_spawn_points();
    
    if let Some(carla_spawn) = carla_spawns.get(0) {
        if let Some(carla_vehicle_actor_bpl) = carla_world.blueprint_library().find("vehicle.mercedes.coupe_2020") {
            let carla_vehicle_actor = carla_world.spawn_actor(&carla_vehicle_actor_bpl, &carla_spawn)?;
            let carla_vehicle: Vehicle = carla_vehicle_actor.try_into().expect("Spawned Actor is no vehicle (check blueprint)");
            carla_tm.set_global_distance_to_leading_vehicle(2.5);
            carla_vehicle.set_autopilot_opt(true, carla_tm_port);
        }
    }

    let mut carla_weather = carla_world.weather();
    let mut direction_up = true;
    let mut cnt: i32 = 0;
    let mut tick = 0;

    loop {
        tick += 1;

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

            carla_weather.wetness = cnt as f32;
            log::info!("Wetness: {}", carla_weather.wetness);

            carla_world.set_weather(&carla_weather);

//            let _ =v2_client.publish_value(
//                "Vehicle.Exterior.Humidity".to_owned(),
//                v2_proto::Value {
//                    typed_value: Some(v2_proto::value::TypedValue::Float(carla_weather.wetness)),
//                },
//            ).await;
            // Publish NICHT blockierend auslagern
            let val = carla_weather.wetness;
            let v2 = Arc::clone(&v2_client);
            tokio::spawn(async move {
                let mut client = v2.lock().await; // &mut KuksaClientV2
                if let Err(e) = client.publish_value(
                    "Vehicle.Exterior.Humidity".to_owned(),
                    v2_proto::Value {
                        typed_value: Some(v2_proto::value::TypedValue::Float(val)),
                    },
                ).await {
                    log::warn!("publish_value failed: {e}");
                }
            });
        }

        //tokio::time::sleep(Duration::from_millis(50)).await;
        sleep(Duration::from_millis(50)).await;
        
        carla_world.tick();
        let _ = carla_tm.synchronous_tick();
    }
}