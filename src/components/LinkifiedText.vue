<template>
  <div class="linkified-text">
    <template v-for="(segment, index) in segments" :key="index">
      <a
        v-if="segment.url"
        class="text-link"
        :class="{ 'github-link': segment.github }"
        href="#"
        @click.prevent="open(segment.url)"
      >
        <v-icon v-if="segment.github" size="14" class="mr-1">mdi-github</v-icon>
        {{ segment.text }}
      </a>
      <span v-else>{{ segment.text }}</span>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{ text: string }>();

interface Segment {
  text: string;
  url?: string;
  github?: boolean;
}

const segments = computed<Segment[]>(() => {
  const result: Segment[] = [];
  const regex =
    /((?:https?:\/\/)?(?:www\.)?github\.com\/[^\s]+|https?:\/\/[^\s]+)/gi;
  let last = 0;
  let match: RegExpExecArray | null;
  while ((match = regex.exec(props.text)) !== null) {
    if (match.index > last) {
      result.push({ text: props.text.slice(last, match.index) });
    }
    const raw = match[0].replace(/[),.;!?，。；！？]+$/, "");
    const normalized = /^https?:\/\//i.test(raw) ? raw : `https://${raw}`;
    result.push({
      text: raw,
      url: normalized,
      github: /^https?:\/\/(www\.)?github\.com\//i.test(normalized),
    });
    last = match.index + match[0].length;
  }
  if (last < props.text.length) {
    result.push({ text: props.text.slice(last) });
  }
  return result;
});

async function open(url: string) {
  try {
    await openUrl(url);
  } catch (e) {
    notify.error(t("打开链接失败：{e}", { e: String(e) }));
  }
}
</script>

<style scoped>
.linkified-text {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.6;
  font-size: 13px;
}
.text-link {
  color: rgb(var(--v-theme-primary));
  text-decoration: underline;
  text-underline-offset: 2px;
  word-break: break-all;
}
.github-link {
  display: inline-flex;
  align-items: center;
  padding: 1px 6px;
  margin: 0 2px;
  border-radius: 5px;
  background: rgba(var(--v-theme-primary), 0.08);
  text-decoration: none;
  font-weight: 500;
}
</style>
