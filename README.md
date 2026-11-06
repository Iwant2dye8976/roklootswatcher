# RoK Loot Watcher (Rust)

Lấy ý tưởng từ: [rokbattles](https://github.com/rokbattles/rokbattles)

Ứng dụng Windows hiệu năng cao theo dõi thư mục `mailcache` của Rise of Kingdoms theo thời gian thực, tự động giải mã cấu trúc nhị phân Lilith, lưu trữ an toàn bằng SQLite Vault và hỗ trợ đồng bộ đám mây (Cloud Sync) từng bản ghi qua Supabase.

---

## Tính Năng Nổi Bật

- **Real-time Mail Watcher:** Giám sát thư mục `mailcache` với cờ chia sẻ tệp Windows (`FILE_SHARE_DELETE`), không bao giờ xung đột khóa file khi game ghi đè hoặc dọn dẹp chiến báo trong lúc tham gia rally / farm.
- **SQLite Vault Archive (WAL Mode):** Cơ sở dữ liệu SQLite cục bộ `rok_history_vault.db` với cơ chế khử trùng lặp O(1), tự động bảo toàn bản sao lưu nhị phân `mail_vault/`, hỗ trợ import/export JSON và tự động nâng cấp từ các phiên bản cũ.
- **Per-Record Cloud Sync (Supabase):**
  - Đồng bộ từng bản ghi chiến báo (`report_records`) lên Cloud bằng batch upsert (`resolution=merge-duplicates`).
  - Worker chạy ngầm theo mô hình Actor, hỗ trợ debounced delta sync, tự động retry với exponential backoff thông minh cho các lỗi mạng tạm thời.
  - Phân loại lỗi mạng vs lỗi xác thực/phân quyền, ngăn chặn vòng lặp retry vô tận.
  - Đăng nhập Google OAuth qua loopback server nội bộ an toàn (bảo vệ CSRF state token & kiểm tra origin).
  - Khôi phục (Restore) dữ liệu từ Cloud bất cứ lúc nào trên thiết bị mới (có cơ chế tương thích ngược với legacy blob).
- **In-Game Overlay:** Lớp phủ nổi trong suốt hiển thị 5 chiến báo mới nhất trực tiếp trên cửa sổ game Rise of Kingdoms, hỗ trợ click-through xuyên chuột và phím điều hướng D-pad tùy chỉnh vị trí.
- **Bảng Thống Kê & Báo Cáo:** Tra cứu, lọc theo ngày/loại trận đánh (Motte, Pháo đài, Dã man, Thu thập tài nguyên), tìm kiếm và sắp xếp đa cột mượt mà với cơ chế cache chỉ mục 60 FPS.
- **Kinh Tế Điểm Hoạt Lực (AP Economics):** Phân tích chi phí AP và tỷ lệ hoàn vốn (ROI) thu được từ quà diệt dã man / pháo đài.
- **Hoàn toàn Portable:** Tích hợp cơ sở dữ liệu vật phẩm `loot.json` nhúng sẵn trong file thực thi EXE (tối ưu hóa tốc độ tải và loại bỏ phụ thuộc bên ngoài).
- **Đa Ngôn Ngữ & Giao Diện:** Hỗ trợ song ngữ Tiếng Việt & English, chế độ giao diện Sáng / Tối (Dark / Light mode).
- **Khởi Động & Khay Hệ Thống:** Tích hợp System Tray thu nhỏ nền và tùy chọn khởi động cùng Windows an toàn.

---

## Hướng Dẫn Cấu Hình Cloud Sync (Supabase)

Nếu bạn muốn sử dụng tính năng đồng bộ và sao lưu đám mây:

1. Tạo một dự án trên [Supabase](https://supabase.com).
2. Vào **SQL Editor** trong Supabase Dashboard và chạy nội dung file [`resources/supabase_migration_report_records.sql`](resources/supabase_migration_report_records.sql) để tạo bảng `report_records`, cấu hình RLS và cấp quyền cho authenticated user.
3. Bật **Google Provider** trong mục **Authentication -> Providers** trên Supabase.
4. Mở ứng dụng `rok_watcher.exe`, vào tab **Cài đặt (Settings)** -> **Đồng bộ Đám mây (Cloud Sync)**, nhập Supabase URL + Anon Key (hoặc dùng mặc định nếu có) và nhấn **Đăng Nhập Google**.

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
# Chạy toàn bộ 314+ kiểm thử tự động
cargo test --all

# Biên dịch bản phát hành tối ưu hóa (LTO + Strip)
cargo build --release
```

File thực thi sau khi biên dịch nằm tại `target/release/rok_watcher.exe`.

---
