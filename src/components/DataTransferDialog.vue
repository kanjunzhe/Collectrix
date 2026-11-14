<template>
  <v-dialog v-model="visible" max-width="640" persistent>
    <v-card>
      <v-card-title>{{ t("导入 / 导出收藏") }}</v-card-title>
      <v-card-text>
        <div class="text-subtitle-2 font-weight-medium mb-2">{{ t("导出") }}</div>
        <v-checkbox
          v-model="includeAll"
          :label="t('导出全部收藏')"
          density="compact"
          hide-details
        />

        <template v-if="!includeAll">
          <v-select
            v-model="selectedCategoryIds"
            :items="categoryOptions"
            item-title="label"
            item-value="id"
            :label="t('选择分类（包含子分类）')"
            multiple
            chips
            closable-chips
            density="compact"
            variant="outlined"
            class="mt-2"
          />
          <v-select
            v-model="selectedTagIds"
            :items="tags"
            item-title="name"
            item-value="id"
            :label="t('选择标签')"
            multiple
            chips
            closable-chips
            density="compact"
            variant="outlined"
            class="mt-2"
          />
        </template>

        <v-btn
          color="primary"
          variant="tonal"
          prepend-icon="mdi-export"
          class="mt-3"
          :loading="exporting"
          @click="onExport"
        >
          {{ t("选择文件夹并导出") }}
        </v-btn>

        <v-divider class="my-4" />

        <div class="text-subtitle-2 font-weight-medium mb-2">{{ t("导入") }}</div>
        <div class="text-caption text-medium-emphasis mb-3">
          {{
            t(
              "导入会按分类路径合并分类，按名称合并标签和视频；不会覆盖本地视频文件。",
            )
          }}
        </div>
        <v-btn
          color="primary"
          variant="tonal"
          prepend-icon="mdi-import"
          :loading="importing"
          @click="onImport"
        >
          {{ t("选择 JSON 文件并导入") }}
        </v-btn>

        <v-alert
          v-if="lastSummary"
          type="success"
          variant="tonal"
          density="compact"
          class="mt-3"
        >
          {{
            t("导入完成：{videos} 个视频，{categories} 个分类，{tags} 个标签", {
              videos: lastSummary.videos,
              categories: lastSummary.categories,
              tags: lastSummary.tags,
            })
          }}
        </v-alert>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="close">{{ t("关闭") }}</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CategoryNode, TagInfo } from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

interface ImportSummary {
  videos: number;
  categories: number;
  tags: number;
}

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  imported: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const includeAll = ref(true);
const categoryOptions = ref<{ id: number; label: string }[]>([]);
const tags = ref<TagInfo[]>([]);
const selectedCategoryIds = ref<number[]>([]);
const selectedTagIds = ref<number[]>([]);
const exporting = ref(false);
const importing = ref(false);
const lastSummary = ref<ImportSummary | null>(null);

async function loadOptions() {
  try {
    const [tree, nextTags] = await Promise.all([
      invoke<CategoryNode[]>("get_category_tree"),
      invoke<TagInfo[]>("list_tags"),
    ]);
    const flat: { id: number; label: string }[] = [];
    const walk = (nodes: CategoryNode[], prefix: string[]) => {
      for (const node of nodes) {
        const path = [...prefix, node.name];
        flat.push({ id: node.id, label: path.join(" / ") });
        walk(node.children, path);
      }
    };
    walk(tree, []);
    categoryOptions.value = flat;
    tags.value = nextTags;
  } catch (e) {
    notify.error(t("加载分类和标签失败：{e}", { e: String(e) }));
  }
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    lastSummary.value = null;
    loadOptions();
  },
);

function close() {
  visible.value = false;
}

async function onExport() {
  if (
    !includeAll.value &&
    selectedCategoryIds.value.length === 0 &&
    selectedTagIds.value.length === 0
  ) {
    notify.warning(t("请选择要导出的分类或标签"));
    return;
  }
  const directory = await invoke<string | null>("pick_directory");
  if (!directory) return;
  exporting.value = true;
  try {
    const path = await invoke<string>("export_data", {
      directory,
      includeAll: includeAll.value,
      categoryIds: selectedCategoryIds.value,
      tagIds: selectedTagIds.value,
    });
    notify.success(t("已导出到 {path}", { path }));
  } catch (e) {
    notify.error(t("导出失败：{e}", { e: String(e) }));
  } finally {
    exporting.value = false;
  }
}

async function onImport() {
  const path = await invoke<string | null>("pick_import_file");
  if (!path) return;
  importing.value = true;
  try {
    const summary = await invoke<ImportSummary>("import_data", { path });
    lastSummary.value = summary;
    notify.success(
      t("导入完成：{videos} 个视频，{categories} 个分类，{tags} 个标签", {
        videos: summary.videos,
        categories: summary.categories,
        tags: summary.tags,
      }),
    );
    emit("imported");
  } catch (e) {
    notify.error(t("导入失败：{e}", { e: String(e) }));
  } finally {
    importing.value = false;
  }
}
</script>
