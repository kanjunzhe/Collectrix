<template>
  <v-row>
    <v-col cols="auto" style="width: 320px">
      <v-sheet border class="pa-3 rounded">
        <div class="d-flex align-center mb-3">
          <div class="text-subtitle-1 font-weight-medium">{{ t("主题") }}</div>
          <v-spacer />
          <v-btn
            color="primary"
            size="small"
            variant="tonal"
            prepend-icon="mdi-plus"
            @click="openCreate"
          >
            {{ t("新建") }}
          </v-btn>
        </div>

        <v-list nav density="compact">
          <v-list-item
            v-for="topic in topics"
            :key="topic.id"
            :active="selectedTopic?.id === topic.id"
            @click="selectTopic(topic)"
          >
            <template #prepend>
              <v-icon :color="topic.color">mdi-bookmark</v-icon>
            </template>
            <v-list-item-title>{{ topic.name }}</v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ t("{n} 个视频", { n: topic.video_count }) }}
            </v-list-item-subtitle>
          </v-list-item>
          <v-list-item v-if="topics.length === 0">
            <div class="text-caption text-medium-emphasis text-center py-4">
              {{ t("还没有主题，点击「新建」创建第一个") }}
            </div>
          </v-list-item>
        </v-list>
      </v-sheet>
    </v-col>

    <v-col>
      <template v-if="selectedTopic">
        <div class="d-flex align-center mb-4">
          <v-icon :color="selectedTopic.color" size="28" class="mr-2"
            >mdi-bookmark</v-icon
          >
          <div>
            <div class="text-h6">{{ selectedTopic.name }}</div>
            <div class="text-caption text-medium-emphasis">
              {{ selectedTopic.description || t("无描述") }}
            </div>
          </div>
          <v-spacer />
          <v-btn
            variant="tonal"
            size="small"
            prepend-icon="mdi-pencil"
            @click="openEdit"
          >
            {{ t("编辑") }}
          </v-btn>
          <v-btn
            variant="tonal"
            size="small"
            color="primary"
            prepend-icon="mdi-plus"
            class="ml-2"
            @click="showAddVideos = true"
          >
            {{ t("添加视频") }}
          </v-btn>
          <v-btn
            variant="text"
            size="small"
            color="error"
            icon="mdi-delete"
            @click="onDelete"
          />
        </div>

        <div class="video-grid">
          <VideoCard
            v-for="v in topicVideos"
            :key="v.id"
            :video="v"
            @play="$emit('play', v)"
            @edit="$emit('edit', v)"
            @delete="onRemoveVideo"
            @contextmenu="openContextMenu"
          />
        </div>

        <v-alert
          v-if="topicVideos.length === 0"
          type="info"
          variant="tonal"
          class="mt-4"
        >
          {{ t("这个主题里还没有视频。点击「添加视频」从视频库中选择。") }}
        </v-alert>
      </template>

      <template v-else>
        <v-alert type="info" variant="tonal">
          {{ t("从左侧选择一个主题，或新建一个。") }}
        </v-alert>
      </template>
    </v-col>
  </v-row>

  <ContextMenu
    v-model="contextMenu.visible"
    :x="contextMenu.x"
    :y="contextMenu.y"
    :items="contextMenuItems"
    @select="onContextMenuSelect"
  />

  <v-dialog v-model="showTopicDialog" max-width="480">
    <v-card>
      <v-card-title>{{
        editingTopic ? t("编辑主题") : t("新建主题")
      }}</v-card-title>
      <v-card-text>
        <v-text-field v-model="form.name" :label="t('名称')" />
        <v-textarea
          v-model="form.description"
          :label="t('描述（可选）')"
          rows="2"
        />
        <div class="d-flex align-center mt-3" style="gap: 12px">
          <span class="text-body-2">{{ t("颜色") }}</span>
          <input v-model="form.color" type="color" class="color-input" />
          <v-chip :color="form.color" variant="flat">{{
            form.name || t("预览")
          }}</v-chip>
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="showTopicDialog = false">{{ t("取消") }}</v-btn>
        <v-btn color="primary" @click="onSave">{{ t("保存") }}</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="showAddVideos" max-width="760">
    <v-card>
      <v-card-title>{{
        t("选择要加入「{name}」的视频", { name: selectedTopic?.name ?? "" })
      }}</v-card-title>
      <v-card-text style="max-height: 480px; overflow-y: auto">
        <v-text-field
          v-model="search"
          prepend-inner-icon="mdi-magnify"
          :label="t('搜索标题')"
          density="compact"
          variant="outlined"
          hide-details
          class="mb-3"
        />
        <v-list dense>
          <v-list-item
            v-for="v in filteredVideos"
            :key="v.id"
            @click="toggleSelect(v.id)"
          >
            <template #prepend>
              <v-checkbox-btn :model-value="selectedIds.includes(v.id)" />
            </template>
            <v-list-item-title>{{ v.title }}</v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ v.author || t("未知作者") }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>
      </v-card-text>
      <v-card-actions>
        <span class="text-caption text-medium-emphasis ml-2">
          {{ t("已选 {n} 个", { n: selectedIds.length }) }}
        </span>
        <v-spacer />
        <v-btn @click="showAddVideos = false">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          :disabled="selectedIds.length === 0"
          @click="onAddVideos"
        >
          {{ t("加入") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { notify, confirmDialog } from "../composables/useUI";
import { useVideoContextMenu } from "../composables/useVideoContextMenu";
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import VideoCard from "../components/VideoCard.vue";
import ContextMenu from "../components/ContextMenu.vue";
import { t } from "../i18n";
import type { TopicInfo, VideoRow } from "../types";

const emit = defineEmits<{
  play: [video: VideoRow];
  edit: [video: VideoRow];
}>();

const topics = ref<TopicInfo[]>([]);
const selectedTopic = ref<TopicInfo | null>(null);
const topicVideos = ref<VideoRow[]>([]);

const showTopicDialog = ref(false);
const editingTopic = ref<TopicInfo | null>(null);
const form = ref({ name: "", description: "", color: "#1976D2" });

const showAddVideos = ref(false);
const allVideos = ref<VideoRow[]>([]);
const search = ref("");
const selectedIds = ref<number[]>([]);

const filteredVideos = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return allVideos.value;
  return allVideos.value.filter((v) => v.title.toLowerCase().includes(q));
});

async function loadTopics() {
  topics.value = await invoke<TopicInfo[]>("list_topics");
  if (selectedTopic.value) {
    const updated = topics.value.find((t) => t.id === selectedTopic.value!.id);
    selectedTopic.value = updated ?? null;
  }
}

async function loadTopicVideos() {
  if (!selectedTopic.value) {
    topicVideos.value = [];
    return;
  }
  topicVideos.value = await invoke<VideoRow[]>("list_videos_by_topic", {
    topicId: selectedTopic.value.id,
  });
}

function selectTopic(t: TopicInfo) {
  selectedTopic.value = t;
  loadTopicVideos();
}

function openCreate() {
  editingTopic.value = null;
  form.value = { name: "", description: "", color: "#1976D2" };
  showTopicDialog.value = true;
}

function openEdit() {
  if (!selectedTopic.value) return;
  editingTopic.value = selectedTopic.value;
  form.value = {
    name: selectedTopic.value.name,
    description: selectedTopic.value.description || "",
    color: selectedTopic.value.color,
  };
  showTopicDialog.value = true;
}

async function onSave() {
  if (!form.value.name.trim()) return;
  if (editingTopic.value) {
    await invoke("update_topic", {
      id: editingTopic.value.id,
      name: form.value.name.trim(),
      description: form.value.description || null,
      color: form.value.color,
    });
  } else {
    await invoke("create_topic", {
      name: form.value.name.trim(),
      description: form.value.description || null,
      color: form.value.color,
    });
  }
  showTopicDialog.value = false;
  await loadTopics();
  if (editingTopic.value && selectedTopic.value) {
    const updated = topics.value.find((t) => t.id === selectedTopic.value!.id);
    if (updated) selectedTopic.value = updated;
  }
}

async function onDelete() {
  if (!selectedTopic.value) return;
  const ok = await confirmDialog({
    title: t("删除主题"),
    message: t("删除主题「{name}」？视频本身不会被删除。", {
      name: selectedTopic.value.name,
    }),
    confirmColor: "error",
    confirmText: t("删除"),
  });
  if (!ok) return;
  try {
    await invoke("delete_topic", { id: selectedTopic.value.id });
    notify.success(t("主题已删除"));
    selectedTopic.value = null;
    topicVideos.value = [];
    await loadTopics();
  } catch (e) {
    notify.error(t("删除失败：{e}", { e: String(e) }));
  }
}

async function onRemoveVideo(id: number) {
  // 用 set_video_topics 把当前主题从该视频中移除
  const video = topicVideos.value.find((v) => v.id === id);
  if (!video || !selectedTopic.value) return;

  const currentTopicIds = (
    await invoke<TopicInfo[]>("list_topics_for_video", {
      videoId: id,
    })
  ).map((t) => t.id);
  const nextIds = currentTopicIds.filter(
    (tid) => tid !== selectedTopic.value!.id,
  );
  await invoke("set_video_topics", { videoId: id, topicIds: nextIds });
  await loadTopicVideos();
  await loadTopics();
}

const {
  contextMenu,
  contextMenuItems,
  openContextMenu,
  onContextMenuSelect,
} = useVideoContextMenu({
  onPlay: (video) => emit("play", video),
  onEdit: (video) => emit("edit", video),
  extraItems: () => [
    {
      key: "removeFromTopic",
      title: t("从主题移除"),
      icon: "mdi-bookmark-remove",
      color: "error",
    },
  ],
  onExtraAction: (key, video) =>
    key === "removeFromTopic" ? onRemoveVideo(video.id) : undefined,
});

async function openAddVideos() {
  allVideos.value = await invoke<VideoRow[]>("list_videos", {
    categoryId: null,
    limit: null,
    offset: null,
  });
  selectedIds.value = topicVideos.value.map((v) => v.id);
  search.value = "";
  showAddVideos.value = true;
}

function toggleSelect(id: number) {
  const i = selectedIds.value.indexOf(id);
  if (i >= 0) selectedIds.value.splice(i, 1);
  else selectedIds.value.push(id);
}

async function onAddVideos() {
  if (!selectedTopic.value) return;
  // 只处理新增，不做移除
  const existing = topicVideos.value.map((v) => v.id);
  const toAdd = selectedIds.value.filter((id) => !existing.includes(id));
  if (toAdd.length > 0) {
    await invoke("add_videos_to_topic", {
      topicId: selectedTopic.value.id,
      videoIds: toAdd,
    });
  }
  showAddVideos.value = false;
  await loadTopicVideos();
  await loadTopics();
}

watch(showAddVideos, (v) => {
  if (v) openAddVideos();
});

onMounted(loadTopics);
</script>

<style scoped>
.video-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: 16px;
}
.color-input {
  width: 42px;
  height: 32px;
  border: 1px solid rgba(128, 128, 128, 0.3);
  border-radius: 6px;
  padding: 2px;
  cursor: pointer;
  background: transparent;
}
</style>
