use global_hotkey::GlobalHotKeyManager;
use global_hotkey::hotkey::HotKey;

pub struct Shortcut {
    manager: GlobalHotKeyManager,
    keys: Option<HotKey>,
}

impl Shortcut {
    pub fn new(keys: &str) -> Option<Self> {
        let manager = GlobalHotKeyManager::new().ok()?;
        let mut shortcut = Self {
            manager,
            keys: None,
        };
        shortcut.set(keys);
        Some(shortcut)
    }

    // empty keys turn the shortcut off
    pub fn set(&mut self, keys: &str) -> bool {
        if let Some(keys) = self.keys.take() {
            let _ = self.manager.unregister(keys);
        }
        let Ok(keys) = keys.parse::<HotKey>() else {
            return keys.is_empty();
        };
        self.keys = self.manager.register(keys).is_ok().then_some(keys);
        self.keys.is_some()
    }
}
