// src/composables/useUI.ts
import { reactive, ref } from "vue";
import { t } from "../i18n";

// ============ 通知 ============

export type NotifyType = "success" | "error" | "warning" | "info";

export interface NotifyItem {
  id: number;
  type: NotifyType;
  message: string;
  timeout: number;
}

let nextId = 1;

/** 内部队列：App.vue 消费它决定当前显示哪一条 */
export const notifyQueue = reactive<{ items: NotifyItem[] }>({ items: [] });
export const currentNotify = ref<NotifyItem | null>(null);
export const notifyVisible = ref(false);

function enqueue(type: NotifyType, message: string, timeout: number) {
  notifyQueue.items.push({
    id: nextId++,
    type,
    message,
    timeout,
  });
  pump();
}

function pump() {
  if (notifyVisible.value) return;
  const next = notifyQueue.items.shift();
  if (!next) return;
  currentNotify.value = next;
  notifyVisible.value = true;
}

export function dismissCurrentNotify() {
  notifyVisible.value = false;
  currentNotify.value = null;
  // 给 Vuetify 动画留一点时间再切下一条
  setTimeout(pump, 200);
}

export const notify = {
  success: (msg: string, timeout = 3500) => enqueue("success", msg, timeout),
  error: (msg: string, timeout = 6000) => enqueue("error", msg, timeout),
  warning: (msg: string, timeout = 5000) => enqueue("warning", msg, timeout),
  info: (msg: string, timeout = 3500) => enqueue("info", msg, timeout),
};

// ============ 确认对话框 ============

export interface ConfirmOptions {
  title?: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  confirmColor?: string;
  persistent?: boolean;
}

export const confirmState = reactive({
  visible: false,
  title: t("确认"),
  message: "",
  confirmText: t("确定"),
  cancelText: t("取消"),
  confirmColor: "primary",
  persistent: true,
  resolve: null as null | ((v: boolean) => void),
});

/**
 * 用法：
 *   const ok = await confirmDialog("确定删除？");
 *   const ok = await confirmDialog({ title: "删除", message: "...", confirmColor: "error" });
 */
export function confirmDialog(
  options: string | ConfirmOptions,
): Promise<boolean> {
  const opts: ConfirmOptions =
    typeof options === "string" ? { message: options } : options;

  confirmState.title = opts.title ?? t("确认");
  confirmState.message = opts.message;
  confirmState.confirmText = opts.confirmText ?? t("确定");
  confirmState.cancelText = opts.cancelText ?? t("取消");
  confirmState.confirmColor = opts.confirmColor ?? "primary";
  confirmState.persistent = opts.persistent ?? true;
  confirmState.visible = true;

  return new Promise<boolean>((resolve) => {
    confirmState.resolve = resolve;
  });
}

export function resolveConfirm(v: boolean) {
  confirmState.visible = false;
  const r = confirmState.resolve;
  confirmState.resolve = null;
  r?.(v);
}
