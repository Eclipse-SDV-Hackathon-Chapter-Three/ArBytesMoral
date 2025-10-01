use eframe::egui;
use egui::Slider;
use egui_gauge::Gauge;
use epaint::Color32;
use std::sync::{mpsc::{self, Receiver}, Arc};
use std::thread;
use tokio::runtime::Builder;
use tokio::io::{AsyncBufReadExt, BufReader};

struct GaugeExample {
    value: u64,
    settings: bool,
    receiver: Receiver<u64>,
    _runtime: Arc<tokio::runtime::Runtime>,
}

impl GaugeExample {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let ctx_clone = cc.egui_ctx.clone();
        
        // Runtime als Arc erstellen
        let runtime = Arc::new(
            Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        );
        
        let runtime_clone = runtime.clone();
        
        // Separater Thread für block_on
        thread::spawn(move || {
            runtime_clone.block_on(async {
                let stdin = tokio::io::stdin();
                let reader = BufReader::new(stdin);
                let mut lines = reader.lines();
                
                println!("Geben Sie Werte ein (0-260):");
                
                while let Ok(Some(line)) = lines.next_line().await {
                    if let Ok(new_value) = line.trim().parse::<u64>() {
                        if new_value <= 260 {
                            if sender.send(new_value).is_ok() {
                                ctx_clone.request_repaint();
                                println!("Wert aktualisiert: {}", new_value);
                            } else {
                                break;
                            }
                        } else {
                            println!("Fehler: Wert muss zwischen 0 und 260 sein");
                        }
                    } else {
                        println!("Fehler: Ungültige Zahl");
                    }
                }
            });
        });
        
        Self {
            value: 0,
            settings: false,
            receiver,
            runtime,
        }
    }
}

impl eframe::App for GaugeExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(new_value) = self.receiver.try_recv() {
            self.value = new_value;
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gauge Example");
            ui.spacing_mut().slider_width = 300.0;
            ui.add(Slider::new(&mut self.value, 0..=260));
            ui.add(Gauge::new(self.value, 0..=260, 300.0, Color32::RED)
                .text("some text"));

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
        "Gauge Example",
        native_options,
        Box::new(|cc| Ok(Box::new(GaugeExample::new(cc)))),
    )
    .unwrap();
}
