#![allow(dead_code, unused_imports, unused_variables)]

#[path = "../src/model.rs"]
mod model;
#[path = "../src/vault_migrate.rs"]
mod vault_migrate;
#[path = "../src/lilith_stream.rs"]
mod lilith_stream;
#[path = "../src/vault.rs"]
mod vault;
#[path = "../src/cloud_sync.rs"]
mod cloud_sync;

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use model::{BattleRecord, CloudAuthConfig};
use vault::MailVault;
use cloud_sync::{spawn_sync_worker, SyncCommand};

fn create_test_record(id: usize, base_time: i64) -> BattleRecord {
    BattleRecord {
        filename: format!("Persistent.Mail.{:06}", id),
        mail_id: format!("mail_{:06}", id),
        battle_type: if id % 2 == 0 { "fort".to_string() } else { "barbarian".to_string() },
        time_raw: base_time + id as i64,
        time_utc: format!("2026-11-01T{:02}:{:02}:00Z", (id / 60) % 24, id % 60),
        time_vn: format!("2026-11-01 {:02}:{:02}:00", ((id / 60) + 7) % 24, id % 60),
        target: format!("Barbarian Fort Level {}", (id % 10) + 1),
        target_level: format!("{}", (id % 10) + 1),
        talent_add: 0,
        pos_x: 100 + (id as i32 % 500),
        pos_y: 200 + (id as i32 % 500),
        damage_pct: 100,
        plunder_tier: 1,
        items: vec![],
        item_totals: HashMap::new(),
        content: format!("Combat report #{}", id),
    }
}

fn create_unique_temp_dir(tag: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("rok_test_{}_{}_{}", tag, std::process::id(), nonce));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn spawn_mock_postgrest_server<F>(mut handler: F, count: usize) -> (String, std::thread::JoinHandle<()>)
where
    F: FnMut(&str) -> (u16, &'static str, String) + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind mock listener");
    let addr = listener.local_addr().expect("failed to get local addr");
    let url = format!("http://{}", addr);

    let handle = std::thread::spawn(move || {
        for _ in 0..count {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut req_bytes = Vec::new();
                let mut buf = [0u8; 8192];
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

/// Task 6 Step 1:
/// - Create mock local vault with 500 battle records.
/// - Verify `get_unsynced_records(200)` returns 200 records.
/// - Verify `mark_records_synced` leaves 300 unsynced.
/// - Verify second batch leaves 100 unsynced.
/// - Verify third batch leaves 0 unsynced.
/// - Verify `Shutdown` completes cleanly within timeout.
#[test]
fn test_per_record_sync_500_records_batch_paging_and_shutdown() {
    let temp_dir = create_unique_temp_dir("per_record_500");
    let mut vault = MailVault::new(&temp_dir);

    // 1. Insert 500 battle records
    let records: Vec<BattleRecord> = (0..500)
        .map(|i| create_test_record(i, 1_700_000_000))
        .collect();
    let inserted = vault.upsert_batch(records);
    assert_eq!(inserted, 500, "All 500 records should be inserted");

    let (total_init, unsynced_init) = vault.get_sync_counts().expect("failed to get initial sync counts");
    assert_eq!(total_init, 500);
    assert_eq!(unsynced_init, 500, "All 500 records must initially be unsynced");

    // 2. Batch 1: Query 200 records
    let batch1 = vault.get_unsynced_records(200).expect("failed to query batch 1");
    assert_eq!(batch1.len(), 200, "Batch 1 must return exactly 200 records");

    // Verify ordering by time_raw ASC
    for w in batch1.windows(2) {
        assert!(
            w[0].1.time_raw <= w[1].1.time_raw,
            "Records must be strictly ordered by time_raw ASC: {} <= {}",
            w[0].1.time_raw,
            w[1].1.time_raw
        );
    }

    // 3. Mark Batch 1 as synced; leaves 300 unsynced
    let keys1: Vec<String> = batch1.iter().map(|(k, _)| k.clone()).collect();
    let marked1 = vault.mark_records_synced(&keys1, 1_700_000_200).expect("failed to mark batch 1");
    assert_eq!(marked1, 200, "Should have marked 200 records as synced in batch 1");

    let (total1, unsynced1) = vault.get_sync_counts().expect("failed to query sync counts after batch 1");
    assert_eq!(total1, 500);
    assert_eq!(unsynced1, 300, "Marking first batch must leave exactly 300 unsynced records");

    // 4. Batch 2: Query next 200 records; leaves 100 unsynced
    let batch2 = vault.get_unsynced_records(200).expect("failed to query batch 2");
    assert_eq!(batch2.len(), 200, "Batch 2 must return exactly 200 records");

    // Verify no overlap between batch 1 and batch 2
    for (k, _) in &batch2 {
        assert!(!keys1.contains(k), "Batch 2 must not contain records from Batch 1: {}", k);
    }

    let keys2: Vec<String> = batch2.iter().map(|(k, _)| k.clone()).collect();
    let marked2 = vault.mark_records_synced(&keys2, 1_700_000_400).expect("failed to mark batch 2");
    assert_eq!(marked2, 200, "Should have marked 200 records as synced in batch 2");

    let (total2, unsynced2) = vault.get_sync_counts().expect("failed to query sync counts after batch 2");
    assert_eq!(total2, 500);
    assert_eq!(unsynced2, 100, "Marking second batch must leave exactly 100 unsynced records");

    // 5. Batch 3: Query remaining records; leaves 0 unsynced
    let batch3 = vault.get_unsynced_records(200).expect("failed to query batch 3");
    assert_eq!(batch3.len(), 100, "Batch 3 must return remaining 100 records");

    for (k, _) in &batch3 {
        assert!(!keys1.contains(k) && !keys2.contains(k), "Batch 3 must not contain keys from earlier batches: {}", k);
    }

    let keys3: Vec<String> = batch3.iter().map(|(k, _)| k.clone()).collect();
    let marked3 = vault.mark_records_synced(&keys3, 1_700_000_500).expect("failed to mark batch 3");
    assert_eq!(marked3, 100, "Should have marked remaining 100 records as synced in batch 3");

    let (total3, unsynced3) = vault.get_sync_counts().expect("failed to query sync counts after batch 3");
    assert_eq!(total3, 500);
    assert_eq!(unsynced3, 0, "Marking third batch must leave exactly 0 unsynced records");

    let empty_check = vault.get_unsynced_records(200).expect("failed to check empty unsynced");
    assert!(empty_check.is_empty(), "get_unsynced_records should return empty vec when all are synced");

    // 6. Verify Shutdown completes cleanly within timeout
    let vault_arc = Arc::new(vault);
    let cfg = CloudAuthConfig::default();
    let worker = spawn_sync_worker(vault_arc, cfg);

    let (ack_tx, ack_rx) = std::sync::mpsc::channel();
    let shutdown_start = Instant::now();
    worker.tx.send(SyncCommand::Shutdown(ack_tx)).expect("failed to send Shutdown command");

    let shutdown_ack = ack_rx.recv_timeout(Duration::from_secs(2));
    let elapsed = shutdown_start.elapsed();

    assert!(shutdown_ack.is_ok(), "Worker shutdown failed to acknowledge within 2-second timeout");
    assert!(elapsed < Duration::from_secs(2), "Shutdown exceeded 2 seconds: {:?}", elapsed);

    let _ = fs::remove_dir_all(&temp_dir);
}

/// End-to-end background worker test streaming 500 records across 3 batches (200 + 200 + 100)
/// to a mock PostgREST endpoint, verifying headers, payloads, SQLite state progression, and shutdown.
#[test]
fn test_worker_sync_now_500_records_e2e_streaming() {
    let temp_dir = create_unique_temp_dir("worker_e2e_500");
    let mut vault_inst = MailVault::new(&temp_dir);

    // Insert 500 records into local SQLite
    let records: Vec<BattleRecord> = (0..500)
        .map(|i| create_test_record(i, 1_710_000_000))
        .collect();
    vault_inst.upsert_batch(records);
    let vault = Arc::new(vault_inst);

    let batch_counts = Arc::new(Mutex::new(Vec::new()));
    let batch_counts_clone = Arc::clone(&batch_counts);

    // Mock PostgREST server expecting exactly 3 batch requests
    let (url, server_handle) = spawn_mock_postgrest_server(move |req| {
        // Verify PostgREST URL path & query
        assert!(
            req.starts_with("POST /rest/v1/report_records?on_conflict=user_id,dedup_key"),
            "Unexpected request path: {}",
            req.lines().next().unwrap_or("")
        );
        // Verify merge-duplicates resolution header
        assert!(
            req.contains("prefer: resolution=merge-duplicates") || req.contains("Prefer: resolution=merge-duplicates"),
            "Missing Prefer resolution header"
        );
        // Verify Authorization token header
        assert!(
            req.contains("authorization: Bearer test_e2e_token") || req.contains("Authorization: Bearer test_e2e_token"),
            "Missing or incorrect Bearer token"
        );

        // Find and parse JSON array body
        if let Some(body_start) = req.find("\r\n\r\n") {
            let body = &req[body_start + 4..];
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(body) {
                if let Some(arr) = json_val.as_array() {
                    batch_counts_clone.lock().unwrap().push(arr.len());
                }
            }
        }

        (201, "Created", "[]".to_string())
    }, 3);

    let mut cfg = CloudAuthConfig::default();
    cfg.supabase_url = Some(url);
    cfg.anon_key = Some("anon_test_key".to_string());
    cfg.access_token = Some("test_e2e_token".to_string());
    cfg.refresh_token = Some("test_refresh_token".to_string());
    cfg.user_id = Some("usr_scale_500".to_string());

    let worker = spawn_sync_worker(vault.clone(), cfg);

    // Initially 500 records pending
    assert_eq!(worker.state.read().unwrap().pending_records_count, 500);

    // Trigger instant sync
    worker.tx.send(SyncCommand::SyncNow(None)).expect("failed to trigger SyncNow");

    // Wait for worker to finish streaming all 500 records (up to 5 seconds)
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if worker.state.read().unwrap().pending_records_count == 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(30));
    }

    assert_eq!(
        worker.state.read().unwrap().pending_records_count,
        0,
        "Worker did not reach 0 pending records within deadline"
    );
    assert!(worker.state.read().unwrap().last_sync_time.is_some(), "last_sync_time should be set");

    // Verify local SQLite sync counts
    let (total, unsynced) = vault.get_sync_counts().expect("failed to get sync counts");
    assert_eq!(total, 500);
    assert_eq!(unsynced, 0, "All 500 records must be marked synced in SQLite");

    // Verify mock server received batches of 200, 200, 100
    let recorded_batches = batch_counts.lock().unwrap().clone();
    assert_eq!(
        recorded_batches,
        vec![200, 200, 100],
        "Mock PostgREST must receive exactly 3 batches of sizes [200, 200, 100]"
    );

    // Verify clean shutdown within 2s
    let (ack_tx, ack_rx) = std::sync::mpsc::channel();
    let shutdown_start = Instant::now();
    worker.tx.send(SyncCommand::Shutdown(ack_tx)).expect("failed to send Shutdown");
    assert!(ack_rx.recv_timeout(Duration::from_secs(2)).is_ok(), "Shutdown timed out");
    assert!(shutdown_start.elapsed() < Duration::from_secs(2));

    server_handle.join().expect("server thread failed to join");
    let _ = fs::remove_dir_all(&temp_dir);
}

/// Verify that worker Shutdown completes bounded within 2 seconds even when the cloud endpoint
/// is offline / unreachable and fails connections immediately.
#[test]
fn test_worker_shutdown_bounded_when_cloud_offline() {
    let temp_dir = create_unique_temp_dir("shutdown_offline");
    let mut vault_inst = MailVault::new(&temp_dir);

    // Insert 50 records
    let records: Vec<BattleRecord> = (0..50)
        .map(|i| create_test_record(i, 1_720_000_000))
        .collect();
    vault_inst.upsert_batch(records);
    let vault = Arc::new(vault_inst);

    // Bind and immediately close an ephemeral port so it is closed/unreachable
    let dummy_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let dummy_port = dummy_listener.local_addr().unwrap().port();
    drop(dummy_listener);

    let mut cfg = CloudAuthConfig::default();
    cfg.supabase_url = Some(format!("http://127.0.0.1:{}", dummy_port));
    cfg.anon_key = Some("anon_test_key".to_string());
    cfg.access_token = Some("dummy_token".to_string());
    cfg.user_id = Some("usr_offline_test".to_string());

    let worker = spawn_sync_worker(vault.clone(), cfg);

    // Send shutdown
    let (ack_tx, ack_rx) = std::sync::mpsc::channel();
    let shutdown_start = Instant::now();
    worker.tx.send(SyncCommand::Shutdown(ack_tx)).expect("failed to send Shutdown");

    let ack_result = ack_rx.recv_timeout(Duration::from_secs(3));
    let elapsed = shutdown_start.elapsed();

    assert!(ack_result.is_ok(), "Shutdown must acknowledge even when offline");
    assert!(
        elapsed < Duration::from_millis(2500),
        "Shutdown took {:?}, must remain strictly bounded around the 2s timeout",
        elapsed
    );

    // Database must remain intact and unsynced records must NOT be corrupted
    let (total, unsynced) = vault.get_sync_counts().expect("vault query after shutdown failed");
    assert_eq!(total, 50);
    assert_eq!(unsynced, 50, "Unsynced records must remain safely stored in SQLite WAL");

    let _ = fs::remove_dir_all(&temp_dir);
}
