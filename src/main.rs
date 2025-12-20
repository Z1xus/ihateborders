#![windows_subsystem = "windows"]

mod app;
mod config;
mod startup;
mod ui;
mod window_manager;

use app::{BorderlessApp, create_app_options};
use startup::{create_scheduled_task, is_elevated};

fn handle_elevated_task_creation(with_admin: bool) -> !
{
    if !is_elevated() {
        eprintln!("Error: Task creation requires administrator privileges");
        std::process::exit(1);
    }

    match create_scheduled_task(with_admin) {
        Ok(_) => {
            let exe_path = std::env::current_exe().expect("Failed to get exe path");
            std::process::Command::new(exe_path)
                .arg("--open-settings")
                .spawn()
                .expect("Failed to relaunch app");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Failed to create scheduled task: {}", e);
            std::process::exit(1);
        }
    }
}

fn main() -> Result<(), eframe::Error>
{
    let args: Vec<String> = std::env::args().collect();

    let mut open_settings = false;
    
    if args.len() > 1 && args[1] == "--install-admin-task" {
        handle_elevated_task_creation(true);
    }
    
    if args.len() > 1 && args[1] == "--create-startup-task" {
        handle_elevated_task_creation(false);
    }
    
    if args.len() > 1 && args[1] == "--open-settings" {
        open_settings = true;
    }

    eframe::run_native(
        "ihateborders",
        create_app_options(),
        Box::new(move |cc| Ok(Box::new(BorderlessApp::new(cc, open_settings)))),
    )
}