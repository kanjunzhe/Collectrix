<template>
  <span class="comment-content">
    <template v-for="(segment, index) in segments" :key="index">
      <img
        v-if="segment.emote"
        :src="segment.emote"
        class="comment-emote"
        :style="segment.size"
        referrerpolicy="no-referrer"
        alt="emote"
      />
      <span v-else>{{ segment.text }}</span>
    </template>
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue";

interface EmoteInfo {
  url: string;
  width?: number | null;
  height?: number | null;
}

interface Segment {
  text: string;
  emote?: string;
  size?: Record<string, string>;
}

const props = defineProps<{
  message: string;
  emotes?: Record<string, EmoteInfo> | null;
}>();

const segments = computed<Segment[]>(() => {
  const result: Segment[] = [];
  const regex = /(\[[^\]]+\])/g;
  let last = 0;
  let match: RegExpExecArray | null;
  while ((match = regex.exec(props.message)) !== null) {
    if (match.index > last) {
      result.push({ text: props.message.slice(last, match.index) });
    }
    const emote = props.emotes?.[match[0]];
    if (emote?.url) {
      const max = 26;
      const width = emote.width && emote.width > 0 ? emote.width : max;
      const height = emote.height && emote.height > 0 ? emote.height : max;
      const scale = Math.min(1, max / Math.max(width, height));
      result.push({
        text: match[0],
        emote: emote.url,
        size: {
          width: `${Math.round(width * scale)}px`,
          height: `${Math.round(height * scale)}px`,
        },
      });
    } else {
      result.push({ text: match[0] });
    }
    last = match.index + match[0].length;
  }
  if (last < props.message.length) {
    result.push({ text: props.message.slice(last) });
  }
  return result;
});
</script>

<style scoped>
.comment-content {
  white-space: pre-wrap;
  word-break: break-word;
}
.comment-emote {
  display: inline-block;
  vertical-align: middle;
  margin: 0 1px;
  object-fit: contain;
}
</style>
