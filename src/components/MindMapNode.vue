<template>
  <div class="mind-node">
    <button
      type="button"
      class="mind-card"
      :class="[
        `mind-card--${node.kind}`,
        node.kind === 'video' ? 'mind-card--clickable' : '',
      ]"
      @click="onClick"
    >
      <v-icon size="18">{{ icon }}</v-icon>
      <span class="mind-label" :title="node.label">{{ node.label }}</span>
      <v-chip v-if="node.count !== undefined" size="x-small" variant="text">
        {{ node.count }}
      </v-chip>
      <v-icon v-if="node.children?.length" size="16">
        {{ open ? "mdi-chevron-down" : "mdi-chevron-right" }}
      </v-icon>
    </button>

    <div v-if="open && node.children?.length" class="mind-children">
      <MindMapNode
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        @play="$emit('play', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { MindMapNodeData, VideoRow } from "../types";

defineOptions({ name: "MindMapNode" });

const props = defineProps<{ node: MindMapNodeData }>();
const emit = defineEmits<{ play: [video: VideoRow] }>();

const open = ref(true);

const icon = computed(() => {
  switch (props.node.kind) {
    case "root":
      return "mdi-video-library";
    case "category":
      return props.node.children?.length ? "mdi-folder" : "mdi-folder-outline";
    case "more":
      return "mdi-dots-horizontal";
    default:
      return "mdi-play-circle-outline";
  }
});

function onClick() {
  if (props.node.kind === "video" && props.node.video) {
    emit("play", props.node.video);
  } else if (props.node.children?.length) {
    open.value = !open.value;
  }
}
</script>

<style scoped>
.mind-node {
  display: flex;
  align-items: center;
  gap: 14px;
}

.mind-card {
  display: flex;
  align-items: center;
  gap: 6px;
  max-width: 260px;
  padding: 7px 10px;
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 9px;
  background: rgb(var(--v-theme-surface));
  color: rgb(var(--v-theme-on-surface));
  text-align: left;
  font-size: 13px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease;
}

.mind-card:hover {
  transform: translateY(-1px);
  box-shadow: 0 5px 16px rgba(0, 0, 0, 0.12);
}

.mind-card--root {
  background: rgba(var(--v-theme-primary), 0.12);
  border-color: rgba(var(--v-theme-primary), 0.35);
  font-weight: 700;
  max-width: 180px;
}

.mind-card--category {
  background: rgba(var(--v-theme-secondary), 0.08);
}

.mind-card--video {
  opacity: 0.92;
}

.mind-card--more {
  opacity: 0.68;
  font-style: italic;
}

.mind-card--clickable {
  cursor: pointer;
}

.mind-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
  flex: 1 1 auto;
}

.mind-children {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-left: 16px;
  border-left: 2px solid rgba(var(--v-theme-primary), 0.22);
}

.mind-children > :deep(.mind-node) {
  position: relative;
}

.mind-children > :deep(.mind-node)::before {
  content: "";
  position: absolute;
  left: -16px;
  top: 50%;
  width: 14px;
  border-top: 2px solid rgba(var(--v-theme-primary), 0.22);
}
</style>
