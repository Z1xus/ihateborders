use std::sync::mpsc::Sender;

use anyhow::Result;
use image::RgbaImage;
use image::imageops::{FilterType, resize};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::app::Asked;

pub fn tray(
    icon: &RgbaImage,
    asked: Sender<Asked>,
    wake: impl Fn() + Clone + Send + Sync + 'static,
) -> Result<TrayIcon> {
    let icon = resize(icon, 32, 32, FilterType::Lanczos3);
    let open = MenuItem::new("Open", true, None);
    let quit = MenuItem::new("Quit", true, None);
    let icon = TrayIconBuilder::new()
        .with_icon(Icon::from_rgba(icon.into_raw(), 32, 32)?)
        .with_tooltip("ihateborders")
        .with_menu(Box::new(Menu::with_items(&[&open, &quit])?))
        .with_menu_on_left_click(false)
        .build()?;
    let (clicks, woken) = (asked.clone(), wake.clone());
    TrayIconEvent::set_event_handler(Some(move |event| {
        let left = (MouseButton::Left, MouseButtonState::Up);
        if matches!(event, TrayIconEvent::Click { button, button_state, .. } if (button, button_state) == left)
        {
            let _ = clicks.send(Asked::Open);
            woken();
        }
    }));
    let quit = quit.id().clone();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let _ = asked.send(if event.id == quit {
            Asked::Quit
        } else {
            Asked::Open
        });
        wake();
    }));
    Ok(icon)
}
