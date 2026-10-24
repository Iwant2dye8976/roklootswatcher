#[cfg(windows)]
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
    IsIconic, IsWindowVisible,
};

#[cfg(not(windows))]
pub type HWND = *mut std::ffi::c_void;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoKWindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayGeometry {
    pub pos_x: f32,
    pub pos_y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn calculate_overlay_geometry(
    bounds: &RoKWindowBounds,
    config: &crate::model::OverlayConfig,
    scale_factor: f32,
) -> OverlayGeometry {
    let scale = if scale_factor <= 0.0 { 1.0 } else { scale_factor };
    let pos_x = (bounds.x as f32 + config.offset_x) / scale;
    let pos_y = (bounds.y as f32 + config.offset_y) / scale;
    let width = config.width.clamp(180.0, 450.0);
    
    // Dynamic height calculation based on font size (approx 3.3x font size per card + spacing)
    let card_height = (config.font_size * 3.3).max(28.0);
    let est_height = (card_height * 5.0 + 20.0).max(150.0);
    let max_height = ((bounds.height as f32 - 40.0) / scale).max(150.0);
    let height = est_height.min(max_height);

    OverlayGeometry {
        pos_x,
        pos_y,
        width,
        height,
    }
}

pub fn is_rok_window_title(title: &str) -> bool {
    let t = title.trim();
    if !t.contains("Rise of Kingdoms") {
        return false;
    }
    let lower = t.to_lowercase();
    !(lower.contains("chrome")
        || lower.contains("edge")
        || lower.contains("firefox")
        || lower.contains("loot watcher")
        || lower.contains("explorer")
        || lower.contains("visual studio")
        || lower.contains("code"))
}

pub fn find_rok_window() -> Option<HWND> {
    #[cfg(windows)]
    unsafe {
        struct SearchData {
            own_pid: u32,
            found_hwnd: Option<HWND>,
        }

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let data = &mut *(lparam as *mut SearchData);
            let mut proc_id = 0;
            GetWindowThreadProcessId(hwnd, &mut proc_id);
            if proc_id == data.own_pid {
                return 1; // Skip our own app
            }

            if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
                return 1;
            }

            let mut buf = [0u16; 512];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            if len > 0 {
                let title = String::from_utf16_lossy(&buf[..len as usize]);
                if is_rok_window_title(&title) {
                    data.found_hwnd = Some(hwnd);
                    return 0; // Found target, stop enumeration
                }
            }
            1
        }

        let mut data = SearchData {
            own_pid: std::process::id(),
            found_hwnd: None,
        };

        EnumWindows(Some(enum_proc), &mut data as *mut _ as LPARAM);
        data.found_hwnd
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn is_rok_active_and_visible(hwnd: HWND) -> bool {
    #[cfg(windows)]
    unsafe {
        if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 {
            return false;
        }
        let fg = GetForegroundWindow();
        if fg == hwnd {
            return true;
        }
        let mut rok_pid = 0;
        GetWindowThreadProcessId(hwnd, &mut rok_pid);
        let mut fg_pid = 0;
        GetWindowThreadProcessId(fg, &mut fg_pid);
        if rok_pid != 0 && fg_pid == rok_pid {
            return true;
        }
        let own_pid = std::process::id();
        fg_pid == own_pid
    }
    #[cfg(not(windows))]
    {
        let _ = hwnd;
        false
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn get_rok_window_bounds(hwnd: HWND) -> Option<RoKWindowBounds> {
    #[cfg(windows)]
    unsafe {
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if GetWindowRect(hwnd, &mut rect) != 0 {
            let w = (rect.right - rect.left).max(0) as u32;
            let h = (rect.bottom - rect.top).max(0) as u32;
            Some(RoKWindowBounds {
                x: rect.left,
                y: rect.top,
                width: w,
                height: h,
            })
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        let _ = hwnd;
        None
    }
}

#[cfg(windows)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn apply_click_through_styles(hwnd: HWND) {
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE,
            WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
        };
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let target_style = ex_style
            | WS_EX_TRANSPARENT as isize
            | WS_EX_TOOLWINDOW as isize
            | WS_EX_TOPMOST as isize
            | WS_EX_NOACTIVATE as isize;
        if ex_style != target_style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, target_style);
        }
    }
}

#[cfg(not(windows))]
pub fn apply_click_through_styles(_hwnd: HWND) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_overlay_geometry_with_custom_offsets() {
        let bounds = RoKWindowBounds {
            x: 100,
            y: 100,
            width: 1920,
            height: 1080,
        };
        let cfg = crate::model::OverlayConfig {
            enabled: true,
            opacity: 0.8,
            text_color_hex: "#FFFFFF".into(),
            accent_color_hex: "#38BDF8".into(),
            filter: crate::model::FilterMode::AllRewards,
            offset_x: 250.0,
            offset_y: 150.0,
            width: 300.0,
            font_size: 12.0,
        };
        let geom = calculate_overlay_geometry(&bounds, &cfg, 1.0);
        assert_eq!(geom.pos_x, 350.0);
        assert_eq!(geom.pos_y, 250.0);
        assert_eq!(geom.width, 300.0);
        assert!(geom.height >= 180.0);
    }

    #[test]
    fn test_calculate_overlay_geometry() {
        let bounds = RoKWindowBounds {
            x: 100,
            y: 50,
            width: 1920,
            height: 1080,
        };
        let cfg = crate::model::OverlayConfig::default();
        let geom = calculate_overlay_geometry(&bounds, &cfg, 1.0);
        assert_eq!(geom.pos_x, 116.0); // 100 + 16 margin
        assert_eq!(geom.pos_y, 110.0); // 50 + 60 margin
        assert_eq!(geom.width, 240.0);
        assert!(geom.height >= 180.0);
    }

    #[test]
    fn test_calculate_overlay_geometry_small_window() {
        let bounds = RoKWindowBounds {
            x: 0,
            y: 0,
            width: 800,
            height: 500,
        };
        let cfg = crate::model::OverlayConfig::default();
        let geom = calculate_overlay_geometry(&bounds, &cfg, 1.0);
        assert_eq!(geom.pos_x, 16.0);
        assert_eq!(geom.pos_y, 60.0);
        assert_eq!(geom.width, 240.0);
        assert!(geom.height >= 180.0);
    }

    #[test]
    fn test_is_rok_window_title() {
        assert!(is_rok_window_title("Rise of Kingdoms"));
        assert!(is_rok_window_title("Rise of Kingdoms Game"));
        assert!(!is_rok_window_title("Rise of Kingdoms Guide - Google Chrome"));
        assert!(!is_rok_window_title("Rise of Kingdoms - Motte, Fort & Barbarian Loot Watcher"));
        assert!(!is_rok_window_title("C:\\Program Files\\Rise of Kingdoms - File Explorer"));
        assert!(!is_rok_window_title("Other Game"));
    }
}
