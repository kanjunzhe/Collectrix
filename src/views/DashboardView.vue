<template>
  <div>
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
      {{ t("正在加载仪表盘…") }}
    </v-alert>

    <div
      class="dashboard-hero d-flex align-center flex-wrap rounded-3xl p-6 mb-5"
      style="gap: 8px"
    >
      <img
        src="/collectrix-logo-flat.svg"
        alt=""
        class="dashboard-hero-mark"
      />
      <div>
        <div class="text-h5 font-weight-bold">{{ t("欢迎回来 👋") }}</div>
        <div class="text-caption dashboard-hero-sub">
          {{ t("管理你的本地视频收藏与知识库") }}
        </div>
      </div>
      <v-spacer />
      <v-btn
        variant="text"
        color="white"
        prepend-icon="mdi-swap-horizontal"
        @click="showTransfer = true"
      >
        {{ t("导入 / 导出收藏") }}
      </v-btn>
    </div>

    <v-row>
      <v-col cols="12" sm="4">
        <v-card color="primary" variant="tonal" class="pa-2">
          <v-card-text>
            <div class="d-flex align-center justify-space-between">
              <div>
                <div class="text-h3 font-weight-bold">
                  {{ stats.videos_count }}
                </div>
                <div class="text-subtitle-2">{{ t("视频总数") }}</div>
              </div>
              <v-icon size="56" color="primary">mdi-video</v-icon>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" sm="4">
        <v-card color="secondary" variant="tonal" class="pa-2">
          <v-card-text>
            <div class="d-flex align-center justify-space-between">
              <div>
                <div class="text-h3 font-weight-bold">
                  {{ stats.categories_count }}
                </div>
                <div class="text-subtitle-2">{{ t("分类总数") }}</div>
              </div>
              <v-icon size="56" color="secondary">mdi-folder-multiple</v-icon>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
      <v-col cols="12" sm="4">
        <v-card color="success" variant="tonal" class="pa-2">
          <v-card-text>
            <div class="d-flex align-center justify-space-between">
              <div>
                <div class="text-h3 font-weight-bold">
                  {{ stats.tags_count }}
                </div>
                <div class="text-subtitle-2">{{ t("标签总数") }}</div>
              </div>
              <v-icon size="56" color="success">mdi-tag-multiple</v-icon>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>

    <div class="d-flex align-center mt-8 mb-3">
      <div class="text-h6">{{ t("标签") }}</div>
      <v-spacer />
      <v-btn
        variant="text"
        append-icon="mdi-arrow-right"
        @click="$emit('navigate', 'tags')"
      >
        {{ t("管理标签") }}
      </v-btn>
    </div>
    <div v-if="visibleTags.length > 0" class="d-flex flex-wrap" style="gap: 8px">
      <v-chip
        v-for="tag in visibleTags"
        :key="tag.id"
        :color="tag.color"
        variant="flat"
        @click="$emit('tag', tag.id)"
      >
        {{ tag.name }}
      </v-chip>
      <v-chip v-if="tags.length > visibleTags.length" variant="text">
        +{{ tags.length - visibleTags.length }}
      </v-chip>
    </div>
    <v-alert v-else type="info" variant="tonal" density="compact">
      {{ t("还没有标签") }}
    </v-alert>

    <div class="d-flex align-center mt-8 mb-3">
      <div class="text-h6">{{ t("最近添加") }}</div>
      <v-spacer />
      <v-btn
        variant="text"
        append-icon="mdi-arrow-right"
        @click="$emit('navigate', 'library')"
      >
        {{ t("进入视频库") }}
      </v-btn>
    </div>

    <v-row v-if="stats.recent_videos.length > 0">
      <v-col
        v-for="video in stats.recent_videos"
        :key="video.id"
        cols="12"
        sm="6"
        md="4"
        lg="3"
      >
        <v-card
          class="recent-video-card"
          hover
          @click="$emit('play', video)"
          @contextmenu.prevent="openContextMenu($event, video)"
        >
          <div class="cover-wrapper">
            <img
              v-if="videoCoverSrc(video)"
              :src="videoCoverSrc(video)"
              referrerpolicy="no-referrer"
              class="cover-img"
              alt="cover"
              loading="lazy"
              decoding="async"
            />
            <v-icon v-else size="48">mdi-video-off-outline</v-icon>
          </div>
          <div class="recent-video-body">
            <div class="recent-video-title" :title="video.title">
              {{ video.title }}
            </div>
            <div class="recent-video-author">
              {{ video.author || t("未知作者") }}
            </div>
          </div>
        </v-card>
      </v-col>
    </v-row>

    <v-alert v-else type="info" variant="tonal" class="mt-4">
      {{ t("还没有收藏任何视频。点击下方进入视频库并添加第一个吧。") }}
    </v-alert>

    <ContextMenu
      v-model="contextMenu.visible"
      :x="contextMenu.x"
      :y="contextMenu.y"
      :items="contextMenuItems"
      @select="onContextMenuSelect"
    />

    <!-- 以下弹窗只在点击时出现，异步加载，不进首屏包。 -->
    <DataTransferDialog
      v-if="showTransfer"
      v-model="showTransfer"
      @imported="onImported"
    />

    <AddFab @select="onAddFabSelect" />

    <AddVideoDialog
      v-if="showAddVideo"
      v-model="showAddVideo"
      :categories="flatCategories"
      :initial-category-id="null"
      @saved="onAdded"
    />
    <LocalImportDialog
      v-if="showLocalImport"
      v-model="showLocalImport"
      :categories="flatCategories"
      :initial-category-id="null"
      @imported="onLocalImported"
    />
    <ImportFavDialog
      v-if="showImportFav"
      v-model="showImportFav"
      :categories="flatCategories"
      @imported="onFavImported"
    />
    <BulkImportDialog
      v-if="showBulkImport"
      v-model="showBulkImport"
      :mode="bulkMode"
      :categories="flatCategories"
      @imported="onBulkImported"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CategoryNode, Stats, TagInfo, VideoRow } from "../types";
import ContextMenu from "../components/ContextMenu.vue";
import AddFab from "../components/AddFab.vue";
import { useVideoContextMenu } from "../composables/useVideoContextMenu";
import { videoCoverSrc } from "../utils/media";
import { t } from "../i18n";

// 弹窗按需加载：首次点击时才下载对应 chunk，启动时只需要仪表盘本身。
const DataTransferDialog = defineAsyncComponent(
  () => import("../components/DataTransferDialog.vue"),
);
const AddVideoDialog = defineAsyncComponent(
  () => import("../components/AddVideoDialog.vue"),
);
const LocalImportDialog = defineAsyncComponent(
  () => import("../components/LocalImportDialog.vue"),
);
const ImportFavDialog = defineAsyncComponent(
  () => import("../components/ImportFavDialog.vue"),
);
const BulkImportDialog = defineAsyncComponent(
  () => import("../components/BulkImportDialog.vue"),
);

const props = defineProps<{
  /** 侧栏已经加载好的标签，避免仪表盘重复请求 list_tags。 */
  tags?: TagInfo[];
}>();

const emit = defineEmits<{
  navigate: [view: string];
  play: [video: VideoRow];
  tag: [id: number];
  "data-changed": [];
}>();

const stats = ref<Stats>({
  videos_count: 0,
  categories_count: 0,
  tags_count: 0,
  recent_videos: [],
});
const tags = computed(() => props.tags ?? []);
const categories = ref<CategoryNode[]>([]);
const visibleTags = computed(() => tags.value.slice(0, 12));
const loading = ref(true);
const showTransfer = ref(false);
const showAddVideo = ref(false);
const showLocalImport = ref(false);
const showImportFav = ref(false);
const showBulkImport = ref(false);
const bulkMode = ref<"collection" | "watchLater">("collection");

// 分类只在要打开导入/添加弹窗时才需要，不在启动路径上。
let categoriesRequest: Promise<void> | null = null;

function ensureCategories() {
  if (!categoriesRequest) {
    categoriesRequest = invoke<CategoryNode[]>("get_category_tree")
      .then((tree) => {
        categories.value = tree;
      })
      .catch((e) => {
        console.warn("加载分类失败", e);
        categoriesRequest = null;
      });
  }
  return categoriesRequest;
}

function invalidateCategories() {
  categoriesRequest = null;
  categories.value = [];
}

const flatCategories = computed(() => {
  const out: { id: number; label: string }[] = [];
  const walk = (nodes: CategoryNode[], depth: number) => {
    for (const node of nodes) {
      out.push({ id: node.id, label: "　".repeat(depth) + node.name });
      walk(node.children, depth + 1);
    }
  };
  walk(categories.value, 0);
  return out;
});

async function loadDashboard() {
  loading.value = true;
  try {
    // 标签由 App.vue 统一加载并通过 props 传入，这里只取统计。
    stats.value = await invoke<Stats>("get_stats");
  } catch (e) {
    console.warn("加载仪表盘失败", e);
  } finally {
    loading.value = false;
  }
}

async function onImported() {
  await loadDashboard();
  invalidateCategories();
  emit("data-changed");
}

function onAddFabSelect(
  mode: "single" | "local" | "collection" | "favorites" | "watchLater",
) {
  // 分类是弹窗里的响应式数据，先打开弹窗、在后台补齐，避免点击等待。
  void ensureCategories();
  if (mode === "single") {
    showAddVideo.value = true;
  } else if (mode === "local") {
    showLocalImport.value = true;
  } else if (mode === "favorites") {
    showImportFav.value = true;
  } else {
    bulkMode.value = mode;
    showBulkImport.value = true;
  }
}

async function onAdded() {
  await loadDashboard();
  emit("data-changed");
}

async function onFavImported(_count: number) {
  await loadDashboard();
  invalidateCategories();
  emit("data-changed");
}

async function onLocalImported(_count: number) {
  await loadDashboard();
  invalidateCategories();
  emit("data-changed");
}

async function onBulkImported(_count: number) {
  await loadDashboard();
  invalidateCategories();
  emit("data-changed");
}

const {
  contextMenu,
  contextMenuItems,
  openContextMenu,
  onContextMenuSelect,
} = useVideoContextMenu({
  onPlay: (video) => emit("play", video),
});

onMounted(loadDashboard);
</script>

<style scoped>
.dashboard-hero {
  background: linear-gradient(120deg, #312e81 0%, #1d4ed8 52%, #0891b2 100%);
  color: white;
  box-shadow: 0 18px 40px rgba(30, 64, 175, 0.25);
}

.dashboard-hero-sub {
  color: rgba(255, 255, 255, 0.72) !important;
}
.dashboard-hero-mark {
  width: 46px;
  height: 46px;
  border-radius: 12px;
  box-shadow: 0 8px 22px rgba(0, 0, 0, 0.18);
  flex: 0 0 auto;
}

.cover-wrapper {
  width: 100%;
  aspect-ratio: 16 / 9;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.06);
  display: flex;
  align-items: center;
  justify-content: center;
}
.recent-video-card {
  overflow: hidden;
}
.recent-video-body {
  padding: 7px 9px 8px;
}
.recent-video-title {
  font-size: 0.82rem;
  line-height: 1.35;
  font-weight: 500;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.recent-video-author {
  margin-top: 2px;
  font-size: 0.72rem;
  line-height: 1.2;
  color: rgba(var(--v-theme-on-surface), 0.62);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.cover-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
</style>
