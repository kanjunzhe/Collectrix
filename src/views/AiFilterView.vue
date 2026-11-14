<template>
  <div style="max-width: 1000px">
    <div class="d-flex align-center flex-wrap mb-4" style="gap: 8px">
      <div>
        <div class="text-h5 font-weight-bold">{{ t("AI 筛选") }}</div>
        <div class="text-caption text-medium-emphasis">
          {{ t("从收藏夹或稍后再看中按话题筛选，由你决定是否入库") }}
        </div>
      </div>
      <v-spacer />
      <v-btn-toggle
        v-model="source"
        mandatory
        density="compact"
        variant="outlined"
        divided
      >
        <v-btn value="favorites" prepend-icon="mdi-folder-star">
          {{ t("收藏夹") }}
        </v-btn>
        <v-btn value="watchLater" prepend-icon="mdi-clock-outline">
          {{ t("稍后再看") }}
        </v-btn>
      </v-btn-toggle>
    </div>

    <v-card variant="outlined" class="mb-4">
      <v-card-text>
        <div class="d-flex flex-wrap" style="gap: 8px">
          <v-select
            v-if="source === 'favorites'"
            v-model="selectedMediaId"
            :items="folderItems"
            item-title="title"
            item-value="value"
            :label="t('选择收藏夹')"
            density="compact"
            variant="outlined"
            :loading="loadingFolders"
            style="min-width: 260px; flex: 1"
          />
          <v-text-field
            v-model="topic"
            :label="t('筛选题材')"
            :placeholder="t('例如：Rust、AI 编程、摄影教程')"
            density="compact"
            variant="outlined"
            hide-details
            style="min-width: 240px; flex: 1"
            @keydown.enter.prevent="onFilter"
          />
          <v-btn
            color="primary"
            variant="tonal"
            prepend-icon="mdi-robot"
            :loading="filtering"
            :disabled="
              !hasCookie ||
              !topic.trim() ||
              (source === 'favorites' && !selectedMediaId)
            "
            @click="onFilter"
          >
            {{ t("AI 筛选") }}
          </v-btn>
        </div>
        <v-alert
          v-if="!hasCookie"
          type="warning"
          variant="tonal"
          density="compact"
          class="mt-3"
        >
          {{ t("请先在设置中登录 B 站，才能读取收藏夹") }}
        </v-alert>
      </v-card-text>
    </v-card>

    <div class="d-flex align-center mb-2">
      <span class="text-caption text-medium-emphasis">
        {{ t("筛选结果：{n} 个", { n: results.length }) }}
      </span>
      <v-spacer />
      <v-btn size="small" variant="text" @click="selectAll">
        {{ t("全选") }}
      </v-btn>
      <v-btn size="small" variant="text" @click="selectedBvids = []">
        {{ t("取消全选") }}
      </v-btn>
    </div>

    <v-progress-linear v-if="filtering" indeterminate color="primary" class="mb-2" />

    <v-list v-if="results.length" density="compact" class="fav-results">
      <v-list-item
        v-for="item in results"
        :key="item.bvid"
        @click="toggle(item.bvid)"
      >
        <template #prepend>
          <v-checkbox-btn
            :model-value="selectedBvids.includes(item.bvid)"
            @click.stop
            @update:model-value="toggle(item.bvid)"
          />
        </template>
        <template #default>
          <div class="d-flex align-center" style="gap: 10px">
            <img
              v-if="item.cover_url"
              :src="item.cover_url"
              class="fav-cover"
              referrerpolicy="no-referrer"
              alt="cover"
              loading="lazy"
            />
            <div class="flex-grow-1 min-w-0">
              <div class="text-body-2 font-weight-medium text-truncate">
                {{ item.title }}
              </div>
              <div class="text-caption text-medium-emphasis">
                {{ item.author || t("未知作者") }}
              </div>
              <div class="d-flex flex-wrap mt-1" style="gap: 4px">
                <v-chip size="x-small" color="primary" variant="flat">
                  {{ t("相关度 {score}", { score: (item.score * 100).toFixed(0) + "%" }) }}
                </v-chip>
                <v-chip v-if="item.in_library" size="x-small" color="success" variant="tonal">
                  {{ t("已在库") }}
                </v-chip>
              </div>
            </div>
          </div>
        </template>
      </v-list-item>
    </v-list>
    <v-alert
      v-else-if="!filtering && topic.trim()"
      type="info"
      variant="tonal"
      density="compact"
    >
      {{ t("没有找到匹配的内容，试试更宽泛的话题或稍后重试") }}
    </v-alert>

    <div class="d-flex flex-wrap align-center mt-4" style="gap: 8px">
      <v-select
        v-model="targetCategory"
        :items="categoryOptions"
        item-title="label"
        item-value="id"
        :label="t('导入到分类（可选）')"
        density="compact"
        variant="outlined"
        clearable
        hide-details
        style="min-width: 260px; max-width: 360px"
      />
      <v-spacer />
      <span class="text-caption text-medium-emphasis">
        {{ t("已选择 {n} 个", { n: selectedBvids.length }) }}
      </span>
      <v-btn
        color="primary"
        :loading="importing"
        :disabled="selectedBvids.length === 0"
        variant="flat"
        prepend-icon="mdi-import"
        @click="onImport"
      >
        {{ t("导入选中 {n} 个", { n: selectedBvids.length }) }}
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AiFavCandidate, CategoryNode, VideoMeta } from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

interface FavFolder {
  id: number;
  title: string;
  media_count: number;
}

const source = ref<"favorites" | "watchLater">("favorites");
const hasCookie = ref(true);
const folders = ref<FavFolder[]>([]);
const loadingFolders = ref(false);
const selectedMediaId = ref<number | null>(null);
const topic = ref("");
const filtering = ref(false);
const results = ref<AiFavCandidate[]>([]);
const selectedBvids = ref<string[]>([]);
const importing = ref(false);
const targetCategory = ref<number | null>(null);
const categoryOptions = ref<{ id: number; label: string }[]>([]);

const folderItems = computed(() =>
  folders.value.map((folder) => ({
    title: `${folder.title}（${folder.media_count}）`,
    value: folder.id,
  })),
);

async function loadFolders() {
  loadingFolders.value = true;
  try {
    const settings = await invoke<{ bilibili_cookie: string }>("get_settings");
    hasCookie.value =
      !!settings.bilibili_cookie && settings.bilibili_cookie.includes("SESSDATA");
    if (!hasCookie.value) return;
    folders.value = await invoke<FavFolder[]>("list_fav_folders");
    if (folders.value.length > 0 && selectedMediaId.value === null) {
      selectedMediaId.value = folders.value[0].id;
    }
  } catch (e) {
    notify.error(t("读取收藏夹失败：{e}", { e: String(e) }));
  } finally {
    loadingFolders.value = false;
  }
}

async function loadCategories() {
  try {
    const tree = await invoke<CategoryNode[]>("get_category_tree");
    const out: { id: number; label: string }[] = [];
    const walk = (nodes: CategoryNode[], prefix: string[]) => {
      for (const node of nodes) {
        const path = [...prefix, node.name];
        out.push({ id: node.id, label: path.join(" / ") });
        walk(node.children, path);
      }
    };
    walk(tree, []);
    categoryOptions.value = out;
  } catch (e) {
    console.warn("加载分类失败", e);
  }
}

async function onFilter() {
  if (!topic.value.trim()) return;
  if (source.value === "favorites" && !selectedMediaId.value) return;
  filtering.value = true;
  results.value = [];
  selectedBvids.value = [];
  try {
    if (source.value === "favorites") {
      results.value = await invoke<AiFavCandidate[]>("ai_filter_favorites", {
        mediaId: selectedMediaId.value,
        topic: topic.value.trim(),
        limit: 120,
        minScore: 0.4,
      });
    } else {
      results.value = await invoke<AiFavCandidate[]>("ai_filter_watch_later", {
        topic: topic.value.trim(),
        limit: 120,
        minScore: 0.4,
      });
    }
    selectedBvids.value = results.value
      .filter((item) => !item.in_library)
      .map((item) => item.bvid);
  } catch (e) {
    notify.error(t("AI 筛选失败：{e}", { e: String(e) }));
  } finally {
    filtering.value = false;
  }
}

function selectAll() {
  selectedBvids.value = results.value.map((item) => item.bvid);
}

function toggle(bvid: string) {
  const index = selectedBvids.value.indexOf(bvid);
  if (index >= 0) selectedBvids.value.splice(index, 1);
  else selectedBvids.value.push(bvid);
}

async function onImport() {
  const selected = results.value.filter((item) =>
    selectedBvids.value.includes(item.bvid),
  );
  if (selected.length === 0) return;
  importing.value = true;
  let success = 0;
  try {
    for (const item of selected) {
      const data: VideoMeta = {
        platform: "bilibili",
        platform_id: item.bvid,
        title: item.title,
        author: item.author,
        cover_url: item.cover_url,
        description: item.description,
        duration: item.duration,
        original_url: `https://www.bilibili.com/video/${item.bvid}`,
        aid: null,
        cid: null,
      };
      await invoke("save_video", {
        data,
        categoryId: targetCategory.value,
        originalUrl: data.original_url,
        tags: [],
        preserveTags: true,
      });
      success++;
    }
    notify.success(t("导入完成：{n} 个视频", { n: success }));
    const imported = new Set(selected.map((item) => item.bvid));
    results.value = results.value.map((item) =>
      imported.has(item.bvid) ? { ...item, in_library: true } : item,
    );
    selectedBvids.value = [];
  } catch (e) {
    notify.error(t("导入失败：{e}", { e: String(e) }));
  } finally {
    importing.value = false;
  }
}

watch(source, (value) => {
  results.value = [];
  selectedBvids.value = [];
  if (value === "favorites") loadFolders();
});

onMounted(async () => {
  await Promise.all([loadFolders(), loadCategories()]);
});
</script>

<style scoped>
.fav-results {
  max-height: calc(100vh - 430px);
  min-height: 160px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 8px;
}
.fav-cover {
  width: 96px;
  height: 56px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
  background: rgba(0, 0, 0, 0.06);
}
.min-w-0 {
  min-width: 0;
}
</style>
