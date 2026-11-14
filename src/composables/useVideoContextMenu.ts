import { computed, reactive } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { ContextMenuItem, VideoRow } from "../types";
import { notify } from "./useUI";
import { t } from "../i18n";

interface VideoContextMenuOptions {
  onPlay: (video: VideoRow) => void;
  onEdit?: (video: VideoRow) => void;
  onDelete?: (video: VideoRow) => void | Promise<void>;
  extraItems?: (video: VideoRow) => ContextMenuItem[];
  onExtraAction?: (key: string, video: VideoRow) => void | Promise<void>;
}

export function useVideoContextMenu(options: VideoContextMenuOptions) {
  const state = reactive({
    visible: false,
    x: 0,
    y: 0,
    video: null as VideoRow | null,
  });

  function open(event: MouseEvent, video: VideoRow) {
    event.preventDefault();
    event.stopPropagation();
    state.video = video;
    state.x = event.clientX;
    state.y = event.clientY;
    state.visible = true;
  }

  function close() {
    state.visible = false;
  }

  const items = computed<ContextMenuItem[]>(() => {
    const video = state.video;
    if (!video) return [];

    const result: ContextMenuItem[] = [
      { key: "play", title: t("播放"), icon: "mdi-play" },
    ];

    if (options.onEdit) {
      result.push({ key: "edit", title: t("编辑"), icon: "mdi-pencil" });
    }

    result.push(
      {
        key: "copyLink",
        title: t("复制链接"),
        icon: "mdi-link-variant",
        disabled: !video.original_url,
      },
      {
        key: "openOriginal",
        title: t("在浏览器打开"),
        icon: "mdi-open-in-new",
        disabled: !video.original_url,
      },
    );

    const extra = options.extraItems?.(video) ?? [];
    if (extra.length > 0) {
      result.push({ key: "extra-divider", divider: true }, ...extra);
    }

    if (options.onDelete) {
      result.push(
        { key: "delete-divider", divider: true },
        { key: "delete", title: t("删除"), icon: "mdi-delete", color: "error" },
      );
    }

    return result;
  });

  async function select(key: string) {
    const video = state.video;
    close();
    if (!video) return;

    switch (key) {
      case "play":
        options.onPlay(video);
        return;
      case "edit":
        options.onEdit?.(video);
        return;
      case "delete":
        await options.onDelete?.(video);
        return;
      case "copyLink":
        try {
          await navigator.clipboard.writeText(video.original_url ?? "");
          notify.success(t("已复制链接"));
        } catch (e) {
          notify.error(t("复制失败：{e}", { e: String(e) }));
        }
        return;
      case "openOriginal":
        if (!video.original_url) return;
        try {
          await openUrl(video.original_url);
        } catch (e) {
          notify.error(t("打开链接失败：{e}", { e: String(e) }));
        }
        return;
      default:
        await options.onExtraAction?.(key, video);
    }
  }

  return {
    contextMenu: state,
    contextMenuItems: items,
    openContextMenu: open,
    closeContextMenu: close,
    onContextMenuSelect: select,
  };
}
