// Copyright (c) 2025 Eclipse Foundation and others.
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use kuksa_rust_sdk::kuksa::common::ClientTraitV2;
use kuksa_rust_sdk::kuksa::val::v2::KuksaClientV2;
use kuksa_rust_sdk::v2_proto;
use std::fmt;
use std::time::Duration;
use up_rust::LocalUriProvider;
use up_rust::StaticUriProvider;
use up_rust::UMessageBuilder;
use up_rust::UPayloadFormat;
use up_rust::UTransport;
use up_transport_zenoh::UPTransportZenoh;
use up_transport_zenoh::zenoh_config;
use zenoh::Config;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long, default_value = "127.0.0.1")]
    host: String,
    #[clap(long, default_value = None)]
    router: Option<String>,
}

const TOPIC_TEMP: u16 = 0x8001;
const TOPIC_WETNESS: u16 = 0x8002;
const TOPIC_SPEED: u16 = 0x8003;
const TOPIC_POS_LAT: u16 = 0x8004;
const TOPIC_POS_LON: u16 = 0x8005;
const TOPIC_POS_ALT: u16 = 0x8006;

struct DisplayDatapoint(v2_proto::Value);

fn display_array<T>(f: &mut fmt::Formatter<'_>, array: &[T]) -> fmt::Result
where
    T: fmt::Display,
{
    f.write_str("[")?;
    let real_delimiter = ", ";
    let mut delimiter = "";
    for value in array {
        write!(f, "{delimiter}")?;
        delimiter = real_delimiter;
        write!(f, "{value}")?;
    }
    f.write_str("]")
}

impl fmt::Display for DisplayDatapoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.typed_value {
            Some(value) => match value {
                v2_proto::value::TypedValue::Bool(value) => f.pad(&format!("{value}")),
                v2_proto::value::TypedValue::Int32(value) => f.pad(&format!("{value}")),
                v2_proto::value::TypedValue::Int64(value) => f.pad(&format!("{value}")),
                v2_proto::value::TypedValue::Uint32(value) => f.pad(&format!("{value}")),
                v2_proto::value::TypedValue::Uint64(value) => f.pad(&format!("{value}")),
                v2_proto::value::TypedValue::Float(value) => f.pad(&format!("{value:.2}")),
                v2_proto::value::TypedValue::Double(value) => f.pad(&format!("{value}")),
                v2_proto::value::TypedValue::String(value) => f.pad(&format!("'{value}'")),
                v2_proto::value::TypedValue::StringArray(array) => display_array(f, &array.values),
                v2_proto::value::TypedValue::BoolArray(array) => display_array(f, &array.values),
                v2_proto::value::TypedValue::Int32Array(array) => display_array(f, &array.values),
                v2_proto::value::TypedValue::Int64Array(array) => display_array(f, &array.values),
                v2_proto::value::TypedValue::Uint32Array(array) => display_array(f, &array.values),
                v2_proto::value::TypedValue::Uint64Array(array) => display_array(f, &array.values),
                v2_proto::value::TypedValue::FloatArray(array) => display_array(f, &array.values),
                v2_proto::value::TypedValue::DoubleArray(array) => display_array(f, &array.values),
            },
            None => f.pad("None"),
        }
    }
}

// Helper function to create a Zenoh configuration
pub(crate) fn get_zenoh_config() -> zenoh_config::Config {
    let args = Args::parse();

    let zenoh_string = if let Some(router) = &args.router {
        format!(
            "{{ mode: 'peer', connect: {{ endpoints: [ 'tcp/{}:7447' ] }} }}",
            router
        )
    } else {
        "{ mode: 'peer' }".to_string()
    };

    println!("Zenoh String: {:?}", zenoh_string);

    let zenoh_config = Config::from_json5(&zenoh_string).expect("Failed to load Zenoh config");

    zenoh_config
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Assumption:
    // - Started after MQTT Broker and Kuksa Data broker
    // --> This is achieved with ankaios configuration

    // Initialize Kuksa Client
    let host = "http://localhost:55555";
    let mut v2_client: KuksaClientV2 = KuksaClientV2::from_host(host);

    // Initialze uProtocol + Zenoh

    // Initialize uProtocol logging
    UPTransportZenoh::try_init_log_from_env();

    // Create a uProtocol URI provider for this vehicle
    // This defines the identity of this node in the uProtocol network
    let uri_provider = StaticUriProvider::new("vehicledataaccessor", 0, 2);
    let authority = uri_provider.get_authority();

    println!("uProtocol Authority: {:?}", authority);

    // Create the uProtocol transport using Zenoh as the underlying transport
    let transport = UPTransportZenoh::builder(authority)
        .expect("invalid authority name")
        .with_config(get_zenoh_config())
        .build()
        .await?;

    loop {
        // let test_payload = format!("{}", 25.5f32);
        // let test_topic = uri_provider.get_resource_uri(TOPIC_TEMP);
        // let test_message = UMessageBuilder::publish(test_topic.clone())
        //     .build_with_payload(test_payload.clone(), UPayloadFormat::UPAYLOAD_FORMAT_TEXT)?;
        // let _ = transport.send(test_message).await;

        let result = v2_client
            .get_value("Vehicle.Cabin.HVAC.AmbientAirTemperature".to_owned())
            .await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!(
                        "Vehicle.Cabin.HVAC.AmbientAirTemperature: {:?}",
                        datapoint.value
                    );
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            println!(
                                "Got value for Vehicle.Cabin.HVAC.AmbientAirTemperature: {:?}",
                                printable.to_string()
                            );
                            // Publish on uProtocol
                            let topic = uri_provider.get_resource_uri(TOPIC_TEMP);
                            let message = UMessageBuilder::publish(topic.clone())
                                .build_with_payload(
                                    printable.to_string(),
                                    UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
                                )?;
                            transport.send(message).await?;
                        }
                        None => {
                            // TODO
                        }
                    }
                }
                None => {
                    println!("Vehicle.Cabin.HVAC.AmbientAirTemperature not set");
                }
            },
            Err(err) => {
                println!(
                    "Getting value for signal {:?} failed: {:?}",
                    "Vehicle.Cabin.HVAC.AmbientAirTemperature", err
                );
            }
        }

        let result = v2_client
            .get_value("Vehicle.Exterior.Humidity".to_owned())
            .await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!("Vehicle.Exterior.Humidity: {:?}", datapoint.value);
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            println!(
                                "Got value for Vehicle.Exterior.Humidity: {:?}",
                                printable.to_string()
                            );
                            // Publish on uProtocol
                            let topic = uri_provider.get_resource_uri(TOPIC_WETNESS);
                            let message = UMessageBuilder::publish(topic.clone())
                                .build_with_payload(
                                    printable.to_string(),
                                    UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
                                )?;
                            transport.send(message).await?;
                        }
                        None => {
                            // TODO
                        }
                    }
                }
                None => {
                    println!("Vehicle.Exterior.Humidity not set");
                }
            },
            Err(err) => {
                println!(
                    "Getting value for signal {:?} failed: {:?}",
                    "Vehicle.Exterior.Humidity", err
                );
            }
        }

        let result = v2_client.get_value("Vehicle.Speed".to_owned()).await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!("Vehicle.Speed: {:?}", datapoint.value);
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            println!("Got value for Vehicle.Speed: {:?}", printable.to_string());
                            // Publish on uProtocol
                            let topic = uri_provider.get_resource_uri(TOPIC_SPEED);
                            let message = UMessageBuilder::publish(topic.clone())
                                .build_with_payload(
                                    printable.to_string(),
                                    UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
                                )?;
                            transport.send(message).await?;
                        }
                        None => {
                            // TODO
                        }
                    }
                }
                None => {
                    println!("Vehicle.Speed not set");
                }
            },
            Err(err) => {
                println!(
                    "Getting value for signal {:?} failed: {:?}",
                    "Vehicle.Speed", err
                );
            }
        }

        let result = v2_client
            .get_value("Vehicle.CurrentLocation.Latitude".to_owned())
            .await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!("Vehicle.CurrentLocation.Latitude: {:?}", datapoint.value);
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            println!(
                                "Got value for Vehicle.CurrentLocation.Latitude: {:?}",
                                printable.to_string()
                            );
                            // Publish on uProtocol
                            let topic = uri_provider.get_resource_uri(TOPIC_POS_LAT);
                            let message = UMessageBuilder::publish(topic.clone())
                                .build_with_payload(
                                    printable.to_string(),
                                    UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
                                )?;
                            transport.send(message).await?;
                        }
                        None => {
                            // TODO
                        }
                    }
                }
                None => {
                    println!("Vehicle.CurrentLocation.Latitude not set");
                }
            },
            Err(err) => {
                println!(
                    "Getting value for signal {:?} failed: {:?}",
                    "Vehicle.CurrentLocation.Latitude", err
                );
            }
        }

        let result = v2_client
            .get_value("Vehicle.CurrentLocation.Longitude".to_owned())
            .await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!("Vehicle.CurrentLocation.Longitude: {:?}", datapoint.value);
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            println!(
                                "Got value for Vehicle.CurrentLocation.Longitude: {:?}",
                                printable.to_string()
                            );
                            // Publish on uProtocol
                            let topic = uri_provider.get_resource_uri(TOPIC_POS_LON);
                            let message = UMessageBuilder::publish(topic.clone())
                                .build_with_payload(
                                    printable.to_string(),
                                    UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
                                )?;
                            transport.send(message).await?;
                        }
                        None => {
                            // TODO
                        }
                    }
                }
                None => {
                    println!("Vehicle.CurrentLocation.Longitude not set");
                }
            },
            Err(err) => {
                println!(
                    "Getting value for signal {:?} failed: {:?}",
                    "Vehicle.CurrentLocation.Longitude", err
                );
            }
        }

        let result = v2_client
            .get_value("Vehicle.CurrentLocation.Altitude".to_owned())
            .await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!("Vehicle.CurrentLocation.Altitude: {:?}", datapoint.value);
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            println!(
                                "Got value for Vehicle.CurrentLocation.Altitude: {:?}",
                                printable.to_string()
                            );
                            // Publish on uProtocol
                            let topic = uri_provider.get_resource_uri(TOPIC_POS_ALT);
                            let message = UMessageBuilder::publish(topic.clone())
                                .build_with_payload(
                                    printable.to_string(),
                                    UPayloadFormat::UPAYLOAD_FORMAT_TEXT,
                                )?;
                            transport.send(message).await?;
                        }
                        None => {
                            // TODO
                        }
                    }
                }
                None => {
                    println!("Vehicle.CurrentLocation.Altitude not set");
                }
            },
            Err(err) => {
                println!(
                    "Getting value for signal {:?} failed: {:?}",
                    "Vehicle.CurrentLocation.Altitude", err
                );
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
