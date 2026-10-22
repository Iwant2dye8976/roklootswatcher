# RoK Loot Watcher (Rust)
Ứng dụng Windows theo dõi thư mục `mailcache` Rise of Kingdoms theo thời gian thực, tự động giải mã và thống kê phần thưởng chiến báo (Motte, Pháo đài, Dã man).
---
## Khởi Chạy
- **Giao diện (GUI)**: Chạy trực tiếp [`rok_watcher.exe`].
- **Dòng lệnh (CLI)**:
  ```powershell
  # Quét tất cả và xuất JSON
  .\rok_watcher.exe --cli --mode all-targets --json

  # Lọc Motte trong 7 ngày gần nhất
  .\rok_watcher.exe --cli --mode motte --days 7
  ```
  *Xem thêm tùy chọn:* `.\rok_watcher.exe --help`
---
## Biên Dịch (Dành cho Developer)

```powershell
cargo build --release
```
File thực thi nằm tại `target/release/rok_watcher.exe`.

Lấy ý tưởng từ: [rokbattles](https://github.com/rokbattles/rokbattles)