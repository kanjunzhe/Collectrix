# Collectrix

Tauri 2 desktop video-library app. Frontend: Vue 3 + TypeScript + Vuetify 4 (`src/`). Backend: Rust (`src-tauri/src/`), SQLite via `rusqlite`. Bilibili is the primary platform; YouTube/Douyin/article providers only fetch metadata. UI is bilingual (zh/en): user-facing strings go through `t("中文文案")` with an English entry in `src/i18n.ts`; code comments remain Chinese.

## Commands

- `npm run tauri dev` — full app. Starts Vite on fixed port 1420, then compiles/runs Rust.
- `npm run dev` — frontend only. `invoke()` calls fail outside a Tauri window.
- `npm run build` — frontend check only: `vue-tsc --noEmit && vite build`.
- `npm run tauri build` — desktop bundle (runs `npm run build` first).
- `cargo check` — run inside `src-tauri/` for backend-only verification.
- There are no test, lint, or formatter scripts/configs. Do not invent `npm test`/`npm run lint`.
- TypeScript is strict with `noUnusedLocals`/`noUnusedParameters`; unused code fails `npm run build`.
- Vite uses strict port 1420 and ignores `src-tauri/`. `TAURI_DEV_HOST` enables remote/mobile HMR.
- `dist/`, `src-tauri/target/`, and `src-tauri/gen/` are build/generated output; do not hand-edit them.

## Wiring Rules

- `src/App.vue` is the router: it switches views with a `currentView` ref; there is no `vue-router`. Add a view to `ViewName`, the drawer, and the template.
- Frontend talks to Rust only through `invoke()` from `@tauri-apps/api/core`; there are no generated bindings. Keep `src/types.ts` manually in sync with `src-tauri/src/models.rs`.
- Every `#[tauri::command]` must be added to the `generate_handler!` list in `src-tauri/src/lib.rs`. Registration is not automatic.
- Video rows depend on `models::SELECT_FIELDS` and `read_video_row` having the **same field order**. When adding a video field, update both, the SQL schema, and `src/types.ts`.
- SQLite is a single `Mutex<Connection>` in `DbState`. Never hold the lock across `.await`; use tight scoped blocks (existing command code is the pattern).
- New platforms require a backend `VideoProvider` plus registration in `all_providers()` in `src-tauri/src/providers/mod.rs`, and a matching entry in both `PLATFORMS` and `guessPlatform()` in `src/platforms.ts`.
- `save_video` accepts a generic `VideoMeta` and keys videos by `(platform, platform_id)`; `refresh_video_info` only supports Bilibili rows.
- The CSP in `src-tauri/tauri.conf.json` allows Bilibili and YouTube iframes. Update `frame-src` before enabling another embedded player.
- Local media uses Tauri's asset protocol: keep `protocol-asset` enabled on the `tauri` Cargo dependency, `assetProtocol.enable` plus scope in `tauri.conf.json`, and `asset:` / `http://asset.localhost` in `media-src`.
- Bilibili subtitle responses may be compressed; `reqwest` must keep `gzip`, `brotli` and `deflate` features or subtitle JSON decoding fails.
- i18n lives in `src/i18n.ts`: wrap user-facing text with `t("中文文案")` and add the English entry to the dictionary. The language is stored in `AppSettings.language` and can be toggled from the app bar or Settings.
- Video context menus use `ContextMenu.vue` + `useVideoContextMenu()` (`src/composables/`). Category and tag menus reuse `ContextMenu.vue`; destructive tag/category actions must use `DeleteConfirmDialog.vue` and require typing the entity name. Tag merging uses `MergeTagDialog.vue`.
- Theme values live in `src/themes.ts` and Vuetify theme definitions in `src/main.ts`. Add new themes to both, plus validation in `set_theme`. Dark variants start with `dark`; `isDark` and `lastLightTheme()` rely on that convention.
- AI organize processes videos in batches (`AI_CHUNK_SIZE` in `ai_task.rs`) and emits `ai-task-update` snapshots; the frontend must treat the task state as authoritative and prune accepted/rejected suggestions.
- AI Chat (`AiChatView.vue` + `ai_chat`) sends the local library context to the configured OpenAI-compatible/Ollama endpoint. Provider presets in Settings include OpenAI, DeepSeek, SiliconFlow, Groq, Moonshot, Qwen and Zhipu; custom OpenAI-compatible endpoints are supported via `custom`. The chat model can be changed from the AI Chat header.
- AI suggestions are previewed through `VideoPreviewDialog.vue`; shared delete/merge/move flows live in `DeleteConfirmDialog.vue`, `MergeTagDialog.vue`, `TagEditDialog.vue` and `MoveCategoryDialog.vue`.
- Download settings (`download_video`, `download_audio`, qualities, subtitles, danmaku, cover) are consumed in `download.rs`. Bilibili downloads use DASH tracks and ffmpeg; subtitle output is SRT and danmaku output is JSON. Video track selection prefers H.264/AVC (`avc1`/`avc3`) because WebView2 may play HEVC/AV1 audio without video.
- Library uses 48-item pages (`PAGE_SIZE`) with an IntersectionObserver sentinel for lazy loading in flat/tag modes; grouped category mode still loads the group set. The category sidebar uses `categoryHover` (persisted as `collectrix.categoryHover`) for fixed vs hover-expanded mode; keep its sticky sizing rules and `drawer-scroll`/fixed drawer footer when editing the shell.
- Keep `LinkCards` hover shadows inside the `.desc-box` padding/overflow; the card uses `overflow: visible` to avoid clipped borders.
- Data transfer lives in `commands/transfer.rs` (`export_data` / `import_data`) and `DataTransferDialog.vue`; imports match categories by path and tags/videos by name/platform id, and do not overwrite local media paths.
- Video cards/list items are HTML5-draggable; `CategoryTree.vue` emits `drop(videoId, categoryId)` and `move_video_to_category` persists the change. Keep the `text/plain` fallback and the sidebar-level drop fallback for WebView2.
- `db.rs` creates a top-level `未分类` category on startup and moves `category_id IS NULL` videos into it; `save_video`/`update_video` default to it when category is omitted.
- Bilibili QR login uses `generate_bili_qr` / `poll_bili_qr` and saves the returned cookies into settings. `QrLoginDialog.vue` renders the QR with the `qrcode` package. `poll_qr_code` merges Set-Cookie headers with the cookie query in `data.url`; `check_bili_login` verifies the saved session before watch-later/favorites operations.
- AI filtering is its own `AI 筛选` view (`AiFilterView.vue`) with favorites and Watch Later sources. Backend commands are `ai_filter_favorites` / `ai_filter_watch_later`; results are scored by topic and selected by the user before import.
- AI suggestions support `category_path` (root-to-leaf): `ensure_category_path` creates missing levels, and the edit UI accepts `父/子/孙` in the category field.
- Notebooks use `commands/notebook.rs`, `NotebookView.vue`, and `NotebookPickerDialog.vue`; tables are `notebooks` / `notebook_videos` in `001_init.sql`.
- Local import lives in `commands/local_import.rs` (`scan_local_videos` / `import_local_videos`); it groups sidecar cover/audio/subtitle files, supports matching via `search_bili_videos`, and stores `local_cover_path` + JSON `local_subtitle_paths` (update `SELECT_FIELDS`, `read_video_row`, and `src/types.ts` together).
- Local video covers use `videoCoverSrc()` from `src/utils/media.ts`; local SRT/VTT/ASS subtitles are parsed by `get_local_subtitle` and prioritized in `DanmakuPlayer`.
- `transcribe_video` falls back from audio transcription to local sidecar subtitles and then Bilibili subtitles; `polish_transcript` uses the AI chat endpoint to add punctuation and proofread.
- `App.vue` lazy-loads views with `defineAsyncComponent`; playback defaults to the in-app `PlayerView`. The desktop title-bar button can create a separate player window via the frontend `WebviewWindow` API (`index.html?player_id=...`); `capabilities/default.json` must keep `core:webview:allow-create-webview-window` and the `player-*` window glob.
- `index.html` contains the pre-Vue boot splash; after mount, `CurveLoader.vue` renders the animated rose-curve loader and `LoadingView.vue` is used by async views. Keep the inline splash independent of Vuetify so first paint is never blank.
- UI branding uses `public/collectrix-logo*.svg`; the Tauri bundle/Android launcher icons come from `src-tauri/icons`.
- Android release signing reads `src-tauri/gen/android/keystore.properties` and the gitignored keystore under `src-tauri/android-keystore/`; `app/build.gradle.kts` must keep the `signingConfigs.release` wiring. `AndroidManifest.xml` uses `fullUser` orientation, and the adaptive launcher foreground is inset to keep the logo inside the safe zone.
- Android edge-to-edge layout relies on `viewport-fit=cover` plus `.app-safe-area` in `App.vue` for the status bar/notch inset. Keep the app bar/drawer/main padding rules when editing the shell.
- Player performance: comments are lazy-loaded behind a button (`CommentSection` async), and danmaku/subtitles are deferred until playback starts and loaded after a short timer. Avoid reintroducing eager requests in `PlayerView`/`DanmakuPlayer`.
- `AppSettings.ai_enabled` controls the AI nav/features. `App.vue` listens for `collectrix:settings-changed` so toggling in Settings immediately hides AI sections.
- `src/main.ts` imports only the Vuetify components/directives actually used and registers them in `createVuetify`. Do not reintroduce `vite-plugin-vuetify` auto-import: its late dynamic component optimization caused full-page reloads in development. Add new `v-*` components to the explicit import list.
- App lock uses `set_app_lock` / `verify_app_lock` with a SHA-256 PIN hash; `AppLockScreen.vue` is rendered outside `v-app` and must keep `v-theme--light` for Vuetify variables.
- About-page update checks use `commands/update.rs` (`check_for_updates`) against the latest GitHub release; keep the repo URL/version comparison in sync if the release host changes.
- `OnboardingDialog.vue` is first-run help stored under `collectrix.onboardingDone` in localStorage. Settings dispatches `collectrix:open-onboarding`; App owns the dialog and marks completion.
- The AI Organize progress strings originate from Rust in Chinese; `localizedProgress()` in `AiOrganizeView.vue` must be updated whenever those strings change.
- Collection/watch-later bulk import uses `fetch_bili_collection` / `list_bili_watch_later` and `BulkImportDialog.vue`; single/favorites flows remain in the existing dialogs.
- `LinkCards.vue` + `utils/linkCards.ts` extract GitHub/Baidu/Quark/Aliyun/123/Lanzou/Drive links and nearby extraction codes; keep passwords in copied/opened URLs. `ContextMenu.vue` has an entrance/loading animation (`opening` state and progress bar).
- `StatsView.vue` uses `get_detailed_stats`; `MindMapView.vue` + `MindMapNode.vue` render the category/video mind map; `AiOrganizeView` uses `VideoPreviewDialog` and `localizedProgress`.
- Blog and direct GitHub links use `web.rs` + `providers/web_article.rs` / `providers/github.rs`; `ArticleView.vue` switches to `fetch_web_article` for non-Bilibili articles. The generic web provider must stay last in `all_providers()`.
- Tailwind CSS v4 is integrated through `@tailwindcss/vite` and `src/styles/tailwind.css` (theme/utilities only, no preflight); Vuetify remains the primary component/theme system.
- Android project lives in `src-tauri/gen/android` (`npm run android:init|dev|build`). Mobile builds need `ANDROID_NDK_HOME`; `whisper-rs`, `symphonia`, `innertube-rs`, and native directory-picking are desktop-only, so local Whisper/YouTube parsing/export-directory picking are unavailable on Android.
- Video playback defaults to `PlayerView` in the current window to avoid WebView creation stalls. The desktop title-bar action uses the frontend `WebviewWindow` API with `index.html?player_id=...`; keep the `player-*` window glob and `core:webview:allow-create-webview-window` permission in `capabilities/default.json`.
- `DanmakuPlayer.vue` exposes danmaku/subtitle visibility toggles; comment emoji rendering lives in `CommentContent.vue` (backend `ReplyContent.emote`). Bilibili danmaku XML is raw DEFLATE, so `fetch_danmaku_xml` disables reqwest auto-decompression and decodes with `flate2`; do not remove that fallback.
- Use `notify.*` and `confirmDialog()` from `src/composables/useUI.ts` for feedback, not native `alert()`/`confirm()`.

## Database / Settings

- App data lives in the Tauri app-data directory, not the repo: `video_vault.db` and `settings.json`. Deleting that directory resets local data.
- `db.rs` executes `migrations/001_init.sql` (the full, re-runnable schema) on every start, then an `ALTER TABLE` compatibility loop for old databases. Add schema changes to `001_init.sql` and, for existing DBs, the compatibility list.
- `wbi_cache` exists but is unused; WBI keys are fetched again on every Bilibili request.
- `AppSettings` is duplicated in Rust (`settings.rs`) and TypeScript (`types.ts`); new fields need both sides plus sensible defaults for old `settings.json` files. The Rust struct uses `#[serde(default)]` so adding fields does not wipe existing settings.

## External Prerequisites

- Bilibili downloads need `ffmpeg` (settings path, resource dir, then PATH) and a valid cookie (`SESSDATA`, `bili_jct`, `DedeUserID`) for favorites, subtitles, and restricted content.
- Local transcription needs a GGML Whisper model path; online transcription uses an OpenAI-compatible `/audio/transcriptions` endpoint.
- AI organize uses an OpenAI-compatible `/chat/completions` endpoint or Ollama `/api/chat`. A base URL containing `localhost` or `11434` is treated as Ollama.
- Downloads are Bilibili-only today; other providers only fetch metadata.

See `README.md` for Bilibili WBI signing and cookie notes.
