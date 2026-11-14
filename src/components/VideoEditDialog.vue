<template>
  <v-dialog v-model="visible" max-width="580">
    <v-card>
      <v-card-title class="d-flex align-center">
        <v-icon class="mr-2">mdi-pencil</v-icon>
        {{ t("编辑视频信息") }}
      </v-card-title>
      <v-card-text v-if="video">
        <!-- 标题（只读） -->
        <v-text-field
          v-model="title"
          :label="t('标题')"
          density="compact"
          variant="outlined"
        />
        <v-text-field
          v-model="author"
          :label="t('作者')"
          density="compact"
          variant="outlined"
        />

        <!-- 平台（只读，仅展示） -->
        <div class="d-flex align-center mb-3" style="gap: 8px">
          <span class="text-caption text-medium-emphasis">{{ t("平台：") }}</span>
          <v-chip size="small" :color="platform.color" variant="flat">
            <v-icon start size="12">{{ platform.icon }}</v-icon>
            {{ t(platform.name) }}
          </v-chip>
          <v-chip size="small" variant="text">{{ video.platform_id }}</v-chip>
        </div>

        <!-- 分类 -->
        <v-select
          v-model="categoryId"
          :items="categories"
          item-title="label"
          item-value="id"
          :label="t('所属分类')"
          density="compact"
          variant="outlined"
          clearable
        />

        <!-- 标签 -->
        <v-combobox
          v-model="tags"
          :label="t('标签（回车添加）')"
          multiple
          chips
          closable-chips
          density="compact"
          variant="outlined"
        />

        <!-- 主题 -->
        <v-select
          v-model="topicIds"
          :items="allTopics"
          item-title="name"
          item-value="id"
          :label="t('所属主题（可多选）')"
          multiple
          chips
          closable-chips
          density="compact"
          variant="outlined"
        />

        <v-textarea
          v-model="description"
          :label="t('简介')"
          rows="3"
          auto-grow
          density="compact"
          variant="outlined"
        />

        <v-progress-linear
          v-if="loadingTopics"
          indeterminate
          color="primary"
          class="mt-2"
        />

        <!-- ★★★ 补全信息门控：仅 B站视频且缺 aid/cid 时显示 ★★★ -->
        <div
          v-if="needsRefresh"
          class="mt-3 pa-3 d-flex align-center"
          style="
            gap: 8px;
            background: rgba(var(--v-theme-warning), 0.08);
            border-radius: 6px;
          "
        >
          <v-icon color="warning" size="20">mdi-alert-circle-outline</v-icon>
          <div class="flex-grow-1 text-caption">
            {{
              t("该视频缺少 {fields} ，评论区和下载功能无法使用。", {
                fields: missingFields,
              })
            }}
          </div>
          <v-btn
            size="small"
            color="warning"
            variant="tonal"
            prepend-icon="mdi-refresh"
            :loading="refreshing"
            @click="onRefreshInfo"
          >
            {{ t("补全信息") }}
          </v-btn>
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn :disabled="saving" @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          :disabled="saving"
          :loading="saving"
          @click="onSave"
        >
          {{ t("保存") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getPlatform } from "../platforms";
import { notify } from "../composables/useUI";
import { t } from "../i18n";
import type { TopicInfo, VideoRow } from "../types";

const props = defineProps<{
  modelValue: boolean;
  video: VideoRow | null;
  categories: { id: number; label: string }[];
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

const categoryId = ref<number | null>(null);
const title = ref("");
const author = ref("");
const description = ref("");
const tags = ref<string[]>([]);
const topicIds = ref<number[]>([]);
const allTopics = ref<TopicInfo[]>([]);
const loadingTopics = ref(false);

const refreshing = ref(false);
const saving = ref(false);

// ============ 派生 ============

const platform = computed(() =>
  getPlatform(props.video?.platform ?? "bilibili"),
);

/**
 * ★★★ 门控条件 ★★★
 * - platform === "bilibili"：只有 B站视频有 aid/cid 概念
 * - 缺 aid 或缺 cid：两者任一为空都需要补全
 * YouTube、抖音、B站专栏等平台的 aid/cid 永远是 null，
 * 但 platform 不等于 "bilibili"，所以不会显示这个按钮。
 */
const needsRefresh = computed(() => {
  const v = props.video;
  if (!v) return false;
  return v.platform === "bilibili" && (!v.aid || !v.cid);
});

const missingFields = computed(() => {
  const v = props.video;
  if (!v) return "";
  const missing: string[] = [];
  if (!v.aid) missing.push("aid");
  if (!v.cid) missing.push("cid");
  return missing.join("、");
});

// ============ 打开时加载 ============

watch(
  () => props.video,
  async (v) => {
    if (!v) return;

    categoryId.value = v.category_id;
    title.value = v.title;
    author.value = v.author ?? "";
    description.value = v.description ?? "";
    tags.value = v.tags.map((t) => t.name);
    topicIds.value = [];

    loadingTopics.value = true;
    try {
      const [topics, current] = await Promise.all([
        invoke<TopicInfo[]>("list_topics"),
        invoke<TopicInfo[]>("list_topics_for_video", { videoId: v.id }),
      ]);
      allTopics.value = topics;
      topicIds.value = current.map((t) => t.id);
    } catch (e) {
      console.error("加载主题失败", e);
    } finally {
      loadingTopics.value = false;
    }
  },
  { immediate: true },
);

// ============ 补全信息 ============

async function onRefreshInfo() {
  if (!props.video) return;
  refreshing.value = true;
  try {
    await invoke("refresh_video_info", { videoId: props.video.id });
    // 拉回最新数据覆盖当前 props.video
    const updated = await invoke<VideoRow>("get_video", {
      id: props.video.id,
    });
    Object.assign(props.video, updated);
    notify.success(t("视频信息已补全"));
  } catch (e) {
    notify.error(t("补全失败：{e}", { e: String(e) }));
  } finally {
    refreshing.value = false;
  }
}

// ============ 保存 ============

async function onSave() {
  if (!props.video) return;
  saving.value = true;

  try {
    const videoId = props.video.id;

    await invoke("update_video", {
      id: videoId,
      categoryId: categoryId.value,
      tags: tags.value,
      title: title.value.trim(),
      author: author.value.trim(),
      description: description.value.trim(),
    });

    await invoke("set_video_topics", {
      videoId,
      topicIds: topicIds.value,
    });

    notify.success(t("已保存"));
    emit("saved");
    close();
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  } finally {
    saving.value = false;
  }
}

function close() {
  visible.value = false;
}
</script>
