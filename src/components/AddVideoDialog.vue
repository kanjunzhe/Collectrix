<template>
  <v-dialog v-model="visible" max-width="640" persistent>
    <v-card>
      <v-card-title class="d-flex align-center">
        <v-icon class="mr-2">mdi-plus-circle</v-icon>
        {{ t("添加视频 / 图文") }}
      </v-card-title>
      <v-card-text>
        <!-- URL 输入 -->
        <v-text-field
          v-model="videoUrl"
          :label="t('链接')"
          :placeholder="t('粘贴任意平台的视频/图文链接')"
          :prepend-inner-icon="guessedPlatform?.icon ?? 'mdi-link'"
          :messages="
            guessedPlatform
              ? t('识别为 {name}', { name: t(guessedPlatform.name) })
              : t('支持 B站、YouTube、抖音、B站专栏、视频号')
          "
          :disabled="fetching || saving"
          @keydown.enter="onFetch()"
        >
          <template #append-inner>
            <v-chip
              v-if="guessedPlatform"
              size="x-small"
              :color="guessedPlatform.color"
              variant="flat"
            >
              {{ t(guessedPlatform.name) }}
            </v-chip>
          </template>
        </v-text-field>

        <v-alert
          v-if="fetchError"
          type="warning"
          variant="tonal"
          density="compact"
          class="mb-3"
        >
          {{ t("自动解析失败：{e}", { e: fetchError }) }}
        </v-alert>

        <!-- 抓取按钮 -->
        <div class="d-flex" style="gap: 8px">
          <v-btn
            color="primary"
            variant="tonal"
            prepend-icon="mdi-cloud-download"
            :loading="fetching"
            :disabled="!videoUrl.trim() || saving"
            @click="onFetch()"
          >
            {{ t("抓取信息") }}
          </v-btn>
          <v-btn
            variant="text"
            prepend-icon="mdi-pencil"
            :disabled="fetching || saving"
            @click="enterManualMode"
          >
            {{ t("手动录入") }}
          </v-btn>
          <v-chip
            v-if="manualMode"
            size="small"
            color="warning"
            variant="tonal"
            class="ml-2"
          >
            {{ t("手动模式") }}
          </v-chip>
        </div>

        <v-divider class="my-4" />

        <!-- 抓取结果预览 -->
        <template v-if="fetched && !manualMode">
          <div class="d-flex align-start" style="gap: 12px">
            <div class="preview-thumb">
              <img
                v-if="fetched.cover_url"
                :src="fetched.cover_url"
                referrerpolicy="no-referrer"
                alt="cover"
              />
              <div v-else class="preview-thumb-placeholder">
                <v-icon size="28" opacity="0.3">mdi-image-off</v-icon>
              </div>
            </div>
            <div class="flex-grow-1 min-w-0">
              <div
                class="text-subtitle-2 font-weight-medium"
                :title="fetched.title"
              >
                {{ fetched.title }}
              </div>
              <div class="text-caption text-medium-emphasis mt-1">
                {{ fetched.author || t("未知作者") }}
              </div>
              <div class="mt-2 d-flex flex-wrap align-center" style="gap: 6px">
                <v-chip
                  v-if="fetchedPlatform"
                  size="x-small"
                  :color="fetchedPlatform.color"
                  variant="flat"
                >
                  <v-icon start size="10">{{ fetchedPlatform.icon }}</v-icon>
                  {{ t(fetchedPlatform.name) }}
                </v-chip>
                <v-chip
                  v-if="fetchedPlatform"
                  size="x-small"
                  variant="text"
                  :color="
                    fetchedPlatform.kind === 'article' ? 'info' : undefined
                  "
                >
                  {{ fetchedPlatform.kind === "article" ? t("图文") : t("视频") }}
                </v-chip>
                <v-chip v-if="fetched.duration" size="x-small" variant="text">
                  {{ formatDuration(fetched.duration) }}
                </v-chip>
              </div>
            </div>
          </div>
        </template>

        <!-- 手动录入 -->
        <template v-if="manualMode">
          <v-alert type="info" variant="tonal" density="compact" class="mb-3">
            {{ t("该平台暂不支持自动解析，请手动填写基本信息。") }}
            <span v-if="guessedPlatform" class="text-caption">
              {{
                t("当前平台：{name}", { name: t(guessedPlatform.name) })
              }}
            </span>
          </v-alert>

          <v-text-field
            v-model="manual.title"
            :label="t('标题 *')"
            density="compact"
            variant="outlined"
            :disabled="saving"
          />
          <v-text-field
            v-model="manual.author"
            :label="t('作者（可选）')"
            density="compact"
            variant="outlined"
            :disabled="saving"
          />
          <v-text-field
            v-model="manual.cover_url"
            :label="t('封面 URL（可选）')"
            placeholder="https://..."
            density="compact"
            variant="outlined"
            :disabled="saving"
          >
            <template v-if="manual.cover_url" #append-inner>
              <img
                :src="manual.cover_url"
                referrerpolicy="no-referrer"
                class="inline-thumb"
                alt="preview"
              />
            </template>
          </v-text-field>
          <v-textarea
            v-model="manual.description"
            :label="t('描述（可选）')"
            rows="2"
            auto-grow
            density="compact"
            variant="outlined"
            :disabled="saving"
          />
        </template>

        <v-divider class="my-4" />

        <!-- 分类 -->
        <v-select
          v-model="targetCategory"
          :items="categories"
          item-title="label"
          item-value="id"
          :label="t('保存到分类（可选）')"
          density="compact"
          variant="outlined"
          clearable
          :disabled="saving"
        />

        <!-- 标签 -->
        <v-combobox
          v-model="tagInput"
          :label="t('标签（回车添加）')"
          multiple
          chips
          closable-chips
          density="compact"
          variant="outlined"
          :disabled="saving"
        />
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn :disabled="saving" @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          :disabled="!canSave || saving"
          :loading="saving"
          prepend-icon="mdi-content-save"
          @click="onSave"
        >
          {{ t("保存") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getPlatform, guessPlatform } from "../platforms";
import { notify } from "../composables/useUI";
import { t } from "../i18n";
import type { VideoMeta } from "../types";

const props = defineProps<{
  modelValue: boolean;
  categories: { id: number; label: string }[];
  initialCategoryId: number | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [v: boolean];
  saved: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit("update:modelValue", v),
});

// ============ 状态 ============

const videoUrl = ref("");
const fetching = ref(false);
const saving = ref(false);
const fetched = ref<VideoMeta | null>(null);
const fetchError = ref<string | null>(null);
let autoFetchTimer: ReturnType<typeof setTimeout> | null = null;
const targetCategory = ref<number | null>(null);
const tagInput = ref<string[]>([]);

const manualMode = ref(false);
const manual = ref({
  title: "",
  author: "",
  cover_url: "",
  description: "",
});

// ============ 派生值 ============

const guessedPlatform = computed(() => {
  const id = guessPlatform(videoUrl.value);
  return id ? getPlatform(id) : null;
});

const fetchedPlatform = computed(() => {
  if (!fetched.value) return null;
  return getPlatform(fetched.value.platform);
});

const canSave = computed(() => {
  if (manualMode.value) {
    return manual.value.title.trim().length > 0;
  }
  return !!fetched.value;
});

// ============ 打开时重置 ============

watch(visible, (v) => {
  clearAutoFetch();
  if (v) {
    videoUrl.value = "";
    fetched.value = null;
    fetchError.value = null;
    targetCategory.value = props.initialCategoryId;
    tagInput.value = [];
    manualMode.value = false;
    manual.value = {
      title: "",
      author: "",
      cover_url: "",
      description: "",
    };
  }
});

// GitHub 链接输入/粘贴后自动解析，省去手动点「抓取信息」。
watch(videoUrl, (value) => {
  fetchError.value = null;
  const url = value.trim();
  if (fetched.value && fetched.value.original_url !== url) {
    fetched.value = null;
  }
  if (!url || manualMode.value) {
    clearAutoFetch();
    return;
  }
  scheduleAutoFetch(url);
});

function clearAutoFetch() {
  if (autoFetchTimer !== null) {
    clearTimeout(autoFetchTimer);
    autoFetchTimer = null;
  }
}

function scheduleAutoFetch(url: string) {
  clearAutoFetch();
  if (!visible.value || fetching.value || manualMode.value) return;
  if (guessPlatform(url) !== "github") return;
  // 至少要有 owner/repo 两段路径，避免只输入 github.com 或用户名时误触发。
  if (
    !/^(https?:\/\/)?(www\.)?github\.com\/[^/?#]+\/[^/?#]+/i.test(
      normalizeUrl(url),
    )
  ) {
    return;
  }
  if (fetched.value?.original_url === url) return;

  autoFetchTimer = setTimeout(() => {
    autoFetchTimer = null;
    if (
      visible.value &&
      !fetching.value &&
      !manualMode.value &&
      videoUrl.value.trim() === url
    ) {
      void onFetch({ auto: true });
    }
  }, 600);
}

// ============ 抓取 ============

async function onFetch(options: { auto?: boolean } = {}) {
  clearAutoFetch();
  const url = normalizeUrl(videoUrl.value);
  if (!url || fetching.value) return;
  if (videoUrl.value.trim() !== url) {
    videoUrl.value = url;
  }

  const auto = options.auto === true;
  fetching.value = true;
  fetchError.value = null;
  manualMode.value = false;

  try {
    const meta = await invoke<VideoMeta>("fetch_video_meta", { url });
    if (videoUrl.value.trim() !== url) return;
    fetched.value = meta;
    if (!auto) {
      notify.success(t("已抓取：{title}", { title: meta.title }));
    }
  } catch (e) {
    const msg = String(e);
    fetchError.value = msg;

    if (auto) {
      // 自动解析失败时保留输入，用户仍可手动点击抓取或手动录入。
      return;
    }

    // 已知平台但无法自动解析时，自动进入手动模式
    if (guessedPlatform.value) {
      notify.warning(
        t("{name} 暂不支持自动解析，已切换为手动录入", {
          name: t(guessedPlatform.value.name),
        }),
      );
      enterManualMode();
    } else {
      notify.error(t("抓取失败：{e}", { e: msg }));
    }
  } finally {
    fetching.value = false;
  }
}

onBeforeUnmount(clearAutoFetch);

// ============ 手动模式 ============

function enterManualMode() {
  if (!videoUrl.value.trim()) {
    notify.warning(t("请先填写链接"));
    return;
  }

  clearAutoFetch();
  manualMode.value = true;

  // 如果已有抓取结果，用它预填
  if (fetched.value) {
    manual.value = {
      title: fetched.value.title,
      author: fetched.value.author ?? "",
      cover_url: fetched.value.cover_url ?? "",
      description: fetched.value.description ?? "",
    };
  }
}

// ============ 保存 ============

async function onSave() {
  if (!canSave.value) return;

  saving.value = true;
  const urlForSave = normalizeUrl(videoUrl.value);

  try {
    let data: VideoMeta;

    if (manualMode.value) {
      const platformId = guessedPlatform.value?.id ?? "manual";
      // 手动模式：platform_id 用 URL 里能提取的 id 或时间戳兜底
      const pid = extractPidFromUrl(urlForSave) ?? `manual_${Date.now()}`;

      data = {
        platform: platformId,
        platform_id: pid,
        title: manual.value.title.trim(),
        author: manual.value.author.trim() || null,
        cover_url: manual.value.cover_url.trim() || null,
        description: manual.value.description.trim() || null,
        duration: null,
        original_url: urlForSave,
        aid: null,
        cid: null,
      };
    } else if (fetched.value) {
      data = fetched.value;
    } else {
      saving.value = false;
      return;
    }

    await invoke("save_video", {
      data,
      categoryId: targetCategory.value,
      originalUrl: urlForSave,
      tags: tagInput.value,
      preserveTags: false,
    });

    notify.success(t("已添加：{title}", { title: data.title }));
    emit("saved");
    close();
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  } finally {
    saving.value = false;
  }
}

// ============ 工具函数 ============

function normalizeUrl(value: string): string {
  const url = value.trim();
  if (/^https?:\/\//i.test(url)) return url;
  if (/^(www\.)?github\.com\//i.test(url)) return `https://${url}`;
  return url;
}

function extractPidFromUrl(url: string): string | null {
  const patterns: RegExp[] = [
    /\/video\/([A-Za-z0-9]+)/, // 抖音
    /\/sph\/([A-Za-z0-9_-]+)/, // 视频号
    /[?&]v=([A-Za-z0-9_-]+)/, // YouTube
    /\/read\/cv(\d+)/, // B站专栏
    /\/(BV[A-Za-z0-9]+)/, // B站视频
  ];
  for (const p of patterns) {
    const m = url.match(p);
    if (m) return m[1];
  }
  return null;
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) {
    return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }
  return `${m}:${String(s).padStart(2, "0")}`;
}

function close() {
  visible.value = false;
}
</script>

<style scoped>
.min-w-0 {
  min-width: 0;
}

/* 抓取结果缩略图 */
.preview-thumb {
  width: 140px;
  height: 80px;
  flex-shrink: 0;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.06);
  display: flex;
  align-items: center;
  justify-content: center;
}
.preview-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.preview-thumb-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

/* 手动模式：封面 URL 输入框里的小预览 */
.inline-thumb {
  width: 28px;
  height: 20px;
  object-fit: cover;
  border-radius: 3px;
}
</style>
