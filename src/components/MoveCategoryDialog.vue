<template>
  <v-dialog v-model="visible" max-width="440" persistent>
    <v-card>
      <v-card-title>{{ t("移动分类") }}</v-card-title>
      <v-card-text v-if="category">
        <div class="text-body-2 mb-3">
          {{ t("将分类「{name}」移动到：", { name: category.name }) }}
        </div>
        <v-select
          v-model="parentId"
          :items="categories"
          item-title="label"
          item-value="id"
          :label="t('目标父分类')"
          density="compact"
          variant="outlined"
          clearable
          :hint="t('留空表示移动到顶级分类')"
          persistent-hint
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="close">{{ t("取消") }}</v-btn>
        <v-btn
          color="primary"
          :loading="moving"
          variant="flat"
          @click="onConfirm"
        >
          {{ t("移动") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { CategoryNode } from "../types";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{
  modelValue: boolean;
  category: CategoryNode | null;
  categories: { id: number; label: string }[];
  currentParentId: number | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  moved: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const parentId = ref<number | null>(null);
const moving = ref(false);

watch(
  () => props.modelValue,
  (open) => {
    if (open) parentId.value = props.currentParentId;
  },
);

function close() {
  visible.value = false;
}

async function onConfirm() {
  if (!props.category) return;
  moving.value = true;
  try {
    await invoke("move_category", {
      id: props.category.id,
      parentId: parentId.value,
    });
    notify.success(t("分类已移动"));
    emit("moved");
    close();
  } catch (e) {
    notify.error(t("移动失败：{e}", { e: String(e) }));
  } finally {
    moving.value = false;
  }
}
</script>
