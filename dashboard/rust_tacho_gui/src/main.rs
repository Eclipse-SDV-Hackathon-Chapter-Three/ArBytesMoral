mod common;

use async_trait::async_trait;
use eframe::egui;
use egui_gauge::Gauge;
use epaint::Color32;
use std::sync::{
    Arc,
    mpsc::{self, Receiver},
};
use tokio::runtime::Builder;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;
use zenoh::config::Config;

const CONFIG_PATH: &str = "src/zenoh_config.json";

// UListener Implementation für Message Handling
struct ValueUpdateListener {
    sender: std::sync::mpsc::Sender<u64>,
    ctx: egui::Context,
}

#[async_trait]
impl UListener for ValueUpdateListener {
    async fn on_receive(&self, msg: UMessage) {
        // Payload aus der UMessage extrahieren
        if let Some(payload) = msg.payload {
            // Annahme: Payload enthält u64 Wert
            if let Some(value) = String::from_utf8(payload.to_vec())
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
            {
                if value <= 260 {
                    if self.sender.send(value).is_ok() {
                        self.ctx.request_repaint();
                        println!("Wert von uProtocol empfangen: {}", value);
                    }
                }
            }
        }
    }
}

struct GaugeExample {
    value: u64,
    settings: bool,
    receiver: Receiver<u64>,
    runtime: Arc<tokio::runtime::Runtime>,
    _transport: Option<Arc<UPTransportZenoh>>,
}

impl GaugeExample {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let ctx_clone = cc.egui_ctx.clone();

        // Tokio Runtime erstellen
        let runtime = Arc::new(Builder::new_multi_thread().enable_all().build().unwrap());

        let runtime_clone = runtime.clone();

        // Transport und Listener in separatem Thread initialisieren
        std::thread::spawn(move || {
            runtime_clone.block_on(async {
                // UPTransportZenoh erstellen
                let transport = UPTransportZenoh::builder("subscriber")
                    .expect("invalid authority name")
                    .with_config_path(CONFIG_PATH.to_string())
                    .build()
                    .await
                    .unwrap();

            

                // UUri für das Topic definieren (anpassen an deinen Use-Case)
                let source_filter = UUri {
                    authority_name: "vehicledataaccessor".to_string(),
                    ue_id: 0x0000, // Entity ID
                    ue_version_major: 2,
                    resource_id: 0x8002, // Resource ID für Publish
                    ..Default::default()
                };

                // Listener erstellen und registrieren
                let listener = Arc::new(ValueUpdateListener {
                    sender,
                    ctx: ctx_clone,
                });

                transport
                    .register_listener(&source_filter, None, listener)
                    .await
                    .unwrap();

                println!(
                    "Listener erfolgreich registriert für Topic: {:?}",
                    source_filter
                );

                // Transport am Leben halten
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });
        });

        Self {
            value: 0,
            settings: false,
            receiver,
            runtime,
            _transport: None,
        }
    }
}

impl eframe::App for GaugeExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Neue Werte vom Channel empfangen
        while let Ok(new_value) = self.receiver.try_recv() {
            self.value = new_value;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gauge Example - uProtocol");

            ui.label(format!("Aktueller Wert: {}", self.value));

            ui.spacing_mut().slider_width = 300.0;
            ui.add(
                Gauge::new(self.value, 0..=260, 300.0, Color32::RED)
                    .text(&format!("{}", self.value)),
            );

            ui.checkbox(&mut self.settings, "Settings");
            egui::Window::new("Settings")
                .open(&mut self.settings)
                .vscroll(true)
                .show(ctx, |ui| {
                    ctx.settings_ui(ui);
                });
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Gauge Example - uProtocol",
        native_options,
        Box::new(|cc| Ok(Box::new(GaugeExample::new(cc)))),
    )
    .unwrap();
}
