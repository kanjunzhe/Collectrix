<template>
  <div class="mind-page p-5">
    <div class="d-flex align-center flex-wrap mb-4" style="gap: 8px">
      <div>
        <div class="text-h5 font-weight-bold">{{ t("思维导图") }}</div>
        <div class="text-caption text-medium-emphasis">
          {{ t("按分类展开视频集合，点击视频可在新窗口播放") }}
        </div>
      </div>
      <v-spacer />
      <v-chip variant="tonal">
        {{ t("{n} 个视频", { n: totalVideos }) }}
      </v-chip>
      <v-chip v-if="totalVideos > 500" variant="text" size="small">
        {{ t("视图最多展示 500 个视频") }}
      </v-chip>
      <v-btn
        variant="tonal"
        prepend-icon="mdi-refresh"
        :loading="loading"
        @click="load"
      >
        {{ t("刷新") }}
      </v-btn>
    </div>

    <div v-if="loading" class="d-flex justify-center pa-10">
      <v-progress-circular indeterminate color="primary" />
    </div>
    <v-sheet
      v-else-if="tree"
      class="mind-canvas rounded-xl overflow-auto"
      border
    >
      <MindMapNode :node="tree" @play="$emit('play', $event)" />
    </v-sheet>

    <v-alert v-else type="info" variant="tonal">
      {{ t("暂无数据") }}
    </v-alert>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import MindMapNode from "../components/MindMapNode.vue";
import type {
  CategoryNode,
  MindMapNodeData,
  Stats,
  VideoRow,
} from "../types";
import { t } from "../i18n";

defineEmits<{ play: [video: VideoRow] }>();

const loading = ref(false);
const tree = ref<MindMapNodeData | null>(null);
const totalVideos = ref(0);

const VIDEOS_PER_CATEGORY = 6;

function buildTree(
  categories: CategoryNode[],
  videosByCategory: Map<number, VideoRow[]>,
): MindMapNodeData[] {
  return categories.map((category) => {
    const children = buildTree(category.children, videosByCategory);
    const directVideos = videosByCategory.get(category.id) ?? [];
    const videoNodes: MindMapNodeData[] = directVideos
      .slice(0, VIDEOS_PER_CATEGORY)
      .map((video) => ({
        id: `video-${video.id}`,
        label: video.title,
        kind: "video",
        video,
      }));
    if (directVideos.length > VIDEOS_PER_CATEGORY) {
      videoNodes.push({
        id: `more-${category.id}`,
        label: t("还有 {n} 个视频", {
          n: directVideos.length - VIDEOS_PER_CATEGORY,
        }),
        kind: "more",
      });
    }
    return {
      id: `category-${category.id}`,
      label: category.name,
      kind: "category",
      count: directVideos.length,
      children: [...children, ...videoNodes],
    };
  });
}

async function load() {
  loading.value = true;
  try {
    const [categories, videos, stats] = await Promise.all([
      invoke<CategoryNode[]>("get_category_tree"),
      invoke<VideoRow[]>("list_videos", {
        categoryId: null,
        limit: 500,
        offset: 0,
      }),
      invoke<Stats>("get_stats"),
    ]);
    totalVideos.value = stats.videos_count;
    const byCategory = new Map<number, VideoRow[]>();
    const uncategorized: VideoRow[] = [];
    for (const video of videos) {
      if (video.category_id === null) {
        uncategorized.push(video);
        continue;
      }
      const list = byCategory.get(video.category_id) ?? [];
      list.push(video);
      byCategory.set(video.category_id, list);
    }
    const children = buildTree(categories, byCategory);
    if (uncategorized.length > 0) {
      children.push({
        id: "uncategorized",
        label: t("未分类"),
        kind: "category",
        count: uncategorized.length,
        children: [
          ...uncategorized.slice(0, VIDEOS_PER_CATEGORY).map((video) => ({
            id: `video-${video.id}`,
            label: video.title,
            kind: "video" as const,
            video,
          })),
          ...(uncategorized.length > VIDEOS_PER_CATEGORY
            ? [
                {
                  id: "more-uncategorized",
                  label: t("还有 {n} 个视频", {
                    n: uncategorized.length - VIDEOS_PER_CATEGORY,
                  }),
                  kind: "more" as const,
                },
              ]
            : []),
        ],
      });
    }
    tree.value = {
      id: "root",
      label: "Collectrix",
      kind: "root",
      count: totalVideos.value,
      children,
    };
  } catch (e) {
    console.error("加载思维导图失败", e);
  } finally {
    loading.value = false;
  }
}

onMounted(load);
</script>

<style scoped>
.mind-page {
  min-height: calc(100vh - 120px);
}
.mind-canvas {
  min-height: calc(100vh - 210px);
  max-height: calc(100vh - 150px);
  padding: 24px;
  background:
    radial-gradient(circle at 1px 1px, rgba(128, 128, 128, 0.13) 1px, transparent 0);
  background-size: 22px 22px;
}
</style>
