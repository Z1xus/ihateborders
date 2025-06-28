use crate::window_manager::WindowInfo;
use egui::{
    Align, Align2, Color32, ColorImage, FontId, Layout, RichText, Sense, Stroke, Style, Visuals,
};

pub trait IconCacheInterface
{
    fn get(&mut self, key: &str) -> Option<&egui::TextureHandle>;
    fn insert(&mut self, key: String, texture: egui::TextureHandle);
    fn contains_key(&self, key: &str) -> bool;
}

pub fn setup_dark_theme(ctx: &egui::Context)
{
    let mut style = Style::default();

    style.visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(Color32::from_rgb(230, 224, 233)),
        widgets: egui::style::Widgets {
            noninteractive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(28, 27, 31),
                weak_bg_fill: Color32::from_rgb(16, 16, 20),
                bg_stroke: Stroke::new(1.0, Color32::from_rgb(73, 69, 79)),
                fg_stroke: Stroke::new(1.0, Color32::from_rgb(202, 196, 208)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 0.0,
            },
            inactive: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(73, 69, 79),
                weak_bg_fill: Color32::from_rgb(28, 27, 31),
                bg_stroke: Stroke::new(1.0, Color32::from_rgb(147, 143, 153)),
                fg_stroke: Stroke::new(1.0, Color32::from_rgb(202, 196, 208)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 0.0,
            },
            hovered: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(103, 80, 164),
                weak_bg_fill: Color32::from_rgb(73, 69, 79),
                bg_stroke: Stroke::new(1.0, Color32::from_rgb(103, 80, 164)),
                fg_stroke: Stroke::new(1.0, Color32::from_rgb(230, 224, 233)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 1.0,
            },
            active: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(79, 55, 139),
                weak_bg_fill: Color32::from_rgb(73, 69, 79),
                bg_stroke: Stroke::new(1.0, Color32::from_rgb(79, 55, 139)),
                fg_stroke: Stroke::new(1.0, Color32::from_rgb(230, 224, 233)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 1.0,
            },
            open: egui::style::WidgetVisuals {
                bg_fill: Color32::from_rgb(73, 69, 79),
                weak_bg_fill: Color32::from_rgb(28, 27, 31),
                bg_stroke: Stroke::new(1.0, Color32::from_rgb(147, 143, 153)),
                fg_stroke: Stroke::new(1.0, Color32::from_rgb(202, 196, 208)),
                corner_radius: egui::CornerRadius::same(4),
                expansion: 0.0,
            },
        },
        selection: egui::style::Selection {
            bg_fill: Color32::from_rgb(79, 55, 139),
            stroke: Stroke::new(1.0, Color32::from_rgb(103, 80, 164)),
        },
        window_fill: Color32::from_rgb(16, 16, 20),
        panel_fill: Color32::from_rgb(16, 16, 20),
        ..Default::default()
    };

    ctx.set_style(style);
}

pub fn render_header(ui: &mut egui::Ui, window_count: usize)
{
    ui.add_space(10.0);

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.label(
            RichText::new("ihateborders")
                .font(FontId::proportional(20.0))
                .color(Color32::from_gray(240)),
        );

        ui.add_space(15.0);

        let counter_text = if window_count == 1 {
            format!("{} window found", window_count)
        } else {
            format!("{} windows found", window_count)
        };

        ui.label(
            RichText::new(counter_text)
                .font(FontId::proportional(12.0))
                .color(Color32::from_gray(180)),
        );
    });

    ui.add_space(10.0);
}

pub fn render_window_selector(
    ui: &mut egui::Ui,
    windows: &[WindowInfo],
    selected_window: &mut Option<usize>,
    icon_cache: &mut dyn IconCacheInterface,
)
{
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Select Window:")
                .font(FontId::proportional(13.0))
                .color(Color32::from_gray(200)),
        );
    });

    ui.add_space(5.0);

    let selected_text = if let Some(index) = selected_window {
        if let Some(window) = windows.get(*index) {
            window.display_text()
        } else {
            "Select a window...".to_string()
        }
    } else {
        "Select a window...".to_string()
    };

    egui::ComboBox::from_id_salt("window_selector")
        .selected_text(selected_text)
        .width(ui.available_width())
        .height(150.0)
        .show_ui(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

            for (index, window) in windows.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.set_min_width(ui.available_width());
                    if let Some(icon_data) = &window.icon_data {
                        let cache_key = format!("icon_{}", window.hwnd);

                        if !icon_cache.contains_key(&cache_key) {
                            let color_image =
                                ColorImage::from_rgba_unmultiplied([16, 16], icon_data);
                            let texture = ui.ctx().load_texture(
                                &cache_key,
                                color_image,
                                egui::TextureOptions::LINEAR,
                            );
                            icon_cache.insert(cache_key.clone(), texture);
                        }

                        if let Some(texture) = icon_cache.get(&cache_key) {
                            ui.image((texture.id(), egui::vec2(16.0, 16.0)));
                        }
                    } else {
                        let (rect, _response) =
                            ui.allocate_exact_size(egui::vec2(16.0, 16.0), Sense::hover());

                        let status_icon = "○";
                        let status_color = if window.is_borderless {
                            Color32::from_rgb(100, 200, 100)
                        } else {
                            Color32::from_rgb(180, 180, 180)
                        };

                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            status_icon,
                            FontId::proportional(12.0),
                            status_color,
                        );
                    }

                    let status_text = if window.is_borderless { "[B]" } else { "[W]" };
                    let status_color = if window.is_borderless {
                        Color32::from_rgb(100, 200, 100)
                    } else {
                        Color32::from_rgb(150, 150, 150)
                    };

                    ui.label(
                        RichText::new(status_text)
                            .color(status_color)
                            .font(FontId::proportional(10.0)),
                    );

                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            let response = ui.selectable_label(
                                *selected_window == Some(index),
                                window.display_text(),
                            );
                            if response.clicked() {
                                *selected_window = Some(index);
                            }
                        },
                    );
                });
            }
        });
}

pub fn render_position_checkbox(ui: &mut egui::Ui, resize_to_screen: &mut bool)
{
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.add_space(5.0);

        let checkbox = egui::Checkbox::new(resize_to_screen, "");

        ui.add(checkbox);

        ui.label(
            RichText::new("Resize to screen")
                .font(FontId::proportional(12.0))
                .color(Color32::from_gray(180)),
        );
    });
}

pub fn render_action_button(
    ui: &mut egui::Ui,
    windows: &[WindowInfo],
    selected_window: Option<usize>,
) -> Option<usize>
{
    ui.add_space(15.0);

    let mut clicked_window = None;

    let button_enabled = selected_window.is_some();
    let button_text = if let Some(index) = selected_window {
        if let Some(window) = windows.get(index) {
            if window.is_borderless { "Restore Borders" } else { "Make Borderless" }
        } else {
            "Make Borderless"
        }
    } else {
        "Make Borderless"
    };

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_enabled_ui(button_enabled, |ui| {
            let button = egui::Button::new(
                RichText::new(button_text).font(FontId::proportional(14.0)).color(
                    if button_enabled { Color32::from_gray(255) } else { Color32::from_gray(120) },
                ),
            )
            .min_size(egui::vec2(180.0, 35.0));

            if ui.add(button).clicked() && button_enabled {
                clicked_window = selected_window;
            }
        });
    });

    clicked_window
}
