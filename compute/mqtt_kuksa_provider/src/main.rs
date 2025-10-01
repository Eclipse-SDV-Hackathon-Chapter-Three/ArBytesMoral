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

use paho_mqtt::{self as mqtt, MQTT_VERSION_5, QOS_1};
use tokio::sync::mpsc;
use std::time::Duration;
use kuksa_rust_sdk::kuksa::common;
use kuksa_rust_sdk::kuksa::common::ClientTraitV2;
use kuksa_rust_sdk::kuksa::val::v2::KuksaClientV2;
use kuksa_rust_sdk::v2_proto;

#[tokio::main]
async fn main() {

    // Initialize Kuksa Client
    let host = "http://localhost:55555";
    let mut v2_client: KuksaClientV2 = KuksaClientV2::from_host(host);

    // Establish connection to MQTT Broker
    let host = "mqtt://localhost:1883".to_string();

    println!("Connecting to the MQTT server at '{}'", host);

    // Create the client
    let mut mqtt_client = mqtt::AsyncClient::new(host).unwrap();

    // Get message stream before connecting.
    let strm = mqtt_client.get_stream(25);

    // Connect with default options and wait for it to complete or fail
    // The default is an MQTT v3.x connection.
    mqtt_client.connect(None).await;

    mqtt_client.subscribe("mcu/temperature", 1).wait().unwrap();

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
    mqtt_client.set_message_callback(move |_cli, msg_opt| {
        if let Some(msg) = msg_opt {
            if msg.topic() == "mcu/temperature" {
                let payload = msg.payload().to_vec();
                println!("Got message payload: {:?}", payload);
                // TODO publish via kuksa
                            // Spawn an async Tokio task to handle publishing asynchronously
                let _ = tx.try_send(payload);
            }
        }
    });

    // tokio::spawn(async move {
    //     while let Some(payload) = rx.recv().await {
    //         println!("Processing payload asynchronously: {:?}", payload);
    //         // TODO asynchronous publish via Kuksa here using await
    //         v2_client.publish_value(
    //             "Vehicle.Speed".to_owned(),
    //             v2_proto::Value {
    //                 typed_value: Some(v2_proto::value::TypedValue::Float(30.0)),
    //             },
    //         ).await;
    //     }
    // });

    // Wait asynchronously for the message on the oneshot receiver

    // We listen for the Ambient Light Color Data Updates
    // match common::ClientTraitV2::subscribe(
    //     &mut v2_client,
    //     vec!["Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color".to_owned()],
    //     None,
    //     None,
    // )
    // .await
    // {
    //     Ok(mut stream) => {
    //         println!("Successfully subscribed to {:?}!", "Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide");
    //         tokio::spawn(async move {
    //             match stream.message().await {
    //                 Ok(option) => {
    //                     let response = option.unwrap();
    //                     for entry_update in response.entries {
    //                         let datapoint = entry_update.1;
    //                         println!("Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide: {datapoint:?}");
    //                     }
    //                 }
    //                 Err(err) => {
    //                     println!("Error: Could not receive response {err:?}");
    //                 }
    //             }
    //         });
    //     }
    //     Err(err) => {
    //         println!("Failed to subscribe to {:?}: {:?}", "Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide", err);
    //     }
    // }



    loop {

        match rx.try_recv() {
            Ok(msg) => {
                println!("Got message: {:?}", msg);
                // TODO Write to correct vss signal
                v2_client.publish_value(
                    "Vehicle.Speed".to_owned(),
                    v2_proto::Value {
                        typed_value: Some(v2_proto::value::TypedValue::Float(30.0)),
                    },
                ).await;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                // No message available now, but sender is still active
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                // Channel closed, no more messages possible
                println!("Channel closed");
            }
        }

        match v2_client.get_value("Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color".to_owned()).await {
            Ok(response) => {
                println!("Got value for Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color: {:?}", response);
                let msg = mqtt::Message::new("compute/color", "test", mqtt::QOS_1); // TODO correct value
                mqtt_client.publish(msg).await;
            }
            Err(err) => {
                println!(
                    "Getting value for signal {:?} failed: {:?}",
                    "Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color", err
                );
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    println!("Disconnecting");
    mqtt_client.disconnect(None).await;
}