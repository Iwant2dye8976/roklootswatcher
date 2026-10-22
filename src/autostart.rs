#[cfg(windows)]
pub mod windows {
    const HKEY_CURRENT_USER: isize = 0x80000001u32 as isize;
    const KEY_SET_VALUE: u32 = 0x0002;
    const KEY_QUERY_VALUE: u32 = 0x0001;
    const REG_SZ: u32 = 1;

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: isize,
            lpSubKey: *const u16,
            ulOptions: u32,
            samDesired: u32,
            phkResult: *mut isize,
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

    /// Kiểm tra trạng thái Autostart trong Windows Registry
    /// Trả về `(is_autostart_enabled, is_start_minimized)`
    pub fn get_autostart_status() -> (bool, bool) {
        unsafe {
            let subkey = to_wide_null(r"Software\Microsoft\Windows\CurrentVersion\Run");
            let mut hkey: isize = 0;
            let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_QUERY_VALUE, &mut hkey);
            if status != 0 || hkey == 0 {
                return (false, false);
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
                    let is_minimized = val_str.contains("--minimized") || val_str.contains("--tray");
                    return (true, is_minimized);
                }
            } else {
                RegCloseKey(hkey);
            }
        }
        (false, false)
    }

    /// Cập nhật trạng thái Autostart trong Windows Registry
    pub fn set_autostart(enabled: bool, start_minimized: bool) -> Result<(), String> {
        unsafe {
            let subkey = to_wide_null(r"Software\Microsoft\Windows\CurrentVersion\Run");
            let mut hkey: isize = 0;
            let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_SET_VALUE, &mut hkey);
            if status != 0 || hkey == 0 {
                return Err(format!("Không thể mở Registry Run (mã lỗi: {})", status));
            }

            let val_name = to_wide_null("RoKLootWatcher");

            if enabled {
                let clean_exe = get_clean_exe_path()?;
                let val_str = if start_minimized {
                    format!("\"{}\" --minimized", clean_exe)
                } else {
                    format!("\"{}\"", clean_exe)
                };

                let val_wide = to_wide_null(&val_str);
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
                let _ = RegDeleteValueW(hkey, val_name.as_ptr());
                RegCloseKey(hkey);
            }
        }
        Ok(())
    }
}

#[cfg(not(windows))]
pub mod windows {
    pub fn get_clean_exe_path() -> Result<String, String> {
        Ok(String::new())
    }

    pub fn get_autostart_status() -> (bool, bool) {
        (false, false)
    }

    pub fn set_autostart(_enabled: bool, _start_minimized: bool) -> Result<(), String> {
        Ok(())
    }
}

pub use windows::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_exe_path() {
        let path = get_clean_exe_path();
        assert!(path.is_ok(), "Phải lấy được đường dẫn exe");
        let p = path.unwrap();
        assert!(!p.starts_with(r"\\?\"), "Đường dẫn không được có tiền tố verbatim");
        assert!(p.ends_with(".exe") || p.contains("rok_watcher") || p.contains("deps"));
    }

    #[test]
    fn test_autostart_toggle() {
        // Kiểm tra đọc trạng thái hiện tại
        let (initial_enabled, initial_minimized) = get_autostart_status();

        // Thử bật với tùy chọn thu nhỏ
        let set_res = set_autostart(true, true);
        assert!(set_res.is_ok(), "Bật autostart phải thành công: {:?}", set_res.err());

        let (enabled, minimized) = get_autostart_status();
        assert!(enabled, "Autostart phải ở trạng thái enabled");
        assert!(minimized, "Tùy chọn minimized phải là true");

        // Thử bật không thu nhỏ
        let set_res2 = set_autostart(true, false);
        assert!(set_res2.is_ok());
        let (enabled2, minimized2) = get_autostart_status();
        assert!(enabled2);
        assert!(!minimized2, "Tùy chọn minimized phải là false");

        // Khôi phục lại trạng thái ban đầu của hệ thống
        let _ = set_autostart(initial_enabled, initial_minimized);
        let (restored_enabled, _) = get_autostart_status();
        assert_eq!(restored_enabled, initial_enabled, "Phải khôi phục đúng trạng thái ban đầu");
    }
}
