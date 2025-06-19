use egui::{Color32, Pos2, Rect, Style};

fn main() -> Result<(), Error> {
    eframe::run_native(
        "Cheers",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_maximized(true)
                .with_clamp_size_to_monitor_size(true),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::from(VisApp::default()))),
    )
    .map_err(Error::GuiFailure)
}

#[derive(Debug)]
enum Error {
    GuiFailure(eframe::Error),
}

#[derive(Default)]
struct VisApp {}

impl eframe::App for VisApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let style = Style::default();
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut rect = ui.available_rect_before_wrap();
            egui::Scene::new().show(ui, &mut rect, |ui| {
                let painter = ui.painter();
                painter.rect_filled(
                    Rect::from_min_max(Pos2 { x: 10., y: 10. }, Pos2 { x: 100., y: 100. }),
                    0.1,
                    Color32::from_rgb(200, 50, 50),
                );
            })
        });
    }
}
