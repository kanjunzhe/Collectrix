# Collectrix

Collectrix 是一个本地优先的视频收藏与知识管理应用。它把 B 站、YouTube、抖音、博客、GitHub 等来源的视频或文章整理到本地 SQLite 资料库中，并提供分类、标签、主题、记事本、思维导图、统计、下载、弹幕、字幕和 AI 辅助整理等能力。

桌面端基于 Tauri 2，前端使用 Vue 3 + TypeScript + Vuetify，后端使用 Rust + SQLite。界面支持中文/English 切换，默认数据全部保存在本机。

## 功能

### 视频与文章库

- 通过链接导入视频或文章元数据，支持：
  - Bilibili 视频、B 站专栏
  - YouTube（仅元数据，桌面端）
  - 抖音（仅元数据）
  - GitHub 仓库
  - 常见博客、个人网站和通用网页文章
- 导入本地视频文件夹：自动识别同目录的封面、音频、字幕和文字稿；只有一个视频的文件夹会优先整体关联 sidecar 文件。
- 本地文件可以匹配 B 站视频：按文件名搜索 B 站，由用户确认后补全标题、作者、封面、aid/cid 等信息（aid/cid 用于评论、下载和字幕）。
- 多级分类树、标签、主题、收藏时间、最近新增等组织方式。
- 网格/列表视图、关键词搜索、按标签/主题筛选、懒加载分页。
- 视频库分类栏支持按内容自适应宽度、长分类名换行，并可切换“固定/悬停展开”模式；悬停模式下分类栏保持吸顶。
- 视频可拖入分类树移动分类；分类、标签、主题支持右键菜单和批量操作。
- B 站收藏夹、稍后再看、合集批量导入。
- JSON 数据导出/导入，导入时按分类路径、标签名和平台 ID 匹配，不覆盖本地媒体路径。

### 播放

- 视频默认在应用内播放；桌面端可在播放页点击“在新窗口播放”打开独立窗口，使用 `index.html?player_id=...` 初始化。
- Bilibili / YouTube 支持 `iframe` 内嵌播放；其它平台在浏览器中打开。
- 本地 B 站视频支持弹幕和字幕显示/隐藏；本地 SRT/VTT/ASS 字幕会优先加载。
- 本地视频加载后检测视频轨；只有音轨或 WebView 不支持的编码会回退到内嵌播放。
- 评论区支持 B 站评论、子评论和表情。

### 下载

仅对 Bilibili 启用下载。可配置：

- 视频质量（best / 4K / 1080p / 720p / 480p / 360p）
- 音频质量（best / high / medium / low）
- 单独下载视频或音频
- 字幕导出为 SRT
- 弹幕导出为 JSON
- 封面下载

视频下载使用 DASH 视频轨 + 音频轨，再通过 ffmpeg 合并。选轨时优先 H.264/AVC，以避免 WebView2 出现“有声音没画面”的 HEVC/AV1 兼容问题。下载产生的封面和字幕路径会写回视频记录，本地播放时可直接读取。

### AI

需要先在设置中开启 AI，并配置 OpenAI 兼容接口或 Ollama。

- **AI 整理**：分批分析视频，生成分类、标签和摘要建议；支持逐条预览、编辑、接受或拒绝。
- 分类建议支持多级路径，例如输入 `编程/Rust/异步` 可一次创建多级分类。
- **AI 筛选**：从 B 站收藏夹或稍后再看中按主题筛选，结果由用户确认后再导入。
- **AI 对话**：把本地视频库上下文发送给聊天模型，支持切换模型。
- 兼容 OpenAI、DeepSeek、SiliconFlow、Groq、Moonshot、Qwen、智谱等 OpenAI 兼容服务，也支持自定义接口和本地 Ollama。
- AI 任务状态由后端快照驱动，离开页面后未处理的建议会保存在本地，不会丢失。

### 转录与文字稿

- 支持本地 Whisper（whisper.cpp）、OpenAI 兼容的在线 `/audio/transcriptions` 接口，以及 Bilibili 字幕三种方式。
- 没有音频时，会尝试读取本地 sidecar 字幕，再尝试获取 B 站字幕作为文字稿来源。
- 已有音频但转录失败时，也会自动回退到本地字幕或 B 站字幕。
- 文字稿生成后可点击“AI 加标点 / 校对”，由当前 AI 模型补标点、纠错和分段；长文本会自动分块处理。

### 知识管理

- **主题**：给视频设置主题，适合跨分类归纳内容。
- **记事本**：记录标题和正文，引用视频库中的视频；也可以在视频详情页直接添加到记事本。
- **思维导图**：以 Collectrix 为根，按分类层级展示分类和视频，支持折叠/展开和直接打开视频。
- **统计**：分类、标签、平台、下载、最近新增等详细统计。

### 其它

- 应用锁：SHA-256 PIN 校验。
- 检查更新：关于页可查询 GitHub Releases 的最新版本，并跳转到发布页/下载资源。
- 新手引导：首次启动会展示功能引导，之后可在设置中重新打开。
- 主题切换、深色模式、双语界面。
- 移动端自适应侧栏；Android 工程已包含。
- 启动优化：首屏内置 Boot Splash，页面按需加载，Vuetify 组件显式按需注册，侧栏标签延后到空闲时加载，列表标签使用批量查询。
- 启动阶段不会加载 AI 页面，也不会请求 AI 服务；AI 栏目只在用户打开或设置允许后加载。
- 加载动画：使用数学玫瑰曲线粒子轨迹，启动页和按需加载页面都有统一的 Collectrix 曲线 Loading。
- 播放优化：视频默认在当前窗口播放，评论区按需加载，弹幕和字幕在视频开始播放后再延迟加载，移动端限制同屏弹幕数量。
- 使用 Collectrix 品牌图标：界面左上角、关于页、应用锁和加载页均使用统一图标。
- Tailwind CSS 用于少量布局和视觉样式，Vuetify 仍是主要组件/主题系统。

## 技术栈

| 层          | 技术                                         |
| ----------- | -------------------------------------------- |
| 桌面/移动壳 | Tauri 2                                      |
| 前端        | Vue 3、TypeScript、Vuetify 4、Tailwind CSS 4 |
| 构建        | Vite 8、vue-tsc                              |
| 后端        | Rust、Tokio、Reqwest、Quick XML              |
| 数据库      | SQLite（rusqlite）                           |
| 媒体处理    | ffmpeg、DASH                                 |
| AI          | OpenAI 兼容 API、Ollama                      |
| 本地转录    | whisper-rs（桌面端）                         |

## 快速开始

### 环境要求

- Node.js 20+（建议使用当前 LTS）
- Rust stable
- Tauri 对应平台的系统依赖
  - Windows：WebView2、Visual Studio C++ 构建工具
  - macOS：Xcode Command Line Tools
  - Linux：WebKitGTK、librsvg、libappindicator 等
- B 站下载需要 ffmpeg
- 下载受限视频、字幕和收藏夹同步需要有效的 B 站 Cookie

### 安装与开发

```bash
npm install
npm run tauri dev
```

仅调试前端：

```bash
npm run dev
```

此时 Vite 会运行在固定端口 `1420`，但 `invoke()` 等 Tauri 命令不可用。

### 构建

```bash
npm run build          # 仅前端类型检查 + 构建
npm run tauri build    # 桌面安装包
```

后端单独检查：

```bash
cd src-tauri
cargo check
```

## Linux / macOS 构建

Windows 本地环境不能直接生成 Linux 和 macOS 安装包，项目提供了 GitHub Actions 工作流：

```text
.github/workflows/build-desktop.yml
```

包含：

- Windows：NSIS / MSI
- Ubuntu 22.04：`.deb` + `.AppImage`
- macOS 15 Intel：`.app` + `.dmg`
- macOS 15 Apple Silicon：`.app` + `.dmg`
- Android：arm64 debug APK（使用 debug 签名，可直接安装测试）

在 GitHub 仓库的 Actions 页面手动运行 `Build all platforms`，或推送 `v*` 标签触发构建。产物会作为 workflow artifacts 上传。

> macOS 工作流未配置 Apple Developer 签名/公证，生成的是未签名开发包；正式分发需要补充 codesign、notarization 和 Apple 证书。

## Android

Android 工程位于 `src-tauri/gen/android`。

```bash
npm run android:init
npm run android:dev
npm run android:build
```

构建前需要配置 Android SDK、NDK 和 Java：

```powershell
$env:ANDROID_NDK_HOME="D:\Android\SDK\ndk\29.0.14206865"
$env:NDK_HOME=$env:ANDROID_NDK_HOME
npm run android:build
```

说明：

- Android 构建脚本会自动为 Gradle 的 Tauri 插件准备 `npm.bat` 兼容入口。
- release 签名配置位于 `src-tauri/gen/android/keystore.properties`，密钥库默认在 `src-tauri/android-keystore/`（已加入 `.gitignore`，请自行备份）。
- 只构建 arm64 release 可以使用 `npm run android:build:arm64`，速度更快。
- Activity 使用 `fullUser` 屏幕方向，支持竖屏/横屏切换；自适应图标 foreground 已加入安全区内缩，避免启动器裁剪。
- release 输出位于 `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`，已使用本地 release keystore 签名。
- 本地 Whisper、YouTube 解析、桌面目录选择、部分文件/下载操作在 Android 上不可用。
- 移动端窄屏会使用抽屉式侧栏，本地视频播放会回退为内嵌播放。

## 配置说明

### Bilibili Cookie

推荐在设置中使用二维码登录，登录成功后会自动保存 Cookie。也可以手动填写：

```plain
SESSDATA=xxx; bili_jct=xxx; DedeUserID=xxx
```

Cookie 用于收藏夹、稍后再看、字幕、评论和受限内容。公开视频的元数据与弹幕 XML 通常不需要登录。

### 下载与 ffmpeg

在设置中填写 ffmpeg 可执行文件路径；留空时依次尝试：

1. 系统 PATH 中的 `ffmpeg`
2. 应用资源目录中的 ffmpeg

下载目录可在设置中分别配置视频、音频和文字稿目录。

### 本地转录

需要准备 GGML 格式的 Whisper 模型文件，并在设置中选择模型路径。也可以切换到 OpenAI 兼容的 `/audio/transcriptions` 接口进行在线转录。

### AI 接口

- OpenAI 兼容服务填写 Base URL、API Key 和模型名。
- 本地 Ollama 默认地址通常包含 `localhost` 或 `11434`，后端会按 Ollama `/api/chat` 协议调用。
- 自定义兼容接口使用 `custom` 提供商配置。

## 数据与备份

应用数据保存在 Tauri 的 app-data 目录，而不是仓库目录。Windows 上通常位于：

```text
%APPDATA%\com.kandimord.collectrix\
```

其中包含：

- `video_vault.db`：SQLite 数据库
- `settings.json`：应用设置

删除该目录会重置本地数据。建议定期使用应用内的“导出数据”功能备份；媒体文件本身不会被打包进 JSON，导出/导入只处理元数据、分类、标签和关联关系。

## Bilibili 技术备注

### WBI 签名

B 站 Web 接口使用 WBI 签名：

1. 请求 `https://api.bilibili.com/x/web-interface/nav`。
2. 从 `wbi_img.img_url` 和 `wbi_img.sub_url` 提取文件名，得到 `img_key` / `sub_key`。
3. 拼接后用固定的 `MIXIN_KEY_ENC_TAB` 重排，取前 32 个字符作为 `mixin_key`。
4. 参数加入当前时间戳 `wts`，按 key 排序、URL encode 后拼接 `mixin_key`，计算 MD5 得到 `w_rid`。
5. 将 `w_rid` 和 `wts` 加入请求参数。

实现中 WBI key 会在每次请求时重新获取；`wbi_cache` 表暂时未参与缓存。

### 弹幕与字幕

- 本地 B 站视频通过 `cid` 请求 `https://comment.bilibili.com/{cid}.xml` 获取弹幕。
- 弹幕 XML 优先按 UTF-8 解码，失败时回退 GBK。
- 字幕通过 B 站播放器接口获取；部分视频需要登录 Cookie。
- 本地播放器会按视频时间推进弹幕，暂停、缓冲、拖动进度、循环播放和切换视频时会同步重置/暂停弹幕。

## 常见问题

### 本地视频只有声音、没有画面

通常是 WebView 不支持 HEVC/AV1 视频轨。重新下载时应用会优先选择 H.264/AVC；旧的 HEVC 文件建议删除后重新下载。

### 下载失败

确认：

- 设置中的 ffmpeg 路径正确
- B 站 Cookie 有效
- 视频不是地区限制、付费或已失效内容
- 磁盘空间和下载目录权限正常

### 弹幕不显示

- 弹幕只对本地 B 站视频启用；iframe 播放使用 B 站自己的弹幕。
- 视频需要保存了 `cid`。
- 弹幕 XML 可能因视频下架或接口限制不可用；失败不会影响视频播放。

### 字幕不显示

- 本地视频会自动读取文件夹中的 SRT/VTT/ASS 字幕；如果用了 ASS 高级特效，可能只保留文字内容。
- B 站字幕接口可能要求有效 Cookie。先在设置中完成 B 站登录，再重新打开视频。

### 本地视频无法播放或只有声音

确认视频文件本身可播放，并尝试重新下载为 H.264/AVC。WebView 对 MKV 容器、HEVC/AV1 等格式支持有限，必要时可用 ffmpeg 转码为 MP4/H.264。

### 本地视频匹配 B 站失败

B 站搜索可能受风控或 Cookie 状态影响。先确认标题关键词，必要时在设置中完成 B 站登录后再试；匹配失败不影响按本地视频直接导入。

### AI 请求失败

检查 Base URL、API Key、模型名和网络。Ollama 需要先启动本地服务；如果地址包含 `localhost` 或 `11434`，会按 Ollama 协议处理。

## 项目结构

```text
src/
  App.vue                 # 视图路由与全局布局
  views/                  # 各功能页面
  components/             # 播放器、弹窗、列表等组件
  composables/            # 通知、确认框、右键菜单等组合式逻辑
  types.ts                # 与 Rust models 手动同步的类型
  i18n.ts                 # 中文文案 -> 英文翻译
  platforms.ts            # 平台能力配置

src-tauri/
  src/lib.rs              # Tauri 命令注册
  src/commands/           # video/category/tag/ai/notebook/local_import/... 命令
  src/providers/          # 各平台元数据解析器
  src/commands/local_import.rs # 本地视频扫描、sidecar 识别与导入
  src/download.rs         # B 站下载与 ffmpeg 处理
  src/bilibili.rs         # WBI、弹幕、字幕、收藏夹接口
  migrations/001_init.sql # 可重复执行的完整数据库结构
```

## 开发约定

- 前端只能通过 `invoke()` 调用 Rust 命令；新增命令必须注册到 `lib.rs` 的 `generate_handler!`。
- `src/types.ts` 需要和 Rust 模型手动保持一致。
- 视频字段顺序必须同时更新 `models::SELECT_FIELDS`、`read_video_row`、SQL schema 和 `src/types.ts`。
- 新增平台需要后端 `VideoProvider`、`all_providers()` 注册，以及前端 `PLATFORMS` / `guessPlatform()`。
- 用户可见文案使用 `t("中文文案")`，并在 `src/i18n.ts` 增加英文条目。
- 项目没有测试、lint 或格式化脚本；提交前至少运行 `npm run build` 和 `cargo check`。

## 0.1.1 完成项

- [x] 1. GitHub 自动获取信息，并显示更详细的内容
  - 仓库名称、作者、语言、Stars、Forks、Watchers、Issues
  - 许可证、默认分支、创建/更新时间、Topics、主页和归档状态
  - 自动读取并展示仓库 README，文章页支持表格、详情折叠等常见 Markdown 内容
- [x] 2. 文字转写增强
  - 支持本地 Whisper、OpenAI 兼容在线接口和 Bilibili 字幕
  - 没有音频时自动回退本地 sidecar 字幕或 Bilibili 字幕
  - 设置中可开启“转录后自动 AI 加标点 / 校对”
  - 播放页仍保留手动“AI 加标点 / 校对”按钮
- [x] 3. GitHub Actions 修复
  - 修复 Linux/macOS/Windows 构建中的前端类型错误入口
  - Android 改用 runner 预装 Android SDK，显式安装 NDK，避免 `setup-android` 的 `tools` 包错误
  - Actions 现在覆盖 Windows、Linux、macOS Intel、macOS Apple Silicon 和 Android arm64 Debug APK

版本更新说明见 [CHANGELOG.md](CHANGELOG.md)。
