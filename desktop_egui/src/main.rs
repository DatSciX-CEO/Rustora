mod app;
mod facade;
mod state;
mod ui;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("Rustora"),
        ..Default::default()
    };

    eframe::run_native(
        "Rustora",
        options,
        Box::new(|cc| Ok(Box::new(app::RustoraApp::new(cc)))),
    )
}
