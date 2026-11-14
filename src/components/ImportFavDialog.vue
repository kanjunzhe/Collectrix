<template>
  <v-dialog v-model="visible" max-width="580" persistent>
    <v-card>
      <v-card-title>{{ t("从 B 站收藏夹导入") }}</v-card-title>
      <v-card-text>
        <v-alert v-if="!hasCookie" type="warning" variant="tonal" class="mb-3">
          {{ t("还没有配置 B 站 Cookie，无法读取收藏夹。") }}
          <span class="text-caption">{{
            t("请在「设置 → Bilibili 登录」中填写。")
          }}</span>
        </v-alert>

        <div class="d-flex align-center mb-3" style="gap: 8px">
          <v-btn
            variant="tonal"
            prepend-icon="mdi-refresh"
            :loading="loadingFolders"
            :disabled="!hasCookie"
            @click="loadFolders"
          >
            {{ t("加载我的收藏夹") }}
          </v-btn>
          <span v-if="foldersLoaded" class="text-caption text-medium-emphasis">
            {{ t("共 {n} 个收藏夹", { n: folders.length }) }}
          </span>
        </div>

        <v-select
          v-model="selectedMediaId"
          :items="folderItems"
          item-title="title"
          item-value="value"
          item-subtitle="subtitle"
          :label="t('选择收藏夹')"
          :disabled="!foldersLoaded"
          :loading="loadingFolders"
        />

        <v-select
          v-model="targetCategory"
          :items="categories"
          item-title="label"
          item-value="id"
          :label="t('导入到分类（可选，留空则不归类）')"
          clearable
        />

        <v-alert type="info" variant="tonal" density="compact" class="mt-2">
          {{
            t(
              "导入会保留标题、作者、封面、简介、时长，但不会自动抓取 aid/cid。如需评论和下载功能，导入后在视频卡片上点「补全信息」即可。",
            )
          }}
        </v-alert>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          prepend-icon="mdi-download"
          :loading="importing"
          :disabled="!selectedMediaId"
          @click="onImport"
        >
          {{ t("开始导入") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

interface FavFolder {
  id: number;
  fid: number;
  title: string;
  media_count: number;
}

const props = defineProps<{
  modelValue: boolean;
  categories: { id: number; label: string }[];
}>();
const emit = defineEmits<{
  "update:modelValue": [v: boolean];
  imported: [count: number];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (v) => emit("update:modelValue", v),
});

const hasCookie = ref(false);
const loadingFolders = ref(false);
const foldersLoaded = ref(false);
const folders = ref<FavFolder[]>([]);
const selectedMediaId = ref<number | null>(null);
const targetCategory = ref<number | null>(null);
const importing = ref(false);

/** 预构造 select 的 items，避免在 #item slot 里访问 .raw 引发类型问题 */
const folderItems = computed(() =>
  folders.value.map((f) => ({
    title: f.title,
    subtitle: t("{n} 个视频", { n: f.media_count }),
    value: f.id,
  })),
);

watch(
  visible,
  async (open) => {
    if (!open) return;
    selectedMediaId.value = null;
    foldersLoaded.value = false;
    folders.value = [];
    try {
      const s = await invoke<{ bilibili_cookie: string }>("get_settings");
      hasCookie.value =
        !!s.bilibili_cookie && s.bilibili_cookie.includes("SESSDATA");
    } catch {
      hasCookie.value = false;
    }
  },
  { immediate: true },
);

async function loadFolders() {
  loadingFolders.value = true;
  try {
    folders.value = await invoke<FavFolder[]>("list_fav_folders");
    foldersLoaded.value = true;
    if (folders.value.length === 0) {
      notify.warning(t("该账号没有收藏夹，或 Cookie 已失效"));
    }
  } catch (e) {
    notify.error(t("操作失败：{e}", { e: String(e) }));
  } finally {
    loadingFolders.value = false;
  }
}

async function onImport() {
  if (!selectedMediaId.value) return;
  importing.value = true;
  try {
    const count = await invoke<number>("import_fav_folder", {
      mediaId: selectedMediaId.value,
      categoryId: targetCategory.value,
    });
    emit("imported", count);
    notify.success(t("导入完成，共处理 {n} 个视频", { n: count }));
    close();
  } catch (e) {
    notify.error(t("操作失败：{e}", { e: String(e) }));
  } finally {
    importing.value = false;
  }
}

function close() {
  visible.value = false;
}
</script>
