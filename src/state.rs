use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};

use crate::config::{self, Config, Fit, Match, Rule};
use crate::system::{self, Before, Display, Window};

const REFUSED: &str = "Couldn't change this window. Try as administrator";

pub struct State {
    pub config: Config,
    pub windows: Vec<Window>,
    pub displays: Vec<Display>,
    pub selected: Option<isize>,
    pub search: String,
    pub settings: bool,
    pub startup: bool,
    pub error: Option<&'static str>,
    pub notes: HashMap<isize, &'static str>,
    pub fonts: Vec<&'static str>,
    before: HashMap<isize, Before>,
    icons: HashMap<PathBuf, Option<TextureHandle>>,
    polled: Option<Instant>,
}

impl State {
    pub fn new() -> Self {
        let config = config::load();
        Self {
            error: config.is_err().then_some("Couldn't read the settings"),
            notes: HashMap::new(),
            fonts: Vec::new(),
            config: config.unwrap_or_default(),
            windows: Vec::new(),
            displays: Vec::new(),
            selected: None,
            search: String::new(),
            settings: false,
            startup: system::startup(),
            before: HashMap::new(),
            icons: HashMap::new(),
            polled: None,
        }
    }

    // a game can put its frame back and windows sends no event for that
    pub fn wait(&self, shown: bool) -> Option<Duration> {
        match shown {
            true => Some(Duration::from_secs(1)),
            false if !self.config.apps.is_empty() => Some(Duration::from_secs(3)),
            false => None,
        }
    }

    pub fn poll(&mut self, shown: bool) {
        let due = |polled: Instant| {
            let wait = self.wait(shown);
            wait.is_some_and(|wait| polled.elapsed() >= wait)
        };
        if !system::changed() && !self.polled.is_none_or(due) {
            return;
        }
        self.refresh();
        let ruled = |window: &&Window| {
            let tried = self.notes.contains_key(&window.id);
            !window.borderless && !tried && self.rule(window).is_some()
        };
        let found: Vec<Window> = self.windows.iter().filter(ruled).cloned().collect();
        for window in &found {
            self.follow(window);
            self.strip(window);
        }
    }

    // an app that moved keeps its rule, and the rule gets the new place
    fn follow(&mut self, window: &Window) {
        let rule = self.rule(window).map(|index| &mut self.config.apps[index]);
        if let Some(rule) = rule.filter(|rule| rule.path != window.path) {
            rule.path.clone_from(&window.path);
            self.save();
        }
    }

    fn refresh(&mut self) {
        self.polled = Some(Instant::now());
        self.windows = system::windows();
        self.displays = system::displays();
        let open = |id: &isize| self.windows.iter().any(|window| window.id == *id);
        self.before.retain(|id, _| open(id));
        self.notes.retain(|id, _| open(id));
        self.selected = self.selected.filter(open);
    }

    pub fn rule(&self, window: &Window) -> Option<usize> {
        let rules = self.config.apps.iter().enumerate();
        let fits = rules.filter(|(_, rule)| rule.fits(window));
        fits.min_by_key(|(_, rule)| rule.by).map(|(index, _)| index)
    }

    pub fn fit(&mut self, window: &Window) -> &mut Fit {
        match self.rule(window) {
            Some(index) => &mut self.config.apps[index].fit,
            None => &mut self.config.fit,
        }
    }

    pub fn strip(&mut self, window: &Window) {
        let fit = self.fit(window).clone();
        let named = |name: &String| self.displays.iter().find(|display| display.name == *name);
        let display = fit.display.as_ref().and_then(named);
        let Some(display) = display.or_else(|| named(&window.display)) else {
            return;
        };
        match system::strip(window.id, display.rect, fit.resize) {
            Ok((before, fits)) => {
                self.before.entry(window.id).or_insert(before);
                self.note(
                    window.id,
                    (!fits).then_some("This app limits its window size"),
                );
            }
            Err(_) => self.note(window.id, Some(REFUSED)),
        }
        self.refresh();
    }

    pub fn restore(&mut self, window: &Window) {
        let restored = system::restore(window.id, self.before.remove(&window.id));
        self.note(window.id, restored.is_err().then_some(REFUSED));
        self.refresh();
    }

    fn note(&mut self, window: isize, note: Option<&'static str>) {
        match note {
            Some(note) => self.notes.insert(window, note),
            None => self.notes.remove(&window),
        };
    }

    pub fn toggle(&mut self, window: &Window) {
        match window.borderless {
            true if self.rule(window).is_some() => {}
            true => self.restore(window),
            false => self.strip(window),
        }
    }

    pub fn always(&mut self, window: &Window, always: bool) {
        match self.rule(window) {
            Some(index) if !always => drop(self.config.apps.remove(index)),
            None if always => self.config.apps.push(Rule {
                path: window.path.clone(),
                title: window.title.clone(),
                by: Match::Name,
                fit: self.config.fit.clone(),
            }),
            _ => {}
        }
        self.save();
        if always && !window.borderless {
            self.strip(window);
        }
    }

    pub fn save(&mut self) {
        let saved = config::save(&self.config);
        self.error = saved.is_err().then_some("Couldn't save the settings");
    }

    pub fn start(&mut self, startup: bool) {
        match system::set_startup(startup) {
            Ok(()) => self.startup = startup,
            Err(_) => self.error = Some("Couldn't change the startup setting"),
        }
    }

    pub fn icon(&mut self, context: &Context, path: &Path) -> Option<TextureHandle> {
        if let Some(icon) = self.icons.get(path) {
            return icon.clone();
        }
        let icon = system::icon(path).map(|icon| {
            let image = ColorImage::from_rgba_unmultiplied(icon.size, &icon.rgba);
            context.load_texture(path.to_string_lossy(), image, TextureOptions::LINEAR)
        });
        self.icons.insert(path.to_owned(), icon.clone());
        icon
    }
}
