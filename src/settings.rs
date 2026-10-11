use std::time::{Duration, Instant};

use eframe::egui::{
    Align, Button, CentralPanel, ComboBox, Context, CursorIcon, Event, Image, Key, Label, Layout,
    Panel, RichText, ScrollArea, Sense, Ui, Vec2, ViewportBuilder, ViewportId, vec2,
};

use crate::config::Match;
use crate::state::State;
use crate::view::name;
use crate::{system, theme};

const ICON: Vec2 = vec2(20.0, 20.0);
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("COMMIT"), ")");
const COPIED: Duration = Duration::from_millis(1500);

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
        Panel::bottom("version").show(ui, |ui| version(ui, state));
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

fn version(ui: &mut Ui, state: &mut State) {
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let copied = state.copied.is_some_and(|copied| copied.elapsed() < COPIED);
            let text = if copied { "Copied" } else { VERSION };
            let version = Label::new(RichText::new(text).small().weak());
            let version = ui.add(version.selectable(false).sense(Sense::click()));
            let version = version.on_hover_cursor(CursorIcon::PointingHand);
            if version.on_hover_text("Copy debug info").clicked() {
                ui.ctx().copy_text(details(ui.ctx(), state));
                state.copied = Some(Instant::now());
            }
            if copied {
                ui.ctx().request_repaint_after(COPIED);
            }
        });
    });
}

// no path or window title: they can name a person or a file
fn details(context: &Context, state: &State) -> String {
    let yes = |on: bool| if on { "yes" } else { "no" };
    let displays = state.displays.iter().map(|display| {
        let rect = &display.rect;
        let (width, height) = (rect.width, rect.height);
        let (name, dpi) = (&display.name, display.dpi);
        format!("{name} {width}x{height} at {},{} dpi {dpi}", rect.x, rect.y)
    });
    let config = &state.config;
    let rules = |by: Match| config.apps.iter().filter(|rule| rule.by == by).count();
    let selected = state
        .windows
        .iter()
        .find(|window| state.selected == Some(window.id));
    let window = selected.map_or("none".into(), |window| {
        let before = state.before.get(&window.id);
        let before = before.map(|before| format!("\nwindow before: {}", before.inspect()));
        format!(
            "{} on {}, borderless {}, always borderless {}\nwindow now: {}{}",
            name(&window.path),
            window.display,
            yes(window.borderless),
            yes(state.rule(window).is_some()),
            system::inspect(window.id),
            before.unwrap_or_default()
        )
    });
    let notes: Vec<&str> = state.notes.values().copied().chain(state.error).collect();
    [
        format!("ihateborders: {VERSION}"),
        format!("windows: {}", system::version()),
        format!(
            "administrator: {}",
            system::administrator(std::process::id())
        ),
        format!("displays: {}", displays.collect::<Vec<_>>().join(", ")),
        format!("scale: {}", context.pixels_per_point()),
        format!("resize to screen: {}", yes(config.fit.resize)),
        format!(
            "display: {}",
            config.fit.display.as_deref().unwrap_or("current")
        ),
        format!("start with windows: {}", yes(state.startup)),
        format!("tray: {}", yes(config.tray)),
        format!("shortcut: {}", config.shortcut),
        format!("shortcut registered: {}", yes(state.bound)),
        format!(
            "always borderless: {} by name, {} by path, {} by title",
            rules(Match::Name),
            rules(Match::Path),
            rules(Match::Title)
        ),
        format!("messages: {}", notes.join(", ")),
        format!("window: {window}"),
    ]
    .join("\n")
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
