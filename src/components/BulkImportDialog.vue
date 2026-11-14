<template>
  <v-dialog v-model="visible" max-width="680" persistent scrollable>
    <v-card>
      <v-card-title>
        {{ mode === "collection" ? t("导入合集") : t("导入稍后再看") }}
      </v-card-title>
      <v-card-text>
        <template v-if="mode === 'collection'">
          <div class="d-flex align-center" style="gap: 8px">
            <v-text-field
              v-model="url"
              :label="t('合集内任意视频链接')"
              :placeholder="t('粘贴合集里任意一集的 B 站链接')"
              density="compact"
              variant="outlined"
              hide-details
              @keydown.enter.prevent="loadCollection"
            />
            <v-btn
              color="primary"
              variant="tonal"
              :loading="loading"
              @click="loadCollection"
            >
              {{ t("获取合集") }}
            </v-btn>
          </div>
        </template>
        <template v-else>
          <div class="d-flex align-center mb-2" style="gap: 8px">
            <span class="text-caption text-medium-emphasis">
              {{ t("从 B 站「稍后再看」读取视频") }}
            </span>
            <v-spacer />
            <v-btn
              size="small"
              variant="tonal"
              prepend-icon="mdi-refresh"
              :loading="loading"
              @click="loadWatchLater"
            >
              {{ t("刷新") }}
            </v-btn>
          </div>
        </template>

        <div class="d-flex align-center mt-3 mb-2">
          <span class="text-caption text-medium-emphasis">
            {{ t("已选择 {selected}/{total} 个视频", {
              selected: selectedIds.length,
              total: videos.length,
            }) }}
          </span>
          <v-spacer />
          <v-btn size="small" variant="text" @click="selectAll">
            {{ t("全选") }}
          </v-btn>
          <v-btn size="small" variant="text" @click="selectedIds = []">
            {{ t("取消全选") }}
          </v-btn>
        </div>

        <v-progress-linear v-if="loading" indeterminate color="primary" class="mb-2" />

        <v-list v-if="videos.length" density="compact" class="bulk-list">
          <v-list-item
            v-for="video in videos"
            :key="video.platform_id"
            @click="toggle(video.platform_id)"
          >
            <template #prepend>
              <v-checkbox-btn :model-value="selectedIds.includes(video.platform_id)" />
            </template>
            <v-list-item-title class="text-body-2">
              {{ video.title }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ video.author || t("未知作者") }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>
        <v-alert v-else-if="!loading" type="info" variant="tonal" density="compact">
          {{ t("没有可导入的视频") }}
        </v-alert>

        <v-select
          v-model="targetCategory"
          :items="categories"
          item-title="label"
          item-value="id"
          :label="t('导入到分类（可选）')"
          density="compact"
          variant="outlined"
          clearable
          class="mt-3"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn :disabled="importing" @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          :loading="importing"
          :disabled="selectedIds.length === 0"
          variant="flat"
          @click="onImport"
        >
          {{ t("导入 {n} 个视频", { n: selectedIds.length }) }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { VideoMeta } from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{
  modelValue: boolean;
  mode: "collection" | "watchLater";
  categories: { id: number; label: string }[];
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  imported: [count: number];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const url = ref("");
const videos = ref<VideoMeta[]>([]);
const selectedIds = ref<string[]>([]);
const targetCategory = ref<number | null>(null);
const loading = ref(false);
const importing = ref(false);

async function loadCollection() {
  const link = url.value.trim();
  if (!link) {
    notify.warning(t("请先填写链接"));
    return;
  }
  loading.value = true;
  try {
    videos.value = await invoke<VideoMeta[]>("fetch_bili_collection", {
      url: link,
    });
    selectAll();
    if (videos.value.length === 0) {
      notify.warning(t("没有可导入的视频"));
    }
  } catch (e) {
    videos.value = [];
    notify.error(t("获取失败：{e}", { e: String(e) }));
  } finally {
    loading.value = false;
  }
}

async function loadWatchLater() {
  loading.value = true;
  try {
    videos.value = await invoke<VideoMeta[]>("list_bili_watch_later");
    selectAll();
  } catch (e) {
    videos.value = [];
    notify.error(t("获取失败：{e}", { e: String(e) }));
  } finally {
    loading.value = false;
  }
}

function selectAll() {
  selectedIds.value = videos.value.map((video) => video.platform_id);
}

function toggle(id: string) {
  const index = selectedIds.value.indexOf(id);
  if (index >= 0) selectedIds.value.splice(index, 1);
  else selectedIds.value.push(id);
}

async function onImport() {
  const selected = videos.value.filter((video) =>
    selectedIds.value.includes(video.platform_id),
  );
  if (selected.length === 0) return;
  importing.value = true;
  let success = 0;
  try {
    for (const video of selected) {
      await invoke("save_video", {
        data: video,
        categoryId: targetCategory.value,
        originalUrl: video.original_url,
        tags: [],
        preserveTags: true,
      });
      success++;
    }
    notify.success(t("导入完成：{n} 个视频", { n: success }));
    emit("imported", success);
    close();
  } catch (e) {
    notify.error(t("导入失败：{e}", { e: String(e) }));
  } finally {
    importing.value = false;
  }
}

function close() {
  visible.value = false;
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    videos.value = [];
    selectedIds.value = [];
    targetCategory.value = null;
    url.value = "";
    if (props.mode === "watchLater") {
      loadWatchLater();
    }
  },
);
</script>

<style scoped>
.bulk-list {
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 8px;
}
</style>
