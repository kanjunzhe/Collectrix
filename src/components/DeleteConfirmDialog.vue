<template>
  <v-dialog v-model="visible" max-width="640" persistent>
    <v-card>
      <v-card-title>{{ title }}</v-card-title>
      <v-card-text>
        <v-alert type="warning" variant="tonal" density="compact" class="mb-3">
          {{ warning }}
        </v-alert>

        <div class="text-caption mb-2">
          {{ t("以下 {n} 个视频会受到影响：", { n: videos.length }) }}
        </div>
        <v-list density="compact" class="affected-videos">
          <v-list-item v-for="video in videos" :key="video.id">
            <v-list-item-title class="text-body-2">
              {{ video.title }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ video.author || t("未知作者") }}
            </v-list-item-subtitle>
          </v-list-item>
          <v-list-item v-if="videos.length === 0">
            <v-list-item-title class="text-caption text-medium-emphasis">
              {{ t("（没有视频）") }}
            </v-list-item-title>
          </v-list-item>
        </v-list>

        <v-text-field
          v-model="confirmation"
          :label="t('请输入「{name}」以确认：', { name: entityName })"
          density="compact"
          variant="outlined"
          hide-details
          class="mt-3"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          :color="confirmColor"
          :disabled="confirmation !== entityName"
          variant="flat"
          @click="confirm"
        >
          {{ confirmText }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { VideoRow } from "../types";
import { t } from "../i18n";

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title: string;
    entityName: string;
    videos: VideoRow[];
    warning: string;
    confirmText: string;
    confirmColor?: string;
  }>(),
  { confirmColor: "error" },
);

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const confirmation = ref("");

watch(
  () => props.modelValue,
  (open) => {
    if (open) confirmation.value = "";
  },
);

function close() {
  visible.value = false;
}

function confirm() {
  if (confirmation.value !== props.entityName) return;
  emit("confirm");
}
</script>

<style scoped>
.affected-videos {
  max-height: 260px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 8px;
}
</style>
