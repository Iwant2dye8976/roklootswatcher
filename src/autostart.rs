#[cfg(windows)]
pub mod windows {
    const HKEY_CURRENT_USER: isize = 0x80000001u32 as isize;
    const KEY_SET_VALUE: u32 = 0x0002;
    const KEY_QUERY_VALUE: u32 = 0x0001;
    const REG_SZ: u32 = 1;

    pub const DEFAULT_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: isize,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut isize,
        ) -> i32;

        fn RegCreateKeyExW(
            hKey: isize,
            lpSubKey: *const u16,
            Reserved: u32,
            lpClass: *const u16,
            dwOptions: u32,
            samDesired: u32,
            lpSecurityAttributes: *const std::ffi::c_void,
            phkResult: *mut isize,
            lpdwDisposition: *mut u32,
        ) -> i32;

        fn RegSetValueExW(
            hKey: isize,
            lpValueName: *const u16,
            Reserved: u32,
            dwType: u32,
            lpData: *const u8,
            cbData: u32,
        ) -> i32;

        fn RegDeleteValueW(
            hKey: isize,
            lpValueName: *const u16,
        ) -> i32;

        #[allow(dead_code)]
        fn RegDeleteKeyW(
            hKey: isize,
            lpSubKey: *const u16,
        ) -> i32;

        fn RegQueryValueExW(
            hKey: isize,
            lpValueName: *const u16,
            lpReserved: *mut u32,
            lpType: *mut u32,
            lpData: *mut u8,
            lpcbData: *mut u32,
        ) -> i32;

        fn RegCloseKey(hKey: isize) -> i32;
    }

    fn to_wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Lấy đường dẫn file exe chuẩn xác của ứng dụng (loại bỏ tiền tố `\\?\` nếu có)
    pub fn get_clean_exe_path() -> Result<String, String> {
        let exe = std::env::current_exe().map_err(|e| format!("Không lấy được đường dẫn exe: {}", e))?;
        let s = exe.to_string_lossy().to_string();
        let clean = s.strip_prefix(r"\\?\").unwrap_or(&s);
        Ok(clean.to_string())
    }

    /// Lấy giá trị chuỗi lệnh thô của Autostart trong Windows Registry theo subkey
    pub fn get_raw_autostart_command_with_key(subkey: &str) -> Option<String> {
        unsafe {
            let subkey_wide = to_wide_null(subkey);
            let mut hkey: isize = 0;
            let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey_wide.as_ptr(), 0, KEY_QUERY_VALUE, &mut hkey);
            if status != 0 || hkey == 0 {
                return None;
            }

            let val_name = to_wide_null("RoKLootWatcher");
            let mut val_type = 0u32;
            let mut data_len = 0u32;

            let q_status = RegQueryValueExW(
                hkey,
                val_name.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                std::ptr::null_mut(),
                &mut data_len,
            );

            if q_status == 0 && data_len > 0 {
                let mut buf = vec![0u8; data_len as usize];
                let q_status2 = RegQueryValueExW(
                    hkey,
                    val_name.as_ptr(),
                    std::ptr::null_mut(),
                    &mut val_type,
                    buf.as_mut_ptr(),
                    &mut data_len,
                );
                RegCloseKey(hkey);

                if q_status2 == 0 {
                    let u16_slice: &[u16] = std::slice::from_raw_parts(
                        buf.as_ptr() as *const u16,
                        buf.len() / 2,
                    );
                    let val_str = String::from_utf16_lossy(u16_slice);
                    return Some(val_str.trim_matches('\0').to_string());
                }
            } else {
                RegCloseKey(hkey);
            }
        }
        None
    }

    /// Lấy giá trị chuỗi lệnh thô của Autostart trong Windows Registry nếu có
    #[allow(dead_code)]
    pub fn get_raw_autostart_command() -> Option<String> {
        get_raw_autostart_command_with_key(DEFAULT_RUN_KEY)
    }

    /// Ghi hoặc xóa chuỗi lệnh thô của Autostart trong Windows Registry theo subkey
    pub fn set_raw_autostart_command_with_key(subkey: &str, cmd: Option<&str>) -> Result<(), String> {
        unsafe {
            let subkey_wide = to_wide_null(subkey);
            let val_name = to_wide_null("RoKLootWatcher");

            if let Some(cmd_str) = cmd {
                let mut hkey: isize = 0;
                let mut disp = 0u32;
                let status = RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    subkey_wide.as_ptr(),
                    0,
                    std::ptr::null(),
                    0,
                    KEY_SET_VALUE,
                    std::ptr::null(),
                    &mut hkey,
                    &mut disp,
                );
                if status != 0 || hkey == 0 {
                    return Err(format!("Không thể mở/tạo Registry key (mã lỗi: {})", status));
                }

                let val_wide = to_wide_null(cmd_str);
                let val_bytes = std::slice::from_raw_parts(
                    val_wide.as_ptr() as *const u8,
                    val_wide.len() * 2,
                );

                let set_status = RegSetValueExW(
                    hkey,
                    val_name.as_ptr(),
                    0,
                    REG_SZ,
                    val_bytes.as_ptr(),
                    val_bytes.len() as u32,
                );
                RegCloseKey(hkey);

                if set_status != 0 {
                    return Err(format!("Không thể ghi giá trị Registry (mã lỗi: {})", set_status));
                }
            } else {
                let mut hkey: isize = 0;
                let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey_wide.as_ptr(), 0, KEY_SET_VALUE, &mut hkey);
                if status != 0 || hkey == 0 {
                    // Nếu key không tồn tại thì giá trị coi như đã không tồn tại
                    return Ok(());
                }
                let _ = RegDeleteValueW(hkey, val_name.as_ptr());
                RegCloseKey(hkey);
            }
        }
        Ok(())
    }

    /// Ghi hoặc xóa chuỗi lệnh thô của Autostart trong Windows Registry
    #[allow(dead_code)]
    pub fn set_raw_autostart_command(cmd: Option<&str>) -> Result<(), String> {
        set_raw_autostart_command_with_key(DEFAULT_RUN_KEY, cmd)
    }

    /// Xóa toàn bộ subkey của Autostart khỏi Windows Registry (dùng cho cleanup và test)
    #[allow(dead_code)]
    pub fn delete_autostart_key(subkey: &str) -> Result<(), String> {
        if subkey == DEFAULT_RUN_KEY || subkey.ends_with(r"\Run") || subkey.ends_with(r"\run") {
            return Err("Refusing to delete system Run key".to_string());
        }
        unsafe {
            let subkey_wide = to_wide_null(subkey);
            let status = RegDeleteKeyW(HKEY_CURRENT_USER, subkey_wide.as_ptr());
            // Mã lỗi 0 là thành công, 2 (ERROR_FILE_NOT_FOUND) là key không tồn tại
            if status != 0 && status != 2 {
                return Err(format!("Không thể xóa registry key (mã lỗi: {})", status));
            }
        }
        Ok(())
    }

    /// Kiểm tra trạng thái Autostart trong Windows Registry theo subkey
    /// Trả về `(is_autostart_enabled, is_start_minimized)`
    pub fn get_autostart_status_with_key(subkey: &str) -> (bool, bool) {
        if let Some(val_str) = get_raw_autostart_command_with_key(subkey) {
            let is_minimized = val_str.contains("--minimized") || val_str.contains("--tray");
            (true, is_minimized)
        } else {
            (false, false)
        }
    }

    /// Kiểm tra trạng thái Autostart trong Windows Registry
    /// Trả về `(is_autostart_enabled, is_start_minimized)`
    pub fn get_autostart_status() -> (bool, bool) {
        get_autostart_status_with_key(DEFAULT_RUN_KEY)
    }

    /// Cập nhật trạng thái Autostart trong Windows Registry theo subkey
    pub fn set_autostart_with_key(subkey: &str, enabled: bool, start_minimized: bool) -> Result<(), String> {
        if enabled {
            let clean_exe = get_clean_exe_path()?;
            let val_str = if start_minimized {
                format!("\"{}\" --minimized", clean_exe)
            } else {
                format!("\"{}\"", clean_exe)
            };
            set_raw_autostart_command_with_key(subkey, Some(&val_str))
        } else {
            set_raw_autostart_command_with_key(subkey, None)
        }
    }

    /// Cập nhật trạng thái Autostart trong Windows Registry
    pub fn set_autostart(enabled: bool, start_minimized: bool) -> Result<(), String> {
        set_autostart_with_key(DEFAULT_RUN_KEY, enabled, start_minimized)
    }
}

#[cfg(not(windows))]
pub mod windows {
    pub const DEFAULT_RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

    pub fn get_clean_exe_path() -> Result<String, String> {
        Ok(String::new())
    }

    pub fn get_raw_autostart_command_with_key(_subkey: &str) -> Option<String> {
        None
    }

    #[allow(dead_code)]
    pub fn get_raw_autostart_command() -> Option<String> {
        None
    }

    pub fn set_raw_autostart_command_with_key(_subkey: &str, _cmd: Option<&str>) -> Result<(), String> {
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_raw_autostart_command(_cmd: Option<&str>) -> Result<(), String> {
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete_autostart_key(subkey: &str) -> Result<(), String> {
        if subkey == DEFAULT_RUN_KEY || subkey.ends_with(r"\Run") || subkey.ends_with(r"\run") {
            return Err("Refusing to delete system Run key".to_string());
        }
        Ok(())
    }

    pub fn get_autostart_status_with_key(_subkey: &str) -> (bool, bool) {
        (false, false)
    }

    pub fn get_autostart_status() -> (bool, bool) {
        (false, false)
    }

    pub fn set_autostart_with_key(_subkey: &str, _enabled: bool, _start_minimized: bool) -> Result<(), String> {
        Ok(())
    }

    pub fn set_autostart(_enabled: bool, _start_minimized: bool) -> Result<(), String> {
        Ok(())
    }
}

pub use windows::*;

/// Đảm bảo thư mục làm việc hiện tại trùng với thư mục chứa file thực thi (.exe).
/// Điều này ngăn chặn lỗi khi Windows khởi chạy từ Autostart (Registry Run key)
/// vốn đặt working directory mặc định là C:\Windows\System32.
pub fn ensure_working_dir_is_exe_dir() -> Result<std::path::PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let parent = exe.parent().ok_or_else(|| "No parent directory for executable".to_string())?;
    std::env::set_current_dir(parent).map_err(|e| e.to_string())?;
    std::env::current_dir().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_working_dir_is_exe_dir() {
        let original_dir = std::env::current_dir().unwrap();
        let res = ensure_working_dir_is_exe_dir();
        assert!(res.is_ok());
        let cur = std::env::current_dir().unwrap();
        let exe = std::env::current_exe().unwrap();
        assert_eq!(cur, exe.parent().unwrap());
        let _ = std::env::set_current_dir(original_dir);
    }

    #[test]
    fn test_clean_exe_path() {
        let path = get_clean_exe_path();
        assert!(path.is_ok(), "Phải lấy được đường dẫn exe");
        let p = path.unwrap();
        assert!(!p.starts_with(r"\\?\"), "Đường dẫn không được có tiền tố verbatim");
        assert!(p.ends_with(".exe") || p.contains("rok_watcher") || p.contains("deps"));
    }

    #[test]
    fn test_delete_autostart_key_refuses_system_run_key() {
        let res = delete_autostart_key(DEFAULT_RUN_KEY);
        assert!(res.is_err(), "Must refuse to delete DEFAULT_RUN_KEY");
        let res_run = delete_autostart_key(r"Software\Something\Run");
        assert!(res_run.is_err(), "Must refuse to delete key ending with \\Run");
        let res_lower = delete_autostart_key(r"Software\Something\run");
        assert!(res_lower.is_err(), "Must refuse to delete key ending with \\run");
    }

    #[test]
    fn test_autostart_toggle() {
        let test_key = r"Software\RoKLootWatcher_Test";
        // Ensure clean state before test
        let _ = delete_autostart_key(test_key);

        let set_res = set_autostart_with_key(test_key, true, true);
        assert!(set_res.is_ok(), "Bật autostart test key phải thành công");

        let (enabled, minimized) = get_autostart_status_with_key(test_key);
        assert!(enabled);
        assert!(minimized);

        let set_res2 = set_autostart_with_key(test_key, true, false);
        assert!(set_res2.is_ok());
        let (enabled2, minimized2) = get_autostart_status_with_key(test_key);
        assert!(enabled2);
        assert!(!minimized2);

        let del_res = delete_autostart_key(test_key);
        assert!(del_res.is_ok());
        let (restored, _) = get_autostart_status_with_key(test_key);
        assert!(!restored);
    }
}
