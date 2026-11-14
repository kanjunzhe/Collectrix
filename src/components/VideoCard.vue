<template>
  <v-card
    class="video-card"
    hover
    draggable="true"
    @dragstart="onDragStart"
    @contextmenu.prevent="$emit('contextmenu', $event, video)"
  >
    <!-- 封面区：固定高度 152px，宽度随网格自适应 -->
    <div class="cover-wrapper" @click="$emit('play', video)">
      <img
        v-if="videoCoverSrc(video)"
        :src="videoCoverSrc(video)"
        referrerpolicy="no-referrer"
        class="cover-img"
        alt="cover"
        loading="lazy"
        decoding="async"
      />
      <div v-else class="cover-placeholder">
        <v-icon size="40" opacity="0.4">mdi-video-off-outline</v-icon>
      </div>

      <!-- 左上角徽标：新 / AI -->
      <div class="badge-overlay">
        <v-chip
          size="x-small"
          :color="platform.color"
          variant="flat"
          class="badge"
        >
          <v-icon start size="10">{{ platform.icon }}</v-icon>
          {{ t(platform.name) }}
        </v-chip>
        <v-chip
          v-if="video.is_recent"
          size="x-small"
          color="info"
          variant="flat"
          class="badge"
        >
          <v-icon start size="10">mdi-clock-fast</v-icon>
          {{ t("新") }}
        </v-chip>
        <v-chip
          v-if="video.is_ai_sorted"
          size="x-small"
          color="deep-purple"
          variant="flat"
          class="badge"
        >
          <v-icon start size="10">mdi-robot</v-icon>
          AI
        </v-chip>
      </div>

      <!-- hover 播放图标 -->
      <div class="cover-overlay">
        <v-icon color="white" size="48">mdi-play-circle</v-icon>
      </div>
    </div>

    <!-- 内容区 -->
    <div class="card-body">
      <div class="video-title" :title="video.title">{{ video.title }}</div>
      <div class="video-author text-truncate" :title="video.author || ''">
        {{ video.author || t("未知作者") }}
      </div>
      <div class="video-desc" :title="video.description || ''">
        {{ video.description || t("（无描述）") }}
      </div>

      <div class="tags-row">
        <v-chip
          v-for="tag in visibleTags"
          :key="tag.id"
          size="x-small"
          :color="tag.color"
          variant="flat"
        >
          {{ tag.name }}
        </v-chip>
        <v-chip
          v-if="hiddenTagCount > 0"
          size="x-small"
          variant="outlined"
          class="hidden-tags"
        >
          +{{ hiddenTagCount }}
        </v-chip>
      </div>

      <div class="card-actions">
        <v-btn
          size="small"
          prepend-icon="mdi-play"
          @click="$emit('play', video)"
        >
          {{ t("播放") }}
        </v-btn>
        <v-spacer />
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
  </v-card>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { VideoRow } from "../types";
import { getPlatform } from "../platforms";
import { videoCoverSrc } from "../utils/media";
import { t } from "../i18n";

const platform = computed(() => getPlatform(props.video.platform));

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

const MAX_VISIBLE_TAGS = 3;
const visibleTags = computed(() => props.video.tags.slice(0, MAX_VISIBLE_TAGS));
const hiddenTagCount = computed(() =>
  Math.max(0, props.video.tags.length - MAX_VISIBLE_TAGS),
);
</script>

<style scoped>
/* 整卡：flex 纵向布局，让 card-actions 始终贴底 */
.video-card {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 240px;
  min-height: 320px; /* 关键：不同内容量卡片高度对齐 */
  overflow: hidden;
}

/* 封面：固定高度 152px，不再用 aspect-ratio */
.cover-wrapper {
  position: relative;
  width: 100%;
  height: 152px;
  flex-shrink: 0; /* 关键：不被 flex 压缩 */
  overflow: hidden;
  background: rgba(0, 0, 0, 0.06);
  cursor: pointer;
}

.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover; /* 关键：任何宽高比都不变形 */
  display: block;
}

.cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.cover-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  transition: background 0.2s;
  pointer-events: none; /* 关键：不拦截封面点击 */
}
.cover-wrapper:hover .cover-overlay {
  background: rgba(0, 0, 0, 0.35);
}

.badge-overlay {
  position: absolute;
  top: 6px;
  left: 6px;
  display: flex;
  gap: 4px;
  z-index: 2;
  pointer-events: none;
}
.badge {
  opacity: 0.92;
}

/* 内容区：flex 撑满剩余空间 */
.card-body {
  display: flex;
  flex-direction: column;
  flex: 1 1 auto;
  min-height: 0;
  padding: 8px 12px 6px;
  gap: 4px;
}

/* 标题：最多 2 行，固定高度保证对齐 */
.video-title {
  font-size: 16px;
  font-weight: 500;
  line-height: 1.35;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
  min-height: calc(1.35em * 2);
  max-height: calc(1.35em * 2);
}

.video-author {
  font-size: 12px;
  opacity: 0.65;
  min-height: 1.4em;
}

/* 描述：最多 2 行 */
.video-desc {
  font-size: 12px;
  line-height: 1.4;
  opacity: 0.55;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
  min-height: calc(1.4em * 2);
  max-height: calc(1.4em * 2);
}

/* 标签：固定高度范围，超出隐藏 */
.tags-row {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  align-content: flex-start;
  overflow: hidden;
  min-height: 20px;
  max-height: 44px;
}

/* 操作栏：贴底 */
.card-actions {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  margin-top: auto;
  border-top: 1px solid rgba(128, 128, 128, 0.15);
  padding-top: 4px;
}

@media (max-width: 960px) {
  .cover-wrapper {
    height: 180px;
  }
}
@media (max-width: 600px) {
  .cover-wrapper {
    height: 200px;
  }
}
</style>
