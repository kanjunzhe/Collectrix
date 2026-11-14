<template>
  <v-dialog v-model="visible" max-width="560" persistent>
    <v-card>
      <v-card-title>{{ t("添加到记事本") }}</v-card-title>
      <v-card-text>
        <v-list v-if="notebooks.length" density="compact" class="notebook-list">
          <v-list-item
            v-for="notebook in notebooks"
            :key="notebook.id"
            @click="toggle(notebook)"
          >
            <template #prepend>
              <v-checkbox-btn
                :model-value="linkedIds.includes(notebook.id)"
                @click.stop
                @update:model-value="toggle(notebook)"
              />
            </template>
            <v-list-item-title>{{ notebook.title }}</v-list-item-title>
            <v-list-item-subtitle class="text-caption">
              {{ t("{n} 个视频", { n: notebook.video_count }) }}
            </v-list-item-subtitle>
          </v-list-item>
        </v-list>
        <v-alert v-else type="info" variant="tonal" density="compact">
          {{ t("还没有记事本，请在下方创建一个") }}
        </v-alert>

        <v-divider class="my-4" />
        <div class="d-flex" style="gap: 8px">
          <v-text-field
            v-model="newTitle"
            :label="t('新建记事本标题')"
            density="compact"
            variant="outlined"
            hide-details
            @keydown.enter.prevent="createAndAdd"
          />
          <v-btn
            color="primary"
            variant="tonal"
            :loading="creating"
            :disabled="!newTitle.trim()"
            @click="createAndAdd"
          >
            {{ t("新建并添加") }}
          </v-btn>
        </div>
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
import type { NotebookInfo } from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{
  modelValue: boolean;
  videoId: number;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  changed: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const notebooks = ref<NotebookInfo[]>([]);
const linkedIds = ref<number[]>([]);
const newTitle = ref("");
const creating = ref(false);

async function load() {
  try {
    const [all, linked] = await Promise.all([
      invoke<NotebookInfo[]>("list_notebooks"),
      invoke<NotebookInfo[]>("list_notebooks_for_video", {
        videoId: props.videoId,
      }),
    ]);
    notebooks.value = all;
    linkedIds.value = linked.map((item) => item.id);
  } catch (e) {
    notify.error(t("加载记事本失败：{e}", { e: String(e) }));
  }
}

async function toggle(notebook: NotebookInfo) {
  const linked = linkedIds.value.includes(notebook.id);
  try {
    if (linked) {
      await invoke("remove_video_from_notebook", {
        notebookId: notebook.id,
        videoId: props.videoId,
      });
      linkedIds.value = linkedIds.value.filter((id) => id !== notebook.id);
    } else {
      await invoke("add_video_to_notebook", {
        notebookId: notebook.id,
        videoId: props.videoId,
      });
      linkedIds.value = [...linkedIds.value, notebook.id];
    }
    notebooks.value = notebooks.value.map((item) =>
      item.id === notebook.id
        ? {
            ...item,
            video_count: item.video_count + (linked ? -1 : 1),
          }
        : item,
    );
    emit("changed");
  } catch (e) {
    notify.error(t("更新记事本失败：{e}", { e: String(e) }));
  }
}

async function createAndAdd() {
  const title = newTitle.value.trim();
  if (!title) return;
  creating.value = true;
  try {
    const id = await invoke<number>("save_notebook", {
      id: null,
      title,
      content: "",
    });
    await invoke("add_video_to_notebook", {
      notebookId: id,
      videoId: props.videoId,
    });
    newTitle.value = "";
    await load();
    emit("changed");
    notify.success(t("已添加到记事本"));
  } catch (e) {
    notify.error(t("创建记事本失败：{e}", { e: String(e) }));
  } finally {
    creating.value = false;
  }
}

function close() {
  visible.value = false;
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) load();
  },
);
</script>

<style scoped>
.notebook-list {
  max-height: 280px;
  overflow-y: auto;
  border: 1px solid rgba(128, 128, 128, 0.18);
  border-radius: 8px;
}
</style>
