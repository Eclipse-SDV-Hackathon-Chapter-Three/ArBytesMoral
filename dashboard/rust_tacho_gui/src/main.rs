use eframe::egui;
use egui::Slider;
use egui_gauge::Gauge;
use epaint::Color32;

#[derive(Default)]
struct GaugeExample {
    value: u64,
    settings: bool,
}

impl GaugeExample {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for GaugeExample {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gauge Example");
            ui.spacing_mut().slider_width = 300.0;
            ui.add(Slider::new(&mut self.value, 0..=260));
            ui.add(Gauge::new(self.value, 0..=260, 300.0, Color32::RED).text("some text"));

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