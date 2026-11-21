<template>
  <div>
    <div class="d-flex align-center flex-wrap mb-4" style="gap: 12px">
      <v-icon size="30" :color="scope === 'github' ? '#24292F' : '#607D8B'">
        {{ scope === "github" ? "mdi-github" : "mdi-web" }}
      </v-icon>
      <div>
        <div class="text-h5 font-weight-bold">{{ pageTitle }}</div>
        <div class="text-caption text-medium-emphasis">{{ pageSubtitle }}</div>
      </div>
      <v-spacer />
      <v-btn
        v-if="scope === 'github'"
        variant="tonal"
        prepend-icon="mdi-cloud-download-outline"
        :loading="loading"
        @click="showGithubStars = true"
      >
        {{ t("读取 GitHub Star") }}
      </v-btn>
      <v-btn color="primary" prepend-icon="mdi-plus" @click="showAddVideo = true">
        {{ t("添加网页") }}
      </v-btn>
    </div>

    <div class="d-flex align-center flex-wrap mb-4" style="gap: 8px">
      <v-text-field
        v-model="search"
        :label="t('搜索标题、作者或描述')"
        prepend-inner-icon="mdi-magnify"
        density="compact"
        variant="outlined"
        hide-details
        clearable
        style="max-width: 360px"
      />
      <v-chip-group v-if="scope === 'web'" v-model="sourceFilter" mandatory>
        <v-chip
          v-for="option in sourceOptions"
          :key="option.value"
          :value="option.value"
          size="small"
          filter
        >
          {{ option.title }}
        </v-chip>
      </v-chip-group>
      <v-spacer />
      <v-chip size="small" variant="text">{{ t("共 {n} 项", { n: items.length }) }}</v-chip>
    </div>

    <v-progress-linear v-if="loading" indeterminate color="primary" class="mb-4" />

    <div v-if="!loading && filteredItems.length === 0" class="empty-state">
      <v-icon size="54" opacity="0.35">
        {{ scope === "github" ? "mdi-github" : "mdi-web-off" }}
      </v-icon>
      <div class="text-subtitle-1 mt-2">{{ emptyTitle }}</div>
      <div class="text-caption text-medium-emphasis mt-1">
        {{ emptyHint }}
      </div>
    </div>

    <div v-else class="web-grid">
      <VideoCard
        v-for="item in filteredItems"
        :key="item.id"
        :video="item"
        @play="onPlay"
        @edit="onEdit"
        @delete="onDelete"
      />
    </div>

    <AddVideoDialog
      v-model="showAddVideo"
      :categories="categories"
      :initial-category-id="null"
      @saved="loadItems"
    />

    <GitHubStarsDialog
      v-if="scope === 'github'"
      v-model="showGithubStars"
      @imported="loadItems"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CategoryNode, VideoRow } from "../types";
import VideoCard from "../components/VideoCard.vue";
import AddVideoDialog from "../components/AddVideoDialog.vue";
import GitHubStarsDialog from "../components/GitHubStarsDialog.vue";
import { confirmDialog, notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{ scope: "github" | "web" }>();
const emit = defineEmits<{
  play: [video: VideoRow];
  edit: [video: VideoRow];
}>();

const items = ref<VideoRow[]>([]);
const loading = ref(false);
const search = ref("");
const sourceFilter = ref<"all" | "blog" | "bilibili_article">("all");
const showAddVideo = ref(false);
const showGithubStars = ref(false);
const categories = ref<{ id: number; label: string }[]>([]);

const isGithub = computed(() => props.scope === "github");
const pageTitle = computed(() =>
  isGithub.value ? t("GitHub 收藏") : t("网页收藏"),
);
const pageSubtitle = computed(() =>
  isGithub.value
    ? t("独立管理收藏的 GitHub 仓库，支持从 GitHub Star 批量导入")
    : t("收藏博客、文章和其他网页，和视频库分开管理"),
);
const emptyTitle = computed(() =>
  isGithub.value ? t("还没有 GitHub 收藏") : t("还没有网页收藏"),
);
const emptyHint = computed(() =>
  isGithub.value
    ? t("添加 GitHub 仓库链接，或从 GitHub Star 导入")
    : t("添加任意网页链接，自动提取标题、描述和正文"),
);

const sourceOptions = computed(() => [
  { title: t("全部"), value: "all" as const },
  { title: t("普通网页"), value: "blog" as const },
  { title: t("B站专栏"), value: "bilibili_article" as const },
]);

const filteredItems = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  return items.value.filter((item) => {
    if (!isGithub.value && sourceFilter.value !== "all" && item.platform !== sourceFilter.value) {
      return false;
    }
    if (!keyword) return true;
    return [item.title, item.author ?? "", item.description ?? ""]
      .join(" ")
      .toLowerCase()
      .includes(keyword);
  });
});

async function loadItems() {
  loading.value = true;
  try {
    items.value = await invoke<VideoRow[]>("list_web_videos", {
      scope: props.scope,
    });
  } catch (e) {
    items.value = [];
    notify.error(t("加载失败：{e}", { e: String(e) }));
  } finally {
    loading.value = false;
  }
}

async function loadCategories() {
  try {
    const tree = await invoke<CategoryNode[]>("get_category_tree");
    const flat: { id: number; label: string }[] = [];
    const walk = (nodes: CategoryNode[], depth: number) => {
      for (const node of nodes) {
        flat.push({ id: node.id, label: "　".repeat(depth) + node.name });
        walk(node.children, depth + 1);
      }
    };
    walk(tree, 0);
    categories.value = flat;
  } catch {
    categories.value = [];
  }
}

function onPlay(video: VideoRow) {
  emit("play", video);
}

function onEdit(video: VideoRow) {
  emit("edit", video);
}

async function onDelete(id: number) {
  const ok = await confirmDialog({
    title: t("删除网页"),
    message: t("确定删除这个收藏吗？相关的标签和笔记关联也会一起删除。"),
    confirmColor: "error",
    confirmText: t("删除"),
  });
  if (!ok) return;
  try {
    await invoke("delete_video", { id });
    notify.success(t("已删除"));
    await loadItems();
    window.dispatchEvent(new CustomEvent("collectrix:data-changed"));
  } catch (e) {
    notify.error(t("删除失败：{e}", { e: String(e) }));
  }
}

function onDataChanged() {
  void loadItems();
}

watch(
  () => props.scope,
  () => {
    sourceFilter.value = "all";
    search.value = "";
    void loadItems();
  },
);

onMounted(() => {
  void loadCategories();
  void loadItems();
  window.addEventListener("collectrix:data-changed", onDataChanged);
});

onUnmounted(() => {
  window.removeEventListener("collectrix:data-changed", onDataChanged);
});
</script>

<style scoped>
.web-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
}
.empty-state {
  padding: 64px 16px;
  text-align: center;
  border: 1px dashed rgba(128, 128, 128, 0.25);
  border-radius: 12px;
}
</style>
