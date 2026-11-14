# Collectrix — Logo 设计

基于 `README.md` 提炼：**多源汇集 → 一个本地可翻阅的库**。探索稿 8 个方向，最终方案把它们收成一件拟物化的图标。

## 最终方案

- **`final/collectrix-icon.svg`** — 主图标，512 画布，拟物：蓝色底板（渐变＋暗角＋左上柔光）上叠三张白色卡片（逐层投影＋6px 书脊厚度＋卡面 sheen），前卡压印点阵、嵌一枚珐琅质感播放键。
- **`final/collectrix-icon-flat.svg`** — 扁平版，去掉投影/高光/渐变，用于 ≤48px 与单色底。
- **`final/collectrix-mark-mono.svg`** — 单色标记（100 画布），托盘/水印/烫印，`currentColor` 可反白。
- **`final.html`** — 最终方案展示页（合成说明、材质拆解、尺寸阶梯、三态、字标组合）。
- **`final.png`** — 展示页截图。

### 综合了哪些方向

| 探索稿 | 在最终方案里的角色 |
| --- | --- |
| 01 Tile Play | 圆角方砖 → 蓝色底板；播放三角 → 凸起的珐琅片 |
| 02 Dot Play | 卡面右侧的压印点阵（低对比材质） |
| 03 Converge | 不画线，改成结构：三层卡片收进同一张前卡 |
| 04 C Play | 不塞进图标，留给字标符号（`final.html` 里的 lockup） |
| 05 Card Stack | 图标主体骨架 |
| 06 Node Graph | 收进点阵，避免两套语言 |
| 07 Spine Library | 每张卡下缘 6px 侧边（书脊） |
| 08 Corner Frame | 未采用，保留在 `variants/` |

### 规范

| 项 | 值 |
| --- | --- |
| 画布 | 512×512（图标）／100×100（单色标记） |
| 圆角 | 底板 112、卡片 64（约 22% / 21%） |
| 品牌色 | 蓝 `#1976D2`，渐变 `#4FA0F2 → #1E6FD0 → #0B4EA2` |
| 卡片 | 面 `#FFFFFF → #E4EAF3`，侧边 `#C9D5E5 → #A9B9CF` |
| 播放键 | 渐变 `#4CA6F5 → #1E77D6 → #0E54A8` |
| 安全区 | 内容外留 72px（14%） |
| 小尺寸 | ≥64px 用拟物版，≤48px 用扁平版 |

## 探索稿

`variants/01-tile-play.svg` … `08-corner-frame.svg`，以及 `showcase.html` / `preview.png`（8 个方向的对照页）。

## 已部署的位置

| 文件 | 位置 | 说明 |
| --- | --- | --- |
| `collectrix-logo.svg` | `public/` | 拟物主 logo，Vite 以 `/collectrix-logo.svg` 提供 |
| `collectrix-logo-flat.svg` | `public/` | 扁平版，`/collectrix-logo-flat.svg` |
| `collectrix-mark-mono.svg` | `public/` | 单色标记，`/collectrix-mark-mono.svg` |
| `app-icon-1024.png` | `logo/final/` | 1024 透明主图（图标生成源） |

`src-tauri/icons/` 由 `npx tauri icon logo/final/app-icon-1024.png` 重新生成：

- 桌面：`32x32.png`、`64x64.png`、`128x128.png`、`128x128@2x.png`(256)、`icon.png`(512)、`icon.ico`、`icon.icns`
- Windows Store：`Square30/44/71/89/107/142/150/284/310Logo.png`、`StoreLogo.png`
- iOS：`icons/ios/AppIcon-*.png`
- Android：`src-tauri/gen/android/app/src/main/res/mipmap-*/ic_launcher*.png`（48/72/96/144/192，含 round 与 foreground 432）

> `32x32.png` 单独用扁平版生成（拟物版在该尺寸会糊）；其余尺寸用拟物版。`tauri.conf.json` 的 `bundle.icon` 已包含 `32x32`、`128x128`、`128x128@2x`、`icon.icns`、`icon.ico`，无需改动。

## 下一步

1. 导出 1024×1024 主图（浅底 / 深底 / 透明三版）。
2. `npm run tauri icon <1024.png>` 覆盖 `src-tauri/icons/`。
3. Android：48/72/96/144/192 五档，前景图形留 66% 安全区，背景 `#1976D2`。
4. 需要“展示图”（Nano Banana / Gemini）再走 `logo-generator` 技能 Phase 4，需要 `GEMINI_API_KEY`。
