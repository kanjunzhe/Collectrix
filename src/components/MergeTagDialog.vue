<template>
  <v-dialog v-model="visible" max-width="640" persistent>
    <v-card>
      <v-card-title>{{ t("合并标签") }}</v-card-title>
      <v-card-text v-if="sourceTag">
        <div class="d-flex align-center flex-wrap mb-3" style="gap: 8px">
          <span class="text-body-2">{{ t("将标签「{name}」合并到：", { name: sourceTag.name }) }}</span>
          <v-chip :color="sourceTag.color" variant="flat" size="small">
            {{ sourceTag.name }}
          </v-chip>
        </div>

        <v-alert
          v-if="availableTags.length === 0"
          type="warning"
          variant="tonal"
          density="compact"
        >
          {{ t("没有可合并的其他标签") }}
        </v-alert>

        <v-select
          v-model="targetId"
          :items="availableTags"
          item-title="name"
          item-value="id"
          :label="t('目标标签')"
          density="compact"
          variant="outlined"
          :disabled="availableTags.length === 0"
        />

        <v-text-field
          v-model="mergedName"
          :label="t('合并后的标签名')"
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />

        <v-alert type="info" variant="tonal" density="compact" class="mb-3">
          {{ t("合并后会保留两个标签下的所有视频关联。") }}
        </v-alert>

        <div class="d-flex align-center mb-2">
          <span class="text-caption">
            {{ t("合并预览：{n} 个视频", { n: videos.length }) }}
          </span>
          <v-progress-circular
            v-if="loadingVideos"
            indeterminate
            size="14"
            width="2"
            class="ml-2"
          />
        </div>
        <v-list density="compact" class="merge-videos">
          <v-list-item v-for="video in videos" :key="video.id">
            <v-list-item-title class="text-body-2">
              {{ video.title }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ video.author || t("未知作者") }}
            </v-list-item-subtitle>
          </v-list-item>
          <v-list-item v-if="!loadingVideos && videos.length === 0">
            <v-list-item-title class="text-caption text-medium-emphasis">
              {{ t("（没有视频）") }}
            </v-list-item-title>
          </v-list-item>
        </v-list>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          :disabled="!canMerge || merging"
          :loading="merging"
          variant="flat"
          @click="onConfirm"
        >
          {{ t("合并") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { TagInfo, VideoRow } from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{
  modelValue: boolean;
  sourceTag: TagInfo | null;
  tags: TagInfo[];
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  merged: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const targetId = ref<number | null>(null);
const mergedName = ref("");
const videos = ref<VideoRow[]>([]);
const loadingVideos = ref(false);
const merging = ref(false);

const availableTags = computed(() =>
  props.tags.filter((tag) => tag.id !== props.sourceTag?.id),
);

const canMerge = computed(
  () =>
    !!props.sourceTag &&
    targetId.value !== null &&
    mergedName.value.trim().length > 0 &&
    targetId.value !== props.sourceTag.id,
);

async function loadVideos() {
  if (!props.sourceTag || targetId.value === null) {
    videos.value = [];
    return;
  }
  loadingVideos.value = true;
  try {
    videos.value = await invoke<VideoRow[]>("list_videos_by_tags", {
      tagIds: [props.sourceTag.id, targetId.value],
    });
  } catch (e) {
    videos.value = [];
    notify.error(t("加载视频失败：{e}", { e: String(e) }));
  } finally {
    loadingVideos.value = false;
  }
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    targetId.value = availableTags.value[0]?.id ?? null;
    mergedName.value = props.sourceTag?.name ?? "";
    loadVideos();
  },
);

watch(targetId, () => {
  if (props.modelValue) loadVideos();
});

function close() {
  visible.value = false;
}

async function onConfirm() {
  if (!canMerge.value || !props.sourceTag || targetId.value === null) return;
  merging.value = true;
  try {
    await invoke("merge_tags", {
      sourceId: props.sourceTag.id,
      targetId: targetId.value,
      mergedName: mergedName.value.trim(),
    });
    notify.success(t("标签已合并"));
    emit("merged");
    close();
  } catch (e) {
    notify.error(t("合并失败：{e}", { e: String(e) }));
  } finally {
    merging.value = false;
  }
}
</script>

<style scoped>
.merge-videos {
  max-height: 240px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 8px;
}
</style>
