<template>
  <div
    class="list-item"
    draggable="true"
    @click="$emit('play', video)"
    @dragstart="onDragStart"
    @contextmenu.prevent="$emit('contextmenu', $event, video)"
  >
    <div class="cover">
      <img
        v-if="videoCoverSrc(video)"
        :src="videoCoverSrc(video)"
        referrerpolicy="no-referrer"
        loading="lazy"
        decoding="async"
      />
      <v-icon v-else size="28">mdi-video-off-outline</v-icon>
      <div class="badge-overlay">
        <v-chip
          v-if="video.is_recent"
          size="x-small"
          color="info"
          variant="flat"
        >
          <v-icon start size="10">mdi-clock-fast</v-icon>
          {{ t("新") }}
        </v-chip>
        <v-chip
          v-if="video.is_ai_sorted"
          size="x-small"
          color="deep-purple"
          variant="flat"
        >
          <v-icon start size="10">mdi-robot</v-icon>
          AI
        </v-chip>
      </div>
    </div>

    <div class="content">
      <div class="title">{{ video.title }}</div>
      <div class="author">{{ video.author || t("未知作者") }}</div>
      <div class="desc">{{ video.description || t("（无描述）") }}</div>
      <div class="tags">
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
    </div>

    <div class="actions" @click.stop>
      <v-btn
        icon
        size="small"
        variant="text"
        :title="t('播放')"
        @click="$emit('play', video)"
      >
        <v-icon>mdi-play</v-icon>
      </v-btn>
      <v-btn
        icon
        size="small"
        variant="text"
        :title="t('编辑')"
        @click="$emit('edit', video)"
      >
        <v-icon>mdi-pencil</v-icon>
      </v-btn>
      <v-btn
        icon
        size="small"
        variant="text"
        color="error"
        :title="t('删除')"
        @click="$emit('delete', video.id)"
      >
        <v-icon>mdi-delete</v-icon>
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { VideoRow } from "../types";
import { videoCoverSrc } from "../utils/media";
import { t } from "../i18n";

const props = defineProps<{ video: VideoRow }>();
defineEmits<{
  play: [video: VideoRow];
  edit: [video: VideoRow];
  delete: [id: number];
  contextmenu: [event: MouseEvent, video: VideoRow];
}>();

function onDragStart(event: DragEvent) {
  if (!event.dataTransfer) return;
  event.dataTransfer.effectAllowed = "move";
  const id = String(props.video.id);
  event.dataTransfer.setData("application/x-collectrix-video-id", id);
  event.dataTransfer.setData("text/plain", id);
  event.dataTransfer.setData("text", id);
}
</script>

<style scoped>
.list-item {
  display: flex;
  align-items: stretch;
  gap: 12px;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.12);
  cursor: pointer;
  transition: background 0.15s;
}
.list-item:hover {
  background: rgba(var(--v-theme-primary), 0.05);
}

.cover {
  width: 140px;
  height: 80px;
  flex-shrink: 0;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.06);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}
.cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.badge-overlay {
  position: absolute;
  top: 4px;
  left: 4px;
  display: flex;
  gap: 4px;
}

.content {
  flex: 1 1 auto;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.title {
  font-size: 14px;
  font-weight: 500;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.author {
  font-size: 12px;
  opacity: 0.65;
}
.desc {
  font-size: 12px;
  opacity: 0.55;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-top: 2px;
}

.actions {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 2px;
}
</style>
