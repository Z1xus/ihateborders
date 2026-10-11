use std::sync::Arc;
use std::time::Duration;

use eframe::egui::{Context, IconData, Ui, ViewportBuilder};

use crate::state::State;
use crate::{settings, system, theme, view};

struct App(State);

impl eframe::App for App {
    fn logic(&mut self, context: &Context, _: &mut eframe::Frame) {
        self.0.poll();
        context.request_repaint_after(Duration::from_secs(1));
    }

    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        view::show(ui, &mut self.0);
        if self.0.settings {
            settings::show(ui.ctx(), &mut self.0);
        }
    }
}

pub fn run() -> eframe::Result {
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
            rgba: icon.into_raw(),
        }));
    let options = eframe::NativeOptions {
        viewport: window,
        ..Default::default()
    };
    eframe::run_native(
        "ihateborders",
        options,
        Box::new(|creation| {
            theme::apply(&creation.egui_ctx);
            let context = creation.egui_ctx.clone();
            system::watch(move || context.request_repaint());
            Ok(Box::new(App(State::new())))
        }),
    )
}
