use std::path::Path;

use eframe::egui::{
    Align, Button, CentralPanel, ComboBox, CornerRadius, Image, Label, Layout, Panel, RichText,
    ScrollArea, Sense, TextEdit, Ui, UiBuilder, vec2,
};

use crate::state::State;
use crate::system::{Display, Window};
use crate::theme;

pub fn name(path: &Path) -> String {
    let name = path.file_name().unwrap_or(path.as_os_str());
    name.to_string_lossy().into_owned()
}

fn label(display: &Display) -> String {
    let number = display.name.trim_start_matches(r"\\.\DISPLAY");
    let (width, height) = (display.rect.width, display.rect.height);
    format!("Display {number} ({width} × {height})")
}

pub fn show(ui: &mut Ui, state: &mut State) {
    let titles = state.windows.iter().map(|window| window.title.as_str());
    let rules = state.config.apps.iter().map(|rule| rule.title.as_str());
    theme::fonts(ui.ctx(), &mut state.fonts, titles.chain(rules));
    Panel::top("search").show(ui, |ui| {
        ui.add_space(4.0);
        let bar = vec2(ui.available_width(), ui.spacing().interact_size.y);
        ui.allocate_ui_with_layout(bar, Layout::right_to_left(Align::Center), |ui| {
            let gray = ui.visuals().text_color();
            let settings = Button::image(theme::icon(theme::SETTINGS, gray));
            if ui.add(settings).on_hover_text("Settings").clicked() {
                state.settings = true;
            }
            let search = TextEdit::singleline(&mut state.search).hint_text("Search");
            let search = search.margin(vec2(8.0, 6.0));
            let width = ui.available_width() - 16.0 - ui.spacing().item_spacing.x;
            ui.add(search.desired_width(width));
            ui.add(theme::icon(theme::SEARCH, ui.visuals().weak_text_color()));
        });
        ui.add_space(4.0);
    });
    Panel::bottom("actions").show(ui, |ui| {
        ui.add_space(8.0);
        actions(ui, state);
        ui.add_space(8.0);
    });
    CentralPanel::default().show(ui, |ui| list(ui, state));
}

fn list(ui: &mut Ui, state: &mut State) {
    let search = state.search.to_lowercase();
    let found = |window: &&Window| {
        let text = format!("{} {}", window.title, name(&window.path)).to_lowercase();
        text.contains(&search)
    };
    let windows: Vec<Window> = state.windows.iter().filter(found).cloned().collect();
    if windows.is_empty() {
        ui.centered_and_justified(|ui| ui.weak("No Windows"));
        return;
    }
    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        for window in &windows {
            row(ui, state, window);
        }
    });
}

fn row(ui: &mut Ui, state: &mut State, window: &Window) {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 44.0), Sense::click());
    let fill = match (state.selected == Some(window.id), response.hovered()) {
        (true, _) => Some(theme::SELECTED),
        (false, true) => Some(theme::HOVERED),
        (false, false) => None,
    };
    if let Some(fill) = fill {
        ui.painter().rect_filled(rect, CornerRadius::same(6), fill);
    }
    let icon = state.icon(ui.ctx(), &window.path);
    let always = state.rule(window).is_some();
    let inside = UiBuilder::new()
        .max_rect(rect.shrink2(vec2(10.0, 0.0)))
        .layout(Layout::left_to_right(Align::Center));
    ui.scope_builder(inside, |ui| {
        match icon {
            Some(icon) => ui.add(Image::new(&icon).fit_to_exact_size(vec2(24.0, 24.0))),
            None => ui.allocate_response(vec2(24.0, 24.0), Sense::hover()),
        };
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if always {
                ui.add(theme::icon(theme::PIN, theme::LIGHT))
                    .on_hover_text("Always borderless");
            }
            if window.borderless {
                ui.add(theme::icon(theme::BORDERLESS, theme::DIM))
                    .on_hover_text("Borderless");
            }
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.add_space(6.0);
                let family = theme::family(ui.ctx(), &window.title);
                let title = RichText::new(&window.title).strong().family(family);
                ui.add(Label::new(title).truncate());
                ui.add(Label::new(RichText::new(name(&window.path)).small().weak()).truncate());
            });
        });
    });
    if response.clicked() {
        state.selected = Some(window.id);
    }
    if response.double_clicked() {
        state.toggle(window);
    }
}

fn actions(ui: &mut Ui, state: &mut State) {
    let selected = |window: &&Window| state.selected == Some(window.id);
    let Some(window) = state.windows.iter().find(selected).cloned() else {
        match state.error {
            Some(error) => ui.colored_label(theme::LIGHT, error),
            None => ui.weak("Select a window"),
        };
        return;
    };
    let note = state.notes.get(&window.id).copied();
    let always = state.rule(&window).is_some();
    let mut fit = state.fit(&window).clone();
    let mut wanted = always;
    ui.horizontal(|ui| {
        theme::check(ui, &mut fit.resize, "Resize to screen");
        let named = |display: &&Display| Some(&display.name) == fit.display.as_ref();
        let chosen = state.displays.iter().find(named).map(label);
        let chosen = chosen.unwrap_or_else(|| "Current display".into());
        let displays = ComboBox::from_id_salt("display").selected_text(chosen);
        displays.width(ui.available_width()).show_ui(ui, |ui| {
            ui.selectable_value(&mut fit.display, None, "Current display");
            for display in &state.displays {
                let name = Some(display.name.clone());
                ui.selectable_value(&mut fit.display, name, label(display));
            }
        });
    });
    ui.horizontal(|ui| {
        theme::check(ui, &mut wanted, "Always borderless");
        if let Some(note) = note {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let text = RichText::new(note).small().color(theme::LIGHT);
                ui.add(Label::new(text).truncate()).on_hover_text(note);
            });
        }
    });
    let text = match window.borderless {
        true => "Restore Borders",
        false => "Make Borderless",
    };
    let size = vec2(ui.available_width(), 32.0);
    let button = ui.add_enabled_ui(!(always && window.borderless), |ui| {
        let button = Button::new(RichText::new(text).strong()).fill(theme::ACCENT);
        ui.add_sized(size, button)
            .on_disabled_hover_text("Turn off Always borderless first")
    });
    if button.inner.clicked() {
        state.toggle(&window);
    }
    if fit != *state.fit(&window) {
        *state.fit(&window) = fit;
        state.save();
        if window.borderless {
            state.strip(&window);
        }
    }
    if wanted != always {
        state.always(&window, wanted);
    }
}
