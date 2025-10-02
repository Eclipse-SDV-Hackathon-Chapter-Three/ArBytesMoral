mod common;

use async_trait::async_trait;
use eframe::egui;
use egui_gauge::Gauge;
use epaint::Color32;
use std::{str::FromStr, sync::Arc};
use std::sync::Mutex;
use tokio::sync::mpsc;
use up_rust::{UListener, UMessage, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;

const CONFIG_PATH: &str = "src/zenoh_config.json";

#[derive(Clone, Debug)]
struct GuiUpdate {
    uri: String,
    payload: String,
    value: f32,  // Extrahierter numerischer Wert
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
            let payload_str = String::from_utf8(payload.to_vec()).unwrap();
            let uri = msg.attributes.unwrap().source.unwrap().to_uri(false);

            println!("Received message [topic: {uri}, payload: {payload_str}]");

            // Extrahiere den numerischen Wert aus dem Payload
            // Format: "event 32" -> 32
            let value = extract_number_from_payload(&payload_str);

            let update = GuiUpdate {
                uri,
                payload: payload_str,
                value,
                timestamp: std::time::SystemTime::now(),
            };

            let _ = gui_tx.send(update);
        });
    }
}

// Hilfsfunktion zum Extrahieren der Zahl aus dem Payload
fn extract_number_from_payload(payload: &str) -> f32 {
    // Versuche zuerst, den kompletten String zu parsen
    if let Ok(num) = payload.trim().parse::<f32>() {
        return num;
    }
    
    // Falls das nicht funktioniert, splitte bei Leerzeichen und parse jeden Teil
    for part in payload.split_whitespace() {
        if let Ok(num) = part.trim().parse::<f32>() {
            return num;
        }
    }
    
    // Fallback: 0.0
    0.0
}

struct DashboardApp {
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<GuiUpdate>>>,
    messages: Vec<GuiUpdate>,
    current_value: f32,
    max_value: f32,
}

impl DashboardApp {
    fn new(receiver: mpsc::UnboundedReceiver<GuiUpdate>) -> Self {
        Self {
            receiver: Arc::new(Mutex::new(receiver)),
            messages: Vec::new(),
            current_value: 0.0,
            max_value: 100.0,  // Standardwert, wird dynamisch angepasst
        }
    }
    
    fn process_updates(&mut self) {
        if let Ok(mut rx) = self.receiver.lock() {
            while let Ok(update) = rx.try_recv() {
                self.current_value = update.value;
                
                // Dynamisch den Maximalwert anpassen
                if update.value > self.max_value {
                    self.max_value = (update.value * 1.2).ceil();
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
            
            // Großer Gauge für den aktuellen Wert
            ui.vertical_centered(|ui| {
                ui.heading("Current Value");
                ui.add_space(10.0);
                
                let gauge = Gauge::new(self.current_value, 0.0..=self.max_value, 200.0, Color32::from_rgb(100, 200, 255))
                    .text(&format!("{:.0}", self.current_value));
                ui.add(gauge);
                
                ui.add_space(10.0);
                ui.label(format!("Value: {:.1}", self.current_value));
            });
            
            ui.separator();
            
            // Message Log
            ui.heading("Recent Messages");
            ui.label(format!("Total messages received: {}", self.messages.len()));
            
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    if self.messages.is_empty() {
                        ui.label("Waiting for messages...");
                    }
                    
                    egui::Grid::new("message_grid")
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Topic");
                            ui.label("Payload");
                            ui.label("Value");
                            ui.label("Time");
                            ui.end_row();
                            
                            for msg in self.messages.iter().rev().take(20) {
                                ui.label(&msg.uri);
                                ui.label(&msg.payload);
                                ui.label(format!("{:.1}", msg.value));
                                
                                if let Ok(elapsed) = msg.timestamp.elapsed() {
                                    ui.label(format!("{:.1}s ago", elapsed.as_secs_f32()));
                                }
                                ui.end_row();
                            }
                        });
                });
        });
        
        ctx.request_repaint();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("uProtocol subscriber example");

    let (gui_tx, gui_rx) = mpsc::unbounded_channel::<GuiUpdate>();

    let _tokio_handle = std::thread::Builder::new()
        .name("tokio-runtime".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");

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
                        Arc::new(SubscriberListener {
                            gui_sender: gui_tx,
                        }),
                    )
                    .await
                    .expect("Failed to register listener");

                println!("Listener registered successfully!");

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            });
        })?;

    std::thread::sleep(std::time::Duration::from_millis(500));

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
