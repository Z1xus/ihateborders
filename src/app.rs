use std::env;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use eframe::egui::{
    Context, IconData, SystemTheme, Ui, ViewportBuilder, ViewportCommand, ViewportId,
};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use tray_icon::TrayIcon;

use crate::state::State;
use crate::{settings, system, theme, tray, view};

pub enum Asked {
    Open,
    Quit,
    Run(String),
}

struct App {
    state: State,
    tray: Option<TrayIcon>,
    asked: Receiver<Asked>,
    hidden: bool,
    shown: bool,
    quit: bool,
}

impl eframe::App for App {
    fn logic(&mut self, context: &Context, _: &mut eframe::Frame) {
        // winit resets the title bar on a windows settings change
        if self.state.poll(self.shown) && self.shown {
            let dark = ViewportCommand::SetTheme(SystemTheme::Dark);
            if self.state.settings {
                context.send_viewport_cmd_to(ViewportId::from_hash_of("settings"), dark.clone());
            }
            context.send_viewport_cmd(dark);
        }
        // eframe shows the window after the first frame, so it hides in the next one
        if self.hidden && context.cumulative_pass_nr() > 0 {
            context.send_viewport_cmd(ViewportCommand::Visible(false));
            (self.hidden, self.shown) = (false, false);
        } else if self.hidden {
            context.request_repaint();
        }
        while let Ok(asked) = self.asked.try_recv() {
            match asked {
                Asked::Open => {
                    context.send_viewport_cmd(ViewportCommand::Visible(true));
                    self.shown = true;
                    context.send_viewport_cmd(ViewportCommand::Focus);
                }
                Asked::Run(command) => self.state.run(&command),
                Asked::Quit => {
                    self.quit = true;
                    context.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }
        if let Some(wait) = self.state.wait(self.shown) {
            context.request_repaint_after(wait);
        }
    }

    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        view::show(ui, &mut self.state);
        if self.state.settings {
            settings::show(ui.ctx(), &mut self.state);
        }
        let stay = self.tray.is_some() && self.state.config.tray && !self.quit;
        if stay && ui.input(|input| input.viewport().close_requested()) {
            ui.send_viewport_cmd(ViewportCommand::CancelClose);
            ui.send_viewport_cmd(ViewportCommand::Visible(false));
            self.shown = false;
        }
    }
}

pub fn run() -> eframe::Result {
    let command = env::args().skip(1).collect::<Vec<_>>().join("\n");
    let hidden = command == "--hidden";
    if system::send(&command) {
        return Ok(());
    }
    // a command with no app to take it runs once, with no window
    if !command.is_empty() && !hidden {
        State::new().run(&command);
        return Ok(());
    }
    let icon = image::load_from_memory(include_bytes!("../assets/icon.ico"))
        .expect("the icon in the app is a valid image")
        .into_rgba8();
    let window = ViewportBuilder::default()
        .with_inner_size([380.0, 520.0])
        .with_min_inner_size([340.0, 320.0])
        .with_max_inner_size([560.0, 900.0])
        .with_icon(Arc::new(IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.to_vec(),
        }));
    let options = eframe::NativeOptions {
        viewport: window,
        ..Default::default()
    };
    eframe::run_native(
        "ihateborders",
        options,
        Box::new(move |creation| {
            theme::apply(&creation.egui_ctx);
            let context = creation.egui_ctx.clone();
            let wake = move || context.request_repaint();
            let (sender, asked) = channel();
            let tray = tray::tray(&icon, sender.clone(), wake.clone()).ok();
            let (pressed, woken) = (sender.clone(), wake.clone());
            GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
                if event.state == HotKeyState::Pressed {
                    let _ = pressed.send(Asked::Run("toggle".into()));
                    woken();
                }
            }));
            system::watch(wake.clone());
            system::listen(move |command| {
                let asked = match command.is_empty() {
                    true => Asked::Open,
                    false => Asked::Run(command),
                };
                let _ = sender.send(asked);
                wake();
            });
            Ok(Box::new(App {
                state: State::new(),
                hidden: hidden && tray.is_some(),
                tray,
                asked,
                shown: true,
                quit: false,
            }))
        }),
    )
}
