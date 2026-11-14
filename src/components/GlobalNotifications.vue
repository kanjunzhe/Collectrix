<template>
  <!-- 通知：右下角 -->
  <v-snackbar
    v-model="notifyVisible"
    :color="color"
    :timeout="currentNotify?.timeout ?? 3500"
    location="bottom right"
    max-width="520"
    @update:model-value="
      (v) => {
        if (!v) dismissCurrentNotify();
      }
    "
  >
    <div class="d-flex align-start" style="gap: 10px; padding: 4px 0">
      <v-icon class="mt-1">{{ icon }}</v-icon>
      <div
        class="text-body-2"
        style="white-space: pre-wrap; word-break: break-word"
      >
        {{ currentNotify?.message }}
      </div>
    </div>
    <template #actions>
      <v-btn variant="text" size="small" @click="dismissCurrentNotify"
        >{{ t("关闭") }}</v-btn
      >
    </template>
  </v-snackbar>

  <!-- 确认：居中 -->
  <v-dialog
    v-model="confirmState.visible"
    max-width="460"
    :persistent="confirmState.persistent"
  >
    <v-card>
      <v-card-title class="d-flex align-center">
        <v-icon :color="confirmState.confirmColor" size="24" class="mr-2">
          {{
            confirmState.confirmColor === "error"
              ? "mdi-alert-circle"
              : "mdi-help-circle"
          }}
        </v-icon>
        {{ confirmState.title }}
      </v-card-title>
      <v-card-text style="white-space: pre-wrap; word-break: break-word">
        {{ confirmState.message }}
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn @click="resolveConfirm(false)">
          {{ confirmState.cancelText }}
        </v-btn>
        <v-btn
          :color="confirmState.confirmColor"
          variant="flat"
          @click="resolveConfirm(true)"
        >
          {{ confirmState.confirmText }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from "vue";
import {
  currentNotify,
  dismissCurrentNotify,
  notifyVisible,
  confirmState,
  resolveConfirm,
} from "../composables/useUI";
import { t } from "../i18n";

const color = computed(() => {
  switch (currentNotify.value?.type) {
    case "success":
      return "success";
    case "error":
      return "error";
    case "warning":
      return "warning";
    default:
      return "info";
  }
});

const icon = computed(() => {
  switch (currentNotify.value?.type) {
    case "success":
      return "mdi-check-circle";
    case "error":
      return "mdi-close-circle";
    case "warning":
      return "mdi-alert-circle";
    default:
      return "mdi-information";
  }
});
</script>
