mod common;

use async_trait::async_trait;
use eframe::egui;
use egui_gauge::Gauge;
use epaint::Color32;
use std::sync::Mutex;
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
    gui_sender: mpsc::UnboundedSender<GuiUpdate>,
}

#[async_trait]
impl UListener for SubscriberListener {
    async fn on_receive(&self, msg: UMessage) {
        let gui_tx = self.gui_sender.clone();

        tokio::spawn(async move {
            let payload = msg.payload.unwrap();
            let value = String::from_utf8(payload.to_vec()).unwrap();
            let uri = msg.attributes.unwrap().source.unwrap().to_uri(false);

            println!("Received message [topic: {uri}, payload: {value}]");
            //println!("{}", value);
            let update = GuiUpdate {
                uri,
                payload: value,
                timestamp: std::time::SystemTime::now(),
            };

            let _ = gui_tx.send(update);
        });
    }
}

struct DashboardApp {
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<GuiUpdate>>>,
    messages: Vec<GuiUpdate>,
    current_speed: f32,
    current_rpm: f32,
}

impl DashboardApp {
    fn new(receiver: mpsc::UnboundedReceiver<GuiUpdate>) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(receiver)),
            messages: Vec::new(),
            current_speed: 0.0,
            current_rpm: 0.0,
        }
    }

    fn process_updates(&mut self) {
        if let Ok(mut rx) = self.receiver.lock() {
            while let Ok(update) = rx.try_recv() {
                if let Ok(value) = update.payload.parse::<f32>() {
                    if update.uri.contains("event") {
                        self.current_speed = value;
                    } else if update.uri.contains("rpm") {
                        self.current_rpm = value;
                    }
                }

                self.messages.push(update);

                if self.messages.len() > 100 {
                    self.messages.remove(0);
                }
            }
        }
    }
}

impl eframe::App for DashboardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_updates();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Vehicle Dashboard");
            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Speed");
                    let gauge =
                        Gauge::new(self.current_speed, 0.0..=200.0, 150.0, Color32::LIGHT_BLUE)
                            .text("km/h");
                    ui.add(gauge);
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.heading("RPM");
                    let gauge =
                        Gauge::new(self.current_rpm, 0.0..=8000.0, 150.0, Color32::LIGHT_RED)
                            .text("RPM");
                    ui.add(gauge);
                });
            });

            ui.separator();

            ui.heading("Recent Messages");
            ui.label(format!("Total messages: {}", self.messages.len()));
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    if self.messages.is_empty() {
                        ui.label("Waiting for messages...");
                    }
                    for msg in self.messages.iter().rev().take(20) {
                        ui.horizontal(|ui| {
                            ui.label(&msg.uri);
                            ui.separator();
                            ui.label(&msg.payload);
                        });
                    }
                });
        });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("uProtocol subscriber example");

    let (gui_tx, gui_rx) = mpsc::unbounded_channel::<GuiUpdate>();

    // Tokio Runtime im separaten Thread - OHNE block_on auf Ctrl+C
    let _tokio_handle = std::thread::Builder::new()
        .name("tokio-runtime".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");

            // Spawne den Setup-Task und halte Runtime am Leben
            rt.block_on(async move {
                UPTransportZenoh::try_init_log_from_env();

                let transport = UPTransportZenoh::builder("subscriber")
                    .expect("invalid authority name")
                    .with_config(common::get_zenoh_config())
                    //.with_config_path(CONFIG_PATH.to_string())
                    .build()
                    .await
                    .expect("Failed to build transport");

                // let source_filter = UUri {
                //     authority_name: "vehicledataaccessor".to_string(),
                //     ue_id: 0x0000,
                //     ue_version_major: 2,
                //     resource_id: 0x8002,
                //     ..Default::default()
                // };

                let source_filter = UUri::from_str("//*/FFFFB1DA/1/8001").unwrap();

                println!(
                    "Registering message listener [source filter: {}]",
                    source_filter.to_uri(false)
                );

                transport
                    .register_listener(
                        &source_filter,
                        None,
                        Arc::new(SubscriberListener { gui_sender: gui_tx }),
                    )
                    .await
                    .expect("Failed to register listener");

                println!("Listener registered successfully!");

                // Debug-Task zum Testen
                tokio::spawn(async {
                    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
                    loop {
                        interval.tick().await;
                        println!("[DEBUG] Tokio runtime is active and running...");
                    }
                });

                // Halte Runtime am Leben - warte ENDLOS statt auf Ctrl+C
                // Die Runtime bleibt aktiv bis der Thread beendet wird (GUI schließt)
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });
        })?;

    // Gib der Runtime Zeit zum Starten
    std::thread::sleep(std::time::Duration::from_millis(500));

    // GUI im Main-Thread starten
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Zenoh Vehicle Dashboard"),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Zenoh Dashboard",
        native_options,
        Box::new(|_cc| Ok(Box::new(DashboardApp::new(gui_rx)))),
    );

    Ok(())
}
