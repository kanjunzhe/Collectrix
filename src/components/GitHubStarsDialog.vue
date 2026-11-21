<template>
  <v-dialog v-model="visible" max-width="720" persistent scrollable>
    <v-card>
      <v-card-title class="d-flex align-center">
        <v-icon class="mr-2">mdi-github</v-icon>
        {{ t("导入 GitHub Star") }}
      </v-card-title>
      <v-card-text>
        <div class="text-caption text-medium-emphasis mb-3">
          {{ t("读取 GitHub 收藏列表后，勾选需要导入的仓库。") }}
        </div>

        <div class="d-flex flex-wrap align-center mb-3" style="gap: 8px">
          <v-text-field
            v-model="username"
            :label="t('GitHub 用户名')"
            placeholder="octocat"
            density="compact"
            variant="outlined"
            hide-details
            style="min-width: 220px; flex: 1"
          />
          <v-text-field
            v-model="token"
            :label="t('Token（可选）')"
            type="password"
            density="compact"
            variant="outlined"
            hide-details
            style="min-width: 220px; flex: 1"
          />
          <v-btn
            color="primary"
            variant="tonal"
            prepend-icon="mdi-refresh"
            :loading="loading"
            @click="loadStars"
          >
            {{ t("读取 Star 列表") }}
          </v-btn>
        </div>

        <v-text-field
          v-model="search"
          :label="t('搜索仓库')"
          prepend-inner-icon="mdi-magnify"
          density="compact"
          variant="outlined"
          hide-details
          clearable
          class="mb-3"
        />

        <v-alert
          v-if="error"
          type="warning"
          variant="tonal"
          density="compact"
          class="mb-3"
        >
          {{ error }}
          <div v-if="error.includes('error sending request')" class="text-caption mt-1">
            {{ t("这通常是网络、代理或系统证书问题。请检查网络后重试。") }}
          </div>
        </v-alert>

        <div class="d-flex align-center mb-2">
          <span class="text-caption text-medium-emphasis">
            {{ t("已选择 {selected}/{total} 个仓库", { selected: selectedIds.length, total: repos.length }) }}
          </span>
          <v-spacer />
          <v-btn size="small" variant="text" @click="selectAll">
            {{ t("全选") }}
          </v-btn>
          <v-btn size="small" variant="text" @click="selectedIds = []">
            {{ t("取消全选") }}
          </v-btn>
        </div>

        <v-progress-linear v-if="loading" indeterminate color="primary" class="mb-2" />

        <v-list v-if="filteredRepos.length" density="compact" class="star-list">
          <v-list-item
            v-for="repo in filteredRepos"
            :key="repo.platform_id"
            @click="toggle(repo.platform_id)"
          >
            <template #prepend>
              <v-checkbox-btn :model-value="selectedIds.includes(repo.platform_id)" />
            </template>
            <v-list-item-title class="text-body-2">
              {{ repo.title }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ repo.description || repo.author || t("（无描述）") }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>

        <v-alert
          v-else-if="!loading && loaded"
          type="info"
          variant="tonal"
          density="compact"
        >
          {{ search ? t("没有匹配的仓库") : t("没有读取到 Star 仓库") }}
        </v-alert>

        <div class="text-caption text-medium-emphasis mt-3">
          {{
            t(
              "只填用户名可以读取公开 Star；填写 Token 可以读取当前账号的 Star，并提高 API 限额。",
            )
          }}
        </div>
      </v-card-text>

      <v-card-actions>
        <v-spacer />
        <v-btn :disabled="importing" @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          :loading="importing"
          :disabled="selectedIds.length === 0"
          @click="onImport"
        >
          {{ t("导入 {n} 个仓库", { n: selectedIds.length }) }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, VideoMeta, VideoRow } from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  imported: [count: number];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const username = ref("");
const token = ref("");
const repos = ref<VideoMeta[]>([]);
const selectedIds = ref<string[]>([]);
const search = ref("");
const loading = ref(false);
const importing = ref(false);
const loaded = ref(false);
const error = ref("");

const filteredRepos = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  if (!keyword) return repos.value;
  return repos.value.filter((repo) =>
    [repo.title, repo.author ?? "", repo.description ?? ""]
      .join(" ")
      .toLowerCase()
      .includes(keyword),
  );
});

async function loadSettings() {
  try {
    const settings = await invoke<AppSettings>("get_settings");
    username.value = settings.github_username ?? "";
    token.value = settings.github_token ?? "";
  } catch {
    username.value = "";
    token.value = "";
  }
}

async function persistSettings() {
  try {
    const current = await invoke<AppSettings>("get_settings");
    current.github_username = username.value.trim();
    current.github_token = token.value.trim();
    await invoke("set_settings", { settings: current });
  } catch {
    // 保存失败不影响本次读取。
  }
}

async function loadStars() {
  error.value = "";
  if (!username.value.trim() && !token.value.trim()) {
    error.value = t("请先填写 GitHub 用户名或 Token");
    return;
  }
  loading.value = true;
  loaded.value = false;
  try {
    await persistSettings();
    const allRepos = await invoke<VideoMeta[]>("list_github_stars", {
      username: username.value.trim(),
      token: token.value.trim(),
    });
    const existing = await invoke<VideoRow[]>("list_web_videos", {
      scope: "github",
    });
    const existingIds = new Set(existing.map((item) => item.platform_id));
    repos.value = allRepos.filter((repo) => !existingIds.has(repo.platform_id));
    selectedIds.value = repos.value.map((repo) => repo.platform_id);
    loaded.value = true;
  } catch (e) {
    repos.value = [];
    selectedIds.value = [];
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function selectAll() {
  selectedIds.value = filteredRepos.value.map((repo) => repo.platform_id);
}

function toggle(id: string) {
  const index = selectedIds.value.indexOf(id);
  if (index >= 0) selectedIds.value.splice(index, 1);
  else selectedIds.value.push(id);
}

async function onImport() {
  const selected = repos.value.filter((repo) =>
    selectedIds.value.includes(repo.platform_id),
  );
  if (selected.length === 0) return;

  importing.value = true;
  let success = 0;
  try {
    for (const repo of selected) {
      await invoke("save_video", {
        data: repo,
        categoryId: null,
        originalUrl: repo.original_url,
        tags: [],
        preserveTags: true,
      });
      success++;
    }
    notify.success(t("导入完成：{n} 个仓库", { n: success }));
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
    repos.value = [];
    selectedIds.value = [];
    search.value = "";
    error.value = "";
    loaded.value = false;
    void loadSettings().then(() => {
      if (username.value.trim() || token.value.trim()) {
        void loadStars();
      }
    });
  },
);
</script>

<style scoped>
.star-list {
  max-height: 340px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 8px;
}
</style>
