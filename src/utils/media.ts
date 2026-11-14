import { convertFileSrc } from "@tauri-apps/api/core";
import type { VideoRow } from "../types";

const REMOTE_URL_RE = /^(https?:|data:|blob:|asset:|\/\/)/i;

/** 把本地封面路径或远程封面 URL 转成 <img> 可用的地址。 */
export function videoCoverSrc(
  video: Pick<VideoRow, "cover_url" | "local_cover_path">,
): string {
  if (video.local_cover_path) return convertFileSrc(video.local_cover_path);
  const url = video.cover_url?.trim();
  if (!url) return "";
  if (REMOTE_URL_RE.test(url)) {
    return url.startsWith("//") ? `https:${url}` : url;
  }
  return convertFileSrc(url);
}

/** 任意本地文件路径转成 Tauri asset URL。 */
export function localFileSrc(path: string | null | undefined): string {
  return path ? convertFileSrc(path) : "";
}
