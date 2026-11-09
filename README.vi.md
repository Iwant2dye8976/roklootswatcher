# RoK Loot Watcher

[English](README.md) | [Tiếng Việt](README.vi.md)

Lấy cảm hứng từ: [rokbattles](https://github.com/rokbattles/rokbattles)

Ứng dụng Windows và công cụ dòng lệnh (CLI) hiệu năng cao theo dõi thư mục `mailcache` của Rise of Kingdoms theo thời gian thực. Ứng dụng tự động giải mã cấu trúc nhị phân độc quyền của Lilith, lưu trữ lịch sử chiến lợi phẩm trong kho dữ liệu SQLite chế độ WAL, đồng bộ hóa bản ghi lên Supabase Cloud và hiển thị số liệu chiến báo trực tiếp bên trong giao diện desktop Tauri v2 xây dựng bằng React 19, TypeScript và Tailwind CSS.

---

## Tính Năng Nổi Bật

### 1. Thu Thập & Giải Mã Thư Lilith Thời Gian Thực
- Giám sát thư mục `mailcache` thời gian thực với các cờ chia sẻ tệp của Windows (`FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE`).
- Giải mã các thẻ tuần tự hóa nhị phân của Lilith (`0x00` Null, `0x01` Bool, `0x02` U8, `0x03` F64, `0x04` String, `0x05` Table, `0xFF` End), không bao giờ gây xung đột khóa tệp hay ảnh hưởng đến game khi đang chơi.
- Giải mã hàng loạt đa luồng tận dụng Rayon (`MAIL_DECODE_BATCH_SIZE`).
- Tự động tách biệt và cô lập dữ liệu theo từng tài khoản (UID).

### 2. Kho Lưu Trữ SQLite Vault & Khử Trùng Lặp
- Cơ sở dữ liệu SQLite cục bộ (`rok_history_vault.db`) chạy chế độ WAL với thời gian chờ kết nối bận (busy timeout) và giao dịch nguyên tử (atomic transactions).
- Khử trùng lặp bản ghi theo thời gian O(1) dựa trên mã định danh chiến báo.
- Tự động sao lưu bản ghi nhị phân gốc vào thư mục `mail_vault/`.
- Xuất dữ liệu CSV và JSON an toàn qua tệp tạm (.tmp) và đổi tên nguyên tử.
- Tự động nâng cấp cấu trúc bảng từ các phiên bản kho JSON cũ.

### 3. Kinh Tế Điểm Hoạt Lực (AP) & Phân Tích Hiệu Suất
- **Biểu Đồ Kết Hợp Hai Trục SVG (Dual-Axis Combo Timeline)**: Theo dõi lượng AP tiêu hao hàng ngày đối chiếu với lượng tăng tốc nhận được bằng các cột màu xanh lá dịu (chi phí đầu vào) và đường cong hiệu suất màu xanh dương dịu (kết quả đầu ra), đi kèm khung chú giải tương tác hiển thị số phút tăng tốc trên mỗi 1.000 AP đã dùng.
- **Bảng So Sánh Hiệu Quả Mục Tiêu (Target Benchmarks)**: Đánh giá và xếp hạng Pháo Đài Dã Man, Pháo Đài Motte and Bailey, và Dã Man Kéo Đàn theo tỷ lệ tăng tốc, tỷ lệ gems và tỷ lệ rơi Sách Giao Ước.
- **Theo Dõi Hoàn AP (Rebate Tracking)**: Tự động tính toán tỷ lệ phần trăm bình AP thu hồi được từ các rương phần thưởng diệt dã man.
- Chuẩn hóa thời gian theo múi giờ Việt Nam (UTC+7) với các khung ngày trượt linh hoạt.

### 4. Lớp Phủ Trong Game (HUD Overlay)
- Lớp phủ WebView2 trong suốt, siêu nhẹ bám sát theo vị trí cửa sổ game Rise of Kingdoms đang hoạt động.
- Tính toán tọa độ chuẩn xác theo tỷ lệ DPI màn hình, tùy chỉnh vị trí bù trừ và hỗ trợ xuyên chuột Win32 (`WS_EX_TRANSPARENT`).
- Phím tắt bật/tắt toàn cục (`Ctrl+Shift+H`).
- Nhiều chế độ hiển thị: Full HUD, Mini HUD, Combined HUD hoặc Ẩn.

### 5. Đồng Bộ Đám Mây Supabase & Xác Thực
- Tiến trình chạy ngầm theo mô hình Actor (`CloudSyncWorker`) giao tiếp qua kênh luồng an toàn.
- Cập nhật từng bản ghi theo lô với cơ chế phân giải trùng lặp (`resolution=merge-duplicates`).
- Tự động thử lại thông minh với thuật toán exponential backoff cho lỗi mạng tạm thời, phân loại chính xác lỗi xác thực để tránh vòng lặp.
- Máy chủ loopback cục bộ bảo mật phục vụ đăng nhập Google OAuth với mã kiểm tra CSRF state token.
- Hàng đợi thay đổi ngoại tuyến (offline queue) và khôi phục toàn diện kho dữ liệu trên thiết bị mới.

### 6. Giao Diện Desktop Hiện Đại & Đa Ngôn Ngữ
- Kiến trúc Tauri v2 kết hợp React 19, TypeScript và bảng màu tối lấy cảm hứng từ Binance.
- Danh sách ảo hóa (virtualized lists) cho trải nghiệm cuộn mượt mà qua hàng chục nghìn trận đánh.
- Cơ sở dữ liệu vật phẩm nhúng sẵn (`loot.json`), khởi động tức thì không cần đọc tệp ngoài.
- Hỗ trợ song ngữ hoàn chỉnh (Tiếng Anh và Tiếng Việt).
- Thu nhỏ xuống khay hệ thống Windows (System Tray) và tùy chọn khởi động cùng Windows qua registry.

---

## Bắt Đầu Sử Dụng

### 1. Giao Diện Đồ Họa (GUI)
Khởi chạy `rok_watcher.exe`. Ứng dụng sẽ tự động quét các ổ đĩa thông dụng để tìm đường dẫn thư mục `mailcache` của Rise of Kingdoms. Bạn cũng có thể thiết lập đường dẫn tùy chỉnh trong tab Cài đặt.

### 2. Giao Diện Dòng Lệnh (CLI)
Hỗ trợ kiểm tra nhanh, tính toán số liệu và trích xuất dữ liệu trực tiếp từ terminal PowerShell hoặc Command Prompt:

```powershell
# Phân tích toàn bộ mục tiêu và xuất kết quả dạng JSON
.\rok_watcher.exe --cli --mode all-targets --json

# Phân tích các trận Motte and Bailey trong 7 ngày gần nhất
.\rok_watcher.exe --cli --mode motte --days 7

# Phân tích thu thập tài nguyên trong 30 ngày gần nhất
.\rok_watcher.exe --cli --mode gathering --days 30

# Xem toàn bộ danh sách cờ lệnh hỗ trợ
.\rok_watcher.exe --help
```

---

## Hướng Dẫn Cấu Hình Đồng Bộ Đám Mây (Supabase)

Để kích hoạt tính năng sao lưu đám mây và đồng bộ giữa nhiều máy tính:

1. Tạo một dự án trên [Supabase](https://supabase.com).
2. Mở mục **SQL Editor** trong bảng điều khiển Supabase và chạy mã khởi tạo tại:
   [`resources/supabase_migration_report_records.sql`](resources/supabase_migration_report_records.sql).
3. Bật **Google Provider** trong phần **Authentication -> Providers**.
4. Trong ứng dụng RoK Loot Watcher, vào mục **Cài đặt -> Đồng bộ Đám mây**, nhập Supabase URL cùng Anon Key, sau đó bấm **Đăng nhập Google**.

---

## Quy Trình Biên Dịch & Phát Triển

### Yêu Cầu Môi Trường
- Rust toolchain 1.77+ (phiên bản 2021)
- Node.js 18+ và npm
- Visual Studio C++ Build Tools (trên Windows)

### Các Lệnh Thao Tác

```powershell
# 1. Cài đặt các gói phụ thuộc cho giao diện
npm --prefix ui install

# 2. Xây dựng gói giao diện React
npm --prefix ui run build

# 3. Chạy toàn bộ kiểm thử đơn vị và tích hợp (339+ tests)
cargo test

# 4. Kiểm tra mã nguồn với linter
cargo clippy -- -D warnings

# 5. Biên dịch file thực thi tối ưu hóa
cargo build --release

# 6. Sao chép file thực thi ra thư mục gốc dự án
Copy-Item -Force .\target\release\rok_watcher.exe .\rok_watcher.exe
```

---

## Tổng Quan Cấu Trúc Dự Án

```
roklootswatcher/
├── src/
│   ├── main.rs                 # Điểm khởi chạy CLI, Windows mutex, điều phối hệ thống
│   ├── model.rs                # Các kiểu dữ liệu nghiệp vụ chính (ReportRecord, BattleReport, LootItem)
│   ├── lilith_stream.rs        # Bộ giải mã nhị phân cấu trúc thư của Lilith
│   ├── classifier.rs           # Phân loại thư và trích xuất chiến lợi phẩm
│   ├── loot_catalog.rs         # Cơ sở dữ liệu vật phẩm nhúng và phân giải rương tài nguyên
│   ├── vault.rs                # Động cơ lưu trữ SQLite WAL và cơ chế khử trùng lặp
│   ├── vault_migrate.rs        # Bộ nâng cấp cấu trúc cơ sở dữ liệu
│   ├── watcher.rs              # Giám sát hệ thống tệp thời gian thực không khóa
│   ├── analyzer.rs             # Tính toán kinh tế AP và tỷ lệ rơi vật phẩm
│   ├── cloud_sync.rs           # Tiến trình worker đồng bộ Supabase chạy nền
│   ├── overlay_tracker.rs      # Bám sát tọa độ cửa sổ game và lắng nghe phím tắt Win32
│   ├── autostart.rs            # Quản lý khởi động cùng Windows qua Registry
│   ├── exporters.rs            # Quy trình xuất dữ liệu CSV và JSON
│   ├── i18n.rs                 # Hệ thống dịch thuật song ngữ (EN/VI)
│   └── tauri_app/              # Các lệnh Tauri v2 backend, DTO và quản lý cửa sổ overlay
├── ui/                         # Giao diện React 19 + TypeScript + Tailwind CSS
│   └── src/
│       ├── store/              # Kho trạng thái trung tâm (lắng nghe sự kiện từ Tauri)
│       ├── views/              # DashboardView, BattlesView, InventoryView, SettingsView, OverlayView
│       └── components/         # Titlebar, Sidebar, GlobalFilterBar, InspectorDrawer
└── resources/                  # Bản vẽ cấu trúc SQL và tài nguyên tĩnh
```

---

## Giấy Phép Sử Dụng

Phục vụ nhu cầu cá nhân và cộng đồng người chơi Rise of Kingdoms. Phát hành theo các điều khoản quy định trong kho lưu trữ của dự án.
