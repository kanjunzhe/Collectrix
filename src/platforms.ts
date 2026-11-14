// src/platforms.ts

export type ContentKind = "video" | "article";

export interface PlatformConfig {
  id: string;
  name: string; // 显示名："Bilibili" / "YouTube"
  kind: ContentKind; // 视频 or 图文
  icon: string; // mdi 图标
  color: string; // 品牌色
  canEmbed: boolean; // 是否支持 iframe 嵌入播放
  canDownload: boolean;
  canComment: boolean;
  canDanmaku: boolean;
  /** 给定 platform_id 返回 iframe 嵌入 URL；canEmbed=false 时忽略 */
  embedUrl?: (platformId: string) => string;
  /** 是否能在应用内直接播放 */
  canPlayInApp: boolean;
}

export const PLATFORMS: Record<string, PlatformConfig> = {
  local: {
    id: "local",
    name: "本地",
    kind: "video",
    icon: "mdi-harddisk",
    color: "#546E7A",
    canEmbed: false,
    canDownload: false,
    canComment: false,
    canDanmaku: false,
    canPlayInApp: true,
  },
  bilibili: {
    id: "bilibili",
    name: "Bilibili",
    kind: "video",
    icon: "mdi-television-classic",
    color: "#FB7299",
    canEmbed: true,
    canDownload: true,
    canComment: true,
    canDanmaku: true,
    canPlayInApp: true,
    embedUrl: (bvid) =>
      `https://player.bilibili.com/player.html?bvid=${encodeURIComponent(bvid)}&page=1&high_quality=1&autoplay=0`,
  },
  bilibili_article: {
    id: "bilibili_article",
    name: "B站专栏",
    kind: "article",
    icon: "mdi-text-box-outline",
    color: "#00A1D6",
    canEmbed: false,
    canDownload: false,
    canComment: true,
    canDanmaku: false,
    canPlayInApp: false,
  },
  github: {
    id: "github",
    name: "GitHub",
    kind: "article",
    icon: "mdi-github",
    color: "#24292F",
    canEmbed: false,
    canDownload: false,
    canComment: false,
    canDanmaku: false,
    canPlayInApp: false,
  },
  blog: {
    id: "blog",
    name: "博客",
    kind: "article",
    icon: "mdi-post-outline",
    color: "#607D8B",
    canEmbed: false,
    canDownload: false,
    canComment: false,
    canDanmaku: false,
    canPlayInApp: false,
  },
  youtube: {
    id: "youtube",
    name: "YouTube",
    kind: "video",
    icon: "mdi-youtube",
    color: "#FF0000",
    canEmbed: true,
    canDownload: false,
    canComment: false,
    canDanmaku: false,
    canPlayInApp: true,
    embedUrl: (vid) =>
      `https://www.youtube.com/embed/${encodeURIComponent(vid)}?autoplay=0`,
  },
  douyin: {
    id: "douyin",
    name: "抖音",
    kind: "video",
    icon: "mdi-music-note",
    color: "#000000",
    canEmbed: false,
    canDownload: false,
    canComment: false,
    canDanmaku: false,
    canPlayInApp: false,
  },
  wechat_channels: {
    id: "wechat_channels",
    name: "视频号",
    kind: "video",
    icon: "mdi-wechat",
    color: "#07C160",
    canEmbed: false,
    canDownload: false,
    canComment: false,
    canDanmaku: false,
    canPlayInApp: false,
  },
};

export function getPlatform(id: string): PlatformConfig {
  return (
    PLATFORMS[id] ?? {
      id,
      name: id,
      kind: "video",
      icon: "mdi-video",
      color: "#607D8B",
      canEmbed: false,
      canDownload: false,
      canComment: false,
      canDanmaku: false,
      canPlayInApp: false,
    }
  );
}

/** 根据 URL 猜测平台（前端预判，最终以后端返回为准） */
export function guessPlatform(url: string): string | null {
  if (url.includes("bilibili.com/video") || url.includes("BV"))
    return "bilibili";
  if (url.includes("bilibili.com/read")) return "bilibili_article";
  if (url.includes("github.com")) return "github";
  if (url.includes("youtube.com") || url.includes("youtu.be")) return "youtube";
  if (url.includes("douyin.com")) return "douyin";
  if (
    url.includes("channels.weixin.qq.com") ||
    url.includes("weixin.qq.com/sph")
  )
    return "wechat_channels";
  if (/^https?:\/\//i.test(url)) return "blog";
  return null;
}
