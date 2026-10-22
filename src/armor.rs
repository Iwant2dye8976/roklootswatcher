use std::path::Path;
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Kiểm tra xem thư mục mailcache hiện có đang được bảo vệ chống xóa (Deny Delete) hay không.
pub fn is_mailcache_armored(mail_dir: &Path) -> bool {
    if !mail_dir.is_dir() {
        return false;
    }

    let mut cmd = Command::new("cmd");
    cmd.args(["/c", "icacls", &mail_dir.to_string_lossy()]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    if let Ok(output) = cmd.output() {
        let text = String::from_utf8_lossy(&output.stdout);
        text.contains("(DENY)") || text.contains(":(DE)") || text.contains(":(D)")
    } else {
        false
    }
}

/// Bật khóa chống xóa (Mailcache Armor) trên thư mục mailcache của game.
/// Yêu cầu quyền Administrator nếu thư mục nằm trong Program Files.
pub fn enable_mailcache_armor(mail_dir: &Path) -> Result<String, String> {
    if !mail_dir.is_dir() {
        return Err(format!("Thư mục không tồn tại: {}", mail_dir.display()));
    }

    let path_str = mail_dir.to_string_lossy().to_string();
    let mut cmd = Command::new("cmd");
    // S-1-1-0 đại diện cho nhóm "Everyone" (Tất cả người dùng) độc lập với ngôn ngữ Windows (Tiếng Anh/Việt/Nga/...)
    cmd.args(["/c", "icacls", &path_str, "/deny", "*S-1-1-0:(DE)"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().map_err(|e| format!("Lỗi thực thi icacls: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() && !stdout.contains("Access is denied") {
        Ok("Đã kích hoạt Khóa chống xóa (Mailcache Armor)! Game sẽ không thể xóa thư cache.".into())
    } else {
        Err(format!(
            "Không thể thiết lập quyền (Cần chạy ứng dụng bằng quyền Administrator - Run as Administrator). Chi tiết: {}{}",
            stdout.trim(), stderr.trim()
        ))
    }
}

/// Gỡ bỏ khóa chống xóa trên thư mục mailcache.
pub fn disable_mailcache_armor(mail_dir: &Path) -> Result<String, String> {
    if !mail_dir.is_dir() {
        return Err(format!("Thư mục không tồn tại: {}", mail_dir.display()));
    }

    let path_str = mail_dir.to_string_lossy().to_string();
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", "icacls", &path_str, "/remove:d", "*S-1-1-0"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().map_err(|e| format!("Lỗi thực thi icacls: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() && !stdout.contains("Access is denied") {
        Ok("Đã gỡ bỏ khóa chống xóa trên mailcache hoàn nguyên mặc định.".into())
    } else {
        Err(format!(
            "Lỗi gỡ bỏ khóa (Cần chạy quyền Administrator). Chi tiết: {}{}",
            stdout.trim(), stderr.trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_armored_check_on_temp_dir() {
        let temp = std::env::temp_dir();
        // Temp dir normally doesn't have deny delete for everyone
        assert!(!is_mailcache_armored(&temp.join("non_existent_folder_xyz")));
    }
}
