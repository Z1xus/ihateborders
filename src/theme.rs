use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};

use eframe::egui::{
    Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontTweak, Image,
    ImageSource, Rect, Response, Sense, Shape, Stroke, StrokeKind, TextStyle, Theme, Ui, Vec2,
    Visuals, WidgetText, include_image, pos2, vec2,
};

const BACKGROUND: Color32 = Color32::from_rgb(16, 16, 20);
const SURFACE: Color32 = Color32::from_rgb(28, 27, 31);
const OUTLINE: Color32 = Color32::from_rgb(73, 69, 79);
const TEXT: Color32 = Color32::from_rgb(230, 224, 233);
pub const DIM: Color32 = Color32::from_rgb(147, 143, 153);
const PRESSED: Color32 = Color32::from_rgb(79, 55, 139);
pub const ACCENT: Color32 = Color32::from_rgb(103, 80, 164);
pub const LIGHT: Color32 = Color32::from_rgb(208, 188, 255);
pub const SELECTED: Color32 = Color32::from_rgb(45, 39, 64);
pub const HOVERED: Color32 = SURFACE;

pub const SETTINGS: ImageSource<'static> = include_image!("../assets/icons/settings.svg");
pub const SEARCH: ImageSource<'static> = include_image!("../assets/icons/search.svg");
pub const REMOVE: ImageSource<'static> = include_image!("../assets/icons/x.svg");
pub const BORDERLESS: ImageSource<'static> = include_image!("../assets/icons/maximize.svg");
pub const PIN: ImageSource<'static> = include_image!("../assets/icons/pin.svg");

pub fn icon(source: ImageSource<'static>, color: Color32) -> Image<'static> {
    let icon = Image::new(source).tint(color);
    icon.fit_to_exact_size(vec2(16.0, 16.0))
}

pub fn check(ui: &mut Ui, on: &mut bool, text: &str) -> Response {
    let text = WidgetText::from(text).into_galley(ui, None, f32::INFINITY, TextStyle::Body);
    let size = vec2(26.0 + text.size().x, ui.spacing().interact_size.y);
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    let center = pos2(rect.left() + 9.0, rect.center().y);
    let square = Rect::from_center_size(center, Vec2::splat(18.0));
    let radius = CornerRadius::same(5);
    if *on {
        let fill = if response.hovered() { PRESSED } else { ACCENT };
        let tick = [vec2(-4.0, 0.5), vec2(-1.0, 3.5), vec2(4.5, -3.0)];
        let tick = tick.map(|point| center + point).to_vec();
        ui.painter().rect_filled(square, radius, fill);
        ui.painter().add(Shape::line(tick, Stroke::new(2.0, TEXT)));
    } else {
        let line = if response.hovered() { DIM } else { OUTLINE };
        let line = Stroke::new(1.5, line);
        ui.painter()
            .rect_stroke(square, radius, line, StrokeKind::Inside);
    }
    if response.has_focus() {
        let ring = Stroke::new(1.0, LIGHT);
        ui.painter()
            .rect_stroke(square.expand(2.0), radius, ring, StrokeKind::Outside);
    }
    let place = pos2(rect.left() + 26.0, rect.center().y - text.size().y / 2.0);
    ui.painter().galley(place, text, ui.visuals().text_color());
    response
}

// file, how far down its letters go, if a text needs it
type Script = (&'static str, f32, fn(&str) -> bool);

// a cjk font is 20 mb, so it loads when a title needs it and not a damn second sooner
const SCRIPTS: [Script; 4] = [
    ("YuGothM.ttc", DOWN, |text| has(text, kana)),
    ("malgun.ttf", 0.0, |text| has(text, hangul)),
    ("msyh.ttc", 0.0, |text| {
        has(text, han) && !has(text, kana) && !has(text, hangul)
    }),
    ("seguiemj.ttf", DOWN, |text| has(text, emoji)),
];
const DOWN: f32 = 0.2;
const FONT: &str = "segoeui.ttf";

fn has(text: &str, script: fn(char) -> bool) -> bool {
    text.chars().any(script)
}

fn kana(letter: char) -> bool {
    matches!(letter, '\u{3040}'..='\u{30ff}')
}

fn hangul(letter: char) -> bool {
    matches!(letter, '\u{1100}'..='\u{11ff}' | '\u{ac00}'..='\u{d7af}')
}

fn han(letter: char) -> bool {
    matches!(letter, '\u{3400}'..='\u{9fff}')
}

fn emoji(letter: char) -> bool {
    matches!(letter, '\u{2600}'..='\u{27bf}' | '\u{2b00}'..='\u{2bff}' | '\u{1f000}'..='\u{1faff}')
}

fn load(context: &Context, extra: &[&'static str]) {
    let windows = PathBuf::from(env::var_os("WINDIR").unwrap_or_default());
    let mut fonts = FontDefinitions::default();
    let mut found = 0;
    for file in [FONT].into_iter().chain(extra.iter().copied()) {
        let Ok(font) = fs::read(windows.join("Fonts").join(file)) else {
            continue;
        };
        let script = SCRIPTS.iter().find(|script| script.0 == file);
        let tweak = FontTweak {
            y_offset_factor: script.map_or(0.0, |script| script.1),
            ..Default::default()
        };
        let font = Arc::new(FontData::from_owned(font).tweak(tweak));
        fonts.font_data.insert(file.into(), font);
        for family in fonts.families.values_mut() {
            family.insert(found, file.into());
        }
        found += 1;
    }
    // han letters differ per language, so each script gets its font first
    let all = fonts.families[&FontFamily::Proportional].clone();
    for file in extra {
        let first = |name: &String| name != FONT && name != file;
        let mut family: Vec<String> = all.iter().filter(|name| !first(name)).cloned().collect();
        family.extend(all.iter().filter(|name| first(name)).cloned());
        fonts
            .families
            .insert(FontFamily::Name((*file).into()), family);
    }
    context.set_fonts(fonts);
}

// loads what the texts need, drops the rest
pub fn fonts<'a>(
    context: &Context,
    loaded: &mut Vec<&'static str>,
    texts: impl Iterator<Item = &'a str> + Clone,
) {
    let needed = |script: &&Script| texts.clone().any(script.2);
    let needed: Vec<&str> = SCRIPTS
        .iter()
        .filter(needed)
        .map(|script| script.0)
        .collect();
    if needed != *loaded {
        *loaded = needed;
        load(context, loaded);
    }
}

// a family exists one frame after its load, an unknown one panics
pub fn family(context: &Context, text: &str) -> FontFamily {
    let own = SCRIPTS.iter().find(|script| script.2(text));
    let own = own.map(|script| FontFamily::Name(script.0.into()));
    let ready = |family: &FontFamily| {
        context.fonts(|fonts| fonts.definitions().families.contains_key(family))
    };
    own.filter(ready).unwrap_or(FontFamily::Proportional)
}

pub fn apply(context: &Context) {
    load(context, &[]);
    egui_extras::install_image_loaders(context);
    context.set_theme(Theme::Dark);
    let mut visuals = Visuals::dark();
    visuals.override_text_color = Some(TEXT);
    visuals.weak_text_color = Some(DIM);
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = BACKGROUND;
    visuals.extreme_bg_color = SURFACE;
    visuals.selection.bg_fill = PRESSED;
    visuals.selection.stroke = Stroke::new(1.0, LIGHT);
    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_stroke.color = SURFACE;
    widgets.inactive.weak_bg_fill = SURFACE;
    widgets.inactive.bg_fill = OUTLINE;
    widgets.hovered.weak_bg_fill = OUTLINE;
    widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    widgets.active.weak_bg_fill = PRESSED;
    widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    widgets.open.weak_bg_fill = SURFACE;
    for widget in [
        &mut widgets.noninteractive,
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.open,
    ] {
        widget.corner_radius = CornerRadius::same(6);
    }
    context.set_visuals(visuals);
    context.global_style_mut(|style| {
        style.spacing.item_spacing = vec2(8.0, 8.0);
        style.spacing.button_padding = vec2(10.0, 6.0);
        style.spacing.interact_size.y = 28.0;
        // the rows take the clicks, not their text
        style.interaction.selectable_labels = false;
    });
}
