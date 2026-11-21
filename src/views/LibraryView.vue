<!--
  改动点：
  1. 顶部工具条新增「网格 / 列表」视图切换按钮
  2. 分组渲染、扁平渲染都支持两种视图
  3. 分组渲染中，每个分组独立显示视图模式
  4. 其余逻辑（分类树、按标签筛选、添加视频/分类）保持不变
-->
<template>
  <v-row>
    <!-- 左侧分类树 -->
    <v-col
      cols="auto"
      class="category-column"
      :class="{ 'category-column--hover': categoryHover }"
      :style="categoryColumnStyle"
    >
      <div
        v-if="categoryHover"
        class="category-hover-handle"
        :title="t('分类')"
      >
        <v-icon size="20">mdi-folder-outline</v-icon>
        <span>{{ t("分类") }}</span>
      </div>
      <v-sheet
        class="pa-3 rounded category-sidebar"
        border
        @dragover.prevent
        @drop.prevent="onDropOnCategorySidebar"
      >
        <div class="d-flex align-center mb-2">
          <span class="text-subtitle-2">{{ t("分类") }}</span>
          <v-spacer />
          <v-btn
            icon
            size="x-small"
            variant="text"
            @click="showAddCategory = true"
          >
            <v-icon>mdi-plus</v-icon>
          </v-btn>
        </div>
        <v-list v-model:opened="openedCategories" density="compact" nav>
          <v-list-item
            :title="t('全部视频')"
            :active="selectedCategory === null && !activeTag"
            data-category-id="root"
            @click="selectCategory(null)"
            @dragover.prevent
            @drop.prevent.stop="onDropToRoot"
          />
          <CategoryTree
            v-for="node in categories"
            :key="node.id"
            :node="node"
            :selected-id="selectedCategory"
            @select="onSelectCategory"
            @delete="onDeleteCategory"
            @contextmenu="openCategoryContextMenu"
            @drop="onDropVideoToCategory"
          />
        </v-list>
      </v-sheet>
    </v-col>

    <!-- 右侧视频区 -->
    <v-col>
      <!-- 工具条 -->
      <div class="d-flex align-center flex-wrap mb-4" style="gap: 8px">
        <v-chip
          v-if="activeTag"
          :color="activeTag.color"
          closable
          @click:close="clearTagFilter"
        >
          {{ t("标签：{name}", { name: activeTag.name }) }}
        </v-chip>
        <v-chip
          v-else-if="selectedCategoryName"
          color="primary"
          variant="tonal"
        >
          {{ selectedCategoryName }}
        </v-chip>
        <v-spacer />

        <!-- 视图切换 -->
        <v-btn-group density="compact" variant="outlined">
          <v-btn
            :variant="viewMode === 'grid' ? 'flat' : 'outlined'"
            :color="viewMode === 'grid' ? 'primary' : undefined"
            icon="mdi-view-grid"
            :title="t('网格视图')"
            @click="emit('update:viewMode', 'grid')"
          />
          <v-btn
            :variant="viewMode === 'list' ? 'flat' : 'outlined'"
            :color="viewMode === 'list' ? 'primary' : undefined"
            icon="mdi-view-list"
            :title="t('列表视图')"
            @click="emit('update:viewMode', 'list')"
          />
        </v-btn-group>

        <v-btn
          :variant="categoryHover ? 'flat' : 'outlined'"
          :color="categoryHover ? 'primary' : undefined"
          :icon="categoryHover ? 'mdi-dock-left' : 'mdi-dock-window'"
          :title="categoryHover ? t('固定分类栏') : t('悬停展开分类栏')"
          @click="toggleCategoryHover"
        />

        <v-btn
          color="primary"
          prepend-icon="mdi-plus"
          @click="showAddVideo = true"
        >
          {{ t("添加视频") }}
        </v-btn>
        <v-btn
          variant="tonal"
          prepend-icon="mdi-harddisk-plus"
          @click="showLocalImport = true"
        >
          {{ t("导入本地视频") }}
        </v-btn>
        <v-btn
          variant="tonal"
          prepend-icon="mdi-folder-download"
          @click="showImportFav = true"
        >
          {{ t("导入收藏夹") }}
        </v-btn>
      </div>

      <!-- 首屏骨架屏：避免等待数据库时出现空白 -->
      <template v-if="loadingVideos && videos.length === 0 && groups.length === 0">
        <div class="d-flex align-center mb-2" style="gap: 8px">
          <v-progress-circular indeterminate size="18" color="primary" />
          <span class="text-caption text-medium-emphasis">
            {{ t("正在加载视频…") }}
          </span>
        </div>
        <div class="video-grid">
          <v-sheet
            v-for="i in 8"
            :key="i"
            class="skeleton-card"
            border
          >
            <div class="skeleton-cover" />
            <div class="skeleton-body">
              <div class="skeleton-line skeleton-line--title" />
              <div class="skeleton-line" />
              <div class="skeleton-line skeleton-line--short" />
            </div>
          </v-sheet>
        </div>
      </template>

      <!-- 分组模式：选分类时 -->
      <template v-else-if="groups.length > 0">
        <div
          v-for="(group, idx) in groups"
          :key="group.category_id ?? 'own'"
          class="mb-4"
        >
          <div class="group-header">
            <v-icon size="18" class="mr-1">mdi-folder-outline</v-icon>
            <span class="text-subtitle-2 font-weight-medium">
              {{ group.category_name }}
            </span>
            <v-chip size="x-small" variant="text" class="ml-2">
              {{ group.videos.length }}
            </v-chip>
          </div>

          <div v-if="viewMode === 'grid'" class="video-grid">
            <VideoCard
              v-for="video in group.videos"
              :key="video.id"
              :video="video"
              @play="onPlay"
              @edit="onEdit"
              @delete="onDelete"
              @contextmenu="openContextMenu"
            />
          </div>
          <div v-else class="video-list">
            <VideoListItem
              v-for="video in group.videos"
              :key="video.id"
              :video="video"
              @play="onPlay"
              @edit="onEdit"
              @delete="onDelete"
              @contextmenu="openContextMenu"
            />
          </div>

          <v-divider v-if="idx < groups.length - 1" class="mt-4" />
        </div>
      </template>

      <!-- 扁平模式：全部视频 / 按标签筛选 -->
      <template v-else-if="!loadingVideos || videos.length > 0">
        <div v-if="viewMode === 'grid'" class="video-grid">
          <VideoCard
            v-for="video in videos"
            :key="video.id"
            :video="video"
            @play="onPlay"
            @edit="onEdit"
            @delete="onDelete"
            @contextmenu="openContextMenu"
          />
        </div>
        <div v-else class="video-list">
          <VideoListItem
            v-for="video in videos"
            :key="video.id"
            :video="video"
            @play="onPlay"
            @edit="onEdit"
            @delete="onDelete"
            @contextmenu="openContextMenu"
          />
        </div>
      </template>

      <div
        v-if="!groups.length && hasMore"
        ref="loadMoreSentinel"
        class="load-more-sentinel"
        @click="loadMore"
      >
        <v-progress-circular v-if="loadingMore" indeterminate size="22" />
        <span v-else class="text-caption text-medium-emphasis">
          {{ t("继续滚动加载") }}
        </span>
      </div>

      <v-alert v-if="isEmpty" type="info" variant="tonal" class="mt-4">
        {{ activeTag ? t("该标签下还没有视频") : t("当前分类下还没有视频") }}
      </v-alert>
    </v-col>
  </v-row>

  <ContextMenu
    v-model="contextMenu.visible"
    :x="contextMenu.x"
    :y="contextMenu.y"
    :items="contextMenuItems"
    @select="onContextMenuSelect"
  />
  <ContextMenu
    v-model="categoryMenu.visible"
    :x="categoryMenu.x"
    :y="categoryMenu.y"
    :items="categoryMenuItems"
    @select="onCategoryMenuSelect"
  />

  <!-- 弹窗 -->
  <AddVideoDialog
    v-model="showAddVideo"
    :categories="flatCategories"
    :initial-category-id="selectedCategory"
    @saved="onVideoSaved"
  />
  <VideoEditDialog
    v-model="showEditVideo"
    :video="editingVideo"
    :categories="flatCategories"
    @saved="onVideoSaved"
  />
  <LocalImportDialog
    v-model="showLocalImport"
    :categories="flatCategories"
    :initial-category-id="selectedCategory"
    @imported="onLocalImported"
  />
  <ImportFavDialog
    v-model="showImportFav"
    :categories="flatCategories"
    @imported="onFavImported"
  />
  <BulkImportDialog
    v-model="showBulkImport"
    :mode="bulkMode"
    :categories="flatCategories"
    @imported="onBulkImported"
  />
  <MoveCategoryDialog
    v-model="showMoveCategory"
    :category="movingCategory"
    :categories="moveCategoryOptions"
    :current-parent-id="movingCategoryParent"
    @moved="onCategoryMoved"
  />
  <DeleteConfirmDialog
    v-model="showDeleteCategory"
    :title="deleteCategoryTitle"
    :entity-name="deleteCategoryNode?.name ?? ''"
    :videos="deleteCategoryVideos"
    :warning="deleteCategoryWarning"
    :confirm-text="
      deleteCategoryVideosToo ? t('删除分类和视频') : t('删除分类')
    "
    @confirm="confirmDeleteCategory"
  />

  <AddFab @select="onAddFabSelect" />

  <v-btn
    v-if="showBackToTop"
    class="back-to-top"
    icon="mdi-arrow-up"
    color="primary"
    elevation="6"
    :title="t('返回顶部')"
    @click="scrollToTop"
  />

  <v-dialog v-model="showAddCategory" max-width="420">
    <v-card>
      <v-card-title>{{ t("新增分类") }}</v-card-title>
      <v-card-text>
        <v-text-field v-model="newCategoryName" :label="t('分类名称')" />
        <v-select
          v-model="newCategoryParent"
          :items="flatCategories"
          item-title="label"
          item-value="id"
          :label="t('父分类（留空为顶级）')"
          clearable
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="showAddCategory = false">{{ t("取消") }}</v-btn>
        <v-btn color="primary" @click="onAddCategory">{{ t("确定") }}</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import {
  computed,
  nextTick,
  onMounted,
  onUnmounted,
  reactive,
  ref,
  watch,
} from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  CategoryNode,
  CategoryVideoGroup,
  ContextMenuItem,
  TagInfo,
  VideoRow,
} from "../types";
import CategoryTree from "../components/CategoryTree.vue";
import VideoCard from "../components/VideoCard.vue";
import VideoListItem from "../components/VideoListItem.vue";
import AddVideoDialog from "../components/AddVideoDialog.vue";
import LocalImportDialog from "../components/LocalImportDialog.vue";
import VideoEditDialog from "../components/VideoEditDialog.vue";
import ImportFavDialog from "../components/ImportFavDialog.vue";
import BulkImportDialog from "../components/BulkImportDialog.vue";
import AddFab from "../components/AddFab.vue";
import ContextMenu from "../components/ContextMenu.vue";
import MoveCategoryDialog from "../components/MoveCategoryDialog.vue";
import DeleteConfirmDialog from "../components/DeleteConfirmDialog.vue";

import { notify, confirmDialog } from "../composables/useUI";
import { useVideoContextMenu } from "../composables/useVideoContextMenu";
import { t } from "../i18n";

const categories = ref<CategoryNode[]>([]);
const openedCategories = ref<number[]>(
  (() => {
    try {
      const raw = localStorage.getItem("collectrix.openedCategories");
      return raw ? (JSON.parse(raw) as number[]) : [];
    } catch {
      return [];
    }
  })(),
);

watch(
  openedCategories,
  (value) => {
    try {
      localStorage.setItem(
        "collectrix.openedCategories",
        JSON.stringify(value),
      );
    } catch {
      // ignore storage errors
    }
  },
  { deep: true },
);

const videos = ref<VideoRow[]>([]);
const groups = ref<CategoryVideoGroup[]>([]);
const selectedCategory = ref<number | null>(null);
const selectedCategoryName = ref("");
const activeTag = ref<TagInfo | null>(null);

const CATEGORY_HOVER_KEY = "collectrix.categoryHover";
const categoryHover = ref(
  (() => {
    try {
      return localStorage.getItem(CATEGORY_HOVER_KEY) === "1";
    } catch {
      return false;
    }
  })(),
);
const categoryColumnStyle = computed(() =>
  categoryHover.value
    ? { width: "44px", alignSelf: "flex-start" }
    : {
        width: "max-content",
        minWidth: "280px",
        maxWidth: "480px",
        alignSelf: "flex-start",
      },
);

function toggleCategoryHover() {
  categoryHover.value = !categoryHover.value;
  try {
    localStorage.setItem(
      CATEGORY_HOVER_KEY,
      categoryHover.value ? "1" : "0",
    );
  } catch {
    // ignore storage errors
  }
}

/** 视图模式：网格 / 列表 */
const props = defineProps<{
  viewMode: "grid" | "list";
}>();

const emit = defineEmits<{
  "update:viewMode": [v: "grid" | "list"];
  play: [video: VideoRow];
  "tags-changed": [];
}>();

const showAddCategory = ref(false);
const newCategoryName = ref("");
const newCategoryParent = ref<number | null>(null);

const showAddVideo = ref(false);
const showEditVideo = ref(false);
const showLocalImport = ref(false);
const editingVideo = ref<VideoRow | null>(null);

const isEmpty = computed(() => {
  if (loadingVideos.value || loadingMore.value) return false;
  if (groups.value.length > 0) return false;
  return videos.value.length === 0;
});

const showImportFav = ref(false);
const showBulkImport = ref(false);
const bulkMode = ref<"collection" | "watchLater">("collection");

function onAddFabSelect(
  mode: "single" | "local" | "collection" | "favorites" | "watchLater",
) {
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

async function onFavImported(_count: number) {
  await loadVideos();
  emit("tags-changed");
}

async function onLocalImported(_count: number) {
  await loadVideos();
  emit("tags-changed");
}

async function onBulkImported(_count: number) {
  await loadVideos();
  emit("tags-changed");
}

const flatCategories = computed(() => {
  const out: { id: number; label: string }[] = [];
  const walk = (nodes: CategoryNode[], depth: number) => {
    for (const n of nodes) {
      out.push({ id: n.id, label: "　".repeat(depth) + n.name });
      walk(n.children, depth + 1);
    }
  };
  walk(categories.value, 0);
  return out;
});

async function loadCategories() {
  try {
    categories.value = await invoke<CategoryNode[]>("get_category_tree");
  } catch (e) {
    console.warn("加载分类失败", e);
  }
}

const PAGE_SIZE = 48;
const loadedOffset = ref(0);
const hasMore = ref(true);
const loadingVideos = ref(false);
const loadingMore = ref(false);
const loadMoreSentinel = ref<HTMLElement | null>(null);
let loadObserver: IntersectionObserver | null = null;

let loadRequestId = 0;

async function loadVideos(reset = true) {
  const requestId = ++loadRequestId;
  loadingVideos.value = true;
  if (reset) {
    loadedOffset.value = 0;
    hasMore.value = true;
    groups.value = [];
    videos.value = [];
  }
  try {
    if (activeTag.value) {
      const page = await invoke<VideoRow[]>("list_videos_by_tag", {
        tagId: activeTag.value.id,
        limit: PAGE_SIZE,
        offset: loadedOffset.value,
        excludeArticles: true,
      });
      if (requestId !== loadRequestId) return;
      videos.value = reset ? page : [...videos.value, ...page];
      loadedOffset.value += page.length;
      hasMore.value = page.length === PAGE_SIZE;
      return;
    }
    if (selectedCategory.value === null) {
      const page = await invoke<VideoRow[]>("list_videos", {
        categoryId: null,
        limit: PAGE_SIZE,
        offset: loadedOffset.value,
        excludeArticles: true,
      });
      if (requestId !== loadRequestId) return;
      videos.value = reset ? page : [...videos.value, ...page];
      loadedOffset.value += page.length;
      hasMore.value = page.length === PAGE_SIZE;
      return;
    }
    const nextGroups = await invoke<CategoryVideoGroup[]>(
      "list_videos_with_children",
      { categoryId: selectedCategory.value, excludeArticles: true },
    );
    if (requestId !== loadRequestId) return;
    groups.value = nextGroups;
    videos.value = [];
    loadedOffset.value = 0;
    hasMore.value = false;
  } finally {
    if (requestId === loadRequestId) {
      loadingVideos.value = false;
    }
  }
}

async function loadMore() {
  if (
    loadingVideos.value ||
    loadingMore.value ||
    !hasMore.value ||
    groups.value.length > 0
  ) {
    return;
  }
  loadingMore.value = true;
  try {
    await loadVideos(false);
  } finally {
    loadingMore.value = false;
  }
}

function setupLoadObserver() {
  loadObserver?.disconnect();
  loadObserver = null;
  if (!loadMoreSentinel.value) return;
  loadObserver = new IntersectionObserver(
    (entries) => {
      if (entries.some((entry) => entry.isIntersecting)) {
        loadMore();
      }
    },
    { rootMargin: "400px 0px" },
  );
  loadObserver.observe(loadMoreSentinel.value);
}

watch(
  [loadMoreSentinel, () => videos.value.length, () => groups.value.length, hasMore],
  () => {
    nextTick(setupLoadObserver);
  },
  { flush: "post" },
);

function selectCategory(id: number | null) {
  activeTag.value = null;
  selectedCategory.value = id;
  selectedCategoryName.value = id
    ? flatCategories.value.find((c) => c.id === id)?.label.trim() || ""
    : "";
  loadVideos();
}

function onSelectCategory(id: number, name: string) {
  selectedCategory.value = id;
  selectedCategoryName.value = name;
  activeTag.value = null;
  loadVideos();
}

function setTagFilter(tagId: number) {
  invoke<TagInfo[]>("list_tags").then((tags) => {
    const t = tags.find((x) => x.id === tagId);
    if (t) {
      activeTag.value = t;
      selectedCategory.value = null;
      selectedCategoryName.value = "";
      loadVideos();
    }
  });
}
defineExpose({ setTagFilter });

function clearTagFilter() {
  activeTag.value = null;
  loadVideos();
}

async function onAddCategory() {
  if (!newCategoryName.value.trim()) return;
  await invoke("add_category", {
    name: newCategoryName.value.trim(),
    parentId: newCategoryParent.value,
  });
  showAddCategory.value = false;
  newCategoryName.value = "";
  newCategoryParent.value = null;
  await loadCategories();
}

function findCategoryNode(nodes: CategoryNode[], id: number): CategoryNode | null {
  for (const node of nodes) {
    if (node.id === id) return node;
    const found = findCategoryNode(node.children, id);
    if (found) return found;
  }
  return null;
}

function collectCategoryIds(node: CategoryNode, out: Set<number>) {
  out.add(node.id);
  for (const child of node.children) collectCategoryIds(child, out);
}

const categoryMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  node: null as CategoryNode | null,
});

const categoryMenuItems = computed<ContextMenuItem[]>(() => {
  if (!categoryMenu.node) return [];
  return [
    { key: "move", title: t("移动分类"), icon: "mdi-folder-move" },
    { key: "delete", title: t("删除分类"), icon: "mdi-delete-outline" },
    {
      key: "deleteAll",
      title: t("删除分类及视频"),
      icon: "mdi-delete-forever",
      color: "error",
    },
  ];
});

function openCategoryContextMenu(event: MouseEvent, node: CategoryNode) {
  event.preventDefault();
  event.stopPropagation();
  categoryMenu.node = node;
  categoryMenu.x = event.clientX;
  categoryMenu.y = event.clientY;
  categoryMenu.visible = true;
}

async function onCategoryMenuSelect(key: string) {
  const node = categoryMenu.node;
  categoryMenu.visible = false;
  if (!node) return;
  if (key === "move") {
    openMoveCategory(node);
  } else if (key === "delete") {
    await openDeleteCategory(node, false);
  } else if (key === "deleteAll") {
    await openDeleteCategory(node, true);
  }
}

const showMoveCategory = ref(false);
const movingCategory = ref<CategoryNode | null>(null);
const movingCategoryParent = ref<number | null>(null);

const moveCategoryOptions = computed(() => {
  const node = movingCategory.value;
  if (!node) return [];
  const excluded = new Set<number>();
  collectCategoryIds(node, excluded);
  return flatCategories.value.filter((category) => !excluded.has(category.id));
});

function openMoveCategory(node: CategoryNode) {
  movingCategory.value = node;
  movingCategoryParent.value = node.parent_id;
  showMoveCategory.value = true;
}

async function onCategoryMoved() {
  await loadCategories();
  await loadVideos();
}

const showDeleteCategory = ref(false);
const deleteCategoryNode = ref<CategoryNode | null>(null);
const deleteCategoryVideos = ref<VideoRow[]>([]);
const deleteCategoryVideosToo = ref(false);

const deleteCategoryTitle = computed(() =>
  deleteCategoryVideosToo.value
    ? t("删除分类及全部视频")
    : t("删除分类"),
);

const deleteCategoryWarning = computed(() => {
  const node = deleteCategoryNode.value;
  if (!node) return "";
  return deleteCategoryVideosToo.value
    ? t("分类「{name}」及其子分类下的 {n} 个视频会被永久删除。", {
        name: node.name,
        n: deleteCategoryVideos.value.length,
      })
    : t("分类「{name}」及其子分类会被删除，视频会保留并变为未分类。", {
        name: node.name,
      });
});

async function openDeleteCategory(
  node: CategoryNode,
  deleteVideosToo: boolean,
) {
  try {
    const videos = await invoke<VideoRow[]>("list_videos_in_category_tree", {
      id: node.id,
    });
    deleteCategoryNode.value = node;
    deleteCategoryVideos.value = videos;
    deleteCategoryVideosToo.value = deleteVideosToo;
    showDeleteCategory.value = true;
  } catch (e) {
    notify.error(t("加载分类视频失败：{e}", { e: String(e) }));
  }
}

async function confirmDeleteCategory() {
  const node = deleteCategoryNode.value;
  if (!node) return;
  try {
    await invoke("delete_category_tree", {
      id: node.id,
      deleteVideos: deleteCategoryVideosToo.value,
    });
    notify.success(t("分类已删除"));
    showDeleteCategory.value = false;
    await loadCategories();
    selectCategory(null);
  } catch (e) {
    notify.error(t("删除失败：{e}", { e: String(e) }));
  }
}

async function onDeleteCategory(id: number) {
  const node = findCategoryNode(categories.value, id);
  if (node) await openDeleteCategory(node, false);
}

async function moveVideoToCategory(videoId: number, categoryId: number | null) {
  try {
    await invoke("move_video_to_category", { videoId, categoryId });
    notify.success(t("已移动到分类"));
    await loadVideos();
  } catch (e) {
    notify.error(t("移动视频失败：{e}", { e: String(e) }));
  }
}

function onDropVideoToCategory(videoId: number, categoryId: number) {
  moveVideoToCategory(videoId, categoryId);
}

function droppedVideoId(event: DragEvent): number {
  const raw =
    event.dataTransfer?.getData("application/x-collectrix-video-id") ||
    event.dataTransfer?.getData("text/plain") ||
    event.dataTransfer?.getData("text");
  return Number(raw);
}

function onDropToRoot(event: DragEvent) {
  const videoId = droppedVideoId(event);
  if (Number.isFinite(videoId) && videoId > 0) {
    moveVideoToCategory(videoId, null);
  }
}

/** 兜底：即使具体分类项没有收到 drop，也可以从 sidebar 的 drop 事件里找到目标分类。 */
function onDropOnCategorySidebar(event: DragEvent) {
  const target = (event.target as HTMLElement | null)?.closest(
    "[data-category-id]",
  ) as HTMLElement | null;
  const categoryValue = target?.dataset.categoryId;
  if (!categoryValue) return;
  const videoId = droppedVideoId(event);
  if (!Number.isFinite(videoId) || videoId <= 0) return;
  const categoryId = categoryValue === "root" ? null : Number(categoryValue);
  moveVideoToCategory(videoId, categoryId);
}

function onPlay(video: VideoRow) {
  emit("play", video);
}

function onEdit(video: VideoRow) {
  editingVideo.value = video;
  showEditVideo.value = true;
}

async function onDelete(id: number) {
  const ok = await confirmDialog({
    title: t("删除视频"),
    message: t("确定删除这个视频吗？相关标签会一并解除关联。"),
    confirmColor: "error",
    confirmText: t("删除"),
  });
  if (!ok) return;
  try {
    await invoke("delete_video", { id });
    notify.success(t("已删除"));
    await loadVideos();
  } catch (e) {
    notify.error(t("删除失败：{e}", { e: String(e) }));
  }
}

const {
  contextMenu,
  contextMenuItems,
  openContextMenu,
  onContextMenuSelect,
} = useVideoContextMenu({
  onPlay,
  onEdit,
  onDelete: (video) => onDelete(video.id),
});

async function onVideoSaved() {
  await loadVideos();
  emit("tags-changed");
}

const showBackToTop = ref(false);

function onScroll() {
  showBackToTop.value = window.scrollY > 400;
}

function scrollToTop() {
  window.scrollTo({ top: 0, behavior: "smooth" });
}

onMounted(async () => {
  // 分类和视频互不依赖，并行请求可以明显缩短首屏等待。
  await Promise.all([loadCategories(), loadVideos()]);
  await nextTick();
  setupLoadObserver();
  window.addEventListener("scroll", onScroll, { passive: true });
});

onUnmounted(() => {
  loadObserver?.disconnect();
  window.removeEventListener("scroll", onScroll);
});
</script>

<style scoped>
.video-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
}
.video-list {
  display: flex;
  flex-direction: column;
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 8px;
  overflow: hidden;
  background: rgba(var(--v-theme-surface), 1);
}
.group-header {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
  padding-bottom: 4px;
  border-bottom: 1px dashed rgba(128, 128, 128, 0.25);
}
.category-sidebar {
  position: sticky;
  top: 88px;
  width: 100%;
  max-height: calc(100vh - 120px);
  overflow-y: auto;
}
.category-sidebar :deep(.v-list-item-title) {
  white-space: normal;
  overflow-wrap: anywhere;
  line-height: 1.35;
}
.category-column--hover {
  position: sticky;
  top: 88px;
  z-index: 20;
}
.category-column--hover .category-sidebar {
  position: absolute;
  top: 0;
  left: 0;
  width: max-content;
  min-width: 280px;
  max-width: 480px;
  max-height: calc(100vh - 120px);
  opacity: 0;
  transform: translateX(-14px);
  pointer-events: none;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.16);
  transition:
    opacity 0.16s ease,
    transform 0.16s ease;
}
.category-column--hover:hover .category-sidebar {
  opacity: 1;
  transform: translateX(0);
  pointer-events: auto;
}
.category-hover-handle {
  position: sticky;
  top: 88px;
  width: 44px;
  min-height: 132px;
  border-radius: 10px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  background: rgb(var(--v-theme-surface));
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  color: rgba(var(--v-theme-on-surface), 0.7);
  cursor: pointer;
}
.category-hover-handle span {
  writing-mode: vertical-rl;
  letter-spacing: 4px;
  font-size: 12px;
}
.category-column--hover:hover .category-hover-handle {
  opacity: 0;
}
.back-to-top {
  position: fixed;
  right: 28px;
  bottom: 92px;
  z-index: 10;
}
.load-more-sentinel {
  min-height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  cursor: pointer;
}
.skeleton-card {
  overflow: hidden;
  border-radius: 10px;
}
.skeleton-cover,
.skeleton-line {
  background: linear-gradient(
    100deg,
    rgba(128, 128, 128, 0.1) 20%,
    rgba(128, 128, 128, 0.2) 40%,
    rgba(128, 128, 128, 0.1) 60%
  );
  background-size: 300% 100%;
  animation: skeleton-shimmer 1.4s ease-in-out infinite;
}
.skeleton-cover {
  aspect-ratio: 16 / 9;
}
.skeleton-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
}
.skeleton-line {
  height: 12px;
  border-radius: 999px;
}
.skeleton-line--title {
  width: 82%;
  height: 15px;
}
.skeleton-line--short {
  width: 52%;
}
@keyframes skeleton-shimmer {
  0% {
    background-position: 100% 0;
  }
  100% {
    background-position: -100% 0;
  }
}

@media (max-width: 760px) {
  .category-column,
  .category-column--hover {
    width: 100% !important;
  }
  .category-sidebar,
  .category-column--hover .category-sidebar {
    position: static !important;
    width: 100% !important;
    max-height: 260px;
    opacity: 1 !important;
    transform: none !important;
    pointer-events: auto !important;
    box-shadow: none;
  }
  .category-hover-handle {
    display: none;
  }
  .back-to-top {
    right: 16px;
    bottom: 88px;
  }
}

</style>
