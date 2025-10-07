// Copyright (c) 2025 CarByte Engineering GmbH

use paho_mqtt::{self as mqtt};
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

    // Establish connection to MQTT Broker
    let host = "mqtt://localhost:1883".to_string();

    println!("Connecting to the MQTT server at '{}'", host);

    // Create the client
    let mqtt_client = mqtt::AsyncClient::new(host).unwrap();

    // Connect with default options and wait for it to complete or fail
    // The default is an MQTT v3.x connection.
    let _ = mqtt_client.connect(None).await;

    mqtt_client.subscribe("mcu/temperature", 1).wait().unwrap();

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(100);
    mqtt_client.set_message_callback(move |_cli, msg_opt| {
        if let Some(msg) = msg_opt {
            if msg.topic() == "mcu/temperature" {
                let payload = msg.payload().to_vec();
                println!("Got message payload: {:?}", payload);
                let _ = tx.try_send(payload);
            }
        }
    });

    loop {
        match rx.try_recv() {
            Ok(msg) => {
                println!("Got message: {:?}", msg);
                let _ =v2_client.publish_value(
                    "Vehicle.Cabin.HVAC.AmbientAirTemperature".to_owned(),
                    v2_proto::Value {
                        typed_value: Some(v2_proto::value::TypedValue::Float(vec_u8_to_f32_from_string(msg))),
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

        let result = v2_client.get_value("Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color".to_owned()).await;
        match result {
            Ok(option) => match option {
                Some(datapoint) => {
                    println!("Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color: {:?}", datapoint.value);
                    match datapoint.value {
                        Some(value) => {
                            let printable = DisplayDatapoint(value);
                            //println!("Got value for Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color: {:?}", response);
                            let msg = mqtt::Message::new("compute/color", printable.to_string(), mqtt::QOS_1);
                            let _ = mqtt_client.publish(msg).await;
                        },
                        None => {
                            let msg = mqtt::Message::new("compute/color", "0", mqtt::QOS_1);
                            let _ = mqtt_client.publish(msg).await;
                        }
                    }
                }
                None => {
                    println!("Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color not set");
                }
            },
            Err(err) => {
                println!(
                    "Getting value for signal {:?} failed: {:?}",
                    "Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color", err
                );
            }
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}