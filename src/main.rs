#![windows_subsystem = "windows"]

mod app;
mod ui;
mod window_manager;

use app::{BorderlessApp, create_app_options};

fn main() -> Result<(), eframe::Error>
{
    eframe::run_native(
        "ihateborders",
        create_app_options(),
        Box::new(|cc| Ok(Box::new(BorderlessApp::new(cc)))),
    )
}
