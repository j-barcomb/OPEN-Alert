#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod compose;
mod history;
mod settings_store;
mod settings_tab;
mod theme;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "IPAWS Alert Console",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("IPAWS Alert Console")
                .with_inner_size([1180.0, 780.0])
                .with_min_inner_size([900.0, 600.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(app::IpawsApp::new(cc)))),
    )
}
