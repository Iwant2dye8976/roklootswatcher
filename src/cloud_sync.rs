use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{mpsc, Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::model::{BattleRecord, CloudAuthConfig};
use crate::vault::MailVault;

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
    #[serde(default)]
    pub settings: Option<crate::model::UserCloudSettings>,
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

pub fn start_google_login(
    supabase_url: &str,
    anon_key: &str,
    cancel_signal: Option<&std::sync::atomic::AtomicBool>,
) -> Result<CloudAuthResult, String> {
    if let Some(cancel) = cancel_signal {
        if cancel.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("Đã hủy đăng nhập Google".to_string());
        }
    }

    let listener = TcpListener::bind("127.0.0.1:8080")
        .or_else(|_| TcpListener::bind("127.0.0.1:0"))
        .map_err(|e| format!("Cannot bind local loopback listener: {}", e))?;
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(8080);
    listener.set_nonblocking(true).map_err(|e| format!("Nonblocking listener error: {}", e))?;

    let expected_state = generate_secure_state();
    // Embed expected_state as a query parameter in redirect_to so Supabase preserves it in the browser callback URL
    let redirect_uri = format!("http://127.0.0.1:{}/callback?app_state={}", port, expected_state);
    let auth_url = format!(
        "{}/auth/v1/authorize?provider=google&redirect_to={}&state={}&prompt=consent&access_type=offline",
        supabase_url.trim_end_matches('/'),
        redirect_uri,
        expected_state
    );

    open::that(&auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;

    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(60);

    let (access_token, refresh_token) = loop {
        if let Some(cancel) = cancel_signal {
            if cancel.load(std::sync::atomic::Ordering::SeqCst) {
                return Err("Đã hủy đăng nhập Google".to_string());
            }
        }

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
            let html = r#"<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>RoK Loot Watcher - Login</title></head>
<body style="font-family:sans-serif;text-align:center;padding:60px;background:#1e1e2e;color:#cdd6f4;">
  <h2 style="color:#89b4fa;">RoK Loot Watcher</h2>
  <p id="msg" style="font-size:16px;">Đang đồng bộ đăng nhập...</p>
  <div id="btn-container" style="margin-top:20px;display:none;">
    <button onclick="window.close()" style="background:#313244;color:#cdd6f4;border:1px solid #45475a;padding:8px 16px;border-radius:4px;cursor:pointer;font-size:14px;">Đóng tab</button>
  </div>
  <script>
    function closeTab(delay) {
      setTimeout(() => {
        window.close();
        document.getElementById('btn-container').style.display = 'block';
      }, delay);
    }
    const hash = window.location.hash ? window.location.hash.substring(1) : '';
    const search = window.location.search ? window.location.search.substring(1) : '';
    const fullParams = (hash && search) ? (hash + '&' + search) : (hash || search);
    const url = fullParams ? ('/token?' + fullParams) : '/token?error=empty_callback';
    fetch(url)
      .then(async res => {
        if (!res.ok) {
          const txt = await res.text();
          throw new Error(txt || ('HTTP ' + res.status));
        }
        document.getElementById('msg').innerText = 'Đăng nhập thành công! Đang đóng tab...';
        closeTab(1500);
      })
      .catch(err => {
        document.getElementById('msg').innerText = 'Đăng nhập thất bại: ' + err.message + '. Đang đóng tab...';
        closeTab(1500);
      });
  </script>
</body>
</html>"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        } else if req_str.starts_with("GET /cancel") {
            let html = r#"<!DOCTYPE html><html><body style="font-family:sans-serif;text-align:center;padding:60px;background:#1e1e2e;color:#cdd6f4;"><h2 style="color:#f38ba8;">RoK Loot Watcher</h2><p>Đã hủy đăng nhập. Đang đóng tab...</p><script>setTimeout(()=>window.close(),1000);</script></body></html>"#;
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
            return Err("Đã hủy đăng nhập Google".to_string());
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
                    return Err(format!("Đăng nhập thất bại: {}", message));
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

    let settings = user_resp
        .get("user_metadata")
        .or_else(|| user_resp.get("raw_user_meta_data"))
        .and_then(|m| m.get("rok_settings"))
        .and_then(|s| serde_json::from_value::<crate::model::UserCloudSettings>(s.clone()).ok());

    Ok(CloudAuthResult {
        access_token,
        refresh_token,
        user: CloudUserInfo { id, email, settings },
    })
}

pub fn refresh_session(
    supabase_url: &str,
    anon_key: &str,
    refresh_token: &str,
    timeout: Option<Duration>,
) -> Result<(String, String), String> {
    let url = format!("{}/auth/v1/token?grant_type=refresh_token", supabase_url.trim_end_matches('/'));
    let effective_timeout = timeout.unwrap_or(Duration::from_secs(15));
    let agent = ureq::builder()
        .timeout(effective_timeout)
        .build();
    let resp: serde_json::Value = agent
        .post(&url)
        .set("apikey", anon_key)
        .set("Authorization", &format!("Bearer {}", anon_key))
        .send_json(serde_json::json!({
            "grant_type": "refresh_token",
            "refresh_token": refresh_token
        }))
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

pub fn upload_user_settings(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    settings: &crate::model::UserCloudSettings,
    timeout: Option<Duration>,
) -> Result<(), String> {
    if supabase_url.is_empty() || anon_key.is_empty() || access_token.is_empty() {
        return Ok(());
    }
    let url = format!("{}/auth/v1/user", supabase_url.trim_end_matches('/'));
    let effective_timeout = timeout.unwrap_or(Duration::from_secs(10));
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(5))
        .timeout(effective_timeout)
        .build();

    let body = serde_json::json!({
        "data": {
            "rok_settings": settings
        }
    });

    let resp = agent
        .put(&url)
        .set("apikey", anon_key)
        .set("Authorization", &format!("Bearer {}", access_token))
        .set("Content-Type", "application/json")
        .send_json(body);

    match resp {
        Ok(_) => Ok(()),
        Err(e) => Err(format_cloud_error(e, "Lỗi lưu cấu hình lên cloud")),
    }
}

pub fn fetch_user_settings(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    timeout: Option<Duration>,
) -> Result<Option<crate::model::UserCloudSettings>, String> {
    if supabase_url.is_empty() || anon_key.is_empty() || access_token.is_empty() {
        return Ok(None);
    }
    let url = format!("{}/auth/v1/user", supabase_url.trim_end_matches('/'));
    let effective_timeout = timeout.unwrap_or(Duration::from_secs(10));
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(5))
        .timeout(effective_timeout)
        .build();

    let resp = agent
        .get(&url)
        .set("apikey", anon_key)
        .set("Authorization", &format!("Bearer {}", access_token))
        .call();

    match resp {
        Ok(r) => {
            let val: serde_json::Value = r
                .into_json()
                .map_err(|e| format!("Parse user json error: {}", e))?;
            let settings_val = val
                .get("user_metadata")
                .or_else(|| val.get("raw_user_meta_data"))
                .and_then(|m| m.get("rok_settings"));

            if let Some(s) = settings_val {
                if !s.is_null() {
                    let parsed: crate::model::UserCloudSettings = serde_json::from_value(s.clone())
                        .map_err(|e| format!("Parse rok_settings error: {}", e))?;
                    return Ok(Some(parsed));
                }
            }
            Ok(None)
        }
        Err(e) => Err(format_cloud_error(e, "Lỗi tải cấu hình từ cloud")),
    }
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

pub(crate) fn is_retryable_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    // Non-retryable errors: client errors, auth/permissions, DB issues, user mismatch
    if lower.contains("400")
        || lower.contains("401")
        || lower.contains("403")
        || lower.contains("404")
        || lower.contains("chưa đăng nhập")
        || lower.contains("không khớp")
        || lower.contains("hết hạn")
        || lower.contains("từ chối")
        || lower.contains("sqlite")
    {
        return false;
    }
    // Retryable: transient network drops, connection timeouts, 5xx server errors, rate limiting (429)
    lower.contains("kết nối")
        || lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("transport")
        || lower.contains("network")
        || lower.contains("dns")
        || lower.contains("refused")
        || lower.contains("500")
        || lower.contains("502")
        || lower.contains("503")
        || lower.contains("504")
        || lower.contains("429")
}

pub fn build_report_record_payload(
    user_id: &str,
    dedup_key: &str,
    record: &BattleRecord,
) -> serde_json::Value {
    serde_json::json!({
        "user_id": user_id,
        "dedup_key": dedup_key,
        "mail_id": record.mail_id,
        "battle_type": record.battle_type,
        "time_raw": record.time_raw,
        "time_utc": record.time_utc,
        "data": serde_json::to_value(record).unwrap_or(serde_json::Value::Null),
    })
}

pub fn upload_report_records_batch(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    user_id: &str,
    records: &[(String, BattleRecord)],
    timeout: Option<Duration>,
) -> Result<usize, String> {
    if records.is_empty() {
        return Ok(0);
    }

    let payloads: Vec<serde_json::Value> = records
        .iter()
        .map(|(dedup_key, rec)| build_report_record_payload(user_id, dedup_key, rec))
        .collect();

    let url = format!("{}/rest/v1/report_records?on_conflict=user_id,dedup_key", supabase_url.trim_end_matches('/'));
    let effective_timeout = timeout.unwrap_or(Duration::from_secs(15));
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(10))
        .timeout(effective_timeout)
        .build();

    let resp = agent
        .post(&url)
        .set("apikey", anon_key)
        .set("Authorization", &format!("Bearer {}", access_token))
        .set("Prefer", "resolution=merge-duplicates")
        .send_json(serde_json::Value::Array(payloads));

    match resp {
        Ok(_) => Ok(records.len()),
        Err(ureq::Error::Status(404, _)) => Err(
            "Bảng report_records chưa được khởi tạo trên Supabase (HTTP 404). Vui lòng chạy file migration SQL.".to_string(),
        ),
        Err(e) => Err(format_cloud_error(e, "Lỗi tải bản ghi lên cloud")),
    }
}

pub fn delete_report_records_batch(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    user_id: &str,
    dedup_keys: &[String],
    timeout: Option<Duration>,
) -> Result<usize, String> {
    if dedup_keys.is_empty() {
        return Ok(0);
    }
    let effective_timeout = timeout.unwrap_or(Duration::from_secs(15));
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(10))
        .timeout(effective_timeout)
        .build();

    let mut total_deleted = 0;
    for chunk in dedup_keys.chunks(50) {
        let keys_filter = chunk
            .iter()
            .map(|k| format!("\"{}\"", k))
            .collect::<Vec<_>>()
            .join(",");
        let url = format!(
            "{}/rest/v1/report_records?user_id=eq.{}&dedup_key=in.({})",
            supabase_url.trim_end_matches('/'),
            user_id,
            keys_filter
        );
        let resp = agent
            .delete(&url)
            .set("apikey", anon_key)
            .set("Authorization", &format!("Bearer {}", access_token))
            .call();

        match resp {
            Ok(_) => total_deleted += chunk.len(),
            Err(e) => return Err(format_cloud_error(e, "Lỗi xóa bản ghi trên cloud")),
        }
    }
    Ok(total_deleted)
}

pub fn fetch_report_records_page(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    user_id: &str,
    limit: usize,
    offset: usize,
    timeout: Option<Duration>,
) -> Result<(Vec<BattleRecord>, bool), String> {
    let url = format!(
        "{}/rest/v1/report_records?user_id=eq.{}&select=data&order=time_raw.asc,dedup_key.asc&limit={}&offset={}",
        supabase_url.trim_end_matches('/'),
        user_id,
        limit,
        offset
    );
    let effective_timeout = timeout.unwrap_or(Duration::from_secs(15));
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(10))
        .timeout(effective_timeout)
        .build();

    let resp = match agent
        .get(&url)
        .set("apikey", anon_key)
        .set("Authorization", &format!("Bearer {}", access_token))
        .call()
    {
        Ok(r) => r,
        Err(ureq::Error::Status(404, _)) => {
            return Err(
                "Bảng report_records chưa được khởi tạo trên Supabase (HTTP 404). Vui lòng chạy file migration SQL.".to_string(),
            );
        }
        Err(e) => return Err(format_cloud_error(e, "Lỗi tải bản ghi từ cloud")),
    };

    let val: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Parse JSON error: {}", e))?;

    let rows = val
        .as_array()
        .ok_or_else(|| "Phản hồi từ cloud không hợp lệ (không phải mảng)".to_string())?;

    let mut records = Vec::with_capacity(rows.len());
    for row in rows {
        let data = row
            .get("data")
            .ok_or_else(|| "Dòng thiếu trường data".to_string())?;
        let rec: BattleRecord = serde_json::from_value(data.clone())
            .map_err(|e| format!("Giải mã BattleRecord thất bại: {}", e))?;
        records.push(rec);
    }

    let has_more = limit > 0 && rows.len() == limit;
    Ok((records, has_more))
}

pub fn check_legacy_user_vault(
    supabase_url: &str,
    anon_key: &str,
    access_token: &str,
    timeout: Option<Duration>,
) -> Result<Option<Vec<BattleRecord>>, String> {
    let url = format!("{}/rest/v1/user_vaults?select=vault_data", supabase_url.trim_end_matches('/'));
    let effective_timeout = timeout.unwrap_or(Duration::from_secs(15));
    let agent = ureq::builder()
        .timeout_connect(Duration::from_secs(10))
        .timeout(effective_timeout)
        .build();

    let resp = match agent
        .get(&url)
        .set("apikey", anon_key)
        .set("Authorization", &format!("Bearer {}", access_token))
        .call()
    {
        Ok(r) => r,
        Err(ureq::Error::Status(404, _)) => return Ok(None),
        Err(e) => return Err(format_cloud_error(e, "Lỗi kiểm tra legacy user_vaults")),
    };

    let val: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Parse JSON error: {}", e))?;

    let rows = val
        .as_array()
        .ok_or_else(|| "Phản hồi từ cloud không hợp lệ".to_string())?;

    if rows.is_empty() {
        return Ok(None);
    }

    let first_row = &rows[0];
    let vault_data = match first_row.get("vault_data") {
        Some(v) if !v.is_null() => v,
        _ => return Ok(None),
    };

    if let Ok(storage_data) = serde_json::from_value::<crate::vault::VaultStorageData>(vault_data.clone()) {
        if storage_data.records.is_empty() {
            return Ok(None);
        }
        return Ok(Some(storage_data.records));
    }

    if let Ok(records) = serde_json::from_value::<Vec<BattleRecord>>(vault_data.clone()) {
        if records.is_empty() {
            return Ok(None);
        }
        return Ok(Some(records));
    }

    Err("Dữ liệu legacy user_vaults không đúng định dạng".to_string())
}

#[derive(Debug)]
pub enum SyncCommand {
    TriggerDelta,
    SyncNow(Option<mpsc::Sender<Result<usize, String>>>),
    Restore(mpsc::Sender<Result<usize, String>>),
    DeleteRecords(Vec<String>),
    Shutdown(mpsc::Sender<()>),
    UpdateConfig(CloudAuthConfig),
}

#[derive(Debug, Clone, Default)]
pub struct SyncWorkerState {
    pub is_syncing: bool,
    pub is_restoring: bool,
    pub pending_records_count: usize,
    pub total_records_count: usize,
    pub last_sync_time: Option<String>,
    pub last_error: Option<String>,
    pub retry_in_secs: Option<u64>,
    pub new_tokens: Option<(String, String)>,
}

pub struct SyncWorkerHandle {
    pub tx: mpsc::Sender<SyncCommand>,
    pub state: Arc<RwLock<SyncWorkerState>>,
}

pub fn spawn_sync_worker(vault: Arc<MailVault>, config: CloudAuthConfig) -> SyncWorkerHandle {
    let (tx, rx) = mpsc::channel();
    let state = Arc::new(RwLock::new(SyncWorkerState::default()));

    if let Ok((total, unsynced)) = vault.get_sync_counts() {
        if let Ok(mut s) = state.write() {
            s.total_records_count = total;
            s.pending_records_count = unsynced;
        }
    }

    let worker_state = Arc::clone(&state);
    let _ = std::thread::Builder::new()
        .name("cloud-sync-worker".to_string())
        .spawn(move || {
            run_sync_worker_loop(vault, config, rx, worker_state);
        });

    SyncWorkerHandle { tx, state }
}

fn can_sync_credentials(config: &CloudAuthConfig) -> bool {
    let has_token = config.access_token.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false)
        || config.refresh_token.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
    let has_user = config.user_id.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false);
    has_token && has_user
}

fn perform_delta_sync(
    vault: &MailVault,
    config: &mut CloudAuthConfig,
    state: &Arc<RwLock<SyncWorkerState>>,
    timeout: Option<Duration>,
    flush_all: bool,
) -> Result<usize, String> {
    let (url, anon_key) = config.get_credentials();
    let user_id = match &config.user_id {
        Some(uid) if !uid.is_empty() => uid.clone(),
        _ => return Err("Chưa cấu hình user_id tài khoản Cloud".to_string()),
    };
    let mut access_token = match &config.access_token {
        Some(tok) if !tok.is_empty() => tok.clone(),
        _ => {
            if let Some(ref ref_tok) = config.refresh_token {
                if let Ok((new_acc, new_ref)) = refresh_session(&url, &anon_key, ref_tok, timeout) {
                    config.access_token = Some(new_acc.clone());
                    config.refresh_token = Some(new_ref.clone());
                    if let Ok(mut s) = state.write() {
                        s.new_tokens = Some((new_acc.clone(), new_ref));
                    }
                    new_acc
                } else {
                    return Err("Chưa đăng nhập tài khoản Cloud".to_string());
                }
            } else {
                return Err("Chưa đăng nhập tài khoản Cloud".to_string());
            }
        }
    };

    if let Ok(Some(existing_uid)) = vault.get_cloud_user_id() {
        if existing_uid != user_id {
            let err = format!(
                "Tài khoản Cloud hiện tại ({}) không khớp với tài khoản đã đồng bộ trước đó ({})",
                user_id, existing_uid
            );
            if let Ok(mut s) = state.write() {
                s.last_error = Some(err.clone());
            }
            return Err(err);
        }
    } else {
        let _ = vault.set_cloud_user_id(&user_id);
    }

    if let Ok(mut s) = state.write() {
        s.is_syncing = true;
    }

    let mut total_uploaded = 0;
    let batch_size = 200;

    loop {
        let records = match vault.get_unsynced_records(batch_size) {
            Ok(r) => r,
            Err(e) => {
                if let Ok(mut s) = state.write() {
                    s.is_syncing = false;
                }
                return Err(format!("Lỗi truy vấn bản ghi chưa đồng bộ từ SQLite: {}", e));
            }
        };

        if records.is_empty() {
            break;
        }

        let mut upload_res = upload_report_records_batch(
            &url,
            &anon_key,
            &access_token,
            &user_id,
            &records,
            timeout,
        );

        if let Err(ref e) = upload_res {
            if e.contains("Token phiên làm việc đã hết hạn") || e.contains("401") {
                if let Some(ref_tok) = &config.refresh_token {
                    if let Ok((new_acc, new_ref)) = refresh_session(&url, &anon_key, ref_tok, timeout) {
                        access_token = new_acc.clone();
                        config.access_token = Some(new_acc.clone());
                        config.refresh_token = Some(new_ref.clone());
                        if let Ok(mut s) = state.write() {
                            s.new_tokens = Some((new_acc.clone(), new_ref));
                        }

                        upload_res = upload_report_records_batch(
                            &url,
                            &anon_key,
                            &new_acc,
                            &user_id,
                            &records,
                            timeout,
                        );
                    }
                }
            }
        }

        match upload_res {
            Ok(count) => {
                let now_ts = chrono::Utc::now().timestamp();
                let keys: Vec<String> = records.into_iter().map(|(k, _)| k).collect();
                let marked = match vault.mark_records_synced(&keys, now_ts) {
                    Ok(m) => m,
                    Err(e) => {
                        let err_msg = format!("Lỗi cập nhật trạng thái đã đồng bộ: {}", e);
                        if let Ok(mut s) = state.write() {
                            s.is_syncing = false;
                            s.last_error = Some(err_msg.clone());
                        }
                        return Err(err_msg);
                    }
                };
                if marked == 0 {
                    break;
                }
                total_uploaded += count;

                if let Ok((total, unsynced)) = vault.get_sync_counts() {
                    if let Ok(mut s) = state.write() {
                        s.total_records_count = total;
                        s.pending_records_count = unsynced;
                        s.last_sync_time = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                        s.last_error = None;
                        s.retry_in_secs = None;
                    }
                }

                if !flush_all {
                    break;
                }
            }
            Err(e) => {
                if let Ok(mut s) = state.write() {
                    s.is_syncing = false;
                    s.last_error = Some(e.clone());
                }
                return Err(e);
            }
        }
    }

    if let Ok(mut s) = state.write() {
        s.is_syncing = false;
        if let Ok((total, unsynced)) = vault.get_sync_counts() {
            s.total_records_count = total;
            s.pending_records_count = unsynced;
        }
    }

    Ok(total_uploaded)
}

fn perform_restore(
    vault: &MailVault,
    config: &mut CloudAuthConfig,
    state: &Arc<RwLock<SyncWorkerState>>,
) -> Result<usize, String> {
    let (url, anon_key) = config.get_credentials();
    let user_id = match &config.user_id {
        Some(uid) if !uid.is_empty() => uid.clone(),
        _ => return Err("Chưa cấu hình user_id tài khoản Cloud".to_string()),
    };
    let mut access_token = match &config.access_token {
        Some(tok) if !tok.is_empty() => tok.clone(),
        _ => {
            if let Some(ref ref_tok) = config.refresh_token {
                if let Ok((new_acc, new_ref)) = refresh_session(&url, &anon_key, ref_tok, Some(Duration::from_secs(15))) {
                    config.access_token = Some(new_acc.clone());
                    config.refresh_token = Some(new_ref.clone());
                    if let Ok(mut s) = state.write() {
                        s.new_tokens = Some((new_acc.clone(), new_ref));
                    }
                    new_acc
                } else {
                    return Err("Chưa đăng nhập tài khoản Cloud".to_string());
                }
            } else {
                return Err("Chưa đăng nhập tài khoản Cloud".to_string());
            }
        }
    };

    if let Ok(Some(existing_uid)) = vault.get_cloud_user_id() {
        if existing_uid != user_id {
            let err = format!(
                "Tài khoản Cloud hiện tại ({}) không khớp với tài khoản đã đồng bộ trước đó ({})",
                user_id, existing_uid
            );
            if let Ok(mut s) = state.write() {
                s.last_error = Some(err.clone());
            }
            return Err(err);
        }
    } else {
        let _ = vault.set_cloud_user_id(&user_id);
    }

    {
        if let Ok(mut s) = state.write() {
            s.is_syncing = true;
            s.is_restoring = true;
        }
    }

    let mut total_restored = 0;
    let mut offset = 0;
    let limit = 200;
    let now_ts = chrono::Utc::now().timestamp();

    loop {
        let mut page_res = fetch_report_records_page(
            &url,
            &anon_key,
            &access_token,
            &user_id,
            limit,
            offset,
            Some(Duration::from_secs(15)),
        );

        if let Err(ref e) = page_res {
            if e.contains("Token phiên làm việc đã hết hạn") || e.contains("401") {
                if let Some(ref_tok) = &config.refresh_token {
                    if let Ok((new_acc, new_ref)) = refresh_session(&url, &anon_key, ref_tok, Some(Duration::from_secs(15))) {
                        access_token = new_acc.clone();
                        config.access_token = Some(new_acc.clone());
                        config.refresh_token = Some(new_ref.clone());
                        if let Ok(mut s) = state.write() {
                            s.new_tokens = Some((new_acc.clone(), new_ref));
                        }

                        page_res = fetch_report_records_page(
                            &url,
                            &anon_key,
                            &access_token,
                            &user_id,
                            limit,
                            offset,
                            Some(Duration::from_secs(15)),
                        );
                    }
                }
            }
        }

        match page_res {
            Ok((records, has_more)) => {
                let count = records.len();
                if count > 0 {
                    let saved = match vault.restore_records_batch(&records, Some(now_ts)) {
                        Ok(s) => s,
                        Err(e) => {
                            let err_msg = format!("Lỗi ghi dữ liệu khôi phục vào SQLite: {}", e);
                            if let Ok(mut s) = state.write() {
                                s.is_syncing = false;
                                s.is_restoring = false;
                                s.last_error = Some(err_msg.clone());
                            }
                            return Err(err_msg);
                        }
                    };
                    total_restored += saved;
                    offset += count;
                }
                if !has_more {
                    break;
                }
            }
            Err(e) => {
                if let Ok(mut s) = state.write() {
                    s.is_syncing = false;
                    s.is_restoring = false;
                    s.last_error = Some(e.clone());
                }
                return Err(e);
            }
        }
    }

    if total_restored == 0 {
        let mut legacy_res = check_legacy_user_vault(
            &url,
            &anon_key,
            &access_token,
            Some(Duration::from_secs(15)),
        );

        if let Err(ref e) = legacy_res {
            if e.contains("Token phiên làm việc đã hết hạn") || e.contains("401") {
                if let Some(ref_tok) = &config.refresh_token {
                    if let Ok((new_acc, new_ref)) = refresh_session(&url, &anon_key, ref_tok, Some(Duration::from_secs(15))) {
                        access_token = new_acc.clone();
                        config.access_token = Some(new_acc.clone());
                        config.refresh_token = Some(new_ref.clone());
                        if let Ok(mut s) = state.write() {
                            s.new_tokens = Some((new_acc.clone(), new_ref));
                        }

                        legacy_res = check_legacy_user_vault(
                            &url,
                            &anon_key,
                            &access_token,
                            Some(Duration::from_secs(15)),
                        );
                    }
                }
            }
        }

        match legacy_res {
            Ok(Some(legacy_records)) => {
                if !legacy_records.is_empty() {
                    let saved = match vault.restore_records_batch(&legacy_records, None) {
                        Ok(s) => s,
                        Err(e) => {
                            let err_msg = format!("Lỗi ghi dữ liệu legacy vào SQLite: {}", e);
                            if let Ok(mut s) = state.write() {
                                s.is_syncing = false;
                                s.is_restoring = false;
                                s.last_error = Some(err_msg.clone());
                            }
                            return Err(err_msg);
                        }
                    };
                    total_restored = saved;
                }
            }
            Ok(None) => {}
            Err(e) => {
                if let Ok(mut s) = state.write() {
                    s.is_syncing = false;
                    s.is_restoring = false;
                    s.last_error = Some(e.clone());
                }
                return Err(e);
            }
        }
    }

    if let Ok(mut s) = state.write() {
        s.is_syncing = false;
        s.is_restoring = false;
        s.last_error = None;
        s.retry_in_secs = None;
        s.last_sync_time = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        if let Ok((total, unsynced)) = vault.get_sync_counts() {
            s.total_records_count = total;
            s.pending_records_count = unsynced;
        }
    }

    Ok(total_restored)
}

fn apply_sync_backoff(
    res: &Result<usize, String>,
    current_backoff_secs: &mut Option<u64>,
    next_retry_at: &mut Option<Instant>,
    state: &Arc<RwLock<SyncWorkerState>>,
) {
    match res {
        Ok(_) => {
            *current_backoff_secs = None;
            *next_retry_at = None;
            if let Ok(mut s) = state.write() {
                s.retry_in_secs = None;
            }
        }
        Err(e) if is_retryable_error(e) => {
            let next_secs = current_backoff_secs.map(|s| (s * 2).min(60)).unwrap_or(2);
            *current_backoff_secs = Some(next_secs);
            *next_retry_at = Some(Instant::now() + Duration::from_secs(next_secs));
            if let Ok(mut s) = state.write() {
                s.retry_in_secs = Some(next_secs);
            }
        }
        Err(_) => {
            *current_backoff_secs = None;
            *next_retry_at = None;
            if let Ok(mut s) = state.write() {
                s.retry_in_secs = None;
            }
        }
    }
}

fn run_sync_worker_loop(
    vault: Arc<MailVault>,
    mut config: CloudAuthConfig,
    rx: mpsc::Receiver<SyncCommand>,
    state: Arc<RwLock<SyncWorkerState>>,
) {
    let mut debounce_deadline: Option<Instant> = None;
    let mut next_retry_at: Option<Instant> = None;
    let mut current_backoff_secs: Option<u64> = None;
    let mut last_periodic_sweep = Instant::now();

    loop {
        let now = Instant::now();
        let mut tick_timeout = Duration::from_secs(1);

        if let Some(deadline) = debounce_deadline {
            let rem = deadline.saturating_duration_since(now);
            if rem < tick_timeout {
                tick_timeout = rem.max(Duration::from_millis(10));
            }
        }
        if let Some(retry_at) = next_retry_at {
            let rem = retry_at.saturating_duration_since(now);
            if rem < tick_timeout {
                tick_timeout = rem.max(Duration::from_millis(10));
            }
        }

        match rx.recv_timeout(tick_timeout) {
            Ok(SyncCommand::UpdateConfig(new_cfg)) => {
                if new_cfg.access_token.is_none() && new_cfg.refresh_token.is_none() {
                    next_retry_at = None;
                    current_backoff_secs = None;
                    debounce_deadline = None;
                    if let Ok(mut s) = state.write() {
                        s.last_error = None;
                        s.retry_in_secs = None;
                    }
                }
                config = new_cfg;
            }
            Ok(SyncCommand::Shutdown(ack_tx)) => {
                let retry_allowed = next_retry_at.map(|r| Instant::now() >= r).unwrap_or(true);
                if retry_allowed && config.backup_on_exit && can_sync_credentials(&config) {
                    let _ = perform_delta_sync(&vault, &mut config, &state, Some(Duration::from_secs(1)), false);
                }
                let _ = ack_tx.send(());
                break;
            }
            Ok(SyncCommand::SyncNow(reply_tx)) => {
                debounce_deadline = None;
                current_backoff_secs = None;

                let res = if can_sync_credentials(&config) {
                    perform_delta_sync(&vault, &mut config, &state, Some(Duration::from_secs(15)), true)
                } else {
                    let err = "Chưa đăng nhập tài khoản Cloud".to_string();
                    if let Ok(mut s) = state.write() {
                        s.last_error = Some(err.clone());
                        s.retry_in_secs = None;
                    }
                    Err(err)
                };

                apply_sync_backoff(&res, &mut current_backoff_secs, &mut next_retry_at, &state);

                if let Some(tx) = reply_tx {
                    let _ = tx.send(res);
                }
            }
            Ok(SyncCommand::TriggerDelta) => {
                if config.auto_backup && can_sync_credentials(&config) {
                    debounce_deadline = Some(Instant::now() + Duration::from_secs(5));
                }
            }
            Ok(SyncCommand::Restore(reply_tx)) => {
                debounce_deadline = None;
                next_retry_at = None;
                current_backoff_secs = None;

                let res = perform_restore(&vault, &mut config, &state);
                let _ = reply_tx.send(res);
            }
            Ok(SyncCommand::DeleteRecords(keys)) => {
                if can_sync_credentials(&config) {
                    let (url, anon_key) = config.get_credentials();
                    let user_id = config.user_id.clone().unwrap_or_default();
                    let mut access_token = config.access_token.clone().unwrap_or_default();
                    if let Err(ref e) = delete_report_records_batch(&url, &anon_key, &access_token, &user_id, &keys, Some(Duration::from_secs(15))) {
                        if e.contains("Token phiên làm việc đã hết hạn") || e.contains("401") {
                            if let Some(ref_tok) = &config.refresh_token {
                                if let Ok((new_acc, new_ref)) = refresh_session(&url, &anon_key, ref_tok, Some(Duration::from_secs(15))) {
                                    access_token = new_acc.clone();
                                    config.access_token = Some(new_acc.clone());
                                    config.refresh_token = Some(new_ref.clone());
                                    if let Ok(mut s) = state.write() {
                                        s.new_tokens = Some((new_acc.clone(), new_ref));
                                    }
                                    let _ = delete_report_records_batch(&url, &anon_key, &access_token, &user_id, &keys, Some(Duration::from_secs(15)));
                                }
                            }
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let now = Instant::now();

                // 1. Automatic retry when backoff delay expires
                if let Some(retry_at) = next_retry_at {
                    if now >= retry_at {
                        next_retry_at = None;
                        debounce_deadline = None;

                        if can_sync_credentials(&config) {
                            let (_, unsynced) = vault.get_sync_counts().unwrap_or((0, 0));
                            if unsynced > 0 {
                                let res = perform_delta_sync(&vault, &mut config, &state, Some(Duration::from_secs(15)), true);
                                apply_sync_backoff(&res, &mut current_backoff_secs, &mut next_retry_at, &state);
                            } else {
                                current_backoff_secs = None;
                                if let Ok(mut s) = state.write() {
                                    s.retry_in_secs = None;
                                }
                            }
                        }
                    }
                }

                // 2. Debounced delta sync (preserved if in backoff)
                if let Some(deadline) = debounce_deadline {
                    if now >= deadline {
                        let retry_allowed = next_retry_at.map(|r| now >= r).unwrap_or(true);
                        if retry_allowed {
                            debounce_deadline = None;
                            if config.auto_backup && can_sync_credentials(&config) {
                                let res = perform_delta_sync(&vault, &mut config, &state, Some(Duration::from_secs(15)), true);
                                apply_sync_backoff(&res, &mut current_backoff_secs, &mut next_retry_at, &state);
                            }
                        }
                    }
                }

                // 3. Periodic sweep if auto_backup is true
                if config.auto_backup && can_sync_credentials(&config) {
                    let sweep_interval = Duration::from_secs((config.clamped_interval_minutes() as u64) * 60);
                    if last_periodic_sweep.elapsed() >= sweep_interval {
                        last_periodic_sweep = Instant::now();
                        let retry_allowed = next_retry_at.map(|r| now >= r).unwrap_or(true);
                        if retry_allowed {
                            let res = perform_delta_sync(&vault, &mut config, &state, Some(Duration::from_secs(15)), true);
                            apply_sync_backoff(&res, &mut current_backoff_secs, &mut next_retry_at, &state);
                        }
                    }
                }

                // Periodic count update
                if let Ok((total, unsynced)) = vault.get_sync_counts() {
                    if let Ok(mut s) = state.write() {
                        s.total_records_count = total;
                        s.pending_records_count = unsynced;
                        if let Some(retry_at) = next_retry_at {
                            if now < retry_at {
                                s.retry_in_secs = Some(retry_at.duration_since(now).as_secs() + 1);
                            } else {
                                s.retry_in_secs = None;
                            }
                        } else {
                            s.retry_in_secs = None;
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
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
    fn test_start_google_login_cancelled_immediately() {
        let cancel = std::sync::atomic::AtomicBool::new(true);
        let res = start_google_login("https://example.supabase.co", "anon_key", Some(&cancel));
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Đã hủy đăng nhập"));
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
    fn test_refresh_session_accepts_timeout_param() {
        let res = refresh_session(
            "https://127.0.0.1:9",
            "dummy_key",
            "dummy_refresh",
            Some(Duration::from_millis(100)),
        );
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Làm mới phiên thất bại"));
    }

    #[test]
    fn test_build_report_record_payload() {
        let rec = BattleRecord {
            filename: "mail1.txt".into(),
            mail_id: "m123".into(),
            battle_type: "fort".into(),
            time_raw: 1690000000,
            time_utc: "2023-07-22T00:00:00Z".into(),
            time_vn: "2023-07-22 07:00:00".into(),
            target: "Barbarian Fort Level 5".into(),
            target_level: "5".into(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 100,
            plunder_tier: 1,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
            ..Default::default()
        };
        let payload = build_report_record_payload("user_abc", "id:m123", &rec);
        assert_eq!(payload["user_id"], "user_abc");
        assert_eq!(payload["dedup_key"], "id:m123");
        assert_eq!(payload["mail_id"], "m123");
        assert_eq!(payload["battle_type"], "fort");
        assert_eq!(payload["time_raw"], 1690000000);
        assert!(payload["data"].is_object());
    }

    fn dummy_record(id: &str) -> BattleRecord {
        BattleRecord {
            filename: format!("{}.txt", id),
            mail_id: id.into(),
            battle_type: "fort".into(),
            time_raw: 1690000000,
            time_utc: "2023-07-22T00:00:00Z".into(),
            time_vn: "2023-07-22 07:00:00".into(),
            target: "Barbarian Fort Level 5".into(),
            target_level: "5".into(),
            talent_add: 0,
            pos_x: 100,
            pos_y: 200,
            damage_pct: 100,
            plunder_tier: 1,
            items: vec![],
            item_totals: HashMap::new(),
            content: "".into(),
            ..Default::default()
        }
    }

    fn spawn_mock_server<F>(handler: F) -> (String, std::thread::JoinHandle<()>)
    where
        F: FnOnce(&str) -> (u16, &'static str, String) + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut req_bytes = Vec::new();
                let mut buf = [0u8; 1024];
                let mut content_length = 0;
                let mut header_len = None;

                loop {
                    let n = match stream.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(_) => break,
                    };
                    req_bytes.extend_from_slice(&buf[..n]);
                    if header_len.is_none() {
                        if let Some(pos) = req_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                            header_len = Some(pos + 4);
                            let headers = String::from_utf8_lossy(&req_bytes[..pos]);
                            for line in headers.lines() {
                                if line.to_ascii_lowercase().starts_with("content-length:") {
                                    if let Some(val) = line.split(':').nth(1) {
                                        content_length = val.trim().parse::<usize>().unwrap_or(0);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(hlen) = header_len {
                        if req_bytes.len() >= hlen + content_length {
                            break;
                        }
                    }
                }

                let req_str = String::from_utf8_lossy(&req_bytes);
                let (status, status_text, body) = handler(&req_str);
                let resp = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status,
                    status_text,
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
            }
        });
        (url, handle)
    }

    #[test]
    fn test_upload_report_records_batch_empty() {
        let res = upload_report_records_batch(
            "http://127.0.0.1:9",
            "dummy_key",
            "dummy_token",
            "user_1",
            &[],
            Some(Duration::from_millis(50)),
        );
        assert_eq!(res, Ok(0));
    }

    #[test]
    fn test_upload_report_records_batch_success() {
        let (url, handle) = spawn_mock_server(|req| {
            assert!(req.starts_with("POST /rest/v1/report_records?on_conflict=user_id,dedup_key"));
            assert!(req.to_lowercase().contains("prefer: resolution=merge-duplicates"));
            assert!(req.to_lowercase().contains("apikey: test_anon"));
            assert!(req.contains("Bearer test_token"));
            (201, "Created", "[]".to_string())
        });

        let rec1 = dummy_record("m1");
        let rec2 = dummy_record("m2");
        let batch = vec![
            ("id:m1".to_string(), rec1),
            ("id:m2".to_string(), rec2),
        ];

        let res = upload_report_records_batch(
            &url,
            "test_anon",
            "test_token",
            "user_abc",
            &batch,
            Some(Duration::from_secs(2)),
        );
        assert_eq!(res, Ok(2));
        handle.join().unwrap();
    }

    #[test]
    fn test_upload_report_records_batch_404_error() {
        let (url, handle) = spawn_mock_server(|_| {
            (404, "Not Found", r#"{"message":"relation report_records does not exist"}"#.to_string())
        });

        let batch = vec![("id:m1".to_string(), dummy_record("m1"))];
        let res = upload_report_records_batch(
            &url,
            "test_anon",
            "test_token",
            "user_abc",
            &batch,
            Some(Duration::from_secs(2)),
        );
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("Bảng report_records chưa được khởi tạo"), "Got error: {}", err);
        handle.join().unwrap();
    }

    #[test]
    fn test_delete_report_records_batch_empty() {
        let res = delete_report_records_batch(
            "http://127.0.0.1:9",
            "dummy_key",
            "dummy_token",
            "user_1",
            &[],
            Some(Duration::from_millis(50)),
        );
        assert_eq!(res, Ok(0));
    }

    #[test]
    fn test_delete_report_records_batch_success() {
        let (url, handle) = spawn_mock_server(|req| {
            assert!(req.starts_with("DELETE /rest/v1/report_records?user_id=eq.user_abc&dedup_key=in.("));
            assert!(req.to_lowercase().contains("apikey: test_anon"));
            assert!(req.contains("Bearer test_token"));
            (204, "No Content", "".to_string())
        });

        let keys = vec!["id:m1".to_string(), "id:m2".to_string()];
        let res = delete_report_records_batch(
            &url,
            "test_anon",
            "test_token",
            "user_abc",
            &keys,
            Some(Duration::from_secs(2)),
        );
        assert_eq!(res, Ok(2));
        handle.join().unwrap();
    }

    #[test]
    fn test_fetch_report_records_page_success() {
        let rec1 = dummy_record("m1");
        let rec2 = dummy_record("m2");
        let rows_json = serde_json::to_string(&vec![
            serde_json::json!({ "data": rec1 }),
            serde_json::json!({ "data": rec2 }),
        ]).unwrap();

        let (url, handle) = spawn_mock_server(move |req| {
            assert!(req.starts_with("GET /rest/v1/report_records?user_id=eq.user_123&select=data&order=time_raw.asc,dedup_key.asc&limit=2&offset=0"));
            assert!(req.to_lowercase().contains("apikey: test_anon"));
            assert!(req.contains("Bearer test_token"));
            (200, "OK", rows_json)
        });

        let res = fetch_report_records_page(
            &url,
            "test_anon",
            "test_token",
            "user_123",
            2,
            0,
            Some(Duration::from_secs(2)),
        );
        let (records, has_more) = res.expect("Should succeed");
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].mail_id, "m1");
        assert_eq!(records[1].mail_id, "m2");
        assert!(has_more, "When records.len() == limit, has_more should be true");
        handle.join().unwrap();
    }

    #[test]
    fn test_fetch_report_records_page_partial_has_more_false() {
        let rec1 = dummy_record("m1");
        let rows_json = serde_json::to_string(&vec![
            serde_json::json!({ "data": rec1 }),
        ]).unwrap();

        let (url, handle) = spawn_mock_server(move |_| {
            (200, "OK", rows_json)
        });

        let res = fetch_report_records_page(
            &url,
            "test_anon",
            "test_token",
            "user_123",
            5,
            0,
            Some(Duration::from_secs(2)),
        );
        let (records, has_more) = res.expect("Should succeed");
        assert_eq!(records.len(), 1);
        assert!(!has_more, "When records.len() < limit, has_more should be false");
        handle.join().unwrap();
    }

    #[test]
    fn test_fetch_report_records_page_404() {
        let (url, handle) = spawn_mock_server(|_| {
            (404, "Not Found", r#"{"message":"relation not found"}"#.to_string())
        });

        let res = fetch_report_records_page(
            &url,
            "test_anon",
            "test_token",
            "user_123",
            10,
            0,
            Some(Duration::from_secs(2)),
        );
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Bảng report_records chưa được khởi tạo"));
        handle.join().unwrap();
    }

    #[test]
    fn test_check_legacy_user_vault() {
        // Case 1: 404 returns Ok(None)
        let (url404, h404) = spawn_mock_server(|_| {
            (404, "Not Found", "{}".to_string())
        });
        let res = check_legacy_user_vault(&url404, "k", "t", Some(Duration::from_secs(2)));
        assert!(matches!(res, Ok(None)));
        h404.join().unwrap();

        // Case 2: empty array returns Ok(None)
        let (url_empty, h_empty) = spawn_mock_server(|_| {
            (200, "OK", "[]".to_string())
        });
        let res = check_legacy_user_vault(&url_empty, "k", "t", Some(Duration::from_secs(2)));
        assert!(matches!(res, Ok(None)));
        h_empty.join().unwrap();

        // Case 3: legacy data with VaultStorageData
        let rec = dummy_record("m_legacy");
        let storage = crate::vault::VaultStorageData {
            version: 1,
            last_updated: "2026-01-01".into(),
            total_records: 1,
            records: vec![rec],
        };
        let body = serde_json::to_string(&vec![serde_json::json!({
            "vault_data": storage
        })]).unwrap();

        let (url_data, h_data) = spawn_mock_server(move |_| {
            (200, "OK", body)
        });
        let res = check_legacy_user_vault(&url_data, "k", "t", Some(Duration::from_secs(2)));
        let records = res.expect("Should succeed").expect("Should have records");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].mail_id, "m_legacy");
        h_data.join().unwrap();
    }

    #[test]
    fn test_sync_worker_spawn_and_shutdown() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_worker_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let vault = std::sync::Arc::new(crate::vault::MailVault::new(&temp_dir));
        let cfg = crate::model::CloudAuthConfig::default();
        let worker = spawn_sync_worker(vault, cfg);

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    fn spawn_multi_mock_server<F>(mut handler: F, count: usize) -> (String, std::thread::JoinHandle<()>)
    where
        F: FnMut(&str) -> (u16, &'static str, String) + Send + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);
        let handle = std::thread::spawn(move || {
            for _ in 0..count {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut req_bytes = Vec::new();
                    let mut buf = [0u8; 1024];
                    let mut content_length = 0;
                    let mut header_len = None;

                    loop {
                        let n = match stream.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => n,
                            Err(_) => break,
                        };
                        req_bytes.extend_from_slice(&buf[..n]);
                        if header_len.is_none() {
                            if let Some(pos) = req_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                                header_len = Some(pos + 4);
                                let headers = String::from_utf8_lossy(&req_bytes[..pos]);
                                for line in headers.lines() {
                                    if line.to_ascii_lowercase().starts_with("content-length:") {
                                        if let Some(val) = line.split(':').nth(1) {
                                            content_length = val.trim().parse::<usize>().unwrap_or(0);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(hlen) = header_len {
                            if req_bytes.len() >= hlen + content_length {
                                break;
                            }
                        }
                    }

                    let req_str = String::from_utf8_lossy(&req_bytes);
                    let (status, status_text, body) = handler(&req_str);
                    let resp = format!(
                        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        status,
                        status_text,
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    let _ = stream.flush();
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                }
            }
        });
        (url, handle)
    }

    #[test]
    fn test_sync_worker_sync_now_success() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_worker_now_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.upsert(dummy_record("m_sync_1"));
        vault_inst.upsert(dummy_record("m_sync_2"));
        let vault = std::sync::Arc::new(vault_inst);

        let (url, handle) = spawn_multi_mock_server(|req| {
            assert!(req.starts_with("POST /rest/v1/report_records?on_conflict=user_id,dedup_key"));
            (201, "Created", "[]".to_string())
        }, 1);

        let cfg = crate::model::CloudAuthConfig {
            supabase_url: Some(url),
            anon_key: Some("test_anon".into()),
            access_token: Some("test_tok".into()),
            user_id: Some("user_sync_test".into()),
            ..Default::default()
        };

        let worker = spawn_sync_worker(vault.clone(), cfg);
        assert_eq!(worker.state.read().unwrap().pending_records_count, 2);

        worker.tx.send(SyncCommand::SyncNow(None)).unwrap();

        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if worker.state.read().unwrap().pending_records_count == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert_eq!(worker.state.read().unwrap().pending_records_count, 0);
        assert!(worker.state.read().unwrap().last_sync_time.is_some());
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 0);

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_debounce_trigger_delta() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_debounce_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.upsert(dummy_record("m_deb"));
        let vault = std::sync::Arc::new(vault_inst);

        let (url, handle) = spawn_multi_mock_server(|req| {
            assert!(req.starts_with("POST /rest/v1/report_records"));
            (201, "Created", "[]".to_string())
        }, 1);

        let cfg = crate::model::CloudAuthConfig {
            supabase_url: Some(url),
            anon_key: Some("test_anon".into()),
            access_token: Some("test_tok".into()),
            user_id: Some("user_deb_test".into()),
            auto_backup: true,
            backup_on_exit: false,
            ..Default::default()
        };

        let worker = spawn_sync_worker(vault.clone(), cfg);
        worker.tx.send(SyncCommand::TriggerDelta).unwrap();

        // After 200ms, 5s debounce has not elapsed, still pending
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert_eq!(worker.state.read().unwrap().pending_records_count, 1);
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 1);

        // Explicit SyncNow flushes immediately without waiting for 5s debounce
        worker.tx.send(SyncCommand::SyncNow(None)).unwrap();

        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if worker.state.read().unwrap().pending_records_count == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert_eq!(worker.state.read().unwrap().pending_records_count, 0);

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_restore_success() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_worker_res_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let vault = std::sync::Arc::new(crate::vault::MailVault::new(&temp_dir));

        let rec1 = dummy_record("m_res_1");
        let rec2 = dummy_record("m_res_2");
        let body = serde_json::to_string(&vec![
            serde_json::json!({ "data": rec1 }),
            serde_json::json!({ "data": rec2 }),
        ]).unwrap();

        let (url, handle) = spawn_multi_mock_server(move |req| {
            assert!(req.starts_with("GET /rest/v1/report_records?user_id=eq.user_res_test"));
            (200, "OK", body.clone())
        }, 1);

        let cfg = crate::model::CloudAuthConfig {
            supabase_url: Some(url),
            anon_key: Some("test_anon".into()),
            access_token: Some("test_tok".into()),
            user_id: Some("user_res_test".into()),
            ..Default::default()
        };

        let worker = spawn_sync_worker(vault.clone(), cfg);
        let (res_tx, res_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Restore(res_tx)).unwrap();

        let outcome = res_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(outcome, Ok(2));

        assert_eq!(vault.get_records().len(), 2);
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 0);

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_restore_legacy_fallback() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_worker_leg_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let vault = std::sync::Arc::new(crate::vault::MailVault::new(&temp_dir));

        let leg_rec = dummy_record("m_legacy_1");
        let leg_body = serde_json::to_string(&vec![
            serde_json::json!({
                "vault_data": {
                    "version": 1,
                    "last_updated": "2026-01-01",
                    "total_records": 1,
                    "records": [leg_rec]
                }
            })
        ]).unwrap();

        let (url, handle) = spawn_multi_mock_server(move |req| {
            if req.starts_with("GET /rest/v1/report_records") {
                (200, "OK", "[]".to_string())
            } else if req.starts_with("GET /rest/v1/user_vaults") {
                (200, "OK", leg_body.clone())
            } else {
                (404, "Not Found", "".to_string())
            }
        }, 2);

        let mut cfg = crate::model::CloudAuthConfig::default();
        cfg.supabase_url = Some(url);
        cfg.anon_key = Some("test_anon".into());
        cfg.access_token = Some("test_tok".into());
        cfg.user_id = Some("user_legacy_test".into());
        cfg.backup_on_exit = false;

        let worker = spawn_sync_worker(vault.clone(), cfg);
        let (res_tx, res_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Restore(res_tx)).unwrap();

        let outcome = res_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(outcome, Ok(1));
        assert_eq!(vault.get_records().len(), 1);
        assert_eq!(vault.get_records()[0].mail_id, "m_legacy_1");
        let (total, unsynced) = vault.get_sync_counts().unwrap();
        assert_eq!(total, 1);
        assert_eq!(unsynced, 1);

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_auto_refresh_on_401() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_worker_401_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.upsert(dummy_record("m_401"));
        let vault = std::sync::Arc::new(vault_inst);

        let (url, handle) = spawn_multi_mock_server(|req| {
            if req.starts_with("POST /rest/v1/report_records") && req.contains("Bearer expired_tok") {
                (401, "Unauthorized", r#"{"message":"JWT expired"}"#.to_string())
            } else if req.starts_with("POST /auth/v1/token") {
                (200, "OK", r#"{"access_token":"refreshed_tok","refresh_token":"refreshed_rt"}"#.to_string())
            } else if req.starts_with("POST /rest/v1/report_records") && req.contains("Bearer refreshed_tok") {
                (201, "Created", "[]".to_string())
            } else {
                (500, "Error", "".to_string())
            }
        }, 3);

        let mut cfg = crate::model::CloudAuthConfig::default();
        cfg.supabase_url = Some(url);
        cfg.anon_key = Some("test_anon".into());
        cfg.access_token = Some("expired_tok".into());
        cfg.refresh_token = Some("valid_rt".into());
        cfg.user_id = Some("user_401_test".into());

        let worker = spawn_sync_worker(vault.clone(), cfg);
        worker.tx.send(SyncCommand::SyncNow(None)).unwrap();

        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if worker.state.read().unwrap().pending_records_count == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert_eq!(worker.state.read().unwrap().pending_records_count, 0);
        assert!(worker.state.read().unwrap().last_error.is_none());
        assert_eq!(
            worker.state.read().unwrap().new_tokens,
            Some(("refreshed_tok".to_string(), "refreshed_rt".to_string()))
        );

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_account_mismatch() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_mismatch_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.set_cloud_user_id("account_A").unwrap();
        let vault = std::sync::Arc::new(vault_inst);

        let mut cfg = crate::model::CloudAuthConfig::default();
        cfg.supabase_url = Some("http://127.0.0.1:9".into());
        cfg.anon_key = Some("anon".into());
        cfg.access_token = Some("tok".into());
        cfg.user_id = Some("account_B".into());

        let worker = spawn_sync_worker(vault, cfg);
        worker.tx.send(SyncCommand::SyncNow(None)).unwrap();

        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if worker.state.read().unwrap().last_error.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        let err = worker.state.read().unwrap().last_error.clone().unwrap();
        assert!(err.contains("không khớp với tài khoản đã đồng bộ trước đó"), "Err was: {}", err);

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_retry_backoff_on_network_failure() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_backoff_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.upsert(dummy_record("m_fail"));
        let vault = std::sync::Arc::new(vault_inst);

        let (url, handle) = spawn_multi_mock_server(|_| {
            (500, "Internal Server Error", r#"{"message":"Database down"}"#.to_string())
        }, 1);

        let mut cfg = crate::model::CloudAuthConfig::default();
        cfg.supabase_url = Some(url);
        cfg.anon_key = Some("test_anon".into());
        cfg.access_token = Some("test_tok".into());
        cfg.user_id = Some("user_fail_test".into());

        let worker = spawn_sync_worker(vault, cfg);
        worker.tx.send(SyncCommand::SyncNow(None)).unwrap();

        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if worker.state.read().unwrap().last_error.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert!(worker.state.read().unwrap().last_error.is_some());
        assert_eq!(worker.state.read().unwrap().retry_in_secs, Some(2));

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_auto_retry_fires_after_backoff() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_autoretry_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.upsert(dummy_record("m_autoretry"));
        let vault = std::sync::Arc::new(vault_inst);

        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let attempts_clone = attempts.clone();
        let (url, handle) = spawn_multi_mock_server(move |req| {
            if req.starts_with("POST /rest/v1/report_records") {
                let attempt = attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if attempt == 0 {
                    (500, "Internal Server Error", r#"{"message":"Database down"}"#.to_string())
                } else {
                    (201, "Created", "[]".to_string())
                }
            } else {
                (404, "Not Found", "".to_string())
            }
        }, 2);

        let mut cfg = crate::model::CloudAuthConfig::default();
        cfg.supabase_url = Some(url);
        cfg.anon_key = Some("test_anon".into());
        cfg.access_token = Some("test_tok".into());
        cfg.user_id = Some("user_retry_test".into());

        let worker = spawn_sync_worker(vault.clone(), cfg);
        worker.tx.send(SyncCommand::SyncNow(None)).unwrap();

        // 1. Initial attempt fails
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if worker.state.read().unwrap().last_error.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert!(worker.state.read().unwrap().last_error.is_some());
        assert_eq!(worker.state.read().unwrap().retry_in_secs, Some(2));
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 1);

        // 2. Wait for the 2-second backoff to expire; auto-retry fires and succeeds
        let retry_start = std::time::Instant::now();
        while retry_start.elapsed() < std::time::Duration::from_secs(4) {
            if worker.state.read().unwrap().pending_records_count == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        assert_eq!(worker.state.read().unwrap().pending_records_count, 0);
        assert!(worker.state.read().unwrap().last_error.is_none());
        assert_eq!(worker.state.read().unwrap().retry_in_secs, None);
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 0);

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_update_config() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_update_cfg_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.upsert(dummy_record("m_cfg_upd"));
        let vault = std::sync::Arc::new(vault_inst);

        let (url, handle) = spawn_multi_mock_server(|req| {
            assert!(req.starts_with("POST /rest/v1/report_records"));
            (201, "Created", "[]".to_string())
        }, 1);

        // Spawn with unconfigured credentials
        let worker = spawn_sync_worker(vault.clone(), crate::model::CloudAuthConfig::default());
        worker.tx.send(SyncCommand::SyncNow(None)).unwrap();

        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if worker.state.read().unwrap().last_error.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(worker.state.read().unwrap().last_error.as_deref(), Some("Chưa đăng nhập tài khoản Cloud"));

        // Update config with valid credentials
        let mut new_cfg = crate::model::CloudAuthConfig::default();
        new_cfg.supabase_url = Some(url);
        new_cfg.anon_key = Some("test_anon".into());
        new_cfg.access_token = Some("test_tok".into());
        new_cfg.user_id = Some("user_upd_test".into());

        worker.tx.send(SyncCommand::UpdateConfig(new_cfg)).unwrap();
        worker.tx.send(SyncCommand::SyncNow(None)).unwrap();

        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            if worker.state.read().unwrap().pending_records_count == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert_eq!(worker.state.read().unwrap().pending_records_count, 0);
        assert!(worker.state.read().unwrap().last_error.is_none());

        // Test logout resets backoff and last_error
        if let Ok(mut s) = worker.state.write() {
            s.last_error = Some("Lỗi tạm thời".to_string());
            s.retry_in_secs = Some(30);
        }
        let logout_cfg = crate::model::CloudAuthConfig::default();
        worker.tx.send(SyncCommand::UpdateConfig(logout_cfg)).unwrap();
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(1) {
            if worker.state.read().unwrap().last_error.is_none() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(worker.state.read().unwrap().last_error.is_none());
        assert!(worker.state.read().unwrap().retry_in_secs.is_none());

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_worker_restore_legacy_error_propagates() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_leg_err_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let vault = std::sync::Arc::new(crate::vault::MailVault::new(&temp_dir));

        let (url, handle) = spawn_multi_mock_server(|req| {
            if req.starts_with("GET /rest/v1/report_records") {
                (200, "OK", "[]".to_string())
            } else if req.starts_with("GET /rest/v1/user_vaults") {
                (500, "Internal Server Error", r#"{"message":"Corrupt table"}"#.to_string())
            } else {
                (404, "Not Found", "".to_string())
            }
        }, 2);

        let mut cfg = crate::model::CloudAuthConfig::default();
        cfg.supabase_url = Some(url);
        cfg.anon_key = Some("test_anon".into());
        cfg.access_token = Some("test_tok".into());
        cfg.user_id = Some("user_leg_err".into());

        let worker = spawn_sync_worker(vault, cfg);
        let (res_tx, res_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Restore(res_tx)).unwrap();

        let outcome = res_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert!(outcome.is_err(), "Expected error on 500 legacy restore");
        assert!(!worker.state.read().unwrap().is_syncing);
        assert!(!worker.state.read().unwrap().is_restoring);
        assert!(worker.state.read().unwrap().last_error.is_some());

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_is_retryable_error_classification() {
        assert!(!is_retryable_error("Bị từ chối (HTTP 403) - permission denied for table report_records"));
        assert!(!is_retryable_error("HTTP 401 Unauthorized: token expired"));
        assert!(!is_retryable_error("Không tìm thấy bảng dữ liệu (HTTP 404)."));
        assert!(!is_retryable_error("Chưa đăng nhập tài khoản Cloud"));
        assert!(!is_retryable_error("Tài khoản Cloud hiện tại không khớp"));
        assert!(!is_retryable_error("Lỗi SQLite: table locked"));

        assert!(is_retryable_error("Lỗi kết nối mạng: Connection timed out"));
        assert!(is_retryable_error("ureq transport error: network down"));
        assert!(is_retryable_error("Lỗi máy chủ Cloud (HTTP 500)"));
        assert!(is_retryable_error("Lỗi máy chủ Cloud (HTTP 502)"));
        assert!(is_retryable_error("Lỗi máy chủ Cloud (HTTP 503)"));
        assert!(is_retryable_error("HTTP 429 Too Many Requests"));
    }

    #[test]
    fn test_sync_now_with_reply_channel_sends_result() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_now_reply_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.upsert(dummy_record("m_sync_now_1"));
        let vault = std::sync::Arc::new(vault_inst);

        let (url, handle) = spawn_multi_mock_server(|req| {
            assert!(req.starts_with("POST /rest/v1/report_records"));
            (201, "Created", "[]".to_string())
        }, 1);

        let cfg = crate::model::CloudAuthConfig {
            supabase_url: Some(url),
            anon_key: Some("test_anon".into()),
            access_token: Some("test_tok".into()),
            user_id: Some("user_reply_test".into()),
            backup_on_exit: false,
            ..Default::default()
        };

        let worker = spawn_sync_worker(vault.clone(), cfg);
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::SyncNow(Some(reply_tx))).unwrap();

        let outcome = reply_rx.recv_timeout(std::time::Duration::from_secs(3));
        assert!(outcome.is_ok(), "SyncNow must reply on provided channel");
        let count = outcome.unwrap().expect("SyncNow must succeed");
        assert_eq!(count, 1);
        assert_eq!(vault.get_unsynced_records(10).unwrap().len(), 0);

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sync_now_non_retryable_error_does_not_loop_backoff() {
        let temp_dir = std::env::temp_dir().join(format!("rok_test_sync_403_stop_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mut vault_inst = crate::vault::MailVault::new(&temp_dir);
        vault_inst.upsert(dummy_record("m_sync_403"));
        let vault = std::sync::Arc::new(vault_inst);

        let (url, handle) = spawn_multi_mock_server(|req| {
            if req.starts_with("POST /rest/v1/report_records") {
                (403, "Forbidden", r#"{"message":"permission denied for table report_records"}"#.to_string())
            } else {
                (404, "Not Found", "".to_string())
            }
        }, 1);

        let cfg = crate::model::CloudAuthConfig {
            supabase_url: Some(url),
            anon_key: Some("test_anon".into()),
            access_token: Some("test_tok".into()),
            user_id: Some("user_403_test".into()),
            backup_on_exit: false,
            ..Default::default()
        };

        let worker = spawn_sync_worker(vault.clone(), cfg);
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::SyncNow(Some(reply_tx))).unwrap();

        let outcome = reply_rx.recv_timeout(std::time::Duration::from_secs(3)).expect("Must reply within timeout");
        assert!(outcome.is_err(), "Must report 403 error");
        let err_msg = outcome.unwrap_err();
        assert!(err_msg.contains("403"));

        // Must NOT schedule retry backoff
        assert_eq!(worker.state.read().unwrap().retry_in_secs, None);
        assert!(worker.state.read().unwrap().last_error.is_some());

        let (ack_tx, ack_rx) = std::sync::mpsc::channel();
        worker.tx.send(SyncCommand::Shutdown(ack_tx)).unwrap();
        assert!(ack_rx.recv_timeout(std::time::Duration::from_secs(2)).is_ok());
        handle.join().unwrap();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_upload_and_fetch_user_settings() {
        let (url, handle) = spawn_multi_mock_server(|req| {
            if req.starts_with("PUT /auth/v1/user") {
                assert!(req.contains(r#""main_account_uids":["987654"]"#));
                (200, "OK", r#"{"id":"user_test"}"#.to_string())
            } else if req.starts_with("GET /auth/v1/user") {
                (200, "OK", r#"{"id":"user_test","email":"test@example.com","user_metadata":{"rok_settings":{"main_account_uids":["987654"]}}}"#.to_string())
            } else {
                (404, "Not Found", "".to_string())
            }
        }, 2);

        let settings = crate::model::UserCloudSettings {
            main_account_uids: vec!["987654".to_string()],
            ..Default::default()
        };

        let up_res = upload_user_settings(&url, "anon_key", "test_tok", &settings, Some(Duration::from_secs(2)));
        assert!(up_res.is_ok());

        let fetched = fetch_user_settings(&url, "anon_key", "test_tok", Some(Duration::from_secs(2))).unwrap();
        assert_eq!(fetched, Some(settings));
        handle.join().unwrap();
    }
}
