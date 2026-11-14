<template>
  <v-dialog v-model="visible" max-width="420" persistent>
    <v-card>
      <v-card-title>{{ t("Bilibili 扫码登录") }}</v-card-title>
      <v-card-text class="text-center">
        <div v-if="generating" class="py-8">
          <v-progress-circular indeterminate color="primary" />
        </div>
        <template v-else>
          <img
            v-if="qrDataUrl"
            :src="qrDataUrl"
            class="qr-image"
            alt="Bilibili QR code"
          />
          <div class="text-body-2 mt-3">{{ statusText }}</div>
          <div v-if="statusMessage && statusCode !== 0" class="text-caption text-medium-emphasis mt-1">
            {{ statusMessage }}
          </div>
        </template>
      </v-card-text>
      <v-card-actions>
        <v-btn variant="text" :loading="generating" @click="start">
          {{ t("刷新二维码") }}
        </v-btn>
        <v-spacer />
        <v-btn @click="close">{{ t("关闭") }}</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { toDataURL } from "qrcode";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

interface QrInfo {
  url: string;
  qrcode_key: string;
}

interface QrStatus {
  code: number;
  message: string;
  logged_in: boolean;
}

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  "logged-in": [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const generating = ref(false);
const polling = ref(false);
const qrDataUrl = ref("");
const qrcodeKey = ref("");
const statusCode = ref(-1);
const statusMessage = ref("");
let timer: ReturnType<typeof setInterval> | null = null;

const statusText = computed(() => {
  switch (statusCode.value) {
    case 0:
      return t("登录成功");
    case 86101:
      return t("请使用 B 站 App 扫码");
    case 86090:
      return t("已扫码，请在手机上确认");
    case 86038:
      return t("二维码已失效，请刷新");
    default:
      return t("等待扫码…");
  }
});

function stopPolling() {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
}

async function start() {
  stopPolling();
  generating.value = true;
  qrDataUrl.value = "";
  qrcodeKey.value = "";
  statusCode.value = -1;
  statusMessage.value = "";
  try {
    const info = await invoke<QrInfo>("generate_bili_qr");
    qrcodeKey.value = info.qrcode_key;
    qrDataUrl.value = await toDataURL(info.url, {
      width: 240,
      margin: 1,
      errorCorrectionLevel: "M",
    });
    timer = setInterval(poll, 2000);
    await poll();
  } catch (e) {
    notify.error(t("生成二维码失败：{e}", { e: String(e) }));
  } finally {
    generating.value = false;
  }
}

async function poll() {
  if (polling.value || !qrcodeKey.value) return;
  polling.value = true;
  try {
    const status = await invoke<QrStatus>("poll_bili_qr", {
      qrcodeKey: qrcodeKey.value,
    });
    statusCode.value = status.code;
    statusMessage.value = status.message;
    if (status.logged_in) {
      stopPolling();
      notify.success(t("Bilibili 登录成功"));
      emit("logged-in");
      close();
    } else if (status.code === 86038) {
      stopPolling();
    }
  } catch (e) {
    statusMessage.value = String(e);
  } finally {
    polling.value = false;
  }
}

function close() {
  stopPolling();
  visible.value = false;
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      start();
    } else {
      stopPolling();
    }
  },
);

onBeforeUnmount(stopPolling);
</script>

<style scoped>
.qr-image {
  width: 240px;
  height: 240px;
  border-radius: 8px;
  background: #fff;
  padding: 6px;
}
</style>
