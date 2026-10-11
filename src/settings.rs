use eframe::egui::{
    Align, Button, CentralPanel, ComboBox, Context, Event, Image, Key, Label, Layout, RichText,
    ScrollArea, Sense, Ui, Vec2, ViewportBuilder, ViewportId, vec2,
};

use crate::config::Match;
use crate::state::State;
use crate::theme;
use crate::view::name;

const ICON: Vec2 = vec2(20.0, 20.0);

fn label(by: Match) -> &'static str {
    match by {
        Match::Name => "App name",
        Match::Path => "App path",
        Match::Title => "Window title",
    }
}

pub fn show(context: &Context, state: &mut State) {
    let window = ViewportBuilder::default()
        .with_title("Settings")
        .with_inner_size([420.0, 360.0])
        .with_resizable(false);
    context.show_viewport_immediate(ViewportId::from_hash_of("settings"), window, |ui, _| {
        CentralPanel::default().show(ui, |ui| {
            ui.weak("General");
            let mut startup = state.startup;
            if theme::check(ui, &mut startup, "Start with Windows").changed() {
                state.start(startup);
            }
            let tray = &mut state.config.tray;
            if theme::check(ui, tray, "Keep running in the tray").changed() {
                state.save();
            }
            shortcut(ui, state);
            if let Some(error) = state.error {
                ui.colored_label(theme::LIGHT, error);
            }
            ui.add_space(8.0);
            ui.weak("Always borderless");
            if state.config.apps.is_empty() {
                ui.weak("Select a window and turn on Always borderless");
            }
            let mut removed = None;
            let mut changed = false;
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                for index in 0..state.config.apps.len() {
                    let path = state.config.apps[index].path.clone();
                    let icon = state.icon(ui.ctx(), &path);
                    let rule = &mut state.config.apps[index];
                    ui.horizontal(|ui| {
                        match icon {
                            Some(icon) => ui.add(Image::new(&icon).fit_to_exact_size(ICON)),
                            None => ui.allocate_response(ICON, Sense::hover()),
                        };
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let gray = ui.visuals().text_color();
                            let remove = Button::image(theme::icon(theme::REMOVE, gray));
                            if ui.add(remove).on_hover_text("Remove").clicked() {
                                removed = Some(index);
                            }
                            let before = rule.by;
                            let matches =
                                ComboBox::from_id_salt(index).selected_text(label(before));
                            matches.width(110.0).show_ui(ui, |ui| {
                                for by in [Match::Name, Match::Path, Match::Title] {
                                    ui.selectable_value(&mut rule.by, by, label(by));
                                }
                            });
                            changed |= rule.by != before;
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.label(name(&path));
                                let detail = match rule.by {
                                    _ if !path.exists() => "Not found".into(),
                                    Match::Title => rule.title.clone(),
                                    _ => path.with_file_name("").to_string_lossy().into_owned(),
                                };
                                let family = theme::family(ui.ctx(), &detail);
                                let detail = RichText::new(detail).small().weak().family(family);
                                ui.add(Label::new(detail).truncate());
                            });
                        });
                    });
                }
            });
            if let Some(index) = removed {
                state.config.apps.remove(index);
            }
            if changed || removed.is_some() {
                state.save();
            }
        });
        if ui.input(|input| input.viewport().close_requested()) {
            state.settings = false;
            state.bind();
        }
    });
}

fn shortcut(ui: &mut Ui, state: &mut State) {
    ui.horizontal(|ui| {
        ui.label("Shortcut");
        let keys = match (state.recording, state.config.shortcut.as_str()) {
            (true, _) => "Press keys…",
            (false, "") => "None",
            (false, keys) => keys,
        };
        let keys = ui.button(keys).on_hover_text("Toggles the active window");
        if keys.clicked() && !state.recording {
            state.record();
        } else if keys.clicked() {
            state.bind();
        }
        if !state.config.shortcut.is_empty() && !state.recording {
            let gray = ui.visuals().text_color();
            let remove = Button::image(theme::icon(theme::REMOVE, gray));
            if ui.add(remove).on_hover_text("Remove").clicked() {
                state.config.shortcut.clear();
                state.bind();
            }
        }
        if !state.bound {
            ui.weak("Can't use these keys");
        }
    });
    if !state.recording {
        return;
    }
    match pressed(ui) {
        Some(Key::Escape) => state.bind(),
        Some(key) => {
            let held = ui.input(|input| input.modifiers);
            let held = [
                (held.ctrl, "Ctrl"),
                (held.alt, "Alt"),
                (held.shift, "Shift"),
            ];
            let mut keys: Vec<&str> = held
                .iter()
                .filter(|held| held.0)
                .map(|held| held.1)
                .collect();
            // a key with no modifier is taken from every app, so only the function keys can be alone
            if keys.is_empty() && !(Key::F1..=Key::F35).contains(&key) {
                return;
            }
            keys.push(key.name());
            state.config.shortcut = keys.join("+");
            state.bind();
        }
        None => {}
    }
}

fn pressed(ui: &Ui) -> Option<Key> {
    let modifier = |key: &Key| {
        matches!(
            key,
            Key::ShiftLeft
                | Key::ShiftRight
                | Key::ControlLeft
                | Key::ControlRight
                | Key::AltLeft
                | Key::AltRight
                | Key::SuperLeft
                | Key::SuperRight
        )
    };
    ui.input(|input| {
        input.events.iter().find_map(|event| match event {
            Event::Key {
                key, pressed: true, ..
            } if !modifier(key) => Some(*key),
            _ => None,
        })
    })
}
