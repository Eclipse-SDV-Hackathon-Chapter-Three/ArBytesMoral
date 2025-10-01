/********************************************************************************
 * Copyright (c) 2024 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

/*!
This example illustrates how uProtocol's Transport Layer API can be used to subscribe
to messages that are published to a topic using the Zenoh transport.

This example works in conjunction with the `publisher`, which should be started in
another terminal after having started this subscriber.
*/

mod common;

use async_trait::async_trait;
use std::{str::FromStr, sync::Arc};
use tokio::sync::mpsc;
use up_rust::{UListener, UMessage, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;

const CONFIG_PATH: &str = "src/zenoh_config.json";

#[derive(Clone, Debug)]
struct GuiUpdate {
    uri: String,
    payload: String,
    timestamp: std::time::SystemTime,
}

struct SubscriberListener {
    message_processing_rt: tokio::runtime::Runtime,
    gui_sender: mpsc::UnboundedSender<GuiUpdate>,
}
#[async_trait]
impl UListener for SubscriberListener {
    async fn on_receive(&self, msg: UMessage) {
        let gui_tx = self.gui_sender.clone();

        self.message_processing_rt.spawn(async move {
            let payload = msg.payload.unwrap();
            let value = String::from_utf8(payload.to_vec()).unwrap();
            let uri = msg.attributes.unwrap().source.unwrap().to_uri(false);

            println!("Received message [topic: {uri}, payload: {value}]");

            // Sende Update an GUI-Thread
            let update = GuiUpdate {
                uri,
                payload: value,
                timestamp: std::time::SystemTime::now(),
            };

            let _ = gui_tx.send(update);
        });
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initiate logging
    UPTransportZenoh::try_init_log_from_env();

    println!("uProtocol subscriber example");

    // Channel für GUI-Updates erstellen
    let (gui_tx, mut gui_rx) = mpsc::unbounded_channel::<GuiUpdate>();

    // GUI-Thread starten (Beispiel mit std::thread)
    let gui_thread = std::thread::Builder::new()
        .name("gui-thread".to_string())
        .spawn(move || {
            // Hier die GUI-Runtime (z.B. egui, iced, gtk-rs)
            run_gui(gui_rx);
        })?;

    let transport = UPTransportZenoh::builder("subscriber")
        .expect("invalid authority name")
        .with_config_path(CONFIG_PATH.to_string())
        .build()
        .await?;

    // create uuri
    let source_filter = UUri {
        authority_name: "vehicledataaccessor".to_string(),
        ue_id: 0x0000, // Entity ID
        ue_version_major: 2,
        resource_id: 0x8002, // Resource ID für Publish
        ..Default::default()
    };

    println!(
        "Registering message listener [source filter: {}]",
        source_filter.to_uri(false)
    );

    let message_processing_rt = tokio::runtime::Builder::new_multi_thread()
        .thread_name("message-processing")
        .worker_threads(1)
        .build()?;
    transport
        .register_listener(
            &source_filter,
            None,
            Arc::new(SubscriberListener {
                message_processing_rt,
                gui_sender: gui_tx,
            }),
        )
        .await?;

    tokio::signal::ctrl_c().await.map_err(Box::from)
}

fn run_gui(mut receiver: mpsc::UnboundedReceiver<GuiUpdate>) {
    // GUI-Initialisierung hier
    // Beispiel für eine einfache Schleife
    while let Some(update) = receiver.blocking_recv() {
        println!("[GUI] Update: {} - {}", update.uri, update.payload);
        // GUI-Elemente hier aktualisieren
    }
}
