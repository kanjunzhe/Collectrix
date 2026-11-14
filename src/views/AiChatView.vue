<template>
  <div class="ai-chat" style="max-width: 960px">
    <div class="d-flex align-center mb-4">
      <div>
        <div class="text-h5 font-weight-bold">{{ t("AI 对话") }}</div>
        <div class="text-caption text-medium-emphasis">
          {{ t("让 AI 帮你从收藏中找到想看的内容") }}
        </div>
      </div>
      <v-spacer />
      <v-combobox
        v-model="selectedModel"
        :items="availableModels"
        :label="t('模型')"
        density="compact"
        variant="outlined"
        hide-details
        style="max-width: 260px"
        :loading="loadingModels"
        :disabled="sending"
        @update:model-value="onModelChange"
      />
      <v-btn
        icon
        variant="text"
        :title="t('刷新模型列表')"
        :loading="loadingModels"
        @click="loadModels"
      >
        <v-icon>mdi-refresh</v-icon>
      </v-btn>
      <v-btn
        variant="text"
        prepend-icon="mdi-delete-sweep-outline"
        :disabled="messages.length === 0 || sending"
        @click="clearChat"
      >
        {{ t("清空对话") }}
      </v-btn>
    </div>

    <v-card variant="outlined" class="chat-shell">
      <div ref="messagesEl" class="chat-messages">
        <div v-if="messages.length === 0" class="empty-state">
          <v-icon size="56" opacity="0.25">mdi-chat-processing-outline</v-icon>
          <div class="text-h6 mt-3">{{ t("问我任何关于视频库的问题") }}</div>
          <div class="text-body-2 text-medium-emphasis mt-2">
            {{
              t(
                "例如：找一下关于 Rust 的视频、上周收藏的编程教程、和某个主题相关的视频……",
              )
            }}
          </div>
          <div class="text-caption text-medium-emphasis mt-2">
            {{
              t(
                "AI 会结合你收藏的视频标题、描述、分类和标签来回答。",
              )
            }}
          </div>
        </div>

        <div
          v-for="(message, index) in messages"
          :key="index"
          class="message-row"
          :class="message.role"
        >
          <v-avatar
            size="32"
            :color="message.role === 'user' ? 'primary' : 'secondary'"
          >
            <v-icon size="18">
              {{ message.role === "user" ? "mdi-account" : "mdi-robot" }}
            </v-icon>
          </v-avatar>
          <div class="message-body">
            <div class="message-bubble">{{ message.content }}</div>
            <div v-if="message.videos?.length" class="result-list">
              <div class="text-caption text-medium-emphasis mb-1">
                {{ t("找到 {n} 个相关视频，点击可播放：", { n: message.videos.length }) }}
              </div>
              <v-card
                v-for="video in message.videos"
                :key="video.id"
                variant="tonal"
                class="result-card"
                @click="$emit('play', video)"
              >
                <div class="d-flex align-center" style="gap: 10px">
                  <img
                    v-if="videoCoverSrc(video)"
                    :src="videoCoverSrc(video)"
                    referrerpolicy="no-referrer"
                    class="result-cover"
                    alt="cover"
                    loading="lazy"
                    decoding="async"
                  />
                  <div v-else class="result-cover result-cover-empty">
                    <v-icon size="24" opacity="0.35">mdi-video-off-outline</v-icon>
                  </div>
                  <div class="flex-grow-1 min-w-0">
                    <div class="text-body-2 font-weight-medium text-truncate">
                      {{ video.title }}
                    </div>
                    <div class="text-caption text-medium-emphasis">
                      {{ video.author || t("未知作者") }}
                    </div>
                    <div class="d-flex flex-wrap mt-1" style="gap: 4px">
                      <v-chip
                        v-for="tag in video.tags.slice(0, 4)"
                        :key="tag.id"
                        size="x-small"
                        :color="tag.color"
                        variant="flat"
                      >
                        {{ tag.name }}
                      </v-chip>
                    </div>
                  </div>
                  <v-btn
                    icon
                    size="small"
                    variant="text"
                    :title="t('编辑')"
                    @click.stop="$emit('edit', video)"
                  >
                    <v-icon>mdi-pencil</v-icon>
                  </v-btn>
                </div>
              </v-card>
            </div>
          </div>
        </div>

        <div v-if="sending" class="message-row assistant">
          <v-avatar size="32" color="secondary">
            <v-icon size="18">mdi-robot</v-icon>
          </v-avatar>
          <div class="message-body">
            <div class="message-bubble typing">
              <v-progress-circular indeterminate size="14" width="2" />
              {{ t("思考中…") }}
            </div>
          </div>
        </div>
      </div>

      <v-divider />
      <v-card-actions class="chat-input">
        <v-textarea
          v-model="input"
          :placeholder="t('输入问题…')"
          rows="1"
          auto-grow
          density="compact"
          variant="outlined"
          hide-details
          :disabled="sending"
          @keydown.enter.exact.prevent="onSend"
        />
        <v-btn
          color="primary"
          :loading="sending"
          :disabled="!input.trim() || sending"
          @click="onSend"
        >
          {{ t("发送") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, VideoRow } from "../types";
import { notify } from "../composables/useUI";
import { videoCoverSrc } from "../utils/media";
import { t } from "../i18n";

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
  videos?: VideoRow[];
}

interface AiChatReply {
  reply: string;
  videos: VideoRow[];
}

const emit = defineEmits<{
  play: [video: VideoRow];
  edit: [video: VideoRow];
}>();

const STORAGE_KEY = "collectrix.aiChat.v1";

function loadMessages(): ChatMessage[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (item) =>
        item &&
        (item.role === "user" || item.role === "assistant") &&
        typeof item.content === "string",
    );
  } catch {
    return [];
  }
}

const messages = ref<ChatMessage[]>(loadMessages());
const input = ref("");
const sending = ref(false);
const messagesEl = ref<HTMLElement | null>(null);
const settings = ref<AppSettings | null>(null);
const selectedModel = ref("");
const availableModels = ref<string[]>([]);
const loadingModels = ref(false);

watch(
  messages,
  (value) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
    } catch {
      // ignore storage errors
    }
  },
  { deep: true },
);

async function loadSettings() {
  try {
    settings.value = await invoke<AppSettings>("get_settings");
    selectedModel.value = settings.value.ai_model;
  } catch (e) {
    console.warn("加载 AI 设置失败", e);
  }
}

async function loadModels() {
  if (!settings.value?.openai_base_url) return;
  loadingModels.value = true;
  try {
    availableModels.value = await invoke<string[]>("list_ai_models");
  } catch (e) {
    notify.warning(t("获取模型列表失败：{e}", { e: String(e) }));
  } finally {
    loadingModels.value = false;
  }
}

async function onModelChange(value: string | null) {
  const model = (value ?? "").trim();
  if (!model || !settings.value || model === settings.value.ai_model) return;
  try {
    settings.value.ai_model = model;
    await invoke("set_settings", { settings: settings.value });
    notify.success(t("已切换模型：{model}", { model }));
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  }
}

onMounted(async () => {
  await loadSettings();
  await loadModels();
  await scrollToBottom();
});

async function scrollToBottom() {
  await nextTick();
  if (messagesEl.value) {
    messagesEl.value.scrollTop = messagesEl.value.scrollHeight;
  }
}

function clearChat() {
  messages.value = [];
}

async function onSend() {
  const text = input.value.trim();
  if (!text || sending.value) return;

  const history = messages.value.map((message) => {
    const refs = message.videos?.length
      ? `\n[相关视频: ${message.videos
          .map((video) => `#${video.id} ${video.title}`)
          .join("; ")}]`
      : "";
    return {
      role: message.role,
      content: message.content + refs,
    };
  });

  messages.value.push({ role: "user", content: text });
  input.value = "";
  sending.value = true;
  await scrollToBottom();

  try {
    const result = await invoke<AiChatReply>("ai_chat", {
      message: text,
      history,
    });
    messages.value.push({
      role: "assistant",
      content: result.reply,
      videos: result.videos,
    });
  } catch (e) {
    const message = t("对话失败：{e}", { e: String(e) });
    messages.value.push({ role: "assistant", content: message });
    notify.error(message);
  } finally {
    sending.value = false;
    await scrollToBottom();
  }
}
</script>

<style scoped>
.chat-shell {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 190px);
  min-height: 420px;
}

.chat-messages {
  flex: 1 1 auto;
  overflow-y: auto;
  padding: 20px;
}

.empty-state {
  min-height: 260px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
}

.message-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-bottom: 18px;
}

.message-row.user {
  flex-direction: row-reverse;
}

.message-body {
  max-width: min(720px, 82%);
  min-width: 0;
}

.message-bubble {
  white-space: pre-wrap;
  word-break: break-word;
  border-radius: 12px;
  padding: 10px 12px;
  background: rgba(var(--v-theme-surface-variant), 0.55);
  line-height: 1.55;
}

.message-row.user .message-bubble {
  background: rgba(var(--v-theme-primary), 0.12);
}

.message-bubble.typing {
  display: flex;
  align-items: center;
  gap: 8px;
}

.result-list {
  margin-top: 10px;
}

.result-card {
  cursor: pointer;
  padding: 8px;
  margin-bottom: 8px;
}

.result-cover {
  width: 96px;
  height: 56px;
  object-fit: cover;
  border-radius: 6px;
  flex-shrink: 0;
  background: rgba(0, 0, 0, 0.06);
}

.result-cover-empty {
  display: flex;
  align-items: center;
  justify-content: center;
}

.chat-input {
  display: flex;
  align-items: flex-end;
  gap: 8px;
  padding: 12px;
}

.chat-input :deep(.v-field) {
  flex: 1 1 auto;
}

.min-w-0 {
  min-width: 0;
}
</style>
