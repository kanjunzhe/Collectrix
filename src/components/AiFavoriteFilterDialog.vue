<template>
  <v-dialog v-model="visible" max-width="820" persistent scrollable>
    <v-card>
      <v-card-title>{{ t("收藏夹 AI 筛选") }}</v-card-title>
      <v-card-text>
        <v-alert
          v-if="!hasCookie"
          type="warning"
          variant="tonal"
          density="compact"
          class="mb-3"
        >
          {{ t("请先在设置中登录 B 站，才能读取收藏夹") }}
        </v-alert>

        <div class="d-flex flex-wrap" style="gap: 8px">
          <v-select
            v-model="selectedMediaId"
            :items="folderItems"
            item-title="title"
            item-value="value"
            :label="t('选择收藏夹')"
            density="compact"
            variant="outlined"
            :loading="loadingFolders"
            :disabled="!hasCookie || filtering"
            style="min-width: 260px; flex: 1"
          />
          <v-text-field
            v-model="topic"
            :label="t('筛选题材')"
            :placeholder="t('例如：Rust、AI 编程、摄影教程')"
            density="compact"
            variant="outlined"
            hide-details
            :disabled="filtering"
            style="min-width: 220px; flex: 1"
            @keydown.enter.prevent="onFilter"
          />
          <v-btn
            color="primary"
            variant="tonal"
            prepend-icon="mdi-robot"
            :loading="filtering"
            :disabled="!selectedMediaId || !topic.trim()"
            @click="onFilter"
          >
            {{ t("AI 筛选") }}
          </v-btn>
        </div>

        <div class="d-flex align-center mt-4 mb-2">
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
                    <v-chip
                      v-if="item.in_library"
                      size="x-small"
                      color="success"
                      variant="tonal"
                    >
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

        <v-select
          v-model="targetCategory"
          :items="categoryOptions"
          item-title="label"
          item-value="id"
          :label="t('导入到分类（可选）')"
          density="compact"
          variant="outlined"
          clearable
          class="mt-3"
        />
      </v-card-text>
      <v-card-actions>
        <span class="text-caption text-medium-emphasis ml-2">
          {{ t("已选择 {n} 个", { n: selectedBvids.length }) }}
        </span>
        <v-spacer />
        <v-btn :disabled="importing" @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          :loading="importing"
          :disabled="selectedBvids.length === 0"
          variant="flat"
          @click="onImport"
        >
          {{ t("导入选中 {n} 个", { n: selectedBvids.length }) }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  AiFavCandidate,
  CategoryNode,
  VideoMeta,
} from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

interface FavFolder {
  id: number;
  title: string;
  media_count: number;
}

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  imported: [count: number];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

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
  if (!selectedMediaId.value || !topic.value.trim()) return;
  filtering.value = true;
  results.value = [];
  selectedBvids.value = [];
  try {
    results.value = await invoke<AiFavCandidate[]>("ai_filter_favorites", {
      mediaId: selectedMediaId.value,
      topic: topic.value.trim(),
      limit: 120,
      minScore: 0.4,
    });
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
    emit("imported", success);
    close();
  } catch (e) {
    notify.error(t("导入失败：{e}", { e: String(e) }));
  } finally {
    importing.value = false;
  }
}

function close() {
  visible.value = false;
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    results.value = [];
    selectedBvids.value = [];
    topic.value = "";
    targetCategory.value = null;
    selectedMediaId.value = null;
    loadFolders();
    loadCategories();
  },
);
</script>

<style scoped>
.fav-results {
  max-height: 360px;
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
