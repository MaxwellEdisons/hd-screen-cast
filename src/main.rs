#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod model;
mod platform;
mod virtual_config;

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--admin") {
        if let Err(error) = platform::admin_entry(args.get(2).map(String::as_str).unwrap_or("")) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("高清投屏 · HD Screen Cast")
            .with_inner_size([1120.0, 780.0])
            .with_min_inner_size([900.0, 650.0]),
        ..Default::default()
    };
    eframe::run_native(
        "HD Screen Cast",
        options,
        Box::new(|cc| Ok(Box::new(app::ControlApp::new(cc)))),
    )
}
