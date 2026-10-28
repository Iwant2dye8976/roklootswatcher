#[cfg(windows)]
pub mod windows {
    use std::sync::Arc;
    use eframe::egui;
    use crate::watcher::RoKMailWatcher;

    #[allow(dead_code)]
    pub const SINGLE_INSTANCE_MUTEX_NAME: &str = "Local\\ROK_Loot_Watcher_SingleInstance_Mutex";
    #[allow(dead_code)]
    pub const SINGLE_INSTANCE_EVENT_NAME: &str = "Local\\ROK_Loot_Watcher_Activate_Event";

    const ERROR_ALREADY_EXISTS: u32 = 183;
    const EVENT_MODIFY_STATE: u32 = 0x0002;
    const WAIT_OBJECT_0: u32 = 0;
    const SW_SHOW: i32 = 5;
    const SW_RESTORE: i32 = 9;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateMutexW(
            lpMutexAttributes: *const std::ffi::c_void,
            bInitialOwner: i32,
            lpName: *const u16,
        ) -> isize;

        fn CreateEventW(
            lpEventAttributes: *const std::ffi::c_void,
            bManualReset: i32,
            bInitialState: i32,
            lpName: *const u16,
        ) -> isize;

        fn OpenEventW(
            dwDesiredAccess: u32,
            bInheritHandle: i32,
            lpName: *const u16,
        ) -> isize;

        fn SetEvent(hEvent: isize) -> i32;

        fn WaitForSingleObject(hHandle: isize, dwMilliseconds: u32) -> u32;

        fn CloseHandle(hObject: isize) -> i32;

        fn GetLastError() -> u32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
        fn EnumWindows(
            lpEnumFunc: Option<unsafe extern "system" fn(isize, isize) -> i32>,
            lParam: isize,
        ) -> i32;
        fn GetWindowTextW(hWnd: isize, lpString: *mut u16, nMaxCount: i32) -> i32;
        fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut u32) -> u32;
        fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
        fn SetForegroundWindow(hWnd: isize) -> i32;
        fn BringWindowToTop(hWnd: isize) -> i32;
        fn AllowSetForegroundWindow(dwProcessId: u32) -> i32;
        #[allow(dead_code)]
        fn MessageBoxW(
            hWnd: isize,
            lpText: *const u16,
            lpCaption: *const u16,
            uType: u32,
        ) -> i32;
    }

    fn to_wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub struct SingleInstanceGuard {
        mutex_handle: isize,
        event_handle: isize,
    }

    impl Drop for SingleInstanceGuard {
        fn drop(&mut self) {
            unsafe {
                if self.mutex_handle != 0 {
                    CloseHandle(self.mutex_handle);
                }
                if self.event_handle != 0 {
                    CloseHandle(self.event_handle);
                }
            }
        }
    }

    pub enum CheckResult {
        Running(SingleInstanceGuard),
        AlreadyExists,
    }

    fn get_app_instance_id() -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        if let Ok(exe) = std::env::current_exe() {
            exe.to_string_lossy().hash(&mut hasher);
        } else {
            "rok_default".hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }

    pub fn default_mutex_name() -> String {
        format!("Local\\ROK_Loot_Watcher_{}_Mutex", get_app_instance_id())
    }

    pub fn default_event_name() -> String {
        format!("Local\\ROK_Loot_Watcher_{}_Event", get_app_instance_id())
    }

    pub fn check_or_acquire() -> CheckResult {
        check_or_acquire_named(&default_mutex_name(), &default_event_name())
    }

    pub fn check_or_acquire_named(mutex_name_str: &str, event_name_str: &str) -> CheckResult {
        unsafe {
            let mutex_wide = to_wide_null(mutex_name_str);
            let mutex = CreateMutexW(std::ptr::null(), 1, mutex_wide.as_ptr());

            let err = GetLastError();
            if err == ERROR_ALREADY_EXISTS {
                let event_signaled = signal_activate_event_named(event_name_str);

                let window_found = activate_existing_window();

                if !event_signaled && !window_found {
                    let event_wide = to_wide_null(event_name_str);
                    let event = CreateEventW(std::ptr::null(), 0, 0, event_wide.as_ptr());
                    return CheckResult::Running(SingleInstanceGuard {
                        mutex_handle: mutex,
                        event_handle: event,
                    });
                }

                if mutex != 0 {
                    CloseHandle(mutex);
                }

                return CheckResult::AlreadyExists;
            }

            let event_wide = to_wide_null(event_name_str);
            let event = CreateEventW(std::ptr::null(), 0, 0, event_wide.as_ptr());

            CheckResult::Running(SingleInstanceGuard {
                mutex_handle: mutex,
                event_handle: event,
            })
        }
    }

    #[allow(dead_code)]
    pub fn signal_activate_event() -> bool {
        signal_activate_event_named(&default_event_name())
    }

    pub fn signal_activate_event_named(event_name_str: &str) -> bool {
        unsafe {
            let event_wide = to_wide_null(event_name_str);
            let event = OpenEventW(EVENT_MODIFY_STATE, 0, event_wide.as_ptr());
            if event != 0 {
                SetEvent(event);
                CloseHandle(event);
                true
            } else {
                false
            }
        }
    }

    pub fn activate_existing_window() -> bool {
        let title = to_wide_null("Rise of Kingdoms - Motte, Fort & Barbarian Loot Watcher");

        unsafe {
            let mut hwnd = FindWindowW(std::ptr::null(), title.as_ptr());

            if hwnd == 0 {
                struct EnumData {
                    found: isize,
                }
                unsafe extern "system" fn enum_proc(h: isize, lparam: isize) -> i32 {
                    if lparam == 0 {
                        return 1;
                    }
                    let data = &mut *(lparam as *mut EnumData);
                    let mut buf = [0u16; 512];
                    let len = GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32);
                    if len > 0 {
                        let text = String::from_utf16_lossy(&buf[..len as usize]);
                        if text.contains("Rise of Kingdoms") && text.contains("Loot Watcher") {
                            data.found = h;
                            return 0;
                        }
                    }
                    1
                }

                let mut data = EnumData { found: 0 };
                EnumWindows(Some(enum_proc), &mut data as *mut _ as isize);
                hwnd = data.found;
            }

            if hwnd != 0 {
                ShowWindow(hwnd, SW_SHOW);
                ShowWindow(hwnd, SW_RESTORE);

                let mut proc_id = 0u32;
                GetWindowThreadProcessId(hwnd, &mut proc_id);
                if proc_id != 0 {
                    AllowSetForegroundWindow(proc_id);
                }
                AllowSetForegroundWindow(0xFFFFFFFF);

                SetForegroundWindow(hwnd);
                BringWindowToTop(hwnd);
                true
            } else {
                false
            }
        }
    }

    #[allow(dead_code)]
    pub fn show_already_running_message() {
        let msg = to_wide_null(
            "Rise of Kingdoms Loot Watcher hiện đang được mở sẵn trên máy tính của bạn.\n\n\
            Nếu cửa sổ chưa xuất hiện, vui lòng kiểm tra biểu tượng ở Khay hệ thống \
            (System Tray cạnh đồng hồ Windows) để mở lại."
        );
        let caption = to_wide_null("ROK Loot Watcher - Ứng dụng đang chạy");

        unsafe {
            const MB_OK: u32 = 0x00000000;
            const MB_ICONINFORMATION: u32 = 0x00000040;
            MessageBoxW(0, msg.as_ptr(), caption.as_ptr(), MB_OK | MB_ICONINFORMATION);
        }
    }

    pub fn start_activation_listener(
        guard: &SingleInstanceGuard,
        egui_ctx: egui::Context,
        watcher: Arc<RoKMailWatcher>,
    ) {
        let event_handle = guard.event_handle;
        if event_handle == 0 {
            return;
        }

        std::thread::spawn(move || {
            while !crate::tray::is_quit_requested() {
                let wait_res = unsafe { WaitForSingleObject(event_handle, 500) };
                if wait_res == WAIT_OBJECT_0 {
                    crate::tray::restore_main_window();
                    egui_ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    egui_ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                    egui_ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    egui_ctx.request_repaint();

                    watcher.log(
                        "INFO",
                        "Phát hiện yêu cầu mở ứng dụng: Đã tự động kích hoạt và chuyển tiêu điểm về cửa sổ đang chạy.",
                    );
                }
            }
        });
    }
}

#[cfg(not(windows))]
pub mod windows {
    use std::sync::Arc;
    use eframe::egui;
    use crate::watcher::RoKMailWatcher;

    pub struct SingleInstanceGuard;
    pub enum CheckResult {
        Running(SingleInstanceGuard),
        AlreadyExists,
    }

    pub fn check_or_acquire() -> CheckResult {
        CheckResult::Running(SingleInstanceGuard)
    }

    pub fn start_activation_listener(
        _guard: &SingleInstanceGuard,
        _egui_ctx: egui::Context,
        _watcher: Arc<RoKMailWatcher>,
    ) {}
}

#[cfg(test)]
mod tests {
    use super::windows::*;

    #[test]
    fn test_single_instance_acquisition_and_duplicate_prevention() {
        let pid = std::process::id();
        let mutex_name = format!("Local\\ROK_Test_Mutex_{}", pid);
        let event_name = format!("Local\\ROK_Test_Event_{}", pid);

        let first = check_or_acquire_named(&mutex_name, &event_name);
        let guard = match first {
            CheckResult::Running(g) => g,
            CheckResult::AlreadyExists => panic!("Lần đầu tiên mở phải là Running"),
        };

        let second = check_or_acquire_named(&mutex_name, &event_name);
        match second {
            CheckResult::AlreadyExists => {
            }
            CheckResult::Running(_) => panic!("Lần thứ hai phải bị chặn với AlreadyExists"),
        }

        drop(guard);

        let third = check_or_acquire_named(&mutex_name, &event_name);
        match third {
            CheckResult::Running(_g) => {
            }
            CheckResult::AlreadyExists => panic!("Sau khi guard được giải phóng, phải mở lại được"),
        }
    }

    #[test]
    fn test_activation_event_signaling() {
        let pid = std::process::id();
        let mutex_name = format!("Local\\ROK_Test_Sig_Mutex_{}", pid);
        let event_name = format!("Local\\ROK_Test_Sig_Event_{}", pid);

        let first = check_or_acquire_named(&mutex_name, &event_name);
        let guard = match first {
            CheckResult::Running(g) => g,
            CheckResult::AlreadyExists => panic!("Không thể khởi tạo guard thử nghiệm"),
        };

        let signaled = signal_activate_event_named(&event_name);
        assert!(signaled, "Phải gửi được tín hiệu kích hoạt đến Event của tiến trình đang chạy");

        drop(guard);
    }

    #[test]
    fn test_single_instance_squatted_mutex_fallback() {
        let pid = std::process::id();
        let mutex_name = format!("Local\\ROK_Squat_Mutex_{}", pid);
        let event_name = format!("Local\\ROK_Squat_Event_{}", pid);

        let mutex_wide: Vec<u16> = mutex_name.encode_utf16().chain(std::iter::once(0)).collect();
        let rogue_mutex = unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn CreateMutexW(lpMutexAttributes: *const std::ffi::c_void, bInitialOwner: i32, lpName: *const u16) -> isize;
            }
            CreateMutexW(std::ptr::null(), 1, mutex_wide.as_ptr())
        };

        let result = check_or_acquire_named(&mutex_name, &event_name);
        match result {
            CheckResult::Running(guard) => {
                drop(guard);
            }
            CheckResult::AlreadyExists => {
                panic!("App must not exit when a squatted mutex has no genuine responding instance");
            }
        }

        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn CloseHandle(h: isize) -> i32;
            }
            CloseHandle(rogue_mutex);
        }
    }
}
