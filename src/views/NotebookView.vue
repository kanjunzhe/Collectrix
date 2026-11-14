<template>
  <v-row>
    <v-col cols="auto" style="width: 300px; align-self: flex-start">
      <v-sheet class="pa-3 rounded-xl" border>
        <div class="d-flex align-center mb-2">
          <span class="text-subtitle-2 font-weight-medium">{{ t("记事本") }}</span>
          <v-spacer />
          <v-btn
            size="x-small"
            variant="tonal"
            prepend-icon="mdi-plus"
            :loading="creating"
            @click="createNotebook"
          >
            {{ t("新建") }}
          </v-btn>
        </div>
        <v-list nav density="compact">
          <v-list-item
            v-for="note in notebooks"
            :key="note.id"
            :active="selected?.id === note.id"
            @click="selectNotebook(note)"
          >
            <template #prepend>
              <v-icon>mdi-notebook-outline</v-icon>
            </template>
            <v-list-item-title>{{ note.title }}</v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ t("{n} 个视频", { n: note.video_count }) }}
            </v-list-item-subtitle>
          </v-list-item>
          <v-list-item v-if="notebooks.length === 0">
            <div class="text-caption text-medium-emphasis pa-3">
              {{ t("还没有记事本") }}
            </div>
          </v-list-item>
        </v-list>
      </v-sheet>
    </v-col>

    <v-col>
      <template v-if="selected">
        <div class="d-flex align-center mb-3" style="gap: 8px">
          <v-text-field
            v-model="title"
            :label="t('标题')"
            density="compact"
            variant="outlined"
            hide-details
          />
          <v-btn
            color="primary"
            prepend-icon="mdi-content-save"
            :loading="saving"
            @click="saveNotebook"
          >
            {{ t("保存记事本") }}
          </v-btn>
          <v-btn
            variant="text"
            color="error"
            icon="mdi-delete"
            :title="t('删除')"
            @click="deleteNotebook"
          />
        </div>

        <v-textarea
          v-model="content"
          :label="t('内容')"
          rows="10"
          auto-grow
          density="compact"
          variant="outlined"
        />

        <div class="d-flex align-center mt-5 mb-2">
          <span class="text-subtitle-2 font-weight-medium">
            {{ t("引用视频") }}
          </span>
          <v-chip size="x-small" variant="text" class="ml-2">
            {{ linkedVideos.length }}
          </v-chip>
          <v-spacer />
          <v-btn
            size="small"
            variant="tonal"
            prepend-icon="mdi-video-plus"
            @click="openVideoPicker"
          >
            {{ t("添加视频") }}
          </v-btn>
        </div>

        <v-list v-if="linkedVideos.length" density="compact" class="linked-videos">
          <v-list-item
            v-for="video in linkedVideos"
            :key="video.id"
            @click="$emit('play', video)"
          >
            <template #prepend>
              <img
                v-if="videoCoverSrc(video)"
                :src="videoCoverSrc(video)"
                class="linked-cover"
                referrerpolicy="no-referrer"
                alt="cover"
                loading="lazy"
              />
              <v-icon v-else>mdi-video-off-outline</v-icon>
            </template>
            <v-list-item-title class="text-body-2">
              {{ video.title }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ video.author || t("未知作者") }}
            </v-list-item-subtitle>
            <template #append>
              <v-btn
                icon
                size="small"
                variant="text"
                color="error"
                :title="t('从记事本移除')"
                @click.stop="removeVideo(video)"
              >
                <v-icon>mdi-close</v-icon>
              </v-btn>
            </template>
          </v-list-item>
        </v-list>
        <v-alert v-else type="info" variant="tonal" density="compact">
          {{ t("还没有引用视频，点击「添加视频」从视频库选择") }}
        </v-alert>
      </template>
      <v-alert v-else type="info" variant="tonal">
        {{ t("请选择或新建一个记事本") }}
      </v-alert>
    </v-col>
  </v-row>

  <v-dialog v-model="showVideoPicker" max-width="720" scrollable>
    <v-card>
      <v-card-title>{{ t("添加视频到记事本") }}</v-card-title>
      <v-card-text>
        <v-text-field
          v-model="videoSearch"
          prepend-inner-icon="mdi-magnify"
          :label="t('搜索视频')"
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />
        <v-list density="compact" class="video-picker">
          <v-list-item
            v-for="video in filteredVideos"
            :key="video.id"
            @click="toggleVideo(video.id)"
          >
            <template #prepend>
              <v-checkbox-btn
                :model-value="pickedIds.includes(video.id)"
                @click.stop
                @update:model-value="toggleVideo(video.id)"
              />
            </template>
            <v-list-item-title class="text-body-2">
              {{ video.title }}
            </v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ video.author || t("未知作者") }}
            </v-list-item-subtitle>
          </v-list-item>
          <v-list-item v-if="filteredVideos.length === 0">
            <div class="text-caption text-medium-emphasis pa-3">
              {{ t("没有找到视频") }}
            </div>
          </v-list-item>
        </v-list>
      </v-card-text>
      <v-card-actions>
        <span class="text-caption text-medium-emphasis ml-2">
          {{ t("已选择 {n} 个", { n: pickedIds.length }) }}
        </span>
        <v-spacer />
        <v-btn @click="showVideoPicker = false">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          variant="flat"
          :disabled="pickedIds.length === 0"
          :loading="addingVideos"
          @click="addPickedVideos"
        >
          {{ t("加入") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { NotebookDetail, NotebookInfo, VideoRow } from "../types";
import { notify, confirmDialog } from "../composables/useUI";
import { videoCoverSrc } from "../utils/media";
import { t } from "../i18n";

defineEmits<{ play: [video: VideoRow] }>();

const notebooks = ref<NotebookInfo[]>([]);
const selected = ref<NotebookInfo | null>(null);
const title = ref("");
const content = ref("");
const linkedVideos = ref<VideoRow[]>([]);
const saving = ref(false);
const creating = ref(false);
const showVideoPicker = ref(false);
const allVideos = ref<VideoRow[]>([]);
const videoSearch = ref("");
const pickedIds = ref<number[]>([]);
const addingVideos = ref(false);

const filteredVideos = computed(() => {
  const q = videoSearch.value.trim().toLowerCase();
  const linked = new Set(linkedVideos.value.map((video) => video.id));
  return allVideos.value.filter(
    (video) =>
      !linked.has(video.id) &&
      (!q ||
        video.title.toLowerCase().includes(q) ||
        (video.author ?? "").toLowerCase().includes(q)),
  );
});

async function loadNotebooks() {
  notebooks.value = await invoke<NotebookInfo[]>("list_notebooks");
  if (selected.value) {
    selected.value =
      notebooks.value.find((note) => note.id === selected.value!.id) ?? null;
  }
}

async function loadDetail(id: number) {
  const detail = await invoke<NotebookDetail>("get_notebook", { id });
  title.value = detail.title;
  content.value = detail.content;
  linkedVideos.value = detail.videos;
}

async function selectNotebook(note: NotebookInfo) {
  selected.value = note;
  await loadDetail(note.id);
}

async function createNotebook() {
  creating.value = true;
  try {
    const id = await invoke<number>("save_notebook", {
      id: null,
      title: t("新建记事本"),
      content: "",
    });
    await loadNotebooks();
    const note = notebooks.value.find((item) => item.id === id);
    if (note) await selectNotebook(note);
  } catch (e) {
    notify.error(t("创建记事本失败：{e}", { e: String(e) }));
  } finally {
    creating.value = false;
  }
}

async function saveNotebook() {
  if (!selected.value) return;
  if (!title.value.trim()) {
    notify.warning(t("记事本标题不能为空"));
    return;
  }
  saving.value = true;
  try {
    await invoke("save_notebook", {
      id: selected.value.id,
      title: title.value,
      content: content.value,
    });
    await loadNotebooks();
    notify.success(t("记事本已保存"));
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  } finally {
    saving.value = false;
  }
}

async function deleteNotebook() {
  if (!selected.value) return;
  const ok = await confirmDialog({
    title: t("删除记事本"),
    message: t("确定删除记事本「{name}」？视频本身不会被删除。", {
      name: selected.value.title,
    }),
    confirmColor: "error",
    confirmText: t("删除"),
  });
  if (!ok) return;
  try {
    await invoke("delete_notebook", { id: selected.value.id });
    selected.value = null;
    title.value = "";
    content.value = "";
    linkedVideos.value = [];
    await loadNotebooks();
    notify.success(t("记事本已删除"));
  } catch (e) {
    notify.error(t("删除失败：{e}", { e: String(e) }));
  }
}

async function openVideoPicker() {
  allVideos.value = await invoke<VideoRow[]>("list_videos", {
    categoryId: null,
    limit: 500,
    offset: 0,
  });
  videoSearch.value = "";
  pickedIds.value = [];
  showVideoPicker.value = true;
}

function toggleVideo(id: number) {
  const index = pickedIds.value.indexOf(id);
  if (index >= 0) pickedIds.value.splice(index, 1);
  else pickedIds.value.push(id);
}

async function addPickedVideos() {
  if (!selected.value) return;
  addingVideos.value = true;
  try {
    for (const id of pickedIds.value) {
      await invoke("add_video_to_notebook", {
        notebookId: selected.value.id,
        videoId: id,
      });
    }
    showVideoPicker.value = false;
    await loadDetail(selected.value.id);
    await loadNotebooks();
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  } finally {
    addingVideos.value = false;
  }
}

async function removeVideo(video: VideoRow) {
  if (!selected.value) return;
  try {
    await invoke("remove_video_from_notebook", {
      notebookId: selected.value.id,
      videoId: video.id,
    });
    await loadDetail(selected.value.id);
    await loadNotebooks();
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  }
}

onMounted(async () => {
  await loadNotebooks();
  if (notebooks.value[0]) await selectNotebook(notebooks.value[0]);
});
</script>

<style scoped>
.linked-videos,
.video-picker {
  max-height: 360px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 8px;
}
.linked-cover {
  width: 80px;
  height: 46px;
  border-radius: 5px;
  object-fit: cover;
  margin-right: 8px;
}
</style>
