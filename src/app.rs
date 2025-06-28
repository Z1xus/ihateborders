use crate::{
    ui::{self, IconCacheInterface},
    window_manager::{DisplayInfo, WindowManager},
};
use eframe::egui;
use std::{
    collections::HashMap,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

struct IconCache
{
    cache: HashMap<String, (egui::TextureHandle, Instant)>,
    max_size: usize,
    ttl: Duration,
}

impl IconCache
{
    fn new() -> Self
    {
        Self { cache: HashMap::new(), max_size: 100, ttl: Duration::from_secs(300) }
    }

    fn cleanup_expired(&mut self)
    {
        let now = Instant::now();
        self.cache.retain(|_, (_, last_used)| now.duration_since(*last_used) < self.ttl);
    }

    fn remove_oldest(&mut self)
    {
        if let Some(oldest_key) = self
            .cache
            .iter()
            .min_by_key(|(_, (_, last_used))| *last_used)
            .map(|(key, _)| key.clone())
        {
            self.cache.remove(&oldest_key);
        }
    }
}

impl IconCacheInterface for IconCache
{
    fn get(&mut self, key: &str) -> Option<&egui::TextureHandle>
    {
        let now = Instant::now();
        if let Some((_, last_used)) = self.cache.get(key) {
            if now.duration_since(*last_used) >= self.ttl {
                self.cache.remove(key);
                return None;
            }
        }

        if let Some((texture, last_used)) = self.cache.get_mut(key) {
            *last_used = now;
            Some(texture)
        } else {
            None
        }
    }

    fn insert(&mut self, key: String, texture: egui::TextureHandle)
    {
        self.cleanup_expired();

        if self.cache.len() >= self.max_size {
            self.remove_oldest();
        }

        self.cache.insert(key, (texture, Instant::now()));
    }

    fn contains_key(&self, key: &str) -> bool
    {
        if let Some((_, last_used)) = self.cache.get(key) {
            last_used.elapsed() < self.ttl
        } else {
            false
        }
    }
}

pub struct BorderlessApp
{
    window_manager: WindowManager,
    selected_window: Option<usize>,
    last_refresh: std::time::Instant,
    icon_cache: IconCache,
    resize_to_screen: bool,
    selected_display: Option<usize>,
    displays: Vec<DisplayInfo>,
    needs_repaint: bool,
    refresh_receiver: Option<Receiver<Vec<crate::window_manager::WindowInfo>>>,
}

impl BorderlessApp
{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self
    {
        ui::setup_dark_theme(&cc.egui_ctx);

        let window_manager = WindowManager::new();
        let displays = window_manager.get_displays();

        let mut app = Self {
            window_manager,
            selected_window: None,
            last_refresh: std::time::Instant::now(),
            icon_cache: IconCache::new(),
            resize_to_screen: false,
            selected_display: if !displays.is_empty() { Some(0) } else { None },
            displays,
            needs_repaint: false,
            refresh_receiver: None,
        };

        app.start_async_refresh();

        app
    }

    fn start_async_refresh(&mut self)
    {
        if self.refresh_receiver.is_none() && !self.window_manager.is_refresh_in_progress() {
            let receiver = self.window_manager.refresh_windows_async();
            self.refresh_receiver = Some(receiver);
        }
    }

    fn handle_refresh(&mut self)
    {
        if let Some(receiver) = &self.refresh_receiver {
            if let Ok(windows) = receiver.try_recv() {
                if !windows.is_empty() {
                    self.window_manager.set_windows(windows);
                    self.last_refresh = std::time::Instant::now();
                    self.needs_repaint = true;

                    if let Some(selected) = self.selected_window {
                        if selected >= self.window_manager.get_windows().len() {
                            self.selected_window = None;
                        }
                    }
                }
                self.refresh_receiver = None;
            }
        }

        let should_refresh = self.last_refresh.elapsed().as_secs() >= 5;
        if should_refresh && self.refresh_receiver.is_none() {
            self.start_async_refresh();
        }
    }

    fn handle_keyboard_input(&mut self, ctx: &egui::Context)
    {
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.refresh_receiver = None;
            self.start_async_refresh();
            self.needs_repaint = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_window = None;
            self.needs_repaint = true;
        }
    }

    fn handle_window_action(&mut self, window_index: usize)
    {
        let windows = self.window_manager.get_windows();
        if let Some(window) = windows.get(window_index) {
            let selected_display = if self.resize_to_screen {
                self.selected_display.and_then(|idx| self.displays.get(idx))
            } else {
                None
            };

            if let Err(e) = self.window_manager.toggle_borderless(
                window.hwnd,
                self.resize_to_screen,
                selected_display,
            ) {
                eprintln!("Failed to toggle borderless for window '{}': {}", window.title, e);
            } else {
                self.refresh_receiver = None;
                self.start_async_refresh();
                self.needs_repaint = true;
            }
        }
    }
}

impl eframe::App for BorderlessApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        self.handle_refresh();
        self.handle_keyboard_input(ctx);

        self.icon_cache.cleanup_expired();

        egui::CentralPanel::default().show(ctx, |ui| {
            let windows = self.window_manager.get_windows();

            ui::render_header(ui, windows.len());

            ui::render_window_selector(
                ui,
                windows,
                &mut self.selected_window,
                &mut self.icon_cache,
            );

            ui::render_position_checkbox(ui, &mut self.resize_to_screen);

            if self.resize_to_screen {
                ui::render_display_selector(ui, &self.displays, &mut self.selected_display);
            }

            if let Some(window_index) = ui::render_action_button(ui, windows, self.selected_window)
            {
                self.handle_window_action(window_index);
            }
        });

        if self.needs_repaint {
            self.needs_repaint = false;
            ctx.request_repaint_after(Duration::from_millis(16));
        } else {
            ctx.request_repaint_after(Duration::from_secs(5));
        }
    }
}

pub fn create_app_options() -> eframe::NativeOptions
{
    let icon_data = load_icon();

    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ihateborders")
            .with_inner_size([350.0, 320.0])
            .with_min_inner_size([350.0, 320.0])
            .with_max_inner_size([350.0, 320.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_icon(icon_data),
        ..Default::default()
    }
}

fn load_icon() -> egui::IconData
{
    let icon_bytes = include_bytes!("../assets/icon.ico");

    let image = image::load_from_memory(icon_bytes).expect("Failed to load icon").into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    egui::IconData { rgba, width: width as u32, height: height as u32 }
}
