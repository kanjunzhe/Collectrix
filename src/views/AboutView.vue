<template>
  <div style="max-width: 720px">
    <div class="d-flex align-center mb-6">
      <img
        src="/collectrix-logo.svg"
        alt="Collectrix"
        class="about-logo mr-4"
      />
      <div>
        <div class="text-h4 font-weight-bold">Collectrix</div>
        <div class="text-subtitle-1 text-medium-emphasis">
          {{ t("本地视频收藏与知识管理") }}
        </div>
      </div>
    </div>

    <v-card variant="outlined" class="mb-4">
      <v-card-title class="d-flex align-center">
        {{ t("版本") }}
        <v-spacer />
        <v-btn
          size="small"
          variant="tonal"
          prepend-icon="mdi-update"
          :loading="checking"
          @click="checkUpdates"
        >
          {{ t("检查更新") }}
        </v-btn>
      </v-card-title>
      <v-card-text>
        <div class="info-row">
          <span>{{ t("应用版本") }}</span><span>0.1.1</span>
        </div>
        <v-alert
          v-if="updateError"
          type="warning"
          variant="tonal"
          density="compact"
          class="mt-3"
        >
          {{ updateError }}
        </v-alert>
        <v-alert
          v-else-if="updateInfo"
          :type="updateInfo.has_update ? 'success' : 'info'"
          variant="tonal"
          density="compact"
          class="mt-3"
        >
          <div class="d-flex align-center flex-wrap" style="gap: 8px">
            <span>
              {{
                updateInfo.has_update
                  ? t("发现新版本 v{version}", {
                      version: updateInfo.latest_version,
                    })
                  : t("当前已是最新版本")
              }}
            </span>
            <v-spacer />
            <v-btn
              v-if="updateInfo.has_update"
              size="small"
              color="primary"
              variant="flat"
              prepend-icon="mdi-download"
              @click="openDownload"
            >
              {{ t("前往下载") }}
            </v-btn>
          </div>
          <div
            v-if="updateInfo.notes && updateInfo.has_update"
            class="update-notes mt-2"
          >
            {{ updateInfo.notes }}
          </div>
        </v-alert>
        <div class="info-row"><span>Tauri</span><span>2.x</span></div>
        <div class="info-row">
          <span>{{ t("前端") }}</span><span>Vue 3 + Vuetify 4</span>
        </div>
        <div class="info-row">
          <span>{{ t("后端") }}</span><span>Rust + rusqlite</span>
        </div>
      </v-card-text>
    </v-card>

    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("技术栈") }}</v-card-title>
      <v-card-text>
        <v-chip class="ma-1" v-for="tech in techs" :key="tech" variant="tonal">
          {{ tech }}
        </v-chip>
      </v-card-text>
    </v-card>

    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("开源许可") }}</v-card-title>
      <v-card-text>
        <p class="text-body-2">
          {{
            t(
              "本项目使用 MIT 许可证。Tauri、Vue、Vuetify、Rust 生态组件均遵循各自的开源协议。",
            )
          }}
        </p>
        <p class="text-body-2 mt-2">
          {{
            t(
              "本工具仅用于个人学习与视频收藏，不提供视频内容的再分发。下载和转录功能请遵守 B 站及各视频平台的服务条款，并尊重原作者版权。",
            )
          }}
        </p>
      </v-card-text>
    </v-card>

    <v-card variant="outlined">
      <v-card-title>{{ t("数据存储") }}</v-card-title>
      <v-card-text>
        <div class="info-row">
          <span>{{ t("数据库") }}</span>
          <span class="text-medium-emphasis">app_data_dir/video_vault.db</span>
        </div>
        <div class="info-row">
          <span>{{ t("设置") }}</span>
          <span class="text-medium-emphasis">app_data_dir/settings.json</span>
        </div>
        <v-btn
          size="small"
          variant="text"
          prepend-icon="mdi-folder-open"
          class="mt-2"
          @click="openDataDir"
        >
          {{ t("打开数据目录") }}
        </v-btn>
      </v-card-text>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { notify } from "../composables/useUI";
import { t } from "../i18n";
import type { UpdateInfo } from "../types";

const checking = ref(false);
const updateInfo = ref<UpdateInfo | null>(null);
const updateError = ref("");

async function checkUpdates() {
  checking.value = true;
  updateError.value = "";
  try {
    updateInfo.value = await invoke<UpdateInfo>("check_for_updates");
  } catch (e) {
    updateInfo.value = null;
    updateError.value = t("检查更新失败：{e}", { e: String(e) });
  } finally {
    checking.value = false;
  }
}

async function openDownload() {
  const url = updateInfo.value?.download_url || updateInfo.value?.release_url;
  if (!url) return;
  try {
    await openUrl(url);
  } catch (e) {
    notify.error(t("打开链接失败：{e}", { e: String(e) }));
  }
}

const techs = [
  "Tauri 2",
  "Rust",
  "rusqlite",
  "reqwest",
  "Vue 3",
  "TypeScript",
  "Vuetify 4",
  "Vite",
  "whisper-rs",
  "symphonia",
  "ffmpeg",
];

async function openDataDir() {
  try {
    await invoke("open_data_dir");
  } catch (e) {
    notify.error(t("操作失败：{e}", { e: String(e) }));
  }
}
</script>

<style scoped>
.info-row {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px dashed rgba(128, 128, 128, 0.15);
  font-size: 13px;
}
.about-logo {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  box-shadow: 0 12px 32px rgba(25, 118, 210, 0.22);
}
.update-notes {
  max-height: 160px;
  overflow-y: auto;
  white-space: pre-wrap;
  font-size: 12px;
  opacity: 0.75;
}

</style>
