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

pub fn calculate_overlay_geometry(bounds: &RoKWindowBounds, scale_factor: f32) -> OverlayGeometry {
    let scale = if scale_factor <= 0.0 { 1.0 } else { scale_factor };
    let pos_x = (bounds.x as f32 + 16.0) / scale;
    let pos_y = (bounds.y as f32 + 60.0) / scale;
    let width = 360.0;
    let max_height = ((bounds.height as f32 - 80.0) / scale).max(200.0);
    let height = 680.0_f32.min(max_height);
    OverlayGeometry {
        pos_x,
        pos_y,
        width,
        height,
    }
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
                if title.contains("Rise of Kingdoms") {
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
        fg == hwnd
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
            WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
        };
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let target_style = ex_style
            | WS_EX_TRANSPARENT as isize
            | WS_EX_LAYERED as isize
            | WS_EX_TOOLWINDOW as isize
            | WS_EX_TOPMOST as isize;
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
    fn test_calculate_overlay_geometry() {
        let bounds = RoKWindowBounds {
            x: 100,
            y: 50,
            width: 1920,
            height: 1080,
        };
        let geom = calculate_overlay_geometry(&bounds, 1.0);
        assert_eq!(geom.pos_x, 116.0); // 100 + 16 margin
        assert_eq!(geom.pos_y, 110.0); // 50 + 60 margin
        assert_eq!(geom.width, 360.0);
        assert_eq!(geom.height, 680.0); // min(680, 1080 - 80)
    }

    #[test]
    fn test_calculate_overlay_geometry_small_window() {
        let bounds = RoKWindowBounds {
            x: 0,
            y: 0,
            width: 800,
            height: 500,
        };
        let geom = calculate_overlay_geometry(&bounds, 1.0);
        assert_eq!(geom.pos_x, 16.0);
        assert_eq!(geom.pos_y, 60.0);
        assert_eq!(geom.width, 360.0);
        assert_eq!(geom.height, 420.0); // 500 - 80
    }
}
