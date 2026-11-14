<template>
  <v-dialog v-model="visible" max-width="660" persistent>
    <v-card class="onboarding-card">
      <div class="onboarding-hero">
        <div class="onboarding-logo-row">
          <img
            src="/collectrix-logo-flat.svg"
            alt="Collectrix"
            class="onboarding-logo"
          />
          <v-chip size="small" variant="tonal" color="primary">
            {{ step + 1 }} / {{ steps.length }}
          </v-chip>
        </div>
        <v-icon size="42" color="primary" class="onboarding-icon">
          {{ current.icon }}
        </v-icon>
        <div class="onboarding-title">{{ t(current.title) }}</div>
        <div class="onboarding-desc">{{ t(current.body) }}</div>
      </div>

      <v-card-text class="pt-0">
        <v-alert
          type="info"
          variant="tonal"
          density="compact"
          class="onboarding-tip"
        >
          {{ t(current.tip) }}
        </v-alert>

        <div class="onboarding-dots">
          <button
            v-for="(item, index) in steps"
            :key="item.title"
            type="button"
            class="onboarding-dot"
            :class="{ 'is-active': index === step }"
            :aria-label="`${t('第 {n} 步', { n: index + 1 })}：${t(item.title)}`"
            @click="step = index"
          />
        </div>
      </v-card-text>

      <v-card-actions class="onboarding-actions">
        <v-btn variant="text" @click="finish">{{ t("跳过") }}</v-btn>
        <v-spacer />
        <v-btn v-if="step > 0" variant="text" @click="step--">
          {{ t("上一步") }}
        </v-btn>
        <v-btn color="primary" variant="flat" @click="next">
          {{ step === steps.length - 1 ? t("开始使用") : t("下一步") }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { t } from "../i18n";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  finish: [];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value) => emit("update:modelValue", value),
});

const step = ref(0);
const steps = [
  {
    icon: "mdi-compass-outline",
    title: "欢迎使用 Collectrix",
    body: "Collectrix 是一个本地优先的视频收藏和知识管理工具。你的资料、分类、笔记和媒体文件都保存在自己的电脑上。",
    tip: "这套引导只需要看一次，之后可以在「设置 → 新手引导」重新打开。",
  },
  {
    icon: "mdi-video-plus-outline",
    title: "添加和导入视频",
    body: "可以从链接添加视频、扫描本地文件夹，或者导入 B 站收藏夹和稍后再看。本地视频文件夹里的封面、音频和字幕会自动关联。",
    tip: "本地导入支持同目录的封面、音频、SRT/VTT/ASS 字幕；只有一个视频的文件夹会自动识别 sidecar 文件。",
  },
  {
    icon: "mdi-folder-multiple-outline",
    title: "整理你的视频库",
    body: "用分类树、标签和主题管理视频。视频可以直接拖进左侧分类，也可以在右键菜单中移动、编辑或批量整理。",
    tip: "视频库右上角可以切换网格/列表视图；分类栏还支持固定和悬停展开两种模式。",
  },
  {
    icon: "mdi-play-circle-outline",
    title: "播放、弹幕和字幕",
    body: "点击视频卡片即可在应用内播放。本地 B 站视频支持弹幕和字幕；B 站视频还支持评论和下载。",
    tip: "播放器中的评论、弹幕和字幕都是按需加载，不会在打开页面时立即请求。",
  },
  {
    icon: "mdi-robot-outline",
    title: "AI 和知识库",
    body: "开启 AI 后可以使用 AI 整理、AI 筛选和 AI 对话；专题、记事本和思维导图可以帮你把视频变成可检索的知识。",
    tip: "AI 设置集中在「设置 → AI」，没有配置 API 时不会影响普通视频管理功能。",
  },
] as const;

const current = computed(() => steps[step.value]);

function next() {
  if (step.value >= steps.length - 1) {
    finish();
    return;
  }
  step.value += 1;
}

function finish() {
  emit("finish");
  visible.value = false;
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) step.value = 0;
  },
);
</script>

<style scoped>
.onboarding-card {
  overflow: hidden;
}
.onboarding-hero {
  padding: 24px 26px 18px;
  background:
    radial-gradient(
      circle at 88% 12%,
      rgba(var(--v-theme-primary), 0.18),
      transparent 38%
    ),
    linear-gradient(
      135deg,
      rgba(var(--v-theme-primary), 0.12),
      rgba(var(--v-theme-surface), 0.5)
    );
}
.onboarding-logo-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}
.onboarding-logo {
  width: 46px;
  height: 46px;
  border-radius: 12px;
  box-shadow: 0 8px 22px rgba(25, 118, 210, 0.22);
}
.onboarding-icon {
  display: block;
  margin-bottom: 8px;
}
.onboarding-title {
  font-size: 1.35rem;
  font-weight: 700;
  letter-spacing: -0.01em;
}
.onboarding-desc {
  margin-top: 8px;
  max-width: 560px;
  font-size: 0.92rem;
  line-height: 1.65;
  color: rgba(var(--v-theme-on-surface), 0.78);
}
.onboarding-tip {
  margin-top: 16px;
}
.onboarding-dots {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-top: 20px;
}
.onboarding-dot {
  width: 8px;
  height: 8px;
  padding: 0;
  border: 0;
  border-radius: 999px;
  background: rgba(var(--v-theme-on-surface), 0.2);
  cursor: pointer;
  transition:
    width 0.2s ease,
    background 0.2s ease;
}
.onboarding-dot.is-active {
  width: 24px;
  background: rgb(var(--v-theme-primary));
}
.onboarding-actions {
  padding: 12px 18px 18px;
}
</style>
