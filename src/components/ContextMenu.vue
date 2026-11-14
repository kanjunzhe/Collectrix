<template>
  <Teleport to="body">
    <Transition name="context-menu-pop">
      <div
        v-if="modelValue"
        class="context-menu-layer"
        :class="`v-theme--${theme.global.name.value}`"
        @mousedown.self="close"
        @contextmenu.prevent="close"
      >
        <v-list
          class="context-menu"
          :style="menuStyle"
          density="compact"
          elevation="8"
          :aria-busy="loading || opening"
          @mousedown.stop
          @contextmenu.stop.prevent
        >
          <v-progress-linear
            v-if="loading || opening"
            indeterminate
            color="primary"
            height="2"
            class="context-menu-progress"
          />
          <template v-for="(item, index) in items" :key="`${item.key}-${index}`">
            <v-divider v-if="item.divider" class="my-1" />
            <v-list-item
              v-else
              :disabled="item.disabled"
              :class="
                (item.color ? 'text-' + item.color + ' ' : '') +
                (opening ? 'context-menu-item--entering' : '')
              "
              :style="{ '--menu-item-index': index }"
              @click="select(item)"
            >
              <template v-if="item.icon" #prepend>
                <v-icon size="18">{{ item.icon }}</v-icon>
              </template>
              <v-list-item-title>{{ item.title }}</v-list-item-title>
            </v-list-item>
          </template>
        </v-list>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useTheme } from "vuetify";
import type { ContextMenuItem } from "../types";

const theme = useTheme();
const opening = ref(false);
let openingTimer: ReturnType<typeof setTimeout> | null = null;

const props = defineProps<{
  modelValue: boolean;
  x: number;
  y: number;
  items: ContextMenuItem[];
  loading?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  select: [key: string];
}>();

const menuStyle = computed(() => {
  const width = 220;
  const itemHeight = 38;
  const dividerHeight = 10;
  const height =
    props.items.reduce(
      (sum, item) => sum + (item.divider ? dividerHeight : itemHeight),
      16,
    ) || 48;
  const left = Math.max(
    8,
    Math.min(props.x, window.innerWidth - width - 8),
  );
  const top = Math.max(
    8,
    Math.min(props.y, window.innerHeight - height - 8),
  );
  return {
    left: `${left}px`,
    top: `${top}px`,
    width: `${width}px`,
  };
});

function close() {
  emit("update:modelValue", false);
}

function select(item: ContextMenuItem) {
  if (item.disabled || item.divider) return;
  emit("select", item.key);
  close();
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") close();
}

watch(
  () => props.modelValue,
  (open) => {
    if (openingTimer) {
      clearTimeout(openingTimer);
      openingTimer = null;
    }
    if (!open) {
      opening.value = false;
      return;
    }
    opening.value = true;
    openingTimer = setTimeout(() => {
      opening.value = false;
      openingTimer = null;
    }, 220);
  },
);

onMounted(() => window.addEventListener("keydown", onKeydown));
onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
  if (openingTimer) clearTimeout(openingTimer);
});
</script>

<style scoped>
.context-menu-layer {
  position: fixed;
  inset: 0;
  z-index: 10000;
}

.context-menu {
  position: fixed;
  overflow: hidden;
  border: 1px solid rgba(128, 128, 128, 0.2);
  border-radius: 8px;
  background: rgb(var(--v-theme-surface, 255 255 255));
  color: rgb(var(--v-theme-on-surface, 27 27 31));
}

.context-menu-progress {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 2;
}

.context-menu-item--entering {
  animation: menu-item-in 180ms ease-out both;
  animation-delay: calc(var(--menu-item-index) * 16ms);
}

.context-menu-pop-enter-active {
  transition:
    opacity 120ms ease-out,
    transform 120ms ease-out;
}

.context-menu-pop-leave-active {
  transition:
    opacity 80ms ease-in,
    transform 80ms ease-in;
}

.context-menu-pop-enter-from,
.context-menu-pop-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.96);
}

@keyframes menu-item-in {
  from {
    opacity: 0;
    transform: translateY(-2px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
