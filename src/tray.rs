use std::path::Path;
use std::sync::Arc;
use eframe::egui;
use muda::{Menu, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use crate::watcher::RoKMailWatcher;

const EMBEDDED_ICON_BYTES: &[u8] = include_bytes!("../rok_icon.png");

pub struct SystemTrayManager {
    _tray_icon: TrayIcon,
    pub show_item: MenuItem,
    pub rescan_item: MenuItem,
    pub toggle_item: MenuItem,
    pub quit_item: MenuItem,
}

impl SystemTrayManager {
    pub fn new(icon_path: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let icon = if let Some(p) = icon_path {
            Self::load_icon(p)
                .or_else(Self::load_embedded_icon)
                .unwrap_or_else(Self::create_fallback_icon)
        } else {
            Self::load_embedded_icon().unwrap_or_else(Self::create_fallback_icon)
        };

        let menu = Menu::new();
        let show_item = MenuItem::new("Hiện giao diện chính", true, None);
        let sep1 = PredefinedMenuItem::separator();
        let rescan_item = MenuItem::new("Quét lại dữ liệu", true, None);
        let toggle_item = MenuItem::new("Dừng / Tiếp tục theo dõi", true, None);
        let sep2 = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("Thoát ứng dụng hoàn toàn", true, None);

        menu.append(&show_item)?;
        menu.append(&sep1)?;
        menu.append(&rescan_item)?;
        menu.append(&toggle_item)?;
        menu.append(&sep2)?;
        menu.append(&quit_item)?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Rise of Kingdoms - Loot Watcher (Nhấp để mở lại)")
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
                    ctx_tray.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx_tray.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
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
                ctx_menu.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx_menu.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
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

pub fn request_quit() {
    SHOULD_QUIT.store(true, Ordering::SeqCst);
}

pub fn is_quit_requested() -> bool {
    SHOULD_QUIT.load(Ordering::SeqCst)
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
    if let Some(hwnd) = get_or_find_main_hwnd() {
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::*;
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
            BringWindowToTop(hwnd);
        }
    }
}

pub fn hide_main_window() {
    if let Some(hwnd) = get_or_find_main_hwnd() {
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::*;
            ShowWindow(hwnd, SW_HIDE);
        }
    }
    // Thu hồi Working Set RAM và kích hoạt EcoQoS khi ẩn vào Tray
    trim_memory_working_set();
    enable_eco_qos();
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

#[cfg(windows)]
pub fn enable_eco_qos() {
    unsafe {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> isize;
            fn SetProcessInformation(
                hProcess: isize,
                ProcessInformationClass: i32,
                ProcessInformation: *const std::ffi::c_void,
                ProcessInformationSize: u32,
            ) -> i32;
            fn SetPriorityClass(hProcess: isize, dwPriorityClass: u32) -> i32;
        }

        const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x00004000;
        const PROCESS_POWER_THROTTLING: i32 = 4;
        const PROCESS_POWER_THROTTLING_EXECUTION_SPEED: u32 = 0x1;

        #[allow(non_snake_case)]
        #[repr(C)]
        struct PROCESS_POWER_THROTTLING_STATE {
            Version: u32,
            ControlMask: u32,
            StateMask: u32,
        }

        let _ = SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);

        let throttling = PROCESS_POWER_THROTTLING_STATE {
            Version: 1,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        };

        let _ = SetProcessInformation(
            GetCurrentProcess(),
            PROCESS_POWER_THROTTLING,
            &throttling as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
        );
    }
}

#[cfg(not(windows))]
pub fn enable_eco_qos() {}

