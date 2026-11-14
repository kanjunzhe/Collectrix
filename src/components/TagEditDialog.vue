<template>
  <v-dialog v-model="visible" max-width="440" persistent>
    <v-card>
      <v-card-title>
        {{ mode === "color" ? t("仅修改颜色") : t("编辑标签") }}
      </v-card-title>
      <v-card-text v-if="tag">
        <v-text-field
          v-if="mode === 'edit'"
          v-model="name"
          :label="t('标签名')"
          density="compact"
          variant="outlined"
        />
        <div class="d-flex align-center mt-3" style="gap: 12px">
          <span class="text-body-2">{{ t("颜色") }}</span>
          <input v-model="color" type="color" class="tag-color-input" />
          <v-chip :color="color" variant="flat">
            {{ mode === "color" ? tag.name : name || tag.name }}
          </v-chip>
          <v-btn
            size="small"
            variant="text"
            prepend-icon="mdi-auto-fix"
            @click="applySmartColor"
          >
            {{ t("智能配色") }}
          </v-btn>
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="close">{{ t("取消") }}</v-btn>
        <v-btn color="primary" :loading="saving" variant="flat" @click="onSave">
          {{ t("保存") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { TagInfo } from "../types";
import { notify } from "../composables/useUI";
import { smartTagColor } from "../utils/tagColor";
import { t } from "../i18n";

const props = defineProps<{
  modelValue: boolean;
  tag: TagInfo | null;
  mode: "edit" | "color";
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  saved: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const name = ref("");
const color = ref("#1976D2");
const saving = ref(false);

watch(
  () => props.modelValue,
  (open) => {
    if (!open || !props.tag) return;
    name.value = props.tag.name;
    color.value = props.tag.color;
  },
);

function close() {
  visible.value = false;
}

function applySmartColor() {
  color.value = smartTagColor(
    (props.mode === "color" ? props.tag?.name : name.value) ||
      props.tag?.name ||
      "",
  );
}

async function onSave() {
  if (!props.tag) return;
  const nextName = (props.mode === "color" ? props.tag.name : name.value).trim();
  if (!nextName) {
    notify.warning(t("标签名不能为空"));
    return;
  }
  saving.value = true;
  try {
    await invoke("update_tag", {
      id: props.tag.id,
      name: nextName,
      color: color.value,
    });
    notify.success(t("标签已更新"));
    emit("saved");
    close();
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  } finally {
    saving.value = false;
  }
}
</script>

<style scoped>
.tag-color-input {
  width: 44px;
  height: 34px;
  border: 1px solid rgba(0, 0, 0, 0.2);
  border-radius: 6px;
  padding: 2px;
  cursor: pointer;
  background: transparent;
}
</style>
