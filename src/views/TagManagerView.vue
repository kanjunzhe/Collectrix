<template>
  <div>
    <div class="d-flex align-center mb-4">
      <div class="text-h5 font-weight-bold">{{ t("标签管理") }}</div>
      <v-spacer />
      <v-btn
        variant="tonal"
        prepend-icon="mdi-auto-fix"
        :loading="coloringAll"
        @click="smartColorAll"
      >
        {{ t("智能配色") }}
      </v-btn>
      <v-btn color="primary" prepend-icon="mdi-plus" @click="showCreate = true">
        {{ t("新建标签") }}
      </v-btn>
    </div>

    <v-sheet border rounded>
      <v-list>
        <v-list-item
          v-for="tag in tags"
          :key="tag.id"
          @contextmenu.prevent="openTagMenu($event, tag)"
        >
          <div class="d-flex align-center gap-2 w-100">
            <v-chip :color="tag.color" variant="flat" size="small">
              {{ tag.name }}
            </v-chip>
            <v-text-field
              v-model="tag.name"
              density="compact"
              variant="outlined"
              hide-details
              class="flex-grow-1"
              style="max-width: 300px"
            />
            <input
              v-model="tag.color"
              type="color"
              class="color-input"
              :title="t('选择颜色')"
            />
            <v-spacer />
            <v-btn
              size="small"
              color="primary"
              variant="tonal"
              prepend-icon="mdi-content-save"
              @click="onSave(tag)"
            >
              {{ t("保存") }}
            </v-btn>
            <v-btn
              size="small"
              color="error"
              variant="text"
              icon="mdi-delete"
              @click="onDelete(tag)"
            />
          </div>
        </v-list-item>
        <v-list-item v-if="tags.length === 0">
          <div class="text-medium-emphasis text-center py-4">
            {{ t("还没有标签") }}
          </div>
        </v-list-item>
      </v-list>
    </v-sheet>

    <v-dialog v-model="showCreate" max-width="420">
      <v-card>
        <v-card-title>{{ t("新建标签") }}</v-card-title>
        <v-card-text>
          <v-text-field v-model="newName" :label="t('标签名')" />
          <div class="d-flex align-center gap-2 mt-3">
            <span class="text-body-2">{{ t("颜色") }}</span>
            <input v-model="newColor" type="color" class="color-input" />
            <v-chip :color="newColor" variant="flat">{{
              newName || t("预览")
            }}</v-chip>
          </div>
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn @click="showCreate = false">{{ t("取消") }}</v-btn>
          <v-btn color="primary" @click="onCreate">{{ t("创建") }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <ContextMenu
      v-model="tagMenu.visible"
      :x="tagMenu.x"
      :y="tagMenu.y"
      :items="tagMenuItems"
      @select="onTagMenuSelect"
    />

    <MergeTagDialog
      v-model="showMerge"
      :source-tag="mergeSource"
      :tags="tags"
      @merged="onTagsChanged"
    />

    <TagEditDialog
      v-model="showEdit"
      :tag="editTarget"
      :mode="editMode"
      @saved="onTagsChanged"
    />

    <DeleteConfirmDialog
      v-model="showDelete"
      :title="t('删除标签')"
      :entity-name="deleteTarget?.name ?? ''"
      :videos="deleteVideos"
      :warning="
        t('标签「{name}」会从所有视频上移除，但视频本身不会删除。', {
          name: deleteTarget?.name ?? '',
        })
      "
      :confirm-text="t('删除标签')"
      @confirm="confirmDelete"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ContextMenuItem, TagInfo, VideoRow } from "../types";
import { notify } from "../composables/useUI";
import ContextMenu from "../components/ContextMenu.vue";
import { smartTagColor } from "../utils/tagColor";
import MergeTagDialog from "../components/MergeTagDialog.vue";
import DeleteConfirmDialog from "../components/DeleteConfirmDialog.vue";
import TagEditDialog from "../components/TagEditDialog.vue";
import { t } from "../i18n";

const emit = defineEmits<{ "tags-changed": [] }>();

const tags = ref<TagInfo[]>([]);
const showCreate = ref(false);
const newName = ref("");
const newColor = ref("#1976D2");

async function load() {
  tags.value = await invoke<TagInfo[]>("list_tags");
}

async function onCreate() {
  if (!newName.value.trim()) return;
  await invoke("create_tag", {
    name: newName.value.trim(),
    color: newColor.value,
  });
  showCreate.value = false;
  newName.value = "";
  newColor.value = "#1976D2";
  await load();
  emit("tags-changed");
}

async function onSave(tag: TagInfo) {
  if (!tag.name.trim()) return;
  await invoke("update_tag", {
    id: tag.id,
    name: tag.name.trim(),
    color: tag.color,
  });
  await load();
  emit("tags-changed");
}

const tagMenu = reactive({
  visible: false,
  x: 0,
  y: 0,
  tag: null as TagInfo | null,
});

const tagMenuItems = computed<ContextMenuItem[]>(() => {
  if (!tagMenu.tag) return [];
  return [
    { key: "edit", title: t("编辑标签"), icon: "mdi-pencil" },
    { key: "color", title: t("仅修改颜色"), icon: "mdi-palette" },
    { key: "smartColor", title: t("智能配色"), icon: "mdi-auto-fix" },
    { key: "merge", title: t("合并标签"), icon: "mdi-merge" },
    {
      key: "delete",
      title: t("删除标签"),
      icon: "mdi-delete",
      color: "error",
    },
  ];
});

function openTagMenu(event: MouseEvent, tag: TagInfo) {
  event.preventDefault();
  event.stopPropagation();
  tagMenu.tag = tag;
  tagMenu.x = event.clientX;
  tagMenu.y = event.clientY;
  tagMenu.visible = true;
}

async function onTagMenuSelect(key: string) {
  const tag = tagMenu.tag;
  tagMenu.visible = false;
  if (!tag) return;
  if (key === "edit") {
    openEdit(tag, "edit");
  } else if (key === "color") {
    openEdit(tag, "color");
  } else if (key === "smartColor") {
    await applySmartColor(tag);
  } else if (key === "merge") {
    openMerge(tag);
  } else if (key === "delete") {
    await openDelete(tag);
  }
}

const showEdit = ref(false);
const editTarget = ref<TagInfo | null>(null);
const editMode = ref<"edit" | "color">("edit");
const coloringAll = ref(false);

async function applySmartColor(tag: TagInfo) {
  try {
    const color = smartTagColor(tag.name);
    await invoke("update_tag", { id: tag.id, name: tag.name, color });
    await load();
    emit("tags-changed");
    notify.success(t("已应用智能配色"));
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  }
}

async function smartColorAll() {
  coloringAll.value = true;
  try {
    for (const tag of [...tags.value]) {
      const color = smartTagColor(tag.name);
      if (color !== tag.color) {
        await invoke("update_tag", { id: tag.id, name: tag.name, color });
      }
    }
    await load();
    emit("tags-changed");
    notify.success(t("已应用智能配色"));
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  } finally {
    coloringAll.value = false;
  }
}

function openEdit(tag: TagInfo, mode: "edit" | "color") {
  editTarget.value = tag;
  editMode.value = mode;
  showEdit.value = true;
}

const showMerge = ref(false);
const mergeSource = ref<TagInfo | null>(null);

function openMerge(tag: TagInfo) {
  mergeSource.value = tag;
  showMerge.value = true;
}

async function onTagsChanged() {
  await load();
  emit("tags-changed");
}

const showDelete = ref(false);
const deleteTarget = ref<TagInfo | null>(null);
const deleteVideos = ref<VideoRow[]>([]);

async function onDelete(tag: TagInfo) {
  await openDelete(tag);
}

async function openDelete(tag: TagInfo) {
  try {
    const videos = await invoke<VideoRow[]>("list_videos_by_tag", {
      tagId: tag.id,
      limit: null,
      offset: null,
    });
    deleteTarget.value = tag;
    deleteVideos.value = videos;
    showDelete.value = true;
  } catch (e) {
    notify.error(t("加载标签视频失败：{e}", { e: String(e) }));
  }
}

async function confirmDelete() {
  const tag = deleteTarget.value;
  if (!tag) return;
  try {
    await invoke("delete_tag", { id: tag.id });
    notify.success(t("标签已删除"));
    showDelete.value = false;
    await load();
    emit("tags-changed");
  } catch (e) {
    notify.error(t("删除失败：{e}", { e: String(e) }));
  }
}

onMounted(load);
</script>

<style scoped>
.w-100 {
  width: 100%;
}
.gap-2 {
  gap: 8px;
}
.color-input {
  width: 42px;
  height: 32px;
  border: 1px solid rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  padding: 2px;
  cursor: pointer;
  background: transparent;
}
</style>
