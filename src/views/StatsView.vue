<template>
  <div style="max-width: 1100px">
    <div class="text-h5 font-weight-bold mb-4">{{ t("统计") }}</div>
    <v-progress-linear
      v-if="loading"
      indeterminate
      color="primary"
      rounded
      class="mb-4"
    />
    <v-alert
      v-if="loading"
      type="info"
      variant="tonal"
      density="compact"
      class="mb-4"
    >
      {{ t("正在加载统计数据…") }}
    </v-alert>

    <v-row dense>
      <v-col
        v-for="item in summary"
        :key="item.label"
        cols="6"
        sm="3"
      >
        <v-card variant="tonal" :color="item.color" class="stat-card">
          <v-card-text>
            <div class="d-flex align-center justify-space-between">
              <div class="stat-copy">
                <div class="text-h5 font-weight-bold stat-value">
                  {{ item.value }}
                </div>
                <div class="text-caption stat-label">{{ item.label }}</div>
              </div>
              <div class="stat-icon">
                <v-icon size="22" :color="item.color">{{ item.icon }}</v-icon>
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <v-row class="mt-2" align="start">
      <v-col cols="12" md="6">
        <v-card variant="outlined" class="stats-panel">
          <v-card-title class="text-subtitle-1">{{ t("平台分布") }}</v-card-title>
          <v-card-text>
            <template v-if="stats.platforms.length">
              <div v-for="item in stats.platforms" :key="item.name" class="mb-3">
                <div class="d-flex align-center mb-1">
                  <span class="text-body-2">{{ t(item.name) }}</span>
                  <v-spacer />
                  <span class="text-caption text-medium-emphasis">
                    {{ item.count }}
                  </span>
                </div>
                <v-progress-linear
                  :model-value="percent(item.count, maxPlatform)"
                  color="primary"
                  height="7"
                  rounded
                />
              </div>
            </template>
            <div v-else class="text-caption text-medium-emphasis">
              {{ t("暂无数据") }}
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="6">
        <v-card variant="outlined" class="stats-panel">
          <v-card-title class="text-subtitle-1">{{ t("分类分布") }}</v-card-title>
          <v-card-text>
            <template v-if="stats.categories.length">
              <div v-for="item in stats.categories" :key="item.name" class="mb-3">
                <div class="d-flex align-center mb-1">
                  <span class="text-body-2 stat-row-label">{{ item.name }}</span>
                  <v-spacer />
                  <span class="text-caption text-medium-emphasis">
                    {{ item.count }}
                  </span>
                </div>
                <v-progress-linear
                  :model-value="percent(item.count, maxCategory)"
                  color="secondary"
                  height="7"
                  rounded
                />
              </div>
            </template>
            <div v-else class="text-caption text-medium-emphasis">
              {{ t("暂无数据") }}
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <v-row class="mt-2" align="start">
      <v-col cols="12" md="6">
        <v-card variant="outlined" class="stats-panel">
          <v-card-title class="text-subtitle-1">{{ t("标签分布") }}</v-card-title>
          <v-card-text>
            <template v-if="stats.tags.length">
              <div v-for="item in stats.tags" :key="item.name" class="mb-3">
                <div class="d-flex align-center mb-1">
                  <span class="text-body-2 stat-row-label">{{ item.name }}</span>
                  <v-spacer />
                  <span class="text-caption text-medium-emphasis">
                    {{ item.count }}
                  </span>
                </div>
                <v-progress-linear
                  :model-value="percent(item.count, maxTag)"
                  color="success"
                  height="7"
                  rounded
                />
              </div>
            </template>
            <div v-else class="text-caption text-medium-emphasis">
              {{ t("暂无数据") }}
            </div>
          </v-card-text>
        </v-card>
      </v-col>

      <v-col cols="12" md="6">
        <v-card variant="outlined" class="stats-panel">
          <v-card-title class="text-subtitle-1">{{ t("每月新增") }}</v-card-title>
          <v-card-text>
            <div v-if="stats.monthly.length" class="monthly-chart">
              <div
                v-for="item in stats.monthly"
                :key="item.month"
                class="month-column"
              >
                <div class="month-count">{{ item.count }}</div>
                <div class="month-track">
                  <div
                    class="month-bar"
                    :style="{ height: barHeight(item.count) }"
                  />
                </div>
                <div class="month-label">{{ item.month.slice(5) }}</div>
              </div>
            </div>
            <div v-else class="text-caption text-medium-emphasis">
              {{ t("暂无数据") }}
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { DetailedStats } from "../types";
import { t } from "../i18n";

const loading = ref(true);
const stats = ref<DetailedStats>({
  videos_count: 0,
  categories_count: 0,
  tags_count: 0,
  topics_count: 0,
  ai_sorted_count: 0,
  local_video_count: 0,
  local_audio_count: 0,
  transcript_count: 0,
  platforms: [],
  categories: [],
  tags: [],
  monthly: [],
});

const summary = computed(() => [
  { label: t("视频"), value: stats.value.videos_count, icon: "mdi-video", color: "primary" },
  { label: t("分类"), value: stats.value.categories_count, icon: "mdi-folder", color: "secondary" },
  { label: t("标签"), value: stats.value.tags_count, icon: "mdi-tag", color: "success" },
  { label: t("主题"), value: stats.value.topics_count, icon: "mdi-bookmark", color: "info" },
  { label: t("已 AI 分类"), value: stats.value.ai_sorted_count, icon: "mdi-robot", color: "deep-purple" },
  { label: t("本地视频"), value: stats.value.local_video_count, icon: "mdi-harddisk", color: "teal" },
  { label: t("本地音频"), value: stats.value.local_audio_count, icon: "mdi-music", color: "orange" },
  { label: t("文字稿"), value: stats.value.transcript_count, icon: "mdi-text-box", color: "brown" },
]);

const maxPlatform = computed(() =>
  Math.max(1, ...stats.value.platforms.map((item) => item.count)),
);
const maxCategory = computed(() =>
  Math.max(1, ...stats.value.categories.map((item) => item.count)),
);
const maxTag = computed(() =>
  Math.max(1, ...stats.value.tags.map((item) => item.count)),
);
const maxMonthly = computed(() =>
  Math.max(1, ...stats.value.monthly.map((item) => item.count)),
);

function percent(value: number, max: number) {
  return Math.round((value / max) * 100);
}

function barHeight(value: number) {
  return `${Math.max(4, percent(value, maxMonthly.value))}%`;
}

onMounted(async () => {
  try {
    stats.value = await invoke<DetailedStats>("get_detailed_stats");
  } catch (e) {
    console.warn("加载统计数据失败", e);
  } finally {
    loading.value = false;
  }
});
</script>

<style scoped>
.stat-card {
  height: auto;
  min-height: 0;
  border: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  background:
    linear-gradient(
      135deg,
      rgba(var(--v-theme-surface), 0.72),
      rgba(var(--v-theme-surface), 0.18)
    ),
    rgba(var(--v-theme-surface), 0.1);
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.05);
}
.stat-card :deep(.v-card-text) {
  padding: 4px 10px !important;
}
.stat-value {
  font-size: 1.2rem !important;
  line-height: 1;
  letter-spacing: -0.02em;
}
.stat-label {
  margin-top: 0;
  font-size: 0.66rem;
  opacity: 0.68;
}
.stat-icon {
  width: 28px;
  height: 28px;
  border-radius: 9px;
  display: grid;
  place-items: center;
  background: rgba(var(--v-theme-surface), 0.55);
}
.stats-panel {
  height: auto;
  overflow: hidden;
}
.stats-panel :deep(.v-card-text) {
  padding-top: 6px !important;
  padding-bottom: 8px !important;
}
.stat-row-label {
  min-width: 0;
  max-width: calc(100% - 34px);
  white-space: normal;
  overflow-wrap: anywhere;
  line-height: 1.35;
}
.monthly-chart {
  display: flex;
  align-items: flex-end;
  gap: 6px;
  height: 180px;
  padding-top: 12px;
}
.month-column {
  flex: 1 1 0;
  min-width: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
}
.month-count {
  font-size: 10px;
  opacity: 0.65;
  margin-bottom: 3px;
}
.month-track {
  flex: 1 1 auto;
  width: 100%;
  display: flex;
  align-items: flex-end;
}
.month-bar {
  width: 100%;
  min-height: 3px;
  border-radius: 4px 4px 2px 2px;
  background: rgb(var(--v-theme-primary));
  transition: height 0.25s ease;
}
.month-label {
  font-size: 10px;
  opacity: 0.55;
  margin-top: 5px;
  white-space: nowrap;
}
</style>
