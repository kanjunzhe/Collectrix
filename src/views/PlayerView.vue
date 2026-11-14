<template>
  <div v-if="!video" class="pa-6 text-medium-emphasis">{{ t("未选择视频") }}</div>

  <div v-else>
    <!-- 标题栏：平台徽标放在标题旁 -->
    <div class="d-flex align-center mb-3">
      <v-btn icon="mdi-arrow-left" variant="text" @click="$emit('back')" />
      <v-chip size="small" :color="platform.color" variant="flat" class="ml-2">
        <v-icon start size="14">{{ platform.icon }}</v-icon>
        {{ t(platform.name) }}
      </v-chip>
      <div class="ml-3 min-w-0 flex-1">
        <div class="text-h6 text-truncate">{{ video.title }}</div>
        <div class="text-caption text-medium-emphasis">{{ video.author }}</div>
      </div>

      <!-- 下载：仅 canDownload 平台显示 -->
      <v-btn
        v-if="platform.canDownload && !hasLocal"
        size="small"
        variant="tonal"
        prepend-icon="mdi-download"
        :loading="downloading"
        :disabled="!video.cid"
        @click="onDownload"
      >
        {{ t("下载视频") }}
      </v-btn>

      <v-btn
        v-if="canOpenNewWindow"
        size="small"
        variant="text"
        :title="t('在新窗口播放')"
        icon="mdi-open-in-new"
        @click="openInNewWindow"
      />

      <v-btn
        size="small"
        variant="text"
        :title="t('编辑')"
        icon="mdi-pencil"
        @click="$emit('edit', video)"
      />

      <!-- 在浏览器打开：始终显示，但不是 canPlayInApp 时作为主按钮 -->
      <v-btn
        v-if="video.original_url"
        size="small"
        :variant="platform.canPlayInApp ? 'text' : 'tonal'"
        :color="platform.canPlayInApp ? undefined : 'primary'"
        :href="video.original_url"
        :title="t('在浏览器打开')"
        target="_blank"
      >
        <v-icon :start="!platform.canPlayInApp" size="18">
          mdi-web
        </v-icon>
        <span v-if="!platform.canPlayInApp">{{ t("在浏览器打开") }}</span>
      </v-btn>
    </div>

    <!-- 分发：图文 → ArticleView；视频 → 播放区 -->
    <ArticleView v-if="platform.kind === 'article'" :video="video" />

    <template v-else>
      <v-row>
        <v-col cols="12" lg="8">
          <!-- 本地视频 -->
          <DanmakuPlayer
            v-if="
              hasLocal &&
              localSrc &&
              !localVideoBroken &&
              (platform.canDanmaku ||
                (video.local_subtitle_paths?.length ?? 0) > 0)
            "
            :video-src="localSrc"
            :bvid="video.platform_id"
            :cid="video.cid ?? 0"
            :local-subtitle-paths="video.local_subtitle_paths"
            @invalid="localVideoBroken = true"
          />
          <video
            v-else-if="hasLocal && localSrc && !localVideoBroken"
            :src="localSrc"
            controls
            class="player-video"
            @error="localVideoBroken = true"
            @loadedmetadata="checkLocalVideoTrack"
          />

          <!-- iframe 嵌入 -->
          <v-sheet
            v-else-if="embedUrl"
            class="rounded"
            style="overflow: hidden"
          >
            <iframe
              :src="embedUrl"
              class="player-iframe"
              allowfullscreen
              frameborder="0"
            />
          </v-sheet>

          <!-- 无法嵌入：显示占位 + 跳转提示 -->
          <v-sheet
            v-else
            class="rounded d-flex flex-column align-center justify-center"
            height="360"
          >
            <v-icon size="64" opacity="0.3" :color="platform.color">
              {{ platform.icon }}
            </v-icon>
            <div class="text-h6 mt-4">
              {{
                t("{platform} 暂不支持内嵌播放", {
                  platform: t(platform.name),
                })
              }}
            </div>
            <div class="text-caption text-medium-emphasis mt-1">
              {{ t("点击右上角「在浏览器打开」查看原内容") }}
            </div>
          </v-sheet>

          <!-- 操作栏：标签 + 转录（仅本地音频时） -->
          <div class="mt-3 d-flex flex-wrap align-center" style="gap: 8px">
            <v-chip
              v-for="tag in video.tags"
              :key="tag.id"
              :color="tag.color"
              variant="flat"
            >
              {{ tag.name }}
            </v-chip>
            <v-spacer />
            <v-btn
              v-if="canTranscribe"
              size="small"
              variant="tonal"
              prepend-icon="mdi-text-recognition"
              :loading="transcribing"
              @click="onTranscribe"
            >
              {{ t("转录文字稿") }}
            </v-btn>
          </div>

          <!-- 文字稿 -->
          <v-expand-transition>
            <v-card v-if="transcript" variant="outlined" class="mt-3">
              <v-card-title class="d-flex align-center">
                {{ t("文字稿") }}
                <v-spacer />
                <v-btn
                  size="small"
                  variant="tonal"
                  prepend-icon="mdi-robot-outline"
                  :loading="polishing"
                  @click="polishTranscript"
                >
                  {{ t("AI 加标点 / 校对") }}
                </v-btn>
                <v-btn
                  size="small"
                  variant="text"
                  prepend-icon="mdi-content-copy"
                  @click="copyTranscript"
                >
                  {{ t("复制") }}
                </v-btn>
              </v-card-title>
              <v-card-text class="transcript-box">{{ transcript }}</v-card-text>
            </v-card>
          </v-expand-transition>
        </v-col>

        <!-- 右侧：分类 / 标签 / 简介 / 链接 -->
        <v-col cols="12" lg="4">
          <v-card variant="outlined" class="mb-3">
            <v-card-title class="text-subtitle-1">{{ t("视频信息") }}</v-card-title>
            <v-card-text>
              <div class="meta-grid">
                <span class="meta-label">{{ t("分类") }}</span>
                <span class="meta-value">{{ categoryName }}</span>
                <span class="meta-label">{{ t("平台：") }}</span>
                <span class="meta-value">{{ t(platform.name) }}</span>
                <span class="meta-label">{{ t("作者") }}</span>
                <span class="meta-value">{{ video.author || t("未知作者") }}</span>
              </div>
              <div class="mt-3">
                <div class="meta-label mb-1">{{ t("标签") }}</div>
                <div class="d-flex flex-wrap" style="gap: 4px">
                  <v-chip
                    v-for="tag in video.tags"
                    :key="tag.id"
                    size="x-small"
                    :color="tag.color"
                    variant="flat"
                  >
                    {{ tag.name }}
                  </v-chip>
                  <span v-if="video.tags.length === 0" class="text-caption text-medium-emphasis">
                    {{ t("（无标签）") }}
                  </span>
                </div>
              </div>
            </v-card-text>
          </v-card>

          <v-card variant="outlined" class="mb-3">
            <v-card-title class="text-subtitle-1">{{ t("简介") }}</v-card-title>
            <v-card-text class="desc-box">
              <LinkCards v-if="video.description" :text="video.description" />
              <LinkifiedText
                v-if="video.description"
                :text="video.description"
              />
              <span v-else class="text-medium-emphasis">{{ t("（无描述）") }}</span>
            </v-card-text>
          </v-card>

          <v-card variant="outlined">
            <v-card-title class="text-subtitle-1">{{ t("相关链接") }}</v-card-title>
            <v-card-text>
              <div class="d-flex flex-wrap" style="gap: 8px">
                <v-btn
                  v-if="authorLink"
                  size="small"
                  variant="tonal"
                  prepend-icon="mdi-account-circle-outline"
                  @click="onOpenLink(authorLink)"
                >
                  {{ t("作者主页") }}
                </v-btn>
                <v-btn
                  v-if="commentsLink"
                  size="small"
                  variant="tonal"
                  prepend-icon="mdi-comment-outline"
                  @click="onOpenLink(commentsLink)"
                >
                  {{ t("查看评论") }}
                </v-btn>
                <v-btn
                  v-if="video.original_url"
                  size="small"
                  variant="text"
                  prepend-icon="mdi-open-in-new"
                  @click="onOpenLink(video.original_url ?? '')"
                >
                  {{ t("打开原视频") }}
                </v-btn>
                <v-btn
                  size="small"
                  variant="tonal"
                  prepend-icon="mdi-notebook-plus-outline"
                  @click="showNotebookPicker = true"
                >
                  {{ t("添加到记事本") }}
                </v-btn>
              </div>
            </v-card-text>
          </v-card>

          <NotebookPickerDialog
            v-model="showNotebookPicker"
            :video-id="video.id"
          />
        </v-col>
      </v-row>

      <!-- 评论：按需加载，避免播放窗口启动时同时请求评论接口 -->
      <div v-if="platform.canComment" class="mt-4">
        <v-btn
          variant="tonal"
          prepend-icon="mdi-comment-outline"
          @click="showComments = !showComments"
        >
          {{ showComments ? t("收起评论") : t("查看评论") }}
        </v-btn>
        <v-expand-transition>
          <div v-if="showComments" class="mt-3">
            <CommentSection :aid="video.aid ?? null" />
          </div>
        </v-expand-transition>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, ref, watch } from "vue";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getPlatform } from "../platforms";
import DanmakuPlayer from "../components/DanmakuPlayer.vue";
import ArticleView from "../components/ArticleView.vue";
import LinkifiedText from "../components/LinkifiedText.vue";
import LinkCards from "../components/LinkCards.vue";
import NotebookPickerDialog from "../components/NotebookPickerDialog.vue";
import type { CategoryNode, DownloadResult, VideoRow } from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{ video: VideoRow | null }>();
defineEmits<{ back: []; edit: [video: VideoRow] }>();

const CommentSection = defineAsyncComponent(
  () => import("../components/CommentSection.vue"),
);

const canOpenNewWindow = !/Android/i.test(navigator.userAgent);
const downloading = ref(false);
const transcribing = ref(false);
const polishing = ref(false);
const transcript = ref("");
const categoryMap = ref<Record<number, string>>({});
const localVideoBroken = ref(false);
const showNotebookPicker = ref(false);
const showComments = ref(false);

watch(
  () => props.video?.id,
  async () => {
    localVideoBroken.value = false;
    showComments.value = false;
    transcript.value = "";
    if (!props.video?.transcript_path) return;
    try {
      transcript.value =
        (await invoke<string | null>("get_transcript", {
          videoId: props.video.id,
        })) ?? "";
    } catch (e) {
      console.warn("加载文字稿失败", e);
    }
  },
  { immediate: true },
);

function checkLocalVideoTrack(event: Event) {
  const video = event.target as HTMLVideoElement;
  if (video.readyState >= 1 && video.videoWidth === 0) {
    localVideoBroken.value = true;
  }
}

const categoryName = computed(() => {
  const id = props.video?.category_id;
  return id ? (categoryMap.value[id] ?? t("未分类")) : t("未分类");
});

const authorLink = computed(() => {
  const video = props.video;
  if (!video?.author) return "";
  if (video.platform === "bilibili") {
    return `https://search.bilibili.com/upuser?keyword=${encodeURIComponent(video.author)}`;
  }
  if (video.platform === "youtube") {
    return `https://www.youtube.com/results?search_query=${encodeURIComponent(video.author)}`;
  }
  return "";
});

const commentsLink = computed(() => {
  const video = props.video;
  if (!video) return "";
  if (video.platform === "bilibili") {
    return `https://www.bilibili.com/video/${video.platform_id}#reply`;
  }
  if (video.platform === "youtube") {
    return `https://www.youtube.com/watch?v=${video.platform_id}`;
  }
  return "";
});

async function onOpenLink(url: string) {
  try {
    await openUrl(url);
  } catch (e) {
    notify.error(t("打开链接失败：{e}", { e: String(e) }));
  }
}

async function loadCategories() {
  try {
    const tree = await invoke<CategoryNode[]>("get_category_tree");
    const map: Record<number, string> = {};
    const walk = (nodes: CategoryNode[], prefix: string[]) => {
      for (const node of nodes) {
        const path = [...prefix, node.name];
        map[node.id] = path.join(" / ");
        walk(node.children, path);
      }
    };
    walk(tree, []);
    categoryMap.value = map;
  } catch (e) {
    console.warn("加载分类失败", e);
  }
}

onMounted(loadCategories);

const platform = computed(() =>
  getPlatform(props.video?.platform ?? "bilibili"),
);

const hasLocal = computed(() => {
  const path = props.video?.local_video_path;
  if (!path) return false;
  // 防止旧数据/音频下载被误当成视频播放。
  return !/\.(m4a|mp3|aac|wav|flac|ogg)$/i.test(path);
});

const canTranscribe = computed(() => {
  const video = props.video;
  if (!video) return false;
  return (
    !!video.local_audio_path ||
    !!video.local_subtitle_paths?.length ||
    (video.platform === "bilibili" && !!video.cid)
  );
});

const localSrc = computed(() => {
  if (!props.video?.local_video_path) return "";
  return convertFileSrc(props.video.local_video_path);
});

/** 由平台配置决定嵌入 URL，不再写 if-else */
const embedUrl = computed(() => {
  if (!props.video || !platform.value.canEmbed || !platform.value.embedUrl) {
    return "";
  }
  return platform.value.embedUrl(props.video.platform_id);
});

async function openInNewWindow() {
  if (!props.video) return;
  try {
    const label = `player-${props.video.id}-${Date.now()}`;
    const window = new WebviewWindow(label, {
      url: `index.html?player_id=${props.video.id}`,
      title: props.video.title,
      width: 1280,
      height: 800,
      focus: true,
    });
    window.once("tauri://error", (event) => {
      notify.error(t("打开新窗口失败：{e}", { e: String(event) }));
    });
  } catch (e) {
    notify.error(t("打开新窗口失败：{e}", { e: String(e) }));
  }
}

async function onDownload() {
  if (!props.video) return;
  downloading.value = true;
  try {
    await invoke<DownloadResult>("download_video", {
      videoId: props.video.id,
    });
    const updated = await invoke<VideoRow>("get_video", { id: props.video.id });
    Object.assign(props.video, updated);
    notify.success(t("下载完成"));
  } catch (e) {
    notify.error(t("下载失败：{e}", { e: String(e) }));
  } finally {
    downloading.value = false;
  }
}

async function onTranscribe() {
  if (!props.video) return;
  transcribing.value = true;
  try {
    transcript.value = await invoke<string>("transcribe_video", {
      videoId: props.video.id,
    });
  } catch (e) {
    notify.error(t("转录失败：{e}", { e: String(e) }));
  } finally {
    transcribing.value = false;
  }
}

async function polishTranscript() {
  if (!props.video || !transcript.value.trim()) return;
  polishing.value = true;
  try {
    transcript.value = await invoke<string>("polish_transcript", {
      videoId: props.video.id,
      text: transcript.value,
    });
    notify.success(t("AI 校对完成"));
  } catch (e) {
    notify.error(t("AI 校对失败：{e}", { e: String(e) }));
  } finally {
    polishing.value = false;
  }
}

async function copyTranscript() {
  await navigator.clipboard.writeText(transcript.value);
}
</script>

<style scoped>
.min-w-0 {
  min-width: 0;
}
.flex-1 {
  flex: 1 1 auto;
}
.player-video {
  width: 100%;
  aspect-ratio: 16 / 9;
  display: block;
  background: #000;
}
.player-iframe {
  width: 100%;
  aspect-ratio: 16 / 9;
  display: block;
  border: none;
}
.desc-box {
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 320px;
  overflow-y: auto;
  padding: 8px 10px;
  font-size: 13px;
  line-height: 1.6;
}
.transcript-box {
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 400px;
  overflow-y: auto;
  font-size: 13px;
  line-height: 1.55;
}
.meta-grid {
  display: grid;
  grid-template-columns: 88px 1fr;
  gap: 6px 12px;
  font-size: 13px;
}
.meta-label {
  opacity: 0.6;
}
.meta-value {
  min-width: 0;
}
</style>
