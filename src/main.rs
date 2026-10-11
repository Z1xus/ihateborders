#![windows_subsystem = "windows"]

mod app;
mod config;
mod settings;
mod shortcut;
mod state;
mod system;
mod theme;
mod tray;
mod view;

fn main() -> eframe::Result {
    app::run()
}
