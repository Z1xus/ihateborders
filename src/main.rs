#![windows_subsystem = "windows"]

mod app;
mod config;
mod settings;
mod state;
mod system;
mod theme;
mod view;

fn main() -> eframe::Result {
    app::run()
}
