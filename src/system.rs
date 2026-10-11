use std::env;
use std::ffi::{OsString, c_void};
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, EnumDisplayMonitors, GetDC,
    GetDIBits, GetMonitorInfoW, GetObjectW, HDC, HMONITOR, MONITOR_DEFAULTTONEAREST,
    MONITORINFOEXW, MonitorFromWindow, ReleaseDC,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook};
use windows::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_NAMECHANGE, EnumWindows,
    GA_ROOT, GW_OWNER, GWL_EXSTYLE, GWL_STYLE, GetAncestor, GetClientRect, GetIconInfo, GetWindow,
    GetWindowLongPtrW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, ICONINFO,
    IsWindowVisible, OBJID_WINDOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WINEVENT_OUTOFCONTEXT, WS_CAPTION,
    WS_EX_CLIENTEDGE, WS_EX_DLGMODALFRAME, WS_EX_STATICEDGE, WS_EX_TOOLWINDOW, WS_EX_WINDOWEDGE,
    WS_THICKFRAME,
};
use windows::core::{BOOL, HSTRING, PWSTR};
use windows_registry::CURRENT_USER;

#[derive(Clone)]
pub struct Window {
    pub id: isize,
    pub title: String,
    pub path: PathBuf,
    pub display: String,
    pub borderless: bool,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct Display {
    pub name: String,
    pub rect: Rect,
}

pub struct Icon {
    pub size: [usize; 2],
    pub rgba: Vec<u8>,
}

const FRAME: isize = (WS_CAPTION.0 | WS_THICKFRAME.0) as isize;
const EDGES: isize =
    (WS_EX_DLGMODALFRAME.0 | WS_EX_CLIENTEDGE.0 | WS_EX_STATICEDGE.0 | WS_EX_WINDOWEDGE.0) as isize;
const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

static CHANGED: AtomicBool = AtomicBool::new(false);
static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();

pub struct Before {
    style: isize,
    extended: isize,
    rect: Rect,
}

fn handle(window: isize) -> HWND {
    HWND(window as *mut c_void)
}

unsafe extern "system" fn each_window(window: HWND, list: LPARAM) -> BOOL {
    // SAFETY: list points to the vector in windows()
    unsafe { (*(list.0 as *mut Vec<HWND>)).push(window) };
    true.into()
}

unsafe extern "system" fn each_monitor(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    list: LPARAM,
) -> BOOL {
    // SAFETY: list points to the vector in displays()
    unsafe { (*(list.0 as *mut Vec<HMONITOR>)).push(monitor) };
    true.into()
}

pub fn windows() -> Vec<Window> {
    let mut handles: Vec<HWND> = Vec::new();
    let _ = unsafe { EnumWindows(Some(each_window), LPARAM(&raw mut handles as isize)) };
    handles.into_iter().filter_map(window).collect()
}

fn window(handle: HWND) -> Option<Window> {
    let mut cloaked = 0u32;
    let mut title = [0u16; 256];
    let mut process = 0;
    let (length, extended, style, monitor) = unsafe {
        let _ = DwmGetWindowAttribute(handle, DWMWA_CLOAKED, (&raw mut cloaked).cast(), 4);
        let owner = GetWindow(handle, GW_OWNER);
        let dialog = owner.is_ok_and(|owner| IsWindowVisible(owner).as_bool());
        if dialog || !IsWindowVisible(handle).as_bool() {
            return None;
        }
        GetWindowThreadProcessId(handle, Some(&mut process));
        (
            GetWindowTextW(handle, &mut title) as usize,
            GetWindowLongPtrW(handle, GWL_EXSTYLE),
            GetWindowLongPtrW(handle, GWL_STYLE),
            MonitorFromWindow(handle, MONITOR_DEFAULTTONEAREST),
        )
    };
    let title = String::from_utf16_lossy(&title[..length]);
    // cloaked: on another desktop, or a store app in the background
    let hidden = cloaked != 0 || extended & WS_EX_TOOLWINDOW.0 as isize != 0;
    if hidden || title.is_empty() || title == "Program Manager" || process == std::process::id() {
        return None;
    }
    Some(Window {
        id: handle.0 as isize,
        title,
        path: path(process)?,
        display: display(monitor)?.name,
        borderless: style & FRAME == 0,
    })
}

fn path(process: u32) -> Option<PathBuf> {
    let mut name = [0u16; 1024];
    let mut length = name.len() as u32;
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process).ok()?;
        let found = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(name.as_mut_ptr()),
            &mut length,
        );
        let _ = CloseHandle(process);
        found.ok()?;
    }
    Some(OsString::from_wide(&name[..length as usize]).into())
}

pub fn displays() -> Vec<Display> {
    let mut monitors: Vec<HMONITOR> = Vec::new();
    let list = LPARAM(&raw mut monitors as isize);
    let _ = unsafe { EnumDisplayMonitors(None, None, Some(each_monitor), list) };
    let mut displays: Vec<Display> = monitors.into_iter().filter_map(display).collect();
    displays.sort_by(|a, b| (a.name.len(), &a.name).cmp(&(b.name.len(), &b.name)));
    displays
}

fn display(monitor: HMONITOR) -> Option<Display> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    let found = unsafe { GetMonitorInfoW(monitor, (&raw mut info).cast()) };
    let RECT {
        left,
        top,
        right,
        bottom,
    } = info.monitorInfo.rcMonitor;
    let name = info.szDevice.split(|unit| *unit == 0).next()?;
    found.as_bool().then(|| Display {
        name: String::from_utf16_lossy(name),
        rect: Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        },
    })
}

// a console ignores a move that comes with a size over its limit
fn place(window: HWND, rect: Option<Rect>) -> windows::core::Result<()> {
    let flags = SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE;
    unsafe {
        match rect {
            Some(Rect {
                x,
                y,
                width,
                height,
            }) => {
                SetWindowPos(window, None, x, y, 0, 0, flags | SWP_NOSIZE)?;
                SetWindowPos(window, None, x, y, width, height, flags)
            }
            None => SetWindowPos(window, None, 0, 0, 0, 0, flags | SWP_NOMOVE | SWP_NOSIZE),
        }
    }
}

fn outside(window: HWND) -> windows::core::Result<Rect> {
    let mut rect = RECT::default();
    unsafe { GetWindowRect(window, &mut rect)? };
    Ok(Rect {
        x: rect.left,
        y: rect.top,
        width: rect.right - rect.left,
        height: rect.bottom - rect.top,
    })
}

pub fn strip(window: isize, display: Rect, fill: bool) -> Result<(Before, bool)> {
    let window = handle(window);
    let mut drawn = RECT::default();
    let before = unsafe {
        GetClientRect(window, &mut drawn)?;
        let before = Before {
            style: GetWindowLongPtrW(window, GWL_STYLE),
            extended: GetWindowLongPtrW(window, GWL_EXSTYLE),
            rect: outside(window)?,
        };
        SetWindowLongPtrW(window, GWL_STYLE, before.style & !FRAME);
        SetWindowLongPtrW(window, GWL_EXSTYLE, before.extended & !EDGES);
        before
    };
    let target = match fill {
        true => display,
        false => Rect {
            x: display.x + (display.width - drawn.right) / 2,
            y: display.y + (display.height - drawn.bottom) / 2,
            width: drawn.right,
            height: drawn.bottom,
        },
    };
    place(window, Some(target))?;
    let placed = outside(window)?;
    let fits = (placed.width, placed.height) == (target.width, target.height);
    Ok((before, fits))
}

pub fn restore(window: isize, before: Option<Before>) -> Result<()> {
    let window = handle(window);
    unsafe {
        match &before {
            Some(before) => {
                SetWindowLongPtrW(window, GWL_STYLE, before.style);
                SetWindowLongPtrW(window, GWL_EXSTYLE, before.extended);
            }
            None => {
                let style = GetWindowLongPtrW(window, GWL_STYLE);
                SetWindowLongPtrW(window, GWL_STYLE, style | FRAME);
            }
        }
    }
    Ok(place(window, before.map(|before| before.rect))?)
}

pub fn icon(path: &Path) -> Option<Icon> {
    let mut info = SHFILEINFOW::default();
    let mut parts = ICONINFO::default();
    let mut bitmap = BITMAP::default();
    let mut header = BITMAPINFO::default();
    let (mut rgba, lines) = unsafe {
        SHGetFileInfoW(
            &HSTRING::from(path),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        let found = GetIconInfo(info.hIcon, &mut parts);
        let _ = DestroyIcon(info.hIcon);
        found.ok()?;
        GetObjectW(
            parts.hbmColor.into(),
            size_of::<BITMAP>() as i32,
            Some((&raw mut bitmap).cast()),
        );
        header.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: bitmap.bmWidth,
            // negative: top row first
            biHeight: -bitmap.bmHeight,
            biPlanes: 1,
            biBitCount: 32,
            ..Default::default()
        };
        let mut rgba = vec![0u8; (bitmap.bmWidth * bitmap.bmHeight * 4) as usize];
        let screen = GetDC(None);
        let lines = GetDIBits(
            screen,
            parts.hbmColor,
            0,
            bitmap.bmHeight as u32,
            Some(rgba.as_mut_ptr().cast()),
            &mut header,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, screen);
        let _ = DeleteObject(parts.hbmColor.into());
        let _ = DeleteObject(parts.hbmMask.into());
        (rgba, lines)
    };
    if lines == 0 {
        return None;
    }
    // an old icon has no alpha
    let opaque = rgba.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 0);
    for pixel in rgba.as_chunks_mut::<4>().0 {
        // windows gives blue first
        pixel.swap(0, 2);
        if opaque {
            pixel[3] = 255;
        }
    }
    Some(Icon {
        size: [bitmap.bmWidth as usize, bitmap.bmHeight as usize],
        rgba,
    })
}

fn command() -> Option<String> {
    let app = env::current_exe().ok()?;
    Some(format!("\"{}\"", app.display()))
}

pub fn startup() -> bool {
    let run = CURRENT_USER.open(RUN);
    let saved = run.and_then(|run| run.get_string("ihateborders")).ok();
    saved
        .zip(command())
        .is_some_and(|(saved, now)| saved.eq_ignore_ascii_case(&now))
}

pub fn set_startup(enabled: bool) -> Result<()> {
    let run = CURRENT_USER.create(RUN)?;
    match command() {
        Some(command) if enabled => run.set_string("ihateborders", command)?,
        _ => run.remove_value("ihateborders")?,
    }
    Ok(())
}

unsafe extern "system" fn on_event(
    _: HWINEVENTHOOK,
    _: u32,
    window: HWND,
    object: i32,
    child: i32,
    _: u32,
    _: u32,
) {
    // the buttons and the other parts of a window send events too
    let whole = object == OBJID_WINDOW.0 && child == 0;
    if whole && unsafe { GetAncestor(window, GA_ROOT) } == window {
        CHANGED.store(true, Ordering::Relaxed);
        if let Some(wake) = WAKE.get() {
            wake();
        }
    }
}

// windows tells when a window opens, closes or gets a new title
pub fn watch(wake: impl Fn() + Send + Sync + 'static) {
    let _ = WAKE.set(Box::new(wake));
    let events = [
        (EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE),
        (EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_NAMECHANGE),
    ];
    for (first, last) in events {
        unsafe {
            SetWinEventHook(
                first,
                last,
                None,
                Some(on_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };
    }
}

pub fn changed() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}
