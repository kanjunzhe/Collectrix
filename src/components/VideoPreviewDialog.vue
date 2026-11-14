<template>
  <v-dialog v-model="visible" max-width="960" scrollable>
    <v-card v-if="video">
      <v-card-title class="d-flex align-center">
        <span class="text-truncate">{{ video.title }}</span>
        <v-spacer />
        <v-btn icon variant="text" @click="close">
          <v-icon>mdi-close</v-icon>
        </v-btn>
      </v-card-title>
      <v-divider />
      <v-card-text>
        <video
          v-if="localSrc"
          :src="localSrc"
          controls
          class="preview-video"
        />
        <iframe
          v-else-if="embedUrl"
          :src="embedUrl"
          class="preview-iframe"
          allowfullscreen
          frameborder="0"
        />
        <v-sheet
          v-else
          class="d-flex flex-column align-center justify-center"
          height="300"
        >
          <v-icon size="56" opacity="0.3" :color="platform.color">
            {{ platform.icon }}
          </v-icon>
          <div class="text-h6 mt-3">
            {{
              t("{platform} 暂不支持内嵌播放", {
                platform: t(platform.name),
              })
            }}
          </div>
          <v-btn
            v-if="video.original_url"
            class="mt-3"
            variant="tonal"
            :href="video.original_url"
            target="_blank"
          >
            {{ t("在浏览器打开") }}
          </v-btn>
        </v-sheet>

        <div class="d-flex flex-wrap align-center mt-3" style="gap: 8px">
          <span class="text-caption text-medium-emphasis">
            {{ video.author || t("未知作者") }}
          </span>
          <v-chip
            v-for="tag in video.tags"
            :key="tag.id"
            size="x-small"
            :color="tag.color"
            variant="flat"
          >
            {{ tag.name }}
          </v-chip>
        </div>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { VideoRow } from "../types";
import { getPlatform } from "../platforms";
import { t } from "../i18n";

const props = defineProps<{
  modelValue: boolean;
  video: VideoRow | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const platform = computed(() =>
  getPlatform(props.video?.platform ?? "bilibili"),
);

const localSrc = computed(() => {
  if (!props.video?.local_video_path) return "";
  return convertFileSrc(props.video.local_video_path);
});

const embedUrl = computed(() => {
  if (
    !props.video ||
    !platform.value.canEmbed ||
    !platform.value.embedUrl ||
    !props.video.platform_id
  ) {
    return "";
  }
  return platform.value.embedUrl(props.video.platform_id);
});

function close() {
  visible.value = false;
}
</script>

<style scoped>
.preview-video,
.preview-iframe {
  width: 100%;
  aspect-ratio: 16 / 9;
  display: block;
  border: none;
  background: #000;
}
</style>
