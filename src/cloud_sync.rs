use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::time::Duration;
use serde::{Deserialize, Serialize};

pub const DEFAULT_SUPABASE_URL: &str = match option_env!("ROK_SUPABASE_URL") {
    Some(val) => val,
    None => "",
};
pub const DEFAULT_SUPABASE_ANON_KEY: &str = match option_env!("ROK_SUPABASE_ANON_KEY") {
    Some(val) => val,
    None => "",
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudUserInfo {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudAuthResult {
    pub access_token: String,
    pub refresh_token: String,
    pub user: CloudUserInfo,
}

pub fn parse_params(raw: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let trimmed = raw.trim_start_matches('?').trim_start_matches('#');
    for pair in trimmed.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            let key = urlencoding_decode(k);
            let val = urlencoding_decode(v);
            map.insert(key, val);
        }
    }
    map
}

fn urlencoding_decode(s: &str) -> String {
    let mut bytes_out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..=i + 2]).unwrap_or(""), 16) {
                bytes_out.push(byte);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            bytes_out.push(b' ');
            i += 1;
            continue;
        }
        bytes_out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes_out).to_string()
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LoopbackTokenOutcome {
    Success {
        access_token: String,
        refresh_token: String,
    },
    Error {
        status_code: u16,
        message: String,
    },
}

pub(crate) fn process_loopback_token_request(
    req_str: &str,
    expected_state: &str,
) -> LoopbackTokenOutcome {
    let mut origin_valid = true;
    for line in req_str.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("origin:") {
            let val = line[7..].trim();
            if !val.is_empty()
                && val != "null"
                && !val.starts_with("http://127.0.0.1:")
                && !val.starts_with("http://localhost:")
            {
                origin_valid = false;
            }
        }
    }

    if !origin_valid {
        return LoopbackTokenOutcome::Error {
            status_code: 403,
            message: "Yêu cầu xác thực không hợp lệ (origin mismatch)".to_string(),
        };
    }

    let query_str = req_str
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|path| path.split_once('?').map(|(_, q)| q))
        .unwrap_or("");

    let params = parse_params(query_str);
    let received_state = params
        .get("app_state")
        .or_else(|| params.get("state"))
        .map(|s| s.as_str())
        .unwrap_or("");

    if expected_state.is_empty() || received_state != expected_state {
        return LoopbackTokenOutcome::Error {
            status_code: 403,
            message: "CSRF state token không khớp hoặc bị thiếu".to_string(),
        };
    }

    if let Some(err) = params.get("error_description").or_else(|| params.get("error")) {
        return LoopbackTokenOutcome::Error {
            status_code: 400,
            message: format!("Đăng nhập thất bại: {}", err),
        };
    }

    let access_token = match params.get("access_token") {
        Some(tok) if !tok.is_empty() => tok.clone(),
        _ => {
            return LoopbackTokenOutcome::Error {
                status_code: 400,
                message: "Không nhận được access_token từ Google OAuth".to_string(),
            };
        }
    };

    let refresh_token = params.get("refresh_token").cloned().unwrap_or_default();

    LoopbackTokenOutcome::Success {
        access_token,
        refresh_token,
    }
}

#[cfg(windows)]
fn generate_secure_state() -> String {
    #[link(name = "bcrypt")]
    extern "system" {
        fn BCryptGenRandom(
            hAlgorithm: *mut std::ffi::c_void,
            pbBuffer: *mut u8,
            cbBuffer: u32,
            dwFlags: u32,
        ) -> i32;
    }
    let mut buf = [0u8; 16];
    let status = unsafe { BCryptGenRandom(std::ptr::null_mut(), buf.as_mut_ptr(), 16, 2) };
    if status == 0 {
        buf.iter().map(|b| format!("{:02x}", b)).collect()
    } else {
        format!("{:016x}{:016x}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0), std::process::id())
    }
}

#[cfg(not(windows))]
fn generate_secure_state() -> String {
    format!("{:016x}{:016x}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0), std::process::id())
}

pub fn start_google_login(supabase_url: &str, anon_key: &str) -> Result<CloudAuthResult, String> {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .or_else(|_| TcpListener::bind("127.0.0.1:0"))
        .map_err(|e| format!("Cannot bind local loopback listener: {}", e))?;
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(8080);
    listener.set_nonblocking(true).map_err(|e| format!("Nonblocking listener error: {}", e))?;

    let expected_state = generate_secure_state();
    let redirect_uri = format!("http://127.0.0.1:{}/callback", port);
    let auth_url = format!(
        "{}/auth/v1/authorize?provider=google&redirect_to={}&state={}",
        supabase_url.trim_end_matches('/'),
        redirect_uri,
        expected_state
    );

    open::that(&auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;

    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(60);

    let (access_token, refresh_token) = loop {
        if start_time.elapsed() > timeout {
            return Err("Hết thời gian chờ đăng nhập Google (60s). Vui lòng thử lại.".to_string());
        }

        let (mut stream, _) = match listener.accept() {
            Ok(pair) => pair,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(e) => return Err(format!("Connection error: {}", e)),
        };

        let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let _ = stream.set_nonblocking(false);

        let mut buf = [0u8; 16384];
        let bytes_read = stream.read(&mut buf).unwrap_or(0);
        let req_str = String::from_utf8_lossy(&buf[..bytes_read]);

        if req_str.starts_with("GET /callback") {
            let html = format!(
                r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>RoK Loot Watcher - Login</title></head>
<body style="font-family:sans-serif;text-align:center;padding:60px;background:#1e1e2e;color:#cdd6f4;">
  <h2 style="color:#89b4fa;">RoK Loot Watcher</h2>
  <p id="msg" style="font-size:16px;">Đang đồng bộ đăng nhập...</p>
  <script>
    const hash = window.location.hash ? window.location.hash.substring(1) : '';
    const search = window.location.search ? window.location.search.substring(1) : '';
    const fullParams = (hash && search) ? (hash + '&' + search) : (hash || search);
    const url = fullParams ? ('/token?app_state={0}&' + fullParams) : '/token?app_state={0}&error=empty_callback';
    fetch(url)
      .then(async res => {{
        if (!res.ok) {{
          const txt = await res.text();
          throw new Error(txt || ('HTTP ' + res.status));
        }}
        document.getElementById('msg').innerText = 'Đăng nhập thành công! Bạn có thể đóng tab này.';
        setTimeout(() => window.close(), 1500);
      }})
      .catch(err => {{
        document.getElementById('msg').innerText = 'Đăng nhập thất bại: ' + err.message;
      }});
  </script>
</body>
</html>"#,
                expected_state
            );
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        } else if req_str.starts_with("GET /token") {
            match process_loopback_token_request(&req_str, &expected_state) {
                LoopbackTokenOutcome::Success { access_token: at, refresh_token: rt } => {
                    let resp = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK";
                    let _ = stream.write_all(resp.as_bytes());
                    let _ = stream.flush();
                    break (at, rt);
                }
                LoopbackTokenOutcome::Error { status_code, message } => {
                    let reason = if status_code == 403 { "Forbidden" } else { "Bad Request" };
                    let resp = format!(
                        "HTTP/1.1 {} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        status_code,
                        reason,
                        message.len(),
                        message
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    let _ = stream.flush();
                    return Err(message);
                }
            }
        }
    };

    if access_token.is_empty() {
        return Err("Không nhận được access_token từ Google OAuth".to_string());
    }

    let user_url = format!("{}/auth/v1/user", supabase_url.trim_end_matches('/'));
    let user_resp: serde_json::Value = ureq::get(&user_url)
        .set("apikey", anon_key)
        .set("Authorization", &format!("Bearer {}", access_token))
        .call()
        .map_err(|e| format_cloud_error(e, "Xác thực tài khoản thất bại"))?
        .into_json()
        .map_err(|e| format!("Lỗi phân tích thông tin người dùng: {}", e))?;

    let id = user_resp
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing user ID in Supabase response".to_string())?
        .to_string();

    let email = user_resp
        .get("email")
        .and_then(|v| v.as_str())
        .unwrap_or("No Email")
        .to_string();

    Ok(CloudAuthResult {
        access_token,
        refresh_token,
        user: CloudUserInfo { id, email },
    })
}

pub fn refresh_session(
    supabase_url: &str,
    anon_key: &str,
    refresh_token: &str,
) -> Result<(String, String), String> {
    let url = format!("{}/auth/v1/token?grant_type=refresh_token", supabase_url.trim_end_matches('/'));
    let resp: serde_json::Value = ureq::post(&url)
        .set("apikey", anon_key)
        .send_json(serde_json::json!({ "refresh_token": refresh_token }))
        .map_err(|e| format_cloud_error(e, "Làm mới phiên thất bại"))?
        .into_json()
        .map_err(|e| format!("Lỗi phân tích dữ liệu phiên: {}", e))?;

    let new_access = resp
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing access_token in refresh response".to_string())?
        .to_string();
    let new_refresh = resp
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or(refresh_token)
        .to_string();

    Ok((new_access, new_refresh))
}

fn extract_safe_message(body: &str) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = val.get("message").and_then(|m| m.as_str()) {
            return msg.to_string();
        }
        if let Some(msg) = val.get("error_description").and_then(|m| m.as_str()) {
            return msg.to_string();
        }
    }
    String::new()
}

fn format_cloud_error(err: ureq::Error, context: &str) -> String {
    match err {
        ureq::Error::Status(401, _) => {
            "Token phiên làm việc đã hết hạn. Vui lòng đăng nhập lại Google.".to_string()
        }
        ureq::Error::Status(403, res) => {
            let body = res.into_string().unwrap_or_default();
            let msg = extract_safe_message(&body);
            if msg.is_empty() {
                format!("{context}: Bị từ chối truy cập (HTTP 403). Vui lòng kiểm tra quyền bảng / RLS policy.")
            } else {
                format!("{context}: Bị từ chối (HTTP 403) - {msg}")
            }
        }
        ureq::Error::Status(404, _) => {
            format!("{context}: Không tìm thấy bảng dữ liệu (HTTP 404).")
        }
        ureq::Error::Status(code, res) => {
            let body = res.into_string().unwrap_or_default();
            let msg = extract_safe_message(&body);
            if msg.is_empty() {
                format!("{context}: Lỗi máy chủ Cloud (HTTP {code}).")
            } else {
                format!("{context} (HTTP {code}): {msg}")
            }
        }
        ureq::Error::Transport(t) => {
            format!("{context}: Lỗi kết nối mạng ({:?}). Vui lòng kiểm tra kết nối Internet.", t.kind())
        }
    }
}

pub fn backup_vault(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    user_id: &str,
    vault_path: &Path,
    timeout: Option<Duration>,
) -> Result<(usize, Option<(String, String)>), String> {
    let data = fs::read_to_string(vault_path)
        .map_err(|e| format!("Cannot read vault file {}: {}", vault_path.display(), e))?;

    let parsed: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("Invalid local vault JSON: {}", e))?;

    let record_count = parsed
        .get("records")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);

    let url = format!("{}/rest/v1/user_vaults", supabase_url.trim_end_matches('/'));
    let now = chrono::Utc::now().to_rfc3339();

    let effective_timeout = timeout.unwrap_or(Duration::from_secs(15));
    let agent = ureq::builder()
        .timeout(effective_timeout)
        .build();

    let send_req = |tok: &str| {
        agent.post(&url)
            .set("apikey", anon_key)
            .set("Authorization", &format!("Bearer {}", tok))
            .set("Prefer", "resolution=merge-duplicates")
            .send_json(serde_json::json!({
                "user_id": user_id,
                "updated_at": now,
                "vault_data": parsed
            }))
    };

    match send_req(access_token) {
        Ok(_) => Ok((record_count, None)),
        Err(ureq::Error::Status(401, _)) if refresh_token.is_some() => {
            let ref_tok = refresh_token.unwrap();
            let (new_acc, new_ref) = refresh_session(supabase_url, anon_key, ref_tok)?;
            match send_req(&new_acc) {
                Ok(_) => Ok((record_count, Some((new_acc, new_ref)))),
                Err(e) => Err(format_cloud_error(e, "Lỗi sao lưu lên cloud")),
            }
        }
        Err(e) => Err(format_cloud_error(e, "Lỗi sao lưu lên cloud")),
    }
}

pub fn restore_vault(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    vault_path: &Path,
) -> Result<(usize, String, Option<(String, String)>), String> {
    let url = format!("{}/rest/v1/user_vaults?select=updated_at,vault_data", supabase_url.trim_end_matches('/'));

    let fetch_req = |tok: &str| {
        ureq::get(&url)
            .set("apikey", anon_key)
            .set("Authorization", &format!("Bearer {}", tok))
            .call()
    };

    let (resp_val, new_tokens) = match fetch_req(access_token) {
        Ok(r) => (
            r.into_json::<serde_json::Value>().map_err(|e| format!("Parse JSON error: {}", e))?,
            None,
        ),
        Err(ureq::Error::Status(401, _)) if refresh_token.is_some() => {
            let ref_tok = refresh_token.unwrap();
            let (new_acc, new_ref) = refresh_session(supabase_url, anon_key, ref_tok)?;
            let r = fetch_req(&new_acc)
                .map_err(|e| format_cloud_error(e, "Lỗi tải dữ liệu từ cloud"))?;
            (
                r.into_json::<serde_json::Value>().map_err(|e| format!("Parse JSON error: {}", e))?,
                Some((new_acc, new_ref)),
            )
        }
        Err(e) => return Err(format_cloud_error(e, "Lỗi tải dữ liệu từ cloud")),
    };

    let rows = resp_val.as_array().ok_or_else(|| "Phản hồi từ cloud không hợp lệ".to_string())?;
    if rows.is_empty() {
        return Err("Không tìm thấy bản sao lưu nào trên tài khoản Cloud này.".to_string());
    }

    let first_row = &rows[0];
    let updated_at = first_row
        .get("updated_at")
        .and_then(|v| v.as_str())
        .unwrap_or("Không rõ")
        .to_string();

    let vault_data = first_row
        .get("vault_data")
        .ok_or_else(|| "Dữ liệu bản sao lưu trống".to_string())?;

    let storage_data: crate::vault::VaultStorageData = serde_json::from_value(vault_data.clone())
        .map_err(|e| format!("Dữ liệu đám mây không đúng định dạng Vault: {}", e))?;

    let record_count = storage_data.records.len();

    let formatted = serde_json::to_string_pretty(vault_data)
        .map_err(|e| format!("Format JSON error: {}", e))?;

    if vault_path.exists() {
        let bak_path = vault_path.with_extension("bak");
        let _ = fs::copy(vault_path, &bak_path);
    }

    let tmp_path = vault_path.with_extension("tmp");
    fs::write(&tmp_path, formatted)
        .map_err(|e| format!("Write tmp file error: {}", e))?;
    fs::rename(&tmp_path, vault_path)
        .map_err(|e| format!("Replace vault file error: {}", e))?;

    Ok((record_count, updated_at, new_tokens))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_params() {
        let raw = "access_token=secret_123&token_type=bearer&expires_in=3600&user=test%40gmail.com";
        let params = parse_params(raw);
        assert_eq!(params.get("access_token").unwrap(), "secret_123");
        assert_eq!(params.get("user").unwrap(), "test@gmail.com");
    }

    #[test]
    fn test_urlencoding_decode() {
        assert_eq!(urlencoding_decode("hello+world"), "hello world");
        assert_eq!(urlencoding_decode("foo%20bar%21"), "foo bar!");
        assert_eq!(urlencoding_decode("Vi%E1%BB%87t%20Nam"), "Việt Nam");
    }

    #[test]
    fn test_credentials_defined() {
        let _ = DEFAULT_SUPABASE_URL;
        let _ = DEFAULT_SUPABASE_ANON_KEY;
    }

    #[test]
    fn test_extract_safe_message() {
        let json = r#"{"code":"42501","details":null,"hint":null,"message":"permission denied for table user_vaults"}"#;
        assert_eq!(extract_safe_message(json), "permission denied for table user_vaults");
        assert_eq!(extract_safe_message("invalid json"), "");
    }

    #[test]
    fn test_generate_secure_state() {
        let s1 = generate_secure_state();
        let s2 = generate_secure_state();
        assert_eq!(s1.len(), 32);
        assert_eq!(s2.len(), 32);
        assert_ne!(s1, s2, "State tokens must be unique and non-empty");
    }

    #[test]
    fn test_restore_vault_schema_validation_rejects_corrupted() {
        let invalid_json = serde_json::json!({
            "invalid_field": 123
        });
        let result = serde_json::from_value::<crate::vault::VaultStorageData>(invalid_json);
        assert!(result.is_err(), "Must reject non-VaultStorageData payload");
    }

    #[test]
    fn test_process_loopback_token_request_success() {
        let req = "GET /token?app_state=safe_state_123&access_token=at_abc&refresh_token=rt_xyz HTTP/1.1\r\nOrigin: null\r\n\r\n";
        let res = process_loopback_token_request(req, "safe_state_123");
        assert_eq!(
            res,
            LoopbackTokenOutcome::Success {
                access_token: "at_abc".to_string(),
                refresh_token: "rt_xyz".to_string(),
            }
        );
    }

    #[test]
    fn test_process_loopback_token_request_invalid_origin() {
        let req = "GET /token?app_state=safe_state_123&access_token=at_abc HTTP/1.1\r\nOrigin: https://evil.com\r\n\r\n";
        let res = process_loopback_token_request(req, "safe_state_123");
        assert!(matches!(res, LoopbackTokenOutcome::Error { status_code: 403, .. }));
    }

    #[test]
    fn test_process_loopback_token_request_invalid_state() {
        let req = "GET /token?app_state=wrong_state&access_token=at_abc HTTP/1.1\r\n\r\n";
        let res = process_loopback_token_request(req, "safe_state_123");
        assert!(matches!(res, LoopbackTokenOutcome::Error { status_code: 403, .. }));
    }

    #[test]
    fn test_process_loopback_token_request_oauth_error() {
        let req = "GET /token?app_state=safe_state_123&error=access_denied&error_description=User+cancelled HTTP/1.1\r\n\r\n";
        let res = process_loopback_token_request(req, "safe_state_123");
        match res {
            LoopbackTokenOutcome::Error { status_code, message } => {
                assert_eq!(status_code, 400);
                assert!(message.contains("User cancelled"));
            }
            _ => panic!("Expected error outcome"),
        }
    }

    #[test]
    fn test_process_loopback_token_request_missing_token() {
        let req = "GET /token?app_state=safe_state_123 HTTP/1.1\r\n\r\n";
        let res = process_loopback_token_request(req, "safe_state_123");
        assert!(matches!(res, LoopbackTokenOutcome::Error { status_code: 400, .. }));
    }

    #[test]
    fn test_backup_vault_accepts_timeout_param() {
        let dummy_path = std::path::Path::new("dummy_nonexistent_vault.json");
        let res = backup_vault(
            "https://example.supabase.co",
            "dummy_key",
            "dummy_token",
            None,
            "dummy_user",
            dummy_path,
            Some(std::time::Duration::from_secs(4)),
        );
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Cannot read vault file"));
    }
}
