# Design Specification: Overlay Position D-Pad Controls, Independent Sizing, and See-Through Transparency

- **Date:** 2026-10-24
- **Status:** Proposed
- **Author:** Antigravity & User

---

## 1. Intent & Problem Statement
Players of *Rise of Kingdoms* have diverse screen resolutions, HUD preferences, and game window placements. Currently:
1. The overlay is locked at a fixed static offset `(16px, 60px)` from the top-left of the game window, which can overlap important in-game HUD elements (e.g. alliance chat, profile frame, rally banners).
2. The overlay dimensions (240px) and font size (10.5pt) are fixed, preventing players on high-resolution displays (1440p / 4K) from enlarging text or players on small laptop screens from shrinking the cards.
3. The dark slate card background, while translucent, obscures the game map terrain and marching troops underneath. Players requested a genuine see-through mode where the background box disappears completely, allowing troop marches and map action to be viewed directly through the text.

This feature adds:
- On-screen D-Pad arrow controls with hold-to-move capabilities and numerical X/Y offset inputs.
- Independent sliders for Overlay Width and Font Size.
- A 0%–100% Background Opacity slider (Option A) that allows 100% see-through transparency (Ghost HUD mode) at 0%.

---

## 2. Goals, Non-Goals & Anti-Goals

### Goals
- **Real-Time Smooth Movement**: Provide an on-screen D-Pad (`▲`, `▼`, `◄`, `►`) in the main app Settings tab that supports continuous holding to smoothly glide the overlay across the screen at ~60fps.
- **Explicit Coordinate Tuning**: Allow manual numeric entry and a "Reset to Default" button for `X` and `Y` offsets.
- **Independent Sizing**: Dedicated sliders for `Overlay Width` (180px–380px) and `Font Size` (8.5pt–15.0pt).
- **Pure See-Through Transparency**: At 0% background opacity, render zero background box or border, floating only high-contrast text, counts, and colored accent bars directly over the game map.
- **Strict Backward Compatibility**: Gracefully deserialize older `rok_config.json` files that lack the new fields, using sensible default values.

### Non-Goals
- In-game mouse dragging directly on the overlay window (the overlay strictly maintains `WS_EX_TRANSPARENT` click-through so game clicks are never intercepted).
- Per-card custom color overrides beyond the existing battle-type accent scheme and global text theme.

### Anti-Goals
- **Zero Emoji Violation**: Strictly NO Unicode emojis in any overlay text, labels, or buttons.
- **No Background Thread Loops**: No dedicated polling threads or OS timers for movement; utilize `egui`'s native pointer sensing and frame requests.

---

## 3. Approaches Considered

### Approach 1: Integrated Egui Pointer-Sensing D-Pad with Dynamic Viewport Geometry (Chosen)
- **Description**: Uses `ui.add(button).is_pointer_button_down_on()` during each render frame to continuously increment/decrement `offset_x` / `offset_y` while held, calling `ctx.request_repaint()` for fluid 60fps movement.
- **Pros**: Zero external dependencies, no background timer threads, immediate visual response in the RoK window while holding buttons, crisp subpixel font rendering.
- **Cons**: Requires dynamic height recalculation in egui layout as font size scales.

### Approach 2: Discrete Stepped Timer with Static Viewport Scaling (Rejected)
- **Description**: Uses an internal timer that steps coordinates in fixed 10px jumps every 100ms on button down, using an egui matrix scale transform.
- **Reason for Rejection**: Stepped movement feels jerky; matrix scaling on high-DPI monitors produces blurry text compared to native typography.

---

## 4. Detailed Design & Architecture

### 4.1 Data Model Changes (`src/model.rs`)
Add four new fields with serde defaults to `OverlayConfig`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OverlayConfig {
    pub enabled: bool,
    #[serde(default = "default_overlay_opacity")]
    pub opacity: f32, // 0.0 ..= 1.0
    #[serde(default = "default_overlay_text_color")]
    pub text_color_hex: String,
    #[serde(default)]
    pub filter: FilterMode,
    #[serde(default = "default_overlay_offset_x")]
    pub offset_x: f32, // Default: 16.0
    #[serde(default = "default_overlay_offset_y")]
    pub offset_y: f32, // Default: 60.0
    #[serde(default = "default_overlay_width")]
    pub width: f32,    // Default: 240.0 (Range: 180.0 ..= 380.0)
    #[serde(default = "default_overlay_font_size")]
    pub font_size: f32,// Default: 10.5 (Range: 8.5 ..= 15.0)
}
```
Default helper functions:
- `default_overlay_offset_x() -> f32 { 16.0 }`
- `default_overlay_offset_y() -> f32 { 60.0 }`
- `default_overlay_width() -> f32 { 240.0 }`
- `default_overlay_font_size() -> f32 { 10.5 }`

### 4.2 Window Tracker & Geometry Calculation (`src/overlay_tracker.rs`)
Update `calculate_overlay_geometry` to accept `&OverlayConfig`:
```rust
pub fn calculate_overlay_geometry(
    bounds: &RoKWindowBounds,
    config: &OverlayConfig,
    scale_factor: f32,
) -> OverlayGeometry {
    let scale = if scale_factor <= 0.0 { 1.0 } else { scale_factor };
    let pos_x = (bounds.x as f32 + config.offset_x) / scale;
    let pos_y = (bounds.y as f32 + config.offset_y) / scale;
    let width = config.width.clamp(180.0, 450.0);
    
    // Dynamic height calculation based on font size (approx 3.2x font size per card + spacing)
    let card_height = (config.font_size * 3.3).max(28.0);
    let est_height = (card_height * 5.0 + 20.0).max(150.0);
    let max_height = ((bounds.height as f32 - 40.0) / scale).max(150.0);
    let height = est_height.min(max_height);

    OverlayGeometry {
        pos_x,
        pos_y,
        width,
        height,
    }
}
```

### 4.3 D-Pad Movement & Hold Controls (`src/gui.rs`)
In `render_view_settings`:
1. Render a clean 3x3 grid D-Pad:
   - Row 1: `[ ]` `[ ▲ ]` `[ ]`
   - Row 2: `[ ◄ ]` `[ RoK ]` `[ ► ]`
   - Row 3: `[ ]` `[ ▼ ]` `[ ]`
2. Detection logic for continuous holding:
   ```rust
   let nudge_step = 2.5_f32; // Pixels per frame while held
   let btn_up = ui.add(egui::Button::new("▲"));
   if btn_up.is_pointer_button_down_on() {
       self.overlay_config.offset_y -= nudge_step;
       self.last_mail_arrival = Instant::now();
       self.save_config();
       ctx.request_repaint();
   }
   ```
   (Similar checks for `btn_down`, `btn_left`, `btn_right`).
3. Center `RoK` button acts as a reset button to restore `(16.0, 60.0)`.
4. Accompanying numeric drag-value inputs for `X` and `Y`:
   ```rust
   ui.add(egui::DragValue::new(&mut self.overlay_config.offset_x).speed(1.0).prefix("X: "));
   ui.add(egui::DragValue::new(&mut self.overlay_config.offset_y).speed(1.0).prefix("Y: "));
   ```

### 4.4 Sizing & Typography Sliders (`src/gui.rs`)
- **Width Slider**: `180.0 ..= 380.0` with formatter `{:.0} px`.
- **Font Size Slider**: `8.5 ..= 15.0` with formatter `{:.1} pt`.
- Modifying either value calls `self.save_config()` and `self.last_mail_arrival = Instant::now()`.

### 4.5 See-Through Transparency Rendering (`src/overlay_ui.rs`)
In `render_overlay_viewport`:
1. **Background & Border Handling**:
   - `let eff_opacity = (config.opacity * dim_factor).clamp(0.0, 1.0);`
   - If `config.opacity <= 0.01`:
     - `bg_color = Color32::TRANSPARENT;`
     - `stroke = Stroke::NONE;`
     - Card background is completely invisible. The game map and terrain are 100% visible through the overlay.
   - If `config.opacity > 0.01`:
     - `bg_color = Color32::from_rgba_unmultiplied(15, 23, 42, (220.0 * eff_opacity) as u8);`
     - `stroke = Stroke::new(1.0, Color32::from_white_alpha((35.0 * eff_opacity) as u8));`
2. **Text Readability in See-Through Mode**:
   - `text_color` retains high contrast (`base_text_color.linear_multiply(eff_opacity.max(0.70))`) so text is never washed out even if background is 0%.
   - Vertical 3px left accent bar renders with `card.accent_color.linear_multiply(eff_opacity.max(0.70))` to visually group the report cards without needing a dark box.
3. **Adaptive Typography**:
   - Header tag & target font size: `config.font_size`.
   - Subtitle, coordinates & drops font size: `(config.font_size * 0.88).max(8.0)`.
   - Time stamp font size: `(config.font_size * 0.82).max(7.5)`.

---

## 5. Constraints & Compliance
- **Zero Emoji Constraint**: Text formatting uses strictly ASCII symbols (`•`, `|`, `(X, Y)`, `+100 Gems`).
- **Click-Through & Win32**: Viewport builder maintains `.with_mouse_passthrough(true)` and `.with_active(false)` without `WS_EX_LAYERED`.
- **Clippy & Warnings**: All code must compile with 0 warnings under `cargo clippy -- -D warnings`.
- **Test Coverage**: Unit tests covering config serde, geometry offset calculation, and transparent color rendering.

---

## 6. What's Left to the Builder
- Precise layout margin alignment within the Settings view.
- Formatting of DragValue speed and debounce frequency for config file saving during continuous drag.
