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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayVisibilityAction {
    Show,
    Hide,
}

pub fn determine_overlay_visibility_action(
    overlay_enabled: bool,
    rok_window_found: bool,
    rok_is_active_and_visible: bool,
) -> OverlayVisibilityAction {
    if overlay_enabled && rok_window_found && rok_is_active_and_visible {
        OverlayVisibilityAction::Show
    } else {
        OverlayVisibilityAction::Hide
    }
}

pub fn calculate_overlay_geometry(
    bounds: &RoKWindowBounds,
    config: &crate::model::OverlayConfig,
    scale_factor: f32,
) -> OverlayGeometry {
    let scale = if scale_factor <= 0.0 { 1.0 } else { scale_factor };
    let width = config.width.clamp(180.0, 450.0);
    
    let card_height = (config.font_size * 3.2 + 8.0).max(36.0);
    let est_height = (card_height * 5.0 + 18.0).ceil();
    let max_allowed_h = ((bounds.height as f32 - 20.0) / scale).max(100.0);
    let height = est_height.min(max_allowed_h);

    let min_x = bounds.x as f32 / scale;
    let max_x = ((bounds.x as f32 + bounds.width as f32) / scale - width - 8.0).max(min_x);
    let raw_pos_x = (bounds.x as f32 + config.offset_x.max(0.0)) / scale;
    let pos_x = raw_pos_x.clamp(min_x, max_x);

    let min_y = bounds.y as f32 / scale;
    let max_y = ((bounds.y as f32 + bounds.height as f32) / scale - height - 8.0).max(min_y);
    let raw_pos_y = (bounds.y as f32 + config.offset_y.max(0.0)) / scale;
    let pos_y = raw_pos_y.clamp(min_y, max_y);

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
                return 1;
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
                    return 0;
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

pub fn compute_window_extended_style(current_style: isize) -> isize {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
            WS_EX_TOPMOST, WS_EX_TRANSPARENT,
        };
        current_style
            | WS_EX_TRANSPARENT as isize
            | WS_EX_LAYERED as isize
            | WS_EX_TOOLWINDOW as isize
            | WS_EX_TOPMOST as isize
            | WS_EX_NOACTIVATE as isize
    }
    #[cfg(not(windows))]
    {
        current_style
            | 0x00000020
            | 0x00080000
            | 0x00000080
            | 0x00000008
            | 0x08000000
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayeredWindowParams {
    pub cr_key: u32,
    pub b_alpha: u8,
    pub flags: u32,
}

pub fn compute_layered_window_params(opacity: f32) -> LayeredWindowParams {
    if opacity <= 0.01 {
        LayeredWindowParams {
            cr_key: 0x00000000,
            b_alpha: 255,
            flags: 1,
        }
    } else {
        let alpha = (160.0 + opacity.clamp(0.0, 1.0) * 95.0).round() as u8;
        LayeredWindowParams {
            cr_key: 0x00000000,
            b_alpha: alpha,
            flags: 1 | 2,
        }
    }
}

#[cfg(windows)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn apply_click_through_styles(hwnd: HWND, opacity: f32) {
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos,
            GWL_EXSTYLE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
        };
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let target_style = compute_window_extended_style(ex_style);
        if ex_style != target_style {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, target_style);
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }

        let params = compute_layered_window_params(opacity);
        SetLayeredWindowAttributes(hwnd, params.cr_key, params.b_alpha, params.flags);
    }
}

#[cfg(not(windows))]
pub fn apply_click_through_styles(_hwnd: HWND, _opacity: f32) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_overlay_geometry_clamps_negative_coordinates_to_zero() {
        let bounds = RoKWindowBounds {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let cfg = crate::model::OverlayConfig {
            offset_x: -50.0,
            offset_y: -100.0,
            ..Default::default()
        };
        let geom = calculate_overlay_geometry(&bounds, &cfg, 1.0);
        assert!(
            geom.pos_x >= 0.0,
            "Calculated pos_x {} must not be negative",
            geom.pos_x
        );
        assert!(
            geom.pos_y >= 0.0,
            "Calculated pos_y {} must not be negative",
            geom.pos_y
        );
    }

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
        assert_eq!(geom.pos_x, 116.0);
        assert_eq!(geom.pos_y, 110.0);
        assert_eq!(geom.width, 240.0);
        assert!(geom.height >= 180.0);
    }

    #[test]
    fn test_calculate_overlay_geometry_height_fits_5_reports() {
        let bounds = RoKWindowBounds {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let cfg = crate::model::OverlayConfig::default();
        let geom = calculate_overlay_geometry(&bounds, &cfg, 1.0);
        assert!(
            geom.height >= 220.0,
            "Calculated overlay height {} is too small to fit 5 cards",
            geom.height
        );

        let mut cfg_max = cfg;
        cfg_max.font_size = 15.0;
        let geom_max = calculate_overlay_geometry(&bounds, &cfg_max, 1.0);
        assert!(
            geom_max.height >= 290.0,
            "Max font size height {} is too small to fit 5 cards",
            geom_max.height
        );
    }

    #[test]
    fn test_calculate_overlay_geometry_clamps_to_prevent_bottom_overflow() {
        let bounds = RoKWindowBounds {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let mut cfg = crate::model::OverlayConfig {
            offset_y: 1000.0,
            ..Default::default()
        };
        let geom = calculate_overlay_geometry(&bounds, &cfg, 1.0);
        assert!(
            geom.pos_y + geom.height <= bounds.height as f32,
            "Overlay bottom {} must not overflow game window height {}",
            geom.pos_y + geom.height,
            bounds.height
        );

        cfg.font_size = 15.0;
        let geom_large = calculate_overlay_geometry(&bounds, &cfg, 1.0);
        assert!(
            geom_large.pos_y + geom_large.height <= bounds.height as f32,
            "Large font overlay bottom {} must not overflow game window height {}",
            geom_large.pos_y + geom_large.height,
            bounds.height
        );
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

    #[test]
    fn test_compute_window_extended_style_preserves_hardware_transparency() {
        let style = compute_window_extended_style(0);
        assert_ne!(style & 0x00000020, 0, "Missing WS_EX_TRANSPARENT");
        assert_ne!(style & 0x00080000, 0, "Missing WS_EX_LAYERED");
        assert_ne!(style & 0x00000080, 0, "Missing WS_EX_TOOLWINDOW");
        assert_ne!(style & 0x00000008, 0, "Missing WS_EX_TOPMOST");
        assert_ne!(style & 0x08000000, 0, "Missing WS_EX_NOACTIVATE");
        assert_eq!(style & 0x00200000, 0, "WS_EX_NOREDIRECTIONBITMAP must not be present");
    }

    #[test]
    fn test_determine_overlay_visibility_action_hides_when_rok_not_visible() {
        assert_eq!(
            determine_overlay_visibility_action(true, true, false),
            OverlayVisibilityAction::Hide,
            "Overlay must hide when RoK is not visible"
        );

        assert_eq!(
            determine_overlay_visibility_action(true, false, false),
            OverlayVisibilityAction::Hide,
            "Overlay must hide when RoK is not running"
        );

        assert_eq!(
            determine_overlay_visibility_action(false, true, true),
            OverlayVisibilityAction::Hide,
            "Overlay must hide when disabled"
        );
    }

    #[test]
    fn test_determine_overlay_visibility_action_shows_only_when_rok_active_and_visible() {
        assert_eq!(
            determine_overlay_visibility_action(true, true, true),
            OverlayVisibilityAction::Show,
            "Overlay must show when enabled, RoK is found and RoK is active & visible"
        );
    }

    #[test]
    fn test_compute_layered_window_params_see_through() {
        let params = compute_layered_window_params(0.0);
        assert_eq!(params.cr_key, 0x00000000, "Color key must be pure black");
        assert_eq!(params.b_alpha, 255, "Alpha must be 255 for full brightness text");
        assert_eq!(params.flags, 1, "Must use LWA_COLORKEY");
    }

    #[test]
    fn test_compute_layered_window_params_partial_and_solid() {
        let p_mid = compute_layered_window_params(0.5);
        assert_eq!(p_mid.cr_key, 0x00000000);
        assert_eq!(p_mid.b_alpha, 208);
        assert_eq!(p_mid.flags, 1 | 2, "Must combine LWA_COLORKEY and LWA_ALPHA");

        let p_solid = compute_layered_window_params(1.0);
        assert_eq!(p_solid.cr_key, 0x00000000);
        assert_eq!(p_solid.b_alpha, 255);
        assert_eq!(p_solid.flags, 1 | 2);
    }
}
