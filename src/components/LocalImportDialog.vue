<template>
  <v-dialog v-model="visible" max-width="900" persistent scrollable>
    <v-card>
      <v-card-title class="d-flex align-center">
        <v-icon class="mr-2">mdi-harddisk-plus</v-icon>
        {{ t("导入本地视频") }}
        <v-spacer />
        <v-chip v-if="candidates.length" size="small" variant="text">
          {{ t("已选择 {n} 个", { n: selectedPaths.length }) }}
        </v-chip>
      </v-card-title>

      <v-card-text>
        <div class="d-flex align-center" style="gap: 8px">
          <v-text-field
            :model-value="folder"
            :label="t('视频文件夹')"
            density="compact"
            variant="outlined"
            hide-details
            readonly
            :placeholder="t('选择一个包含本地视频的文件夹')"
          />
          <v-btn
            variant="tonal"
            prepend-icon="mdi-folder-open"
            :loading="scanning"
            @click="chooseFolder"
          >
            {{ t("选择文件夹") }}
          </v-btn>
          <v-btn
            variant="text"
            prepend-icon="mdi-refresh"
            :disabled="!folder || scanning"
            :loading="scanning"
            @click="scan"
          >
            {{ t("重新扫描") }}
          </v-btn>
        </div>

        <v-alert
          v-if="candidates.length === 0 && !scanning"
          type="info"
          variant="tonal"
          density="compact"
          class="mt-4"
        >
          {{ folder ? t("没有找到视频文件") : t("请先选择视频文件夹") }}
        </v-alert>

        <div v-if="candidates.length" class="candidate-list mt-4">
          <div
            v-for="candidate in candidates"
            :key="candidate.video_path"
            class="candidate-row"
          >
            <v-checkbox-btn
              :model-value="selectedPaths.includes(candidate.video_path)"
              @update:model-value="toggle(candidate)"
            />
            <div class="candidate-cover">
              <img
                v-if="coverSrc(candidate)"
                :src="coverSrc(candidate)"
                alt="cover"
                referrerpolicy="no-referrer"
              />
              <v-icon v-else size="26" opacity="0.35">mdi-video-outline</v-icon>
            </div>
            <div class="candidate-main">
              <v-text-field
                v-model="candidate.title"
                density="compact"
                variant="outlined"
                hide-details
                :disabled="!!candidate.matched"
              />
              <div class="candidate-path" :title="candidate.video_path">
                {{ candidate.folder }}
              </div>
              <div v-if="candidate.matched" class="matched-title">
                {{ t("匹配结果：{title}", { title: candidate.matched.title }) }}
                <span v-if="candidate.matched.author">
                  · {{ candidate.matched.author }}
                </span>
              </div>
              <div class="d-flex flex-wrap align-center mt-1" style="gap: 4px">
                <v-chip
                  v-if="candidate.already_imported"
                  size="x-small"
                  color="warning"
                  variant="tonal"
                >
                  {{ t("已在库中") }}
                </v-chip>
                <v-chip
                  v-if="candidate.cover_path"
                  size="x-small"
                  color="teal"
                  variant="tonal"
                >
                  <v-icon start size="11">mdi-image-outline</v-icon>
                  {{ t("封面") }}
                </v-chip>
                <v-chip
                  v-if="candidate.audio_path"
                  size="x-small"
                  color="blue"
                  variant="tonal"
                >
                  <v-icon start size="11">mdi-music-note</v-icon>
                  {{ t("音频") }}
                </v-chip>
                <v-chip
                  v-if="candidate.subtitle_paths.length"
                  size="x-small"
                  color="deep-purple"
                  variant="tonal"
                >
                  <v-icon start size="11">mdi-subtitles-outline</v-icon>
                  {{ t("字幕 {n}", { n: candidate.subtitle_paths.length }) }}
                </v-chip>
                <v-chip
                  v-if="candidate.transcript_path"
                  size="x-small"
                  color="indigo"
                  variant="tonal"
                >
                  <v-icon start size="11">mdi-text-box-outline</v-icon>
                  {{ t("文字稿") }}
                </v-chip>
                <v-chip
                  v-if="candidate.matched"
                  size="x-small"
                  color="success"
                  variant="flat"
                >
                  <v-icon start size="11">mdi-check-decagram</v-icon>
                  {{ t("已匹配 B 站") }}
                </v-chip>
                <v-chip
                  v-if="candidate.matched"
                  size="x-small"
                  variant="text"
                  closable
                  @click:close="candidate.matched = null"
                >
                  {{ candidate.matched.platform_id }}
                </v-chip>
              </div>
            </div>
            <div class="candidate-actions">
              <v-btn
                size="small"
                variant="tonal"
                prepend-icon="mdi-magnify"
                :disabled="scanning || !!candidate.matched"
                @click="openMatch(candidate)"
              >
                {{ t("B 站匹配") }}
              </v-btn>
            </div>
          </div>
        </div>

        <v-divider class="my-4" />
        <v-select
          v-model="targetCategory"
          :items="categories"
          item-title="label"
          item-value="id"
          :label="t('保存到分类（可选）')"
          density="compact"
          variant="outlined"
          clearable
        />
        <v-combobox
          v-model="tagInput"
          :label="t('标签（回车添加）')"
          multiple
          chips
          closable-chips
          density="compact"
          variant="outlined"
        />
      </v-card-text>

      <v-card-actions>
        <span class="text-caption text-medium-emphasis ml-2">
          {{ t("如果文件夹里只有一个视频，会自动关联同目录的封面、音频和字幕。") }}
        </span>
        <v-spacer />
        <v-btn :disabled="importing" @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          variant="flat"
          prepend-icon="mdi-import"
          :disabled="selectedPaths.length === 0"
          :loading="importing"
          @click="importSelected"
        >
          {{ t("导入选中视频") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- B 站搜索匹配 -->
  <v-dialog v-model="showMatch" max-width="820" scrollable>
    <v-card>
      <v-card-title class="d-flex align-center">
        {{ t("在 B 站搜索匹配") }}
        <v-spacer />
        <v-btn icon="mdi-close" variant="text" @click="showMatch = false" />
      </v-card-title>
      <v-card-text>
        <v-alert type="info" variant="tonal" density="compact" class="mb-3">
          {{ t("请选择与本地视频对应的 B 站结果。匹配后可自动补全标题、作者、封面、aid/cid 和字幕信息。") }}
        </v-alert>
        <div class="d-flex" style="gap: 8px">
          <v-text-field
            v-model="searchQuery"
            :label="t('搜索关键词')"
            density="compact"
            variant="outlined"
            hide-details
            @keydown.enter.prevent="runSearch"
          />
          <v-btn
            color="primary"
            variant="tonal"
            prepend-icon="mdi-magnify"
            :loading="searching"
            @click="runSearch"
          >
            {{ t("搜索") }}
          </v-btn>
        </div>

        <v-list v-if="searchResults.length" density="compact" class="search-results mt-3">
          <v-list-item
            v-for="item in searchResults"
            :key="item.bvid"
            @click="chooseMatch(item)"
          >
            <template #prepend>
              <img
                v-if="item.cover_url"
                :src="item.cover_url"
                class="search-cover"
                referrerpolicy="no-referrer"
                alt="cover"
                loading="lazy"
              />
              <v-icon v-else>mdi-video-off-outline</v-icon>
            </template>
            <v-list-item-title class="text-body-2">{{ item.title }}</v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ item.author || t("未知作者") }} · {{ item.bvid }}
            </v-list-item-subtitle>
            <template #append>
              <v-btn
                size="small"
                variant="text"
                color="primary"
                :loading="matchingBvid === item.bvid"
                @click.stop="chooseMatch(item)"
              >
                {{ t("使用此匹配") }}
              </v-btn>
            </template>
          </v-list-item>
        </v-list>
        <v-alert v-else-if="searched && !searching" type="info" variant="tonal" density="compact" class="mt-3">
          {{ t("没有搜索到结果") }}
        </v-alert>
      </v-card-text>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  BiliSearchItem,
  BiliVideoData,
  LocalVideoCandidate,
  VideoMeta,
} from "../types";
import { notify } from "../composables/useUI";
import { localFileSrc } from "../utils/media";
import { t } from "../i18n";

const props = defineProps<{
  modelValue: boolean;
  categories: { id: number; label: string }[];
  initialCategoryId: number | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  imported: [count: number];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const folder = ref("");
const scanning = ref(false);
const importing = ref(false);
const candidates = ref<LocalVideoCandidate[]>([]);
const selectedPaths = ref<string[]>([]);
const targetCategory = ref<number | null>(props.initialCategoryId);
const tagInput = ref<string[]>([]);

const showMatch = ref(false);
const matchTarget = ref<LocalVideoCandidate | null>(null);
const searchQuery = ref("");
const searching = ref(false);
const searched = ref(false);
const searchResults = ref<BiliSearchItem[]>([]);
const matchingBvid = ref("");

function coverSrc(candidate: LocalVideoCandidate): string {
  if (candidate.cover_path) return localFileSrc(candidate.cover_path);
  return candidate.matched?.cover_url ?? "";
}

function toggle(candidate: LocalVideoCandidate) {
  const index = selectedPaths.value.indexOf(candidate.video_path);
  if (index >= 0) selectedPaths.value.splice(index, 1);
  else selectedPaths.value.push(candidate.video_path);
}

async function chooseFolder() {
  try {
    const selected = await invoke<string | null>("pick_directory");
    if (!selected) return;
    folder.value = selected;
    await scan();
  } catch (e) {
    notify.error(t("选择文件夹失败：{e}", { e: String(e) }));
  }
}

async function scan() {
  if (!folder.value) return;
  scanning.value = true;
  try {
    candidates.value = await invoke<LocalVideoCandidate[]>(
      "scan_local_videos",
      { dir: folder.value },
    );
    selectedPaths.value = candidates.value
      .filter((candidate) => !candidate.already_imported)
      .map((candidate) => candidate.video_path);
    if (candidates.value.length === 0) {
      notify.info(t("没有找到视频文件"));
    }
  } catch (e) {
    notify.error(t("扫描本地视频失败：{e}", { e: String(e) }));
  } finally {
    scanning.value = false;
  }
}

function openMatch(candidate: LocalVideoCandidate) {
  matchTarget.value = candidate;
  searchQuery.value = candidate.title;
  searchResults.value = [];
  searched.value = false;
  showMatch.value = true;
  runSearch();
}

async function runSearch() {
  const keyword = searchQuery.value.trim();
  if (!keyword) return;
  searching.value = true;
  searched.value = false;
  try {
    searchResults.value = await invoke<BiliSearchItem[]>(
      "search_bili_videos",
      { keyword },
    );
    searched.value = true;
  } catch (e) {
    notify.error(t("B 站搜索失败：{e}", { e: String(e) }));
  } finally {
    searching.value = false;
  }
}

function normalizeCover(url: string | null | undefined): string | null {
  if (!url) return null;
  return url.startsWith("//") ? `https:${url}` : url;
}

async function chooseMatch(item: BiliSearchItem) {
  matchingBvid.value = item.bvid;
  try {
    const info = await invoke<BiliVideoData>("fetch_bili_info", {
      url: `https://www.bilibili.com/video/${item.bvid}`,
    });
    const matched: VideoMeta = {
      platform: "bilibili",
      platform_id: info.bvid,
      title: info.title,
      author: info.owner.name,
      cover_url: normalizeCover(info.pic),
      description: info.desc,
      duration: info.duration,
      original_url: `https://www.bilibili.com/video/${info.bvid}`,
      aid: info.aid,
      cid: info.cid,
    };
    if (matchTarget.value) {
      matchTarget.value.matched = matched;
      const index = selectedPaths.value.indexOf(matchTarget.value.video_path);
      if (index < 0) selectedPaths.value.push(matchTarget.value.video_path);
    }
    showMatch.value = false;
    notify.success(t("已匹配：{title}", { title: matched.title }));
  } catch (e) {
    notify.error(t("获取 B 站视频信息失败：{e}", { e: String(e) }));
  } finally {
    matchingBvid.value = "";
  }
}

async function importSelected() {
  const items = candidates.value
    .filter((candidate) => selectedPaths.value.includes(candidate.video_path))
    .map((candidate) => ({
      video_path: candidate.video_path,
      title: candidate.title,
      cover_path: candidate.cover_path,
      audio_path: candidate.audio_path,
      subtitle_paths: candidate.subtitle_paths,
      transcript_path: candidate.transcript_path,
      matched: candidate.matched ?? null,
    }));
  if (items.length === 0) return;
  importing.value = true;
  try {
    const count = await invoke<number>("import_local_videos", {
      items,
      categoryId: targetCategory.value,
      tags: tagInput.value,
    });
    notify.success(t("已导入 {n} 个本地视频", { n: count }));
    emit("imported", count);
    visible.value = false;
  } catch (e) {
    notify.error(t("导入本地视频失败：{e}", { e: String(e) }));
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
    targetCategory.value = props.initialCategoryId;
    tagInput.value = [];
    if (folder.value) scan();
  },
  { immediate: true },
);
</script>

<style scoped>
.candidate-list {
  max-height: 440px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 10px;
}
.candidate-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.12);
}
.candidate-row:last-child {
  border-bottom: none;
}
.candidate-cover {
  width: 96px;
  height: 54px;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(128, 128, 128, 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
}
.candidate-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.candidate-main {
  flex: 1 1 auto;
  min-width: 0;
}
.candidate-path {
  margin-top: 4px;
  font-size: 11px;
  color: rgba(128, 128, 128, 0.9);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.matched-title {
  margin-top: 3px;
  font-size: 12px;
  color: rgb(var(--v-theme-success));
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.candidate-actions {
  flex: 0 0 auto;
  padding-top: 2px;
}
.search-results {
  max-height: 440px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 8px;
}
.search-cover {
  width: 96px;
  height: 54px;
  object-fit: cover;
  border-radius: 6px;
  margin-right: 10px;
}
@media (max-width: 700px) {
  .candidate-cover {
    display: none;
  }
  .candidate-row {
    flex-wrap: wrap;
  }
  .candidate-actions {
    width: 100%;
    padding-left: 34px;
  }
}
</style>
