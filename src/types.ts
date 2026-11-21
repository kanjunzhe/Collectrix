export interface ContextMenuItem {
  key: string;
  title?: string;
  icon?: string;
  color?: string;
  disabled?: boolean;
  divider?: boolean;
}

export interface CategoryNode {
  id: number;
  name: string;
  parent_id: number | null;
  children: CategoryNode[];
}

export interface TagInfo {
  id: number;
  name: string;
  color: string;
}

export interface CountItem {
  name: string;
  count: number;
}

export interface MonthlyCount {
  month: string;
  count: number;
}

export interface DetailedStats {
  videos_count: number;
  categories_count: number;
  tags_count: number;
  topics_count: number;
  ai_sorted_count: number;
  local_video_count: number;
  local_audio_count: number;
  transcript_count: number;
  platforms: CountItem[];
  categories: CountItem[];
  tags: CountItem[];
  monthly: MonthlyCount[];
}

export interface Stats {
  videos_count: number;
  categories_count: number;
  tags_count: number;
  recent_videos: VideoRow[];
}

export interface BiliVideoData {
  bvid: string;
  title: string;
  pic: string;
  desc: string;
  duration: number;
  owner: { name: string };
  aid: number;
  cid: number; // 下载需要 cid
}

export interface VideoRow {
  id: number;
  platform: string;
  platform_id: string;
  title: string;
  author: string | null;
  cover_url: string | null;
  category_id: number | null;
  original_url: string | null;
  aid: number | null;
  cid: number | null;
  local_video_path: string | null;
  local_audio_path: string | null;
  transcript_path: string | null;
  local_cover_path: string | null;
  local_subtitle_paths: string[];
  description: string | null;
  is_ai_sorted: boolean;
  is_recent: boolean;
  tags: TagInfo[];
}

export interface CategoryVideoGroup {
  category_id: number | null;
  category_name: string;
  videos: VideoRow[];
}
export interface TopicInfo {
  id: number;
  name: string;
  description: string | null;
  color: string;
  video_count: number;
}

export interface UpdateInfo {
  current_version: string;
  latest_version: string;
  has_update: boolean;
  release_url: string | null;
  download_url: string | null;
  notes: string | null;
  published_at: string | null;
}

export interface NotebookInfo {
  id: number;
  title: string;
  content: string;
  video_count: number;
  created_at: string;
  updated_at: string;
}

export interface NotebookDetail {
  id: number;
  title: string;
  content: string;
  videos: VideoRow[];
}

export interface MindMapNodeData {
  id: string;
  label: string;
  kind: "root" | "category" | "video" | "more";
  count?: number;
  children?: MindMapNodeData[];
  video?: VideoRow;
}

export interface AiFavCandidate {
  bvid: string;
  title: string;
  author: string | null;
  cover_url: string | null;
  description: string | null;
  duration: number | null;
  score: number;
  in_library: boolean;
}

export interface AiSuggestion {
  video_id: number;
  title: string;
  cover_url: string | null;
  suggested_category: string;
  suggested_category_id: number | null;
  parent_category_id: number | null;
  category_path?: string[] | null;
  suggested_tags: string[];
  summary: string;
  confidence: number;
}

export interface CategoryFlat {
  id: number;
  name: string;
  parent_id: number | null;
  path: string; // "编程 / Rust / 异步"
  depth: number;
}

export interface AppSettings {
  // 外观
  theme: string; // "light" | "dark" | "system"
  language: string; // "zh" | "en"

  // 保存路径
  video_dir: string;
  audio_dir: string;
  transcript_dir: string;
  ffmpeg_path: string; // 空字符串 = 用系统 PATH

  // 下载选项
  download_video_quality: string; // "best" | "4k" | "1080p" | "720p" | "480p" | "360p"
  download_audio_quality: string; // "best" | "high" | "medium" | "low"
  download_video: boolean;
  download_audio: boolean;
  download_subtitles: boolean;
  download_danmaku: boolean;
  download_cover: boolean;

  // 本地语音转录
  whisper_model_path: string;
  whisper_language: string; // "auto" | "zh" | "en" | "ja" ...

  // 在线语音转录
  transcribe_provider: string; // "local" | "openai" | "bilibili"
  transcribe_auto_polish: boolean;
  openai_api_key: string;
  openai_base_url: string;
  openai_transcribe_model: string; // "whisper-1" 等

  // AI 智能整理（复用上面的 API Key / Base URL）
  ai_enabled: boolean;
  ai_provider: string; // "openai" | "ollama" | "custom"
  ai_model: string; // "gpt-4o-mini" / "qwen2.5:3b" ...
  ollama_path: string; // Ollama 可执行文件路径，空 = 系统 PATH

  // Bilibili
  bilibili_cookie: string; // 完整 Cookie 字符串

  // GitHub
  github_username: string;
  github_token: string;

  // UI
  recent_days: number; // 「近期新增」徽标判定天数

  // 应用锁
  lock_enabled: boolean;
  lock_pin_hash: string;
}

export interface DownloadResult {
  video_path: string | null;
  audio_path: string | null;
  subtitle_paths: string[];
  danmaku_path: string | null;
  cover_path: string | null;
}

export interface BiliSearchItem {
  bvid: string;
  title: string;
  author: string | null;
  cover_url: string | null;
  description: string | null;
  duration: number | null;
}

export interface LocalVideoCandidate {
  video_path: string;
  file_name: string;
  title: string;
  folder: string;
  cover_path: string | null;
  audio_path: string | null;
  subtitle_paths: string[];
  transcript_path: string | null;
  already_imported: boolean;
  matched?: VideoMeta | null;
}

export interface LocalImportItem {
  video_path: string;
  title: string;
  cover_path: string | null;
  audio_path: string | null;
  subtitle_paths: string[];
  transcript_path: string | null;
  matched: VideoMeta | null;
}

export interface VideoMeta {
  platform: string;
  platform_id: string;
  title: string;
  author: string | null;
  cover_url: string | null;
  description: string | null;
  duration: number | null;
  original_url: string;
  aid: number | null;
  cid: number | null;
}
