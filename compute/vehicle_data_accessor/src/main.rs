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

use tokio::sync::mpsc;
use std::time::Duration;
use kuksa_rust_sdk::kuksa::common::ClientTraitV2;
use kuksa_rust_sdk::kuksa::val::v2::KuksaClientV2;
use kuksa_rust_sdk::v2_proto;
use std::fmt;

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

fn vec_u8_to_f32_from_string(bytes: Vec<u8>) -> f32 {
    let s = str::from_utf8(&bytes).unwrap_or("");  // Convert, default empty string on error
    s.trim().parse().unwrap_or(0.0)                // Parse float, return 0.0 on error
}

#[tokio::main]
async fn main() {

    // Assumption:
    // - Started after MQTT Broker and Kuksa Data broker
    // --> This is achieved with ankaios configuration

    // Initialize Kuksa Client
    let host = "http://localhost:55555";
    let mut v2_client: KuksaClientV2 = KuksaClientV2::from_host(host);

    loop {

        let result = v2_client.get_value("Vehicle.Cabin.HVAC.AmbientAirTemperature".to_owned()).await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!("Vehicle.Cabin.HVAC.AmbientAirTemperature: {:?}", datapoint.value);
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            println!("Got value for Vehicle.Cabin.HVAC.AmbientAirTemperature: {:?}", printable.to_string());
                            // TODO: Publish on uProtocol
                            //let msg = mqtt::Message::new("compute/color", printable.to_string(), mqtt::QOS_1);
                            //let _ = mqtt_client.publish(msg).await;
                        },
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

        let result = v2_client.get_value("Vehicle.Exterior.Humidity".to_owned()).await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!("Vehicle.Exterior.Humidity: {:?}", datapoint.value);
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            println!("Got value for Vehicle.Exterior.Humidity: {:?}", printable.to_string());
                            // TODO: Publish on uProtocol
                            //let msg = mqtt::Message::new("compute/color", printable.to_string(), mqtt::QOS_1);
                            //let _ = mqtt_client.publish(msg).await;
                        },
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

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}