use std::sync::{Arc, Mutex};
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC,
        DIB_RGB_COLORS, DeleteDC, DeleteObject, EnumDisplayMonitors, GetDC, GetDIBits,
        GetMonitorInfoW, HBITMAP, HDC, HGDIOBJ, HMONITOR, MONITORINFO, ReleaseDC, SelectObject,
    },
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    },
    UI::WindowsAndMessaging::{
        DrawIconEx, EnumWindows, GCLP_HICON, GWL_STYLE, GetClassLongPtrW, GetSystemMetrics,
        GetWindowLongW, GetWindowTextW, GetWindowThreadProcessId, HWND_TOP, ICON_SMALL,
        IsWindowVisible, SM_CXSCREEN, SM_CYSCREEN, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE,
        SWP_NOZORDER, SWP_NOACTIVATE, SendMessageW, SetWindowLongW, SetWindowPos, WM_GETICON, WS_BORDER,
        WS_CAPTION, WS_DLGFRAME, WS_THICKFRAME,
    },
};

#[derive(Debug, Clone)]
pub struct WindowInfo
{
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
    pub is_borderless: bool,
    pub icon_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct DisplayInfo
{
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub is_primary: bool,
}

impl WindowInfo
{
    pub fn display_text(&self) -> String
    {
        let max_title_len = 30;
        let max_process_len = 15;

        let truncated_title = if self.title.chars().count() > max_title_len {
            let truncated: String = self.title.chars().take(max_title_len - 3).collect();
            format!("{}...", truncated)
        } else {
            self.title.clone()
        };

        let truncated_process = if self.process_name.chars().count() > max_process_len {
            let truncated: String = self.process_name.chars().take(max_process_len - 3).collect();
            format!("{}...", truncated)
        } else {
            self.process_name.clone()
        };

        format!("{} ({})", truncated_title, truncated_process)
    }
}

impl DisplayInfo
{
    pub fn display_text(&self) -> String
    {
        let primary_indicator = if self.is_primary { " (Primary)" } else { "" };
        format!("{} - {}x{}{}", self.name, self.width, self.height, primary_indicator)
    }
}

pub struct WindowManager
{
    windows: Vec<WindowInfo>,
    refresh_in_progress: Arc<Mutex<bool>>,
}

impl WindowManager
{
    pub fn new() -> Self
    {
        Self { windows: Vec::new(), refresh_in_progress: Arc::new(Mutex::new(false)) }
    }

    pub fn refresh_windows_async(&self) -> std::sync::mpsc::Receiver<Vec<WindowInfo>>
    {
        let (sender, receiver) = std::sync::mpsc::channel();
        let refresh_flag = Arc::clone(&self.refresh_in_progress);

        std::thread::spawn(move || {
            {
                let mut in_progress = refresh_flag.lock().unwrap();
                if *in_progress {
                    let _ = sender.send(Vec::new());
                    return;
                }
                *in_progress = true;
            }

            let mut windows = Vec::new();

            unsafe {
                if EnumWindows(
                    Some(enum_windows_proc),
                    LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
                )
                .is_ok()
                {
                    windows.sort_by(|a: &WindowInfo, b: &WindowInfo| a.title.cmp(&b.title));
                }
            }

            *refresh_flag.lock().unwrap() = false;

            let _ = sender.send(windows);
        });

        receiver
    }

    pub fn get_windows(&self) -> &[WindowInfo]
    {
        &self.windows
    }

    pub fn set_windows(&mut self, windows: Vec<WindowInfo>)
    {
        self.windows = windows;
    }

    pub fn get_displays(&self) -> Vec<DisplayInfo>
    {
        let mut displays = Vec::new();

        unsafe {
            let _ = EnumDisplayMonitors(
                Some(HDC::default()),
                None,
                Some(enum_monitors_proc),
                LPARAM(&mut displays as *mut Vec<DisplayInfo> as isize),
            );
        }

        displays.sort_by(|a: &DisplayInfo, b: &DisplayInfo| {
            if a.is_primary && !b.is_primary {
                std::cmp::Ordering::Less
            } else if !a.is_primary && b.is_primary {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        displays
    }

    pub fn toggle_borderless(
        &self,
        hwnd: isize,
        resize_to_screen: bool,
        selected_display: Option<&DisplayInfo>,
    ) -> anyhow::Result<()>
    {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);

        unsafe {
            let current_style = GetWindowLongW(hwnd, GWL_STYLE) as u32;

            let border_styles = WS_BORDER.0 | WS_CAPTION.0 | WS_THICKFRAME.0 | WS_DLGFRAME.0;

            let new_style = if (current_style & border_styles) != 0 {
                current_style & !border_styles
            } else {
                current_style | WS_CAPTION.0 | WS_THICKFRAME.0
            };

            SetWindowLongW(hwnd, GWL_STYLE, new_style as i32);

            if resize_to_screen && (current_style & border_styles) != 0 {
                let (x, y, width, height) = if let Some(display) = selected_display {
                    (display.x, display.y, display.width, display.height)
                } else {
                    let screen_width = GetSystemMetrics(SM_CXSCREEN);
                    let screen_height = GetSystemMetrics(SM_CYSCREEN);
                    (0, 0, screen_width, screen_height)
                };

                SetWindowPos(
                    hwnd,
                    Some(HWND_TOP),
                    x,
                    y,
                    width,
                    height,
                    SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE,
                )?;
            } else {
                SetWindowPos(
                    hwnd,
                    Some(HWND_TOP),
                    0,
                    0,
                    0,
                    0,
                    SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                )?;
            }
        }

        Ok(())
    }

    pub fn get_window_mut(&mut self, index: usize) -> Option<&mut WindowInfo> {
        self.windows.get_mut(index)
    }

    pub fn is_refresh_in_progress(&self) -> bool
    {
        *self.refresh_in_progress.lock().unwrap()
    }
    
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL
{
    unsafe {
        let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }

        let mut title_buffer = [0u16; 256];
        let title_len = GetWindowTextW(hwnd, &mut title_buffer);
        if title_len == 0 {
            return true.into();
        }

        let title = String::from_utf16_lossy(&title_buffer[..title_len as usize]);

        if title.trim().is_empty()
            || title.starts_with("Program Manager")
            || title == "ihateborders"
        {
            return true.into();
        }

        let mut process_id = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let process_name = get_process_name(process_id).unwrap_or_else(|| "Unknown".to_string());

        if process_name.to_lowercase() == "ihateborders" {
            return true.into();
        }

        let current_style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
        let border_styles = WS_BORDER.0 | WS_CAPTION.0 | WS_THICKFRAME.0 | WS_DLGFRAME.0;
        let is_borderless = (current_style & border_styles) == 0;

        let icon_data = extract_window_icon(hwnd);

        windows.push(WindowInfo {
            hwnd: hwnd.0 as isize,
            title,
            process_name,
            is_borderless,
            icon_data,
        });

        true.into()
    }
}

fn get_process_name(process_id: u32) -> Option<String>
{
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                if entry.th32ProcessID == process_id {
                    let name = String::from_utf16_lossy(&entry.szExeFile);
                    let name = name.trim_end_matches('\0');
                    if let Some(pos) = name.rfind('.') {
                        return Some(name[..pos].to_string());
                    }
                    return Some(name.to_string());
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
    }

    None
}

unsafe extern "system" fn enum_monitors_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> windows::core::BOOL
{
    unsafe {
        let displays_ptr = lparam.0 as *mut Vec<DisplayInfo>;
        let displays = &mut *displays_ptr;

        let mut monitor_info =
            MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };

        if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
            let width = monitor_info.rcMonitor.right - monitor_info.rcMonitor.left;
            let height = monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top;
            let is_primary = monitor_info.dwFlags == 1;

            let name = format!("Display {}", displays.len() + 1);

            displays.push(DisplayInfo {
                name,
                x: monitor_info.rcMonitor.left,
                y: monitor_info.rcMonitor.top,
                width,
                height,
                is_primary,
            });
        }

        true.into()
    }
}

struct GdiResources
{
    hdc_screen: HDC,
    hdc_mem: HDC,
    hbitmap: HBITMAP,
    old_bitmap: HGDIOBJ,
}

impl GdiResources
{
    fn new(size: i32) -> Option<Self>
    {
        unsafe {
            let hdc_screen = GetDC(Some(HWND::default()));
            if hdc_screen.is_invalid() {
                return None;
            }

            let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
            if hdc_mem.is_invalid() {
                ReleaseDC(Some(HWND::default()), hdc_screen);
                return None;
            }

            let hbitmap = CreateCompatibleBitmap(hdc_screen, size, size);
            if hbitmap.is_invalid() {
                let _ = DeleteDC(hdc_mem);
                ReleaseDC(Some(HWND::default()), hdc_screen);
                return None;
            }

            let old_bitmap = SelectObject(hdc_mem, hbitmap.into());

            Some(Self { hdc_screen, hdc_mem, hbitmap, old_bitmap })
        }
    }

    fn draw_icon(
        &self,
        icon_handle: windows::Win32::UI::WindowsAndMessaging::HICON,
        size: i32,
    ) -> windows::core::Result<()>
    {
        unsafe {
            DrawIconEx(
                self.hdc_mem,
                0,
                0,
                icon_handle,
                size,
                size,
                0,
                Some(windows::Win32::Graphics::Gdi::HBRUSH::default()),
                windows::Win32::UI::WindowsAndMessaging::DI_NORMAL,
            )
        }
    }

    fn get_bitmap_data(&self, size: i32) -> Option<Vec<u8>>
    {
        unsafe {
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: size,
                    biHeight: -size,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default(); 1],
            };

            let mut rgba_data = vec![0u8; (size * size * 4) as usize];
            let result = GetDIBits(
                self.hdc_mem,
                self.hbitmap,
                0,
                size as u32,
                Some(rgba_data.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            if result == 0 {
                return None;
            }

            for chunk in rgba_data.chunks_exact_mut(4) {
                chunk.swap(0, 2);
            }

            Some(rgba_data)
        }
    }
}

impl Drop for GdiResources
{
    fn drop(&mut self)
    {
        unsafe {
            SelectObject(self.hdc_mem, self.old_bitmap);
            let _ = DeleteObject(self.hbitmap.into());
            let _ = DeleteDC(self.hdc_mem);
            ReleaseDC(Some(HWND::default()), self.hdc_screen);
        }
    }
}

fn extract_window_icon(hwnd: HWND) -> Option<Vec<u8>>
{
    unsafe {
        let icon = SendMessageW(
            hwnd,
            WM_GETICON,
            Some(windows::Win32::Foundation::WPARAM(ICON_SMALL as usize)),
            Some(windows::Win32::Foundation::LPARAM(0)),
        );

        let icon_handle = if icon.0 != 0 {
            windows::Win32::UI::WindowsAndMessaging::HICON(icon.0 as *mut std::ffi::c_void)
        } else {
            let class_icon = GetClassLongPtrW(hwnd, GCLP_HICON);
            if class_icon != 0 {
                windows::Win32::UI::WindowsAndMessaging::HICON(class_icon as *mut std::ffi::c_void)
            } else {
                return None;
            }
        };

        let size = 16;

        let gdi_resources = GdiResources::new(size)?;

        if gdi_resources.draw_icon(icon_handle, size).is_err() {
            return None;
        }

        gdi_resources.get_bitmap_data(size)
    }
}
