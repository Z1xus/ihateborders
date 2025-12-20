use crate::{
    config::Config,
    startup::{is_elevated, relaunch_elevated, remove_scheduled_task, task_exists},
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
    last_refresh: Instant,
    icon_cache: IconCache,
    resize_to_screen: bool,
    selected_display: Option<usize>,
    displays: Vec<DisplayInfo>,
    needs_repaint: bool,
    refresh_receiver: Option<Receiver<Vec<crate::window_manager::WindowInfo>>>,
    config: Config,
    auto_borderless_enabled: bool,
    show_settings: bool,
    applied_auto_borderless: std::collections::HashSet<isize>,
}

impl BorderlessApp
{
    pub fn new(cc: &eframe::CreationContext<'_>, open_settings: bool) -> Self
    {
        ui::setup_dark_theme(&cc.egui_ctx);

        let window_manager = WindowManager::new();
        let displays = window_manager.get_displays();
        
        let config = Config::load().unwrap_or_default();

        let mut app = Self {
            window_manager,
            selected_window: None,
            last_refresh: Instant::now(),
            icon_cache: IconCache::new(),
            resize_to_screen: true,
            selected_display: if !displays.is_empty() { Some(0) } else { None },
            displays,
            needs_repaint: false,
            refresh_receiver: None,
            config,
            auto_borderless_enabled: false,
            show_settings: open_settings,
            applied_auto_borderless: std::collections::HashSet::new(),
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
                    self.last_refresh = Instant::now();
                    self.needs_repaint = true;

                    if let Some(selected) = self.selected_window {
                        if selected >= self.window_manager.get_windows().len() {
                            self.selected_window = None;
                        }
                    }
                    self.apply_auto_borderless();
                }
                self.refresh_receiver = None;
            }
        }

        let should_refresh = self.last_refresh.elapsed().as_secs() >= 5;
        if should_refresh && self.refresh_receiver.is_none() {
            self.start_async_refresh();
        }
    }
    
    fn apply_auto_borderless(&mut self)
    {
        let windows = self.window_manager.get_windows().to_vec();
        
        for window in windows.iter() {
            if self.config.is_auto_borderless(&window.process_name) 
                && !window.is_borderless 
                && !self.applied_auto_borderless.contains(&window.hwnd)
            {
                let selected_display = if self.resize_to_screen {
                    self.selected_display.and_then(|idx| self.displays.get(idx))
                } else {
                    None
                };
                
                if let Ok(_) = self.window_manager.toggle_borderless(
                    window.hwnd,
                    self.resize_to_screen,
                    selected_display,
                ) {
                    self.applied_auto_borderless.insert(window.hwnd);
                }
            }
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
            if self.show_settings {
                self.show_settings = false;
            } else {
                self.selected_window = None;
            }
            self.needs_repaint = true;
        }
    }

    fn handle_window_action(&mut self, window_index: usize)
    {
        let hwnd = self.window_manager.get_windows()[window_index].hwnd;
        let selected_display = if self.resize_to_screen {
            self.selected_display.and_then(|idx| self.displays.get(idx))
        } else {
            None
        };

        if let Err(e) = self.window_manager.toggle_borderless(
            hwnd,
            self.resize_to_screen,
            selected_display,
        ) {
            eprintln!("Failed to toggle borderless for window: {}", e);
        } else {
            if let Some(window) = self.window_manager.get_window_mut(window_index) {
                window.is_borderless = !window.is_borderless;
            }
            self.refresh_receiver = None;
            self.start_async_refresh();
            self.needs_repaint = true;
        }
    }
    
    fn save_config(&self)
    {
        if let Err(e) = self.config.save() {
            eprintln!("Failed to save config: {}", e);
        }
    }
    
    fn handle_settings_window(&mut self, ctx: &egui::Context)
    {
        if !self.show_settings {
            return;
        }
        
        egui::Window::new("Settings")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_min_width(300.0);
                let mut run_on_startup = self.config.run_on_startup;
                if ui.checkbox(&mut run_on_startup, "Run ihateborders when my computer starts").changed() {
                    if run_on_startup {
                        if !is_elevated() {
                            self.config.run_on_startup = true;
                            self.save_config();
                            if let Err(e) = relaunch_elevated(&["--create-startup-task"]) {
                                eprintln!("Failed to relaunch elevated: {}", e);
                                self.config.run_on_startup = false;
                                self.save_config();
                            }
                        } else {
                            match crate::startup::create_scheduled_task(false) {
                                Ok(_) => {
                                    self.config.run_on_startup = true;
                                    self.save_config();
                                }
                                Err(e) => {
                                    eprintln!("Failed to create startup task: {}", e);
                                }
                            }
                        }
                    } else {
                        self.config.run_on_startup = false;
                        self.config.startup_admin = false;
                        if task_exists() {
                            let _ = remove_scheduled_task();
                        }
                        self.save_config();
                    }
                }
                ui.add_enabled_ui(self.config.run_on_startup, |ui| {
                    let mut startup_admin = self.config.startup_admin;
                    
                    if ui.checkbox(&mut startup_admin, "Enable startup with administrator privileges").changed() {
                        if startup_admin {
                            if !is_elevated() {
                                self.config.startup_admin = true;
                                self.save_config();
                                if let Err(e) = relaunch_elevated(&["--install-admin-task"]) {
                                    eprintln!("Failed to relaunch elevated: {}", e);
                                    self.config.startup_admin = false;
                                    self.save_config();
                                }
                            } else {
                                match crate::startup::create_scheduled_task(true) {
                                    Ok(_) => {
                                        self.config.startup_admin = true;
                                        self.save_config();
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to create scheduled task: {}", e);
                                    }
                                }
                            }
                        } else {
                            if !is_elevated() {
                                self.config.startup_admin = false;
                                self.save_config();
                                if let Err(e) = relaunch_elevated(&["--create-startup-task"]) {
                                    eprintln!("Failed to relaunch elevated: {}", e);
                                }
                            } else {
                                if task_exists() {
                                    let _ = remove_scheduled_task();
                                }
                                
                                match crate::startup::create_scheduled_task(false) {
                                    Ok(_) => {
                                        self.config.startup_admin = false;
                                        self.save_config();
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to recreate startup task: {}", e);
                                    }
                                }
                            }
                        }
                    }
                });
                
                ui.add_space(10.0);
                
                if ui.button("Close").clicked() {
                    self.show_settings = false;
                }
            });
    }
}

impl eframe::App for BorderlessApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        if let Some(pos) = ctx.input(|i| i.viewport().outer_rect).map(|r| r.min) {
            if self.config.window_position.as_ref().map_or(true, |saved_pos| {
                (saved_pos.x - pos.x).abs() > 1.0 || (saved_pos.y - pos.y).abs() > 1.0
            }) {
                self.config.window_position = Some(crate::config::WindowPosition {
                    x: pos.x,
                    y: pos.y,
                });
            }
        }
        
        self.handle_refresh();
        self.handle_keyboard_input(ctx);

        self.icon_cache.cleanup_expired();

    egui::CentralPanel::default().show(ctx, |ui| {
        
        ui::render_header(ui, self.window_manager.get_windows().len());

        ui::render_window_selector(
            ui,
            self.window_manager.get_windows(),
            &mut self.selected_window,
            &mut self.icon_cache,
        );

        ui::render_position_checkbox(ui, &mut self.resize_to_screen);

        if self.resize_to_screen {
            ui::render_display_selector(ui, &self.displays, &mut self.selected_display);
        }

        let mut auto_changed = false;
        let checkbox_enabled = self.selected_window.is_some();
        
        ui::render_auto_borderless_checkbox(
            ui, 
            &mut self.auto_borderless_enabled,
            &mut auto_changed,
            checkbox_enabled,
        );

        if auto_changed {
            if let Some(index) = self.selected_window {
                let current_windows = self.window_manager.get_windows();
                let window = &current_windows[index];
                let process_name = window.process_name.clone();
                let hwnd = window.hwnd;
                let currently_borderless = window.is_borderless;

                let should_toggle = (self.auto_borderless_enabled && !currently_borderless)
                    || (!self.auto_borderless_enabled && currently_borderless);

                if should_toggle {
                    let selected_display = if self.resize_to_screen {
                        self.selected_display.and_then(|idx| self.displays.get(idx))
                    } else {
                        None
                    };

                    if self.window_manager
                        .toggle_borderless(hwnd, self.resize_to_screen, selected_display)
                        .is_ok()
                    {
                        if let Some(w) = self.window_manager.get_window_mut(index) {
                            w.is_borderless = !w.is_borderless;
                        }
                        self.needs_repaint = true;
                        self.refresh_receiver = None;
                        self.start_async_refresh();
                    }
                }

                if self.auto_borderless_enabled {
                    self.config.add_auto_borderless(process_name);
                    if !currently_borderless && should_toggle {
                        self.applied_auto_borderless.insert(hwnd);
                    }
                } else {
                    self.config.remove_auto_borderless(&process_name);
                    if currently_borderless && should_toggle {
                        self.applied_auto_borderless.remove(&hwnd);
                    }
                }
                self.save_config();
            }
        }

        if let Some(index) = self.selected_window {
            let windows = self.window_manager.get_windows();
            if let Some(window) = windows.get(index) {
                self.auto_borderless_enabled = self.config.is_auto_borderless(&window.process_name);
            }
        }

        let action_button_enabled = self.selected_window.is_some() && !self.auto_borderless_enabled;
        
        let clicked_index = ui::render_action_button(
            ui, 
            self.window_manager.get_windows(),
            self.selected_window,
            action_button_enabled,
        );

        if let Some(window_index) = clicked_index {
            self.handle_window_action(window_index);
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(5.0);
            if ui.button("⚙ Settings").clicked() {
                self.show_settings = !self.show_settings;
            }
        });
    });

        self.handle_settings_window(ctx);

        if self.needs_repaint {
            self.needs_repaint = false;
            ctx.request_repaint_after(Duration::from_millis(16));
        } else {
            ctx.request_repaint_after(Duration::from_secs(5));
        }
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>)
    {
        let _ = self.config.save();
    }
}

pub fn create_app_options() -> eframe::NativeOptions
{
    let icon_data = load_icon();
    let config = Config::load().unwrap_or_default();
    let initial_position = config.window_position.map(|pos| egui::Pos2::new(pos.x, pos.y));

    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ihateborders")
            .with_inner_size([350.0, 360.0])
            .with_min_inner_size([350.0, 360.0])
            .with_max_inner_size([350.0, 360.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_position(initial_position.unwrap_or(egui::Pos2::new(100.0, 100.0)))
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

    egui::IconData { rgba, width, height }
}