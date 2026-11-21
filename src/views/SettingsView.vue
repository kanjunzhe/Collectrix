<!--
  改动点：
  1. 「AI 智能分类」卡片新增：提供者选择（OpenAI / Ollama / 自定义）
  2. Ollama 模式自动填 Base URL、模型名，并显示 ollama_path 选择
  3. 保留所有原有卡片（外观、路径、ffmpeg、转录、B站、其他）
  4. 外观切换即时生效（watch settings.theme）
-->
<template>
  <div style="max-width: 760px">
    <div class="text-h5 font-weight-bold mb-4">{{ t("设置") }}</div>

    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("新手引导") }}</v-card-title>
      <v-card-text class="d-flex align-center flex-wrap" style="gap: 12px">
        <span class="text-body-2 text-medium-emphasis">
          {{ t("第一次使用？可以重新查看 Collectrix 的功能介绍。") }}
        </span>
        <v-spacer />
        <v-btn
          variant="tonal"
          prepend-icon="mdi-compass-outline"
          @click="openOnboarding"
        >
          {{ t("重新查看新手引导") }}
        </v-btn>
      </v-card-text>
    </v-card>

    <!-- 外观 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("外观") }}</v-card-title>
      <v-card-text>
        <div class="d-flex flex-wrap" style="gap: 8px">
          <v-btn
            v-for="option in themeOptions"
            :key="option.value"
            :variant="settings.theme === option.value ? 'flat' : 'outlined'"
            :color="settings.theme === option.value ? 'primary' : undefined"
            @click="settings.theme = option.value"
          >
            <span
              class="theme-swatch"
              :style="{ backgroundColor: option.swatch }"
            />
            {{ t(option.label) }}
          </v-btn>
          <v-btn
            :variant="settings.theme === 'system' ? 'flat' : 'outlined'"
            :color="settings.theme === 'system' ? 'primary' : undefined"
            prepend-icon="mdi-monitor"
            @click="settings.theme = 'system'"
          >
            {{ t("跟随系统") }}
          </v-btn>
        </div>
        <div class="text-caption text-medium-emphasis mt-2">
          {{ t("切换立即生效，点「保存设置」后持久化。") }}
        </div>
      </v-card-text>
    </v-card>

    <!-- 语言 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("语言") }}</v-card-title>
      <v-card-text>
        <div class="d-flex flex-wrap" style="gap: 8px">
          <v-btn
            v-for="option in languageOptions"
            :key="option.value"
            :variant="settings.language === option.value ? 'flat' : 'outlined'"
            :color="settings.language === option.value ? 'primary' : undefined"
            @click="onLanguageChange(option.value)"
          >
            {{ t(option.title) }}
          </v-btn>
        </div>
        <div class="text-caption text-medium-emphasis mt-2">
          {{ t("切换立即生效，点「保存设置」后持久化。") }}
        </div>
      </v-card-text>
    </v-card>

    <!-- 保存路径 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("保存路径") }}</v-card-title>
      <v-card-text>
        <div
          v-for="item in pathItems"
          :key="item.key"
          class="d-flex align-center mb-2"
          style="gap: 8px"
        >
          <v-text-field
            :model-value="(settings as any)[item.key]"
            :label="t(item.label)"
            density="compact"
            variant="outlined"
            hide-details
            readonly
            :placeholder="t(item.placeholder)"
            style="flex: 1"
          />
          <v-btn variant="tonal" @click="pickPath(item.key)">{{ t("选择") }}</v-btn>
        </div>
      </v-card-text>
    </v-card>

    <!-- ffmpeg -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>ffmpeg</v-card-title>
      <v-card-text>
        <div class="d-flex align-center" style="gap: 8px">
          <v-text-field
            v-model="settings.ffmpeg_path"
            :label="t('ffmpeg 可执行文件路径（留空则用系统 PATH）')"
            density="compact"
            variant="outlined"
            hide-details
            style="flex: 1"
          />
          <v-btn variant="tonal" @click="pickFfmpeg">{{ t("选择") }}</v-btn>
        </div>
        <v-alert type="info" variant="tonal" density="compact" class="mt-2">
          {{
            t(
              "Windows 从 gyan.dev 下载，解压后选择 ffmpeg.exe；macOS/Linux 通常 /usr/local/bin/ffmpeg。",
            )
          }}
        </v-alert>
      </v-card-text>
    </v-card>

    <!-- 下载 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("下载") }}</v-card-title>
      <v-card-text>
        <v-row dense>
          <v-col cols="12" sm="6">
            <v-select
              v-model="settings.download_video_quality"
              :items="videoQualityOptions"
              item-title="title"
              item-value="value"
              :label="t('视频画质')"
              density="compact"
              variant="outlined"
            />
          </v-col>
          <v-col cols="12" sm="6">
            <v-select
              v-model="settings.download_audio_quality"
              :items="audioQualityOptions"
              item-title="title"
              item-value="value"
              :label="t('音频音质')"
              density="compact"
              variant="outlined"
            />
          </v-col>
        </v-row>

        <div class="text-subtitle-2 font-weight-medium mb-1">
          {{ t("下载内容") }}
        </div>
        <div class="d-flex flex-wrap" style="gap: 4px 16px">
          <v-checkbox
            v-model="settings.download_video"
            :label="t('视频')"
            density="compact"
            hide-details
          />
          <v-checkbox
            v-model="settings.download_audio"
            :label="t('音频')"
            density="compact"
            hide-details
          />
          <v-checkbox
            v-model="settings.download_subtitles"
            :label="t('字幕')"
            density="compact"
            hide-details
          />
          <v-checkbox
            v-model="settings.download_danmaku"
            :label="t('弹幕')"
            density="compact"
            hide-details
          />
          <v-checkbox
            v-model="settings.download_cover"
            :label="t('封面')"
            density="compact"
            hide-details
          />
        </div>
      </v-card-text>
    </v-card>

    <!-- 语音转录 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("语音转录") }}</v-card-title>
      <v-card-text>
        <v-select
          v-model="settings.transcribe_provider"
          :items="transcribeProviderOptions"
          item-title="title"
          item-value="value"
          :label="t('转录方式')"
        />

        <v-alert
          v-if="settings.transcribe_provider === 'bilibili'"
          type="info"
          variant="tonal"
          density="compact"
          class="mt-2"
        >
          {{ t("将优先读取本地同目录字幕，再尝试获取 Bilibili 字幕，不需要音频文件。") }}
        </v-alert>

        <template v-if="settings.transcribe_provider === 'local'">
          <div class="d-flex align-center mb-2" style="gap: 8px">
            <v-text-field
              v-model="settings.whisper_model_path"
              :label="t('Whisper GGML 模型路径')"
              density="compact"
              variant="outlined"
              hide-details
              readonly
              style="flex: 1"
            />
            <v-btn variant="tonal" @click="pickWhisper">{{ t("选择") }}</v-btn>
          </div>
        </template>

        <template v-else-if="settings.transcribe_provider === 'openai'">
          <v-text-field
            v-model="settings.openai_base_url"
            label="API Base URL"
            :placeholder="t('https://api.openai.com/v1 或 https://api.siliconflow.cn/v1')"
            density="compact"
            variant="outlined"
          />
          <v-text-field
            v-model="settings.openai_api_key"
            label="API Key"
            type="password"
            density="compact"
            variant="outlined"
          />
          <v-text-field
            v-model="settings.openai_transcribe_model"
            :label="t('转录模型（whisper-1 / FunAudioLLM/SenseVoiceSmall 等）')"
            density="compact"
            variant="outlined"
          />
        </template>

        <v-checkbox
          v-model="settings.transcribe_auto_polish"
          :label="t('转录后自动使用 AI 加标点 / 校对')"
          density="compact"
          hide-details
          class="mt-1"
        />

        <v-select
          v-if="settings.transcribe_provider !== 'bilibili'"
          v-model="settings.whisper_language"
          :items="whisperLanguageOptions"
          item-title="title"
          item-value="value"
          :label="t('识别语言')"
          class="mt-2"
        />
      </v-card-text>
    </v-card>

    <!-- AI 智能分类（改动点） -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>AI</v-card-title>
      <v-card-text>
        <v-switch
          v-model="settings.ai_enabled"
          :label="t('启用 AI 功能')"
          color="primary"
          density="compact"
          hide-details
          @update:model-value="onAiToggle"
        />

        <v-alert
          v-if="!settings.ai_enabled"
          type="info"
          variant="tonal"
          density="compact"
          class="mt-2"
        >
          {{ t("AI 功能已关闭，AI 相关栏目已隐藏。") }}
        </v-alert>

        <template v-if="settings.ai_enabled">
        <v-select
          v-model="settings.ai_provider"
          :items="aiProviderOptions"
          item-title="title"
          item-value="value"
          :label="t('AI 提供者')"
          @update:model-value="onProviderChange"
        />

        <v-text-field
          v-model="settings.openai_base_url"
          label="API Base URL"
          :placeholder="openaiBaseUrlPlaceholder"
          density="compact"
          variant="outlined"
        />

        <v-text-field
          v-model="settings.openai_api_key"
          :label="t('API Key（Ollama 可填任意值）')"
          type="password"
          density="compact"
          variant="outlined"
        />

        <v-text-field
          v-model="settings.ai_model"
          :label="t('模型名')"
          :placeholder="aiModelPlaceholder"
          density="compact"
          variant="outlined"
        />

        <template v-if="settings.ai_provider === 'ollama'">
          <div class="d-flex align-center mt-2" style="gap: 8px">
            <v-text-field
              v-model="settings.ollama_path"
              :label="t('Ollama 可执行文件路径（可选，留空则用系统 PATH）')"
              density="compact"
              variant="outlined"
              hide-details
              style="flex: 1"
            />
            <v-btn variant="tonal" @click="pickOllama">{{ t("选择") }}</v-btn>
          </div>
          <v-alert type="info" variant="tonal" density="compact" class="mt-2">
            {{ t("先在终端运行") }}
            <code>ollama pull qwen2.5:7b</code>
            {{ t("拉取模型，然后在「AI 整理」页点击「启动 Ollama」。") }}
          </v-alert>
        </template>

        <v-alert type="info" variant="tonal" density="compact" class="mt-2">
          {{
            t(
              "复用「语音转录」里的 API Key 和 Base URL。云端可用 OpenAI、硅基流动、Groq 等兼容服务。",
            )
          }}
        </v-alert>
        </template>
      </v-card-text>
    </v-card>

    <!-- GitHub -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>GitHub</v-card-title>
      <v-card-text>
        <v-text-field
          v-model="settings.github_username"
          :label="t('GitHub 用户名')"
          placeholder="octocat"
          density="compact"
          variant="outlined"
        />
        <v-text-field
          v-model="settings.github_token"
          :label="t('GitHub Token（可选，用于私有 Star 和更高 API 限额）')"
          type="password"
          density="compact"
          variant="outlined"
        />
        <v-alert type="info" variant="tonal" density="compact">
          {{
            t(
              "只读取公开 Star 时可以只填用户名；填写 Token 后读取当前账号的 Star，并支持私有仓库。Token 只需要 public_repo 或 repo 读取权限。",
            )
          }}
        </v-alert>
      </v-card-text>
    </v-card>

    <!-- B 站登录 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title class="d-flex align-center">
        {{ t("Bilibili 登录") }}
        <v-chip
          v-if="biliLogin"
          size="small"
          class="ml-2"
          :color="biliLogin.is_login ? 'success' : 'warning'"
          variant="flat"
        >
          {{
            biliLogin.is_login
              ? t("已登录 {name}", { name: biliLogin.uname || "" })
              : t("未登录")
          }}
        </v-chip>
        <v-spacer />
        <v-btn
          size="small"
          variant="tonal"
          prepend-icon="mdi-qrcode-scan"
          @click="showQrLogin = true"
        >
          {{ t("扫码登录") }}
        </v-btn>
      </v-card-title>
      <v-card-text>
        <v-textarea
          v-model="settings.bilibili_cookie"
          label="Cookie"
          rows="3"
          density="compact"
          variant="outlined"
          placeholder="SESSDATA=xxx; bili_jct=xxx; DedeUserID=xxx;"
        />
        <v-alert type="info" variant="tonal" density="compact">
          {{
            t(
              "用于导入 B 站收藏夹。浏览器登录后 F12 → Application → Cookies → 复制含 SESSDATA 和 DedeUserID 的整段。",
            )
          }}
        </v-alert>
      </v-card-text>
    </v-card>

    <!-- 应用锁 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("应用锁") }}</v-card-title>
      <v-card-text>
        <v-switch
          v-model="settings.lock_enabled"
          :label="t('启用应用锁')"
          color="primary"
          density="compact"
          hide-details
        />
        <template v-if="settings.lock_enabled">
          <v-text-field
            v-model="lockPin"
            type="password"
            inputmode="numeric"
            maxlength="12"
            :label="t('数字密钥（4-12 位，留空则不修改）')"
            density="compact"
            variant="outlined"
            class="mt-2"
          />
          <v-text-field
            v-model="lockPinConfirm"
            type="password"
            inputmode="numeric"
            maxlength="12"
            :label="t('确认数字密钥')"
            density="compact"
            variant="outlined"
          />
          <v-btn
            color="primary"
            variant="tonal"
            prepend-icon="mdi-lock-check"
            :loading="savingLock"
            @click="saveLock"
          >
            {{ t("保存密钥") }}
          </v-btn>
        </template>
        <v-alert type="info" variant="tonal" density="compact" class="mt-2">
          {{
            t(
              "开启后每次启动应用都需要输入数字密钥。忘记密钥将无法解锁，请妥善保管。",
            )
          }}
        </v-alert>
      </v-card-text>
    </v-card>

    <!-- 其他 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-title>{{ t("其他") }}</v-card-title>
      <v-card-text>
        <v-text-field
          v-model.number="settings.recent_days"
          :label="t('「近期新增」徽标判定天数')"
          type="number"
          min="1"
          max="60"
          density="compact"
          variant="outlined"
        />
      </v-card-text>
    </v-card>

    <div class="d-flex" style="gap: 8px">
      <v-spacer />
      <v-btn @click="load">{{ t("重置") }}</v-btn>
      <v-btn color="primary" prepend-icon="mdi-content-save" @click="save">
        {{ t("保存设置") }}
      </v-btn>
    </div>

    <QrLoginDialog v-model="showQrLogin" @logged-in="onQrLoggedIn" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "vuetify";
import type { AppSettings } from "../types";
import { notify } from "../composables/useUI";
import { languageOptions, locale, setLocale, t } from "../i18n";
import { resolveTheme, themeOptions } from "../themes";
import QrLoginDialog from "../components/QrLoginDialog.vue";

const theme = useTheme();

const settings = ref<AppSettings>({
  theme: "light",
  language: "zh",
  video_dir: "",
  audio_dir: "",
  transcript_dir: "",
  ffmpeg_path: "",
  download_video_quality: "best",
  download_audio_quality: "best",
  download_video: true,
  download_audio: true,
  download_subtitles: false,
  download_danmaku: false,
  download_cover: false,
  whisper_model_path: "",
  whisper_language: "auto",
  transcribe_provider: "local",
  transcribe_auto_polish: false,
  openai_api_key: "",
  openai_base_url: "https://api.openai.com/v1",
  openai_transcribe_model: "whisper-1",
  ai_enabled: true,
  ai_model: "gpt-4o-mini",
  ai_provider: "openai",
  ollama_path: "",
  bilibili_cookie: "",
  github_username: "",
  github_token: "",
  recent_days: 7,
  lock_enabled: false,
  lock_pin_hash: "",
});

const showQrLogin = ref(false);
const lockPin = ref("");
const lockPinConfirm = ref("");
const savingLock = ref(false);
const biliLogin = ref<{ is_login: boolean; uname: string | null } | null>(null);

async function saveLock() {
  const pin = lockPin.value.trim();
  if (settings.value.lock_enabled) {
    if (pin && pin !== lockPinConfirm.value.trim()) {
      notify.warning(t("两次输入的数字密钥不一致"));
      return;
    }
    if (!pin && !settings.value.lock_pin_hash) {
      notify.warning(t("请设置数字密钥"));
      return;
    }
  }
  savingLock.value = true;
  try {
    await invoke("set_app_lock", {
      enabled: settings.value.lock_enabled,
      pin: pin || null,
    });
    const s = await invoke<AppSettings>("get_settings");
    settings.value.lock_enabled = s.lock_enabled;
    settings.value.lock_pin_hash = s.lock_pin_hash;
    lockPin.value = "";
    lockPinConfirm.value = "";
    notify.success(t("应用锁已更新"));
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  } finally {
    savingLock.value = false;
  }
}

async function checkBiliLogin() {
  try {
    biliLogin.value = await invoke<{ is_login: boolean; uname: string | null }>(
      "check_bili_login",
    );
  } catch {
    biliLogin.value = { is_login: false, uname: null };
  }
}

async function onQrLoggedIn() {
  try {
    const s = await invoke<AppSettings>("get_settings");
    settings.value.bilibili_cookie = s.bilibili_cookie;
    await checkBiliLogin();
  } catch (e) {
    console.warn("刷新 B 站 Cookie 失败", e);
  }
}

const transcribeProviderOptions = computed(() => [
  { title: t("本地 Whisper (whisper.cpp)"), value: "local" },
  { title: t("在线 Whisper (OpenAI 兼容)"), value: "openai" },
  { title: t("Bilibili 字幕"), value: "bilibili" },
]);

const whisperLanguageOptions = computed(() => [
  { title: t("自动检测"), value: "auto" },
  { title: t("中文"), value: "zh" },
  { title: t("英语"), value: "en" },
  { title: t("日语"), value: "ja" },
]);

const aiProviderOptions = computed(() => [
  { title: t("OpenAI / 兼容服务（云）"), value: "openai" },
  { title: t("DeepSeek"), value: "deepseek" },
  { title: t("硅基流动 SiliconFlow"), value: "siliconflow" },
  { title: t("Groq"), value: "groq" },
  { title: t("Kimi / Moonshot"), value: "moonshot" },
  { title: t("通义千问 (DashScope)"), value: "qwen" },
  { title: t("智谱 GLM"), value: "zhipu" },
  { title: t("Ollama（本地）"), value: "ollama" },
  { title: t("自定义"), value: "custom" },
]);

const openaiBaseUrlPlaceholder = computed(() =>
  settings.value.ai_provider === "ollama"
    ? "http://localhost:11434/v1"
    : "https://api.openai.com/v1",
);

const aiModelPlaceholder = computed(() =>
  settings.value.ai_provider === "ollama" ? "qwen2.5:7b" : "gpt-4o-mini",
);

const videoQualityOptions = computed(() => [
  { title: t("最高画质"), value: "best" },
  { title: "4K", value: "4k" },
  { title: "1080P", value: "1080p" },
  { title: "720P", value: "720p" },
  { title: "480P", value: "480p" },
  { title: "360P", value: "360p" },
]);

const audioQualityOptions = computed(() => [
  { title: t("最高音质"), value: "best" },
  { title: t("高"), value: "high" },
  { title: t("中"), value: "medium" },
  { title: t("低"), value: "low" },
]);

const pathItems = [
  {
    key: "video_dir",
    label: "视频保存目录",
    placeholder: "留空则用系统下载目录/Collectrix/videos",
  },
  {
    key: "audio_dir",
    label: "音频保存目录",
    placeholder: "留空则用系统下载目录/Collectrix/audio",
  },
  {
    key: "transcript_dir",
    label: "文字稿保存目录",
    placeholder: "留空则用系统下载目录/Collectrix/transcripts",
  },
] as const;

/** 主题立即生效 */
watch(
  () => settings.value.theme,
  (v) => {
    if (v) applyTheme(v);
  },
);

/** 顶部工具栏切换语言时，同步设置页里的选中状态。 */
watch(locale, (value) => {
  if (settings.value.language !== value) {
    settings.value.language = value;
  }
});

/** 顶部工具栏切换深浅色时，同步设置页里的选中状态。 */
watch(
  () => theme.global.name.value,
  (name) => {
    if (settings.value.theme !== "system" && settings.value.theme !== name) {
      settings.value.theme = name;
    }
  },
);

function applyTheme(mode: string) {
  theme.global.name.value = resolveTheme(mode);
}

function openOnboarding() {
  window.dispatchEvent(new CustomEvent("collectrix:open-onboarding"));
}

async function load() {
  settings.value = await invoke<AppSettings>("get_settings");
  applyTheme(settings.value.theme);
  setLocale(settings.value.language);
  await checkBiliLogin();
}

async function save() {
  // 应用锁通过「保存密钥」单独持久化，避免主保存写入未设置的锁状态。
  const current = await invoke<AppSettings>("get_settings");
  const payload = {
    ...settings.value,
    lock_enabled: current.lock_enabled,
    lock_pin_hash: current.lock_pin_hash,
  };
  await invoke("set_settings", { settings: payload });
  settings.value.lock_enabled = current.lock_enabled;
  settings.value.lock_pin_hash = current.lock_pin_hash;
  applyTheme(settings.value.theme);
  setLocale(settings.value.language);
  await checkBiliLogin();
  window.dispatchEvent(new CustomEvent("collectrix:settings-changed"));
  notify.success(t("设置已保存"));
}

async function onAiToggle(value: boolean | null) {
  try {
    const current = await invoke<AppSettings>("get_settings");
    current.ai_enabled = value !== false;
    await invoke("set_settings", { settings: current });
    settings.value.ai_enabled = current.ai_enabled;
    window.dispatchEvent(new CustomEvent("collectrix:settings-changed"));
  } catch (e) {
    notify.error(t("保存失败：{e}", { e: String(e) }));
  }
}

function onLanguageChange(value: string) {
  settings.value.language = value;
  setLocale(value);
}

async function pickPath(key: string) {
  const p = await invoke<string | null>("pick_directory");
  if (p) (settings.value as any)[key] = p;
}

async function pickFfmpeg() {
  const p = await invoke<string | null>("pick_executable");
  if (p) settings.value.ffmpeg_path = p;
}

async function pickWhisper() {
  const p = await invoke<string | null>("pick_whisper_model");
  if (p) settings.value.whisper_model_path = p;
}

async function pickOllama() {
  const p = await invoke<string | null>("pick_executable");
  if (p) settings.value.ollama_path = p;
}

const AI_PRESETS: Record<string, { base: string; model: string; key?: string }> =
  {
    openai: {
      base: "https://api.openai.com/v1",
      model: "gpt-4o-mini",
    },
    deepseek: {
      base: "https://api.deepseek.com/v1",
      model: "deepseek-chat",
    },
    siliconflow: {
      base: "https://api.siliconflow.cn/v1",
      model: "Qwen/Qwen2.5-7B-Instruct",
    },
    groq: {
      base: "https://api.groq.com/openai/v1",
      model: "llama-3.3-70b-versatile",
    },
    moonshot: {
      base: "https://api.moonshot.cn/v1",
      model: "moonshot-v1-8k",
    },
    qwen: {
      base: "https://dashscope.aliyuncs.com/compatible-mode/v1",
      model: "qwen-plus",
    },
    zhipu: {
      base: "https://open.bigmodel.cn/api/paas/v4",
      model: "glm-4-flash",
    },
    ollama: {
      base: "http://localhost:11434/v1",
      model: "qwen2.5:7b",
      key: "ollama",
    },
  };

/** 切换 AI 提供者时填入对应服务的默认端点/模型，之后仍可手动修改。 */
function onProviderChange(v: string) {
  const preset = AI_PRESETS[v];
  if (!preset) return;
  settings.value.openai_base_url = preset.base;
  settings.value.ai_model = preset.model;
  if (preset.key) {
    settings.value.openai_api_key = preset.key;
  } else if (settings.value.openai_api_key === "ollama") {
    settings.value.openai_api_key = "";
  }
}

onMounted(load);
</script>

<style scoped>
.theme-swatch {
  display: inline-block;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  margin-right: 8px;
  border: 1px solid rgba(0, 0, 0, 0.15);
}
</style>
