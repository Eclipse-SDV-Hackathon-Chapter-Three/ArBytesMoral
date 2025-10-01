mod common;

use async_trait::async_trait;

use eframe::egui;
use egui::Slider;
use egui_gauge::Gauge;
use epaint::Color32;
use std::io::{self, BufRead};
use std::str::FromStr;
use std::sync::{Arc, mpsc::{self, Receiver}};
use std::thread;
use up_rust::{UListener, UMessage, UTransport, UUri};
use up_transport_zenoh::UPTransportZenoh;


struct SubscriberListener(tokio::runtime::Runtime);
#[async_trait]
impl UListener for SubscriberListener {
    async fn on_receive(&self, msg: UMessage) {
        // Offload processing of the message to a dedicated tokio runtime using
        // threads not used by Zenoh.
        self.0.spawn(async move {
            let payload = msg.payload.unwrap();
            let value = String::from_utf8(payload.to_vec()).unwrap();
            let uri = msg.attributes.unwrap().source.unwrap().to_uri(false);
            println!("Received message [topic: {uri}, payload: {value}]");
        });
    }
}
struct GaugeExample {
    value: u64,
    settings: bool,
    receiver: Receiver<u64>,
}

impl GaugeExample {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let ctx_clone = cc.egui_ctx.clone();

        // Separater Thread für CLI-Input
        thread::spawn(move || {
            let stdin = io::stdin();
            let reader = stdin.lock();

            println!("Geben Sie Werte ein (0-260):");

            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        if let Ok(new_value) = text.trim().parse::<u64>() {
                            if new_value <= 260 {
                                if sender.send(new_value).is_ok() {
                                    ctx_clone.request_repaint();
                                    println!("Wert aktualisiert: {}", new_value);
                                }
                            } else {
                                println!("Fehler: Wert muss zwischen 0 und 260 sein");
                            }
                        } else {
                            println!("Fehler: Ungültige Zahl");
                        }
                    }
                    Err(e) => eprintln!("Fehler beim Lesen: {}", e),
                }
            }
        });

        Self {
            value: 0,
            settings: false,
            receiver,
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
            ui.heading("Team ArBytesMoral");
            ui.spacing_mut().slider_width = 300.0;
            // ui.add(Slider::new(&mut self.value, 0..=260));
            ui.add(Gauge::new(self.value, 0..=260, 300.0, Color32::RED).text("Speed [km/h]"));

            /* ui.checkbox(&mut self.settings, "Settings");
            egui::Window::new("Settings")
                .open(&mut self.settings)
                .vscroll(true)
                .show(ctx, |ui| {
                    ctx.settings_ui(ui);
                }); */
        });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Gauge Example",
        native_options,
        Box::new(|cc| Ok(Box::new(GaugeExample::new(cc)))),
    )
    .unwrap();
}
