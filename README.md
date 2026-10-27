# RoK Loot Watcher (Rust)

Ứng dụng Windows hiệu năng cao theo dõi thư mục `mailcache` Rise of Kingdoms theo thời gian thực, tự động giải mã cấu trúc nhị phân Lilith, lưu trữ vault lịch sử và thống kê phần thưởng chiến báo (Motte, Pháo đài, Dã man, Thu thập tài nguyên).

---

## Tính Năng Nổi Bật

- **Real-time Mail Watcher:** Giám sát thư mục `mailcache` với cơ chế chia sẻ tệp Windows (`FILE_SHARE_DELETE`), tránh xung đột khóa file khi game ghi đè hoặc dọn dẹp chiến báo trong lúc tham gia rally.
- **In-Game Overlay:** Lớp phủ nổi trong suốt thời gian thực hiển thị 5 chiến báo mới nhất trực tiếp trên cửa sổ game Rise of Kingdoms, hỗ trợ click-through và phím điều hướng D-pad.
- **Bảng Thống Kê & Báo Cáo:** Tra cứu, lọc theo ngày/loại trận đánh, tìm kiếm và sắp xếp đa cột mượt mà với cơ chế cache chỉ mục 60 FPS.
- **Persistent Vault:** Lưu trữ toàn bộ lịch sử chiến báo an toàn vào `rok_history_vault.json`, cơ chế khử trùng lặp O(1) và tự động ghi đĩa debounced.
- **Hoàn toàn Portable:** Tích hợp cơ sở dữ liệu vật phẩm `loot.json` nhúng sẵn trong file thực thi EXE (tối ưu hóa tốc độ tải và loại bỏ phụ thuộc bên ngoài).
- **Đa Ngôn Ngữ & Giao Diện:** Hỗ trợ song ngữ Tiếng Việt & English, chế độ giao diện Sáng / Tối (Dark / Light mode).
- **Khởi Động & Khay Hệ Thống:** Tích hợp System Tray thu nhỏ nền và tùy chọn khởi động cùng Windows an toàn.

---

## Khởi Chạy

### 1. Giao diện Đồ Họa (GUI)
Chạy trực tiếp file `rok_watcher.exe`. Ứng dụng sẽ tự động dò tìm thư mục `mailcache` trên các ổ đĩa của máy tính.

### 2. Dòng Lệnh (CLI)
Hỗ trợ quét nhanh và tính toán chiến lợi phẩm trực tiếp từ terminal mà không cần mở GUI:

```powershell
# Quét tất cả trận đánh và xuất báo cáo JSON
.\rok_watcher.exe --cli --mode all-targets --json

# Lọc Motte trong 7 ngày gần nhất
.\rok_watcher.exe --cli --mode motte --days 7

# Lọc thu thập tài nguyên trong 30 ngày
.\rok_watcher.exe --cli --mode gathering --days 30
```

Xem danh sách đầy đủ các tùy chọn:
```powershell
.\rok_watcher.exe --help
```

---

## Biên Dịch (Dành cho Developer)

Yêu cầu: Rust toolchain (2021 edition).

```powershell
# Chạy kiểm thử tự động
cargo test

# Biên dịch bản phát hành tối ưu hóa (LTO + Strip)
cargo build --release
```

File thực thi sau khi biên dịch nằm tại `target/release/rok_watcher.exe`.

---

Lấy ý tưởng từ: [rokbattles](https://github.com/rokbattles/rokbattles)
