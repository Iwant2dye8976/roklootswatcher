use std::path::Path;
use std::sync::Arc;
use eframe::egui;
use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use crate::model::AppLanguage;
use crate::watcher::RoKMailWatcher;

const EMBEDDED_ICON_BYTES: &[u8] = include_bytes!("../rok_icon.png");

pub fn tray_show_text(lang: AppLanguage) -> &'static str {
    match lang {
        AppLanguage::Vietnamese => "Hiện giao diện chính",
        AppLanguage::English => "Open Main Window",
    }
}

pub fn tray_rescan_text(lang: AppLanguage) -> &'static str {
    match lang {
        AppLanguage::Vietnamese => "Quét lại dữ liệu",
        AppLanguage::English => "Rescan Data",
    }
}

pub fn tray_toggle_text(lang: AppLanguage) -> &'static str {
    match lang {
        AppLanguage::Vietnamese => "Dừng / Tiếp tục theo dõi",
        AppLanguage::English => "Pause / Resume Watching",
    }
}

pub fn tray_quit_text(lang: AppLanguage) -> &'static str {
    match lang {
        AppLanguage::Vietnamese => "Thoát ứng dụng hoàn toàn",
        AppLanguage::English => "Exit Application",
    }
}

pub fn tray_tooltip_text(lang: AppLanguage) -> &'static str {
    match lang {
        AppLanguage::Vietnamese => "Rise of Kingdoms - Loot Watcher (Nhấp để mở lại)",
        AppLanguage::English => "Rise of Kingdoms - Loot Watcher (Click to restore)",
    }
}

pub struct SystemTrayManager {
    _tray_icon: TrayIcon,
    pub show_item: MenuItem,
    pub rescan_item: MenuItem,
    pub toggle_item: MenuItem,
    pub quit_item: MenuItem,
}

impl SystemTrayManager {
    pub fn new(icon_path: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_with_lang(icon_path, AppLanguage::default())
    }

    pub fn new_with_lang(
        icon_path: Option<&Path>,
        lang: AppLanguage,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let icon = if let Some(p) = icon_path {
            Self::load_icon(p)
                .or_else(Self::load_embedded_icon)
                .unwrap_or_else(Self::create_fallback_icon)
        } else {
            Self::load_embedded_icon().unwrap_or_else(Self::create_fallback_icon)
        };

        let menu = Menu::new();
        let show_item = MenuItem::new(tray_show_text(lang), true, None);
        let sep1 = PredefinedMenuItem::separator();
        let rescan_item = MenuItem::new(tray_rescan_text(lang), true, None);
        let toggle_item = MenuItem::new(tray_toggle_text(lang), true, None);
        let sep2 = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new(tray_quit_text(lang), true, None);

        menu.append(&show_item)?;
        menu.append(&sep1)?;
        menu.append(&rescan_item)?;
        menu.append(&toggle_item)?;
        menu.append(&sep2)?;
        menu.append(&quit_item)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip(tray_tooltip_text(lang))
            .with_icon(icon)
            .build()?;

        Ok(Self {
            _tray_icon: tray_icon,
            show_item,
            rescan_item,
            toggle_item,
            quit_item,
        })
    }

    pub fn update_language(&self, lang: AppLanguage) {
        self.show_item.set_text(tray_show_text(lang));
        self.rescan_item.set_text(tray_rescan_text(lang));
        self.toggle_item.set_text(tray_toggle_text(lang));
        self.quit_item.set_text(tray_quit_text(lang));
        let tooltip = match lang {
            AppLanguage::Vietnamese => "Rise of Kingdoms - Loot Watcher (Nhấp để mở lại)",
            AppLanguage::English => "Rise of Kingdoms - Loot Watcher (Click to restore)",
        };
        let _ = self._tray_icon.set_tooltip(Some(tooltip));
    }

    pub fn hide_icon(&self) {
        let _ = self._tray_icon.set_visible(false);
    }

    pub fn setup_event_handlers(
        &self,
        egui_ctx: egui::Context,
        watcher: Arc<RoKMailWatcher>,
    ) {
        let show_id = self.show_item.id().clone();
        let rescan_id = self.rescan_item.id().clone();
        let toggle_id = self.toggle_item.id().clone();
        let quit_id = self.quit_item.id().clone();

        // Tray icon click handler (Left click up or double click)
        let ctx_tray = egui_ctx.clone();
        tray_icon::TrayIconEvent::set_event_handler(Some(move |event: tray_icon::TrayIconEvent| {
            match event {
                tray_icon::TrayIconEvent::Click {
                    button: tray_icon::MouseButton::Left,
                    button_state: tray_icon::MouseButtonState::Up,
                    ..
                }
                | tray_icon::TrayIconEvent::DoubleClick {
                    button: tray_icon::MouseButton::Left,
                    ..
                } => {
                    restore_main_window();
                    ctx_tray.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx_tray.request_repaint();
                }
                _ => {}
            }
        }));

        // Context menu item click handler
        let watcher_tray = Arc::clone(&watcher);
        let ctx_menu = egui_ctx;
        muda::MenuEvent::set_event_handler(Some(move |event: muda::MenuEvent| {
            if event.id == show_id {
                restore_main_window();
                ctx_menu.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx_menu.request_repaint();
            } else if event.id == rescan_id {
                watcher_tray.rescan();
            } else if event.id == toggle_id {
                if watcher_tray.is_watching() {
                    watcher_tray.stop();
                } else {
                    watcher_tray.start(false);
                }
            } else if event.id == quit_id {
                request_quit();
                watcher_tray.stop();

                // Request egui to wake up and exit
                ctx_menu.send_viewport_cmd(egui::ViewportCommand::Close);
                ctx_menu.request_repaint();

                // Post WM_CLOSE message to main window
                if let Some(hwnd) = get_or_find_main_hwnd() {
                    unsafe {
                        windows_sys::Win32::UI::WindowsAndMessaging::PostMessageW(
                            hwnd,
                            windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE,
                            0,
                            0,
                        );
                    }
                }

                // Safety fallback: ensure process terminates cleanly within 200ms
                std::thread::spawn(|| {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    std::process::exit(0);
                });
            }
        }));
    }

    pub fn load_icon(path: &Path) -> Option<Icon> {
        if path.is_file() {
            if let Ok(img) = image::open(path) {
                let rgba = img.into_rgba8();
                let (width, height) = rgba.dimensions();
                return Icon::from_rgba(rgba.into_raw(), width, height).ok();
            }
        }
        None
    }

    pub fn load_embedded_icon() -> Option<Icon> {
        if let Ok(img) = image::load_from_memory(EMBEDDED_ICON_BYTES) {
            let rgba = img.into_rgba8();
            let (width, height) = rgba.dimensions();
            return Icon::from_rgba(rgba.into_raw(), width, height).ok();
        }
        None
    }

    fn create_fallback_icon() -> Icon {
        let width = 32u32;
        let height = 32u32;
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            rgba.push(0xF0); // R
            rgba.push(0xCC); // G
            rgba.push(0x71); // B
            rgba.push(0xFF); // A
        }
        Icon::from_rgba(rgba, width, height)
            .unwrap_or_else(|_| Icon::from_rgba(vec![0xFF; 16], 2, 2).expect("2x2 icon must succeed"))
    }
}

// ---------------------------------------------------------
// Win32 Window Visibility & Restore Helpers
// ---------------------------------------------------------

use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);
static RESTORE_REQUESTED: AtomicBool = AtomicBool::new(false);
static IS_HIDDEN_TO_TRAY: AtomicBool = AtomicBool::new(false);

#[cfg(windows)]
static PREV_RECT: std::sync::Mutex<Option<windows_sys::Win32::Foundation::RECT>> = std::sync::Mutex::new(None);

pub fn is_hidden_to_tray() -> bool {
    IS_HIDDEN_TO_TRAY.load(Ordering::SeqCst)
}

pub fn request_quit() {
    SHOULD_QUIT.store(true, Ordering::SeqCst);
}

pub fn is_quit_requested() -> bool {
    SHOULD_QUIT.load(Ordering::SeqCst)
}

pub fn request_restore() {
    RESTORE_REQUESTED.store(true, Ordering::SeqCst);
}

pub fn check_and_clear_restore_request() -> bool {
    RESTORE_REQUESTED.swap(false, Ordering::SeqCst)
}

#[allow(dead_code)]
pub fn set_main_hwnd(hwnd: isize) {
    MAIN_HWND.store(hwnd, Ordering::SeqCst);
}

pub fn get_or_find_main_hwnd() -> Option<windows_sys::Win32::Foundation::HWND> {
    let stored = MAIN_HWND.load(Ordering::SeqCst);
    if stored != 0 {
        return Some(stored as _);
    }

    // Try finding by exact title
    let title: Vec<u16> = "Rise of Kingdoms - Motte, Fort & Barbarian Loot Watcher"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let hwnd = windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW(
            std::ptr::null(),
            title.as_ptr(),
        );
        if !hwnd.is_null() {
            MAIN_HWND.store(hwnd as isize, Ordering::SeqCst);
            return Some(hwnd);
        }

        // Fallback: enumerate windows for current process
        struct EnumData {
            pid: u32,
            found: windows_sys::Win32::Foundation::HWND,
        }

        unsafe extern "system" fn enum_proc(
            hwnd: windows_sys::Win32::Foundation::HWND,
            lparam: windows_sys::Win32::Foundation::LPARAM,
        ) -> windows_sys::Win32::Foundation::BOOL {
            if lparam == 0 {
                return 1;
            }
            let data = &mut *(lparam as *mut EnumData);
            let mut proc_id = 0u32;
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, &mut proc_id);
            if proc_id == data.pid {
                let mut buf = [0u16; 512];
                let len = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowTextW(
                    hwnd,
                    buf.as_mut_ptr(),
                    buf.len() as i32,
                );
                if len > 0 {
                    let title = String::from_utf16_lossy(&buf[..len as usize]);
                    if title.contains("Rise of Kingdoms") || title.contains("RoK") {
                        data.found = hwnd;
                        return 0; // stop enumeration
                    }
                }
            }
            1 // continue
        }

        let mut data = EnumData {
            pid: std::process::id(),
            found: std::ptr::null_mut(),
        };

        windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(enum_proc),
            &mut data as *mut _ as isize,
        );

        if !data.found.is_null() {
            MAIN_HWND.store(data.found as isize, Ordering::SeqCst);
            Some(data.found)
        } else {
            None
        }
    }
}

pub fn restore_main_window() {
    request_restore();
    IS_HIDDEN_TO_TRAY.store(false, Ordering::SeqCst);
    #[cfg(windows)]
    if let Some(hwnd) = get_or_find_main_hwnd() {
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::*;
            // Restore normal taskbar style
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let normal_style = (ex_style & !(WS_EX_TOOLWINDOW as isize)) | (WS_EX_APPWINDOW as isize);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, normal_style);

            // Restore position
            let saved = if let Ok(lock) = PREV_RECT.lock() {
                *lock
            } else {
                None
            };

            if let Some(r) = saved {
                SetWindowPos(
                    hwnd,
                    0 as _,
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_FRAMECHANGED,
                );
            } else {
                SetWindowPos(
                    hwnd,
                    0 as _,
                    100,
                    100,
                    1380,
                    860,
                    SWP_NOZORDER | SWP_FRAMECHANGED,
                );
            }

            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
            BringWindowToTop(hwnd);
        }
    }
}

pub fn hide_main_window() {
    IS_HIDDEN_TO_TRAY.store(true, Ordering::SeqCst);
    #[cfg(windows)]
    if let Some(hwnd) = get_or_find_main_hwnd() {
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::*;
            let mut rect = windows_sys::Win32::Foundation::RECT { left: 0, top: 0, right: 0, bottom: 0 };
            if GetWindowRect(hwnd, &mut rect) != 0
                && rect.left > -10000 && rect.top > -10000
            {
                if let Ok(mut lock) = PREV_RECT.lock() {
                    *lock = Some(rect);
                }
            }
            // Remove from taskbar by switching to WS_EX_TOOLWINDOW
            let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let tool_style = (ex_style & !(WS_EX_APPWINDOW as isize)) | (WS_EX_TOOLWINDOW as isize);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, tool_style);

            // Move off-screen while keeping WS_VISIBLE so Win32 message loop & overlay updates continue
            SetWindowPos(
                hwnd,
                0 as _,
                -32000,
                -32000,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
            ShowWindow(hwnd, SW_SHOWNA);
        }
    }
    trim_memory_working_set();
}

#[cfg(windows)]
pub fn trim_memory_working_set() {
    unsafe {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> isize;
            fn SetProcessWorkingSetSize(
                hProcess: isize,
                dwMinimumWorkingSetSize: usize,
                dwMaximumWorkingSetSize: usize,
            ) -> i32;
        }
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

#[cfg(not(windows))]
pub fn trim_memory_working_set() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AppLanguage;

    #[test]
    fn test_tray_menu_text_constants() {
        assert_eq!(tray_show_text(AppLanguage::Vietnamese), "Hiện giao diện chính");
        assert_eq!(tray_show_text(AppLanguage::English), "Open Main Window");
        assert_eq!(tray_rescan_text(AppLanguage::Vietnamese), "Quét lại dữ liệu");
        assert_eq!(tray_rescan_text(AppLanguage::English), "Rescan Data");
        assert_eq!(tray_toggle_text(AppLanguage::Vietnamese), "Dừng / Tiếp tục theo dõi");
        assert_eq!(tray_toggle_text(AppLanguage::English), "Pause / Resume Watching");
        assert_eq!(tray_quit_text(AppLanguage::Vietnamese), "Thoát ứng dụng hoàn toàn");
        assert_eq!(tray_quit_text(AppLanguage::English), "Exit Application");
        assert_eq!(tray_tooltip_text(AppLanguage::Vietnamese), "Rise of Kingdoms - Loot Watcher (Nhấp để mở lại)");
        assert_eq!(tray_tooltip_text(AppLanguage::English), "Rise of Kingdoms - Loot Watcher (Click to restore)");
    }

    #[test]
    fn test_tray_manager_update_language() {
        if let Ok(tray) = SystemTrayManager::new(None) {
            tray.update_language(AppLanguage::English);
            assert_eq!(tray.show_item.text(), "Open Main Window");
            assert_eq!(tray.rescan_item.text(), "Rescan Data");
            assert_eq!(tray.toggle_item.text(), "Pause / Resume Watching");
            assert_eq!(tray.quit_item.text(), "Exit Application");

            tray.update_language(AppLanguage::Vietnamese);
            assert_eq!(tray.show_item.text(), "Hiện giao diện chính");
            assert_eq!(tray.rescan_item.text(), "Quét lại dữ liệu");
            assert_eq!(tray.toggle_item.text(), "Dừng / Tiếp tục theo dõi");
            assert_eq!(tray.quit_item.text(), "Thoát ứng dụng hoàn toàn");
        }
    }

    #[test]
    fn test_tray_hidden_state_toggle() {
        assert!(!is_hidden_to_tray());
        hide_main_window();
        assert!(is_hidden_to_tray());
        restore_main_window();
        assert!(!is_hidden_to_tray());
    }
}
