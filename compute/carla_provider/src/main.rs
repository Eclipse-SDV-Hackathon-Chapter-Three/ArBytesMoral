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
use kuksa_rust_sdk::kuksa::common::ClientTraitV2;
use kuksa_rust_sdk::kuksa::val::v2::KuksaClientV2;
use kuksa_rust_sdk::v2_proto;
use carla::client::Client;
use carla::client::ActorBase;
use log;

const CLIENT_TIME_MS: u64 = 5_000;
const POLLING_EGO_MS: u64 = 1_000;

#[tokio::main]
async fn main() {

    // Assumption:
    // - Started after Carla and Kuksa Data broker
    // --> This is achieved with ankaios configuration

    // Initialize Kuksa Client
    let host = "http://localhost:55555";
    let mut v2_client: KuksaClientV2 = KuksaClientV2::from_host(host);

    // Connect to Carla
    let mut carla_client = Client::connect("192.168.1.1", 2000, None);

    carla_client.set_timeout(Duration::from_millis(CLIENT_TIME_MS));

    // Configure Carla's World
    let carla_world = carla_client.world();

    // Wait for the Ego Vehicle actor
    let mut ego_vehicle_id: Option<u32> = None;

    while ego_vehicle_id.is_none() {
        log::info!("Waiting for the Ego Vehicle actor...");

        // Syncronize Carla's world
        let _ = carla_world.wait_for_tick();

        // Check if the Ego Vehicle actor exists in the world
        for actor in carla_world.actors().iter() {
            for attribute in actor.attributes().iter() {
                if attribute.id() == "role_name"
                    && attribute.value_string() == "ego_vehicle"
                {
                    log::info!(
                        "Found '{}' actor with id: {}",
                        "ego_vehicle",
                        actor.id()
                    );
                    ego_vehicle_id = Some(actor.id());
                    break;
                }
            }
        }

        // Sleep to avoid busy-waiting
        tokio::time::sleep(Duration::from_millis(POLLING_EGO_MS)).await;
    }

    let weather = carla_world.weather();

    loop {
        log::info!("Wetness: {}", weather.wetness);

        let _ =v2_client.publish_value(
            "Vehicle.Exterior.HVAC.Humidity".to_owned(),
            v2_proto::Value {
                typed_value: Some(v2_proto::value::TypedValue::Float(weather.wetness)),
            },
        ).await;

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}