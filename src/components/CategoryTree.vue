<template>
  <v-list-group v-if="node.children.length" :value="node.id">
    <template #activator="{ props }">
      <v-list-item
        v-bind="props"
        :title="node.name"
        :active="selectedId === node.id"
        :class="{ 'category-drop-target': dragOver }"
        :data-category-id="node.id"
        prepend-icon="mdi-folder"
        @click.stop="$emit('select', node.id, node.name)"
        @contextmenu.prevent="$emit('contextmenu', $event, node)"
        @dragenter.prevent="onDragOver"
        @dragover.prevent="onDragOver"
        @dragleave="dragOver = false"
        @drop.prevent.stop="onDrop"
      >
        <template #append>
          <v-btn
            icon="mdi-delete-outline"
            size="x-small"
            variant="text"
            color="error"
            @click.stop="$emit('delete', node.id)"
          />
        </template>
      </v-list-item>
    </template>

    <CategoryTree
      v-for="child in node.children"
      :key="child.id"
      :node="child"
      :selected-id="selectedId"
      @select="(id, name) => $emit('select', id, name)"
      @delete="(id) => $emit('delete', id)"
      @contextmenu="(event, child) => $emit('contextmenu', event, child)"
      @drop="(videoId, categoryId) => $emit('drop', videoId, categoryId)"
    />
  </v-list-group>

  <v-list-item
    v-else
    :title="node.name"
    :active="selectedId === node.id"
    :class="{ 'category-drop-target': dragOver }"
    :data-category-id="node.id"
    prepend-icon="mdi-folder-outline"
    @click="$emit('select', node.id, node.name)"
    @contextmenu.prevent="$emit('contextmenu', $event, node)"
    @dragenter.prevent="onDragOver"
    @dragover.prevent="onDragOver"
    @dragleave="dragOver = false"
    @drop.prevent.stop="onDrop"
  >
    <template #append>
      <v-btn
        icon="mdi-delete-outline"
        size="x-small"
        variant="text"
        color="error"
        @click.stop="$emit('delete', node.id)"
      />
    </template>
  </v-list-item>
</template>

<script setup lang="ts">
import { ref } from "vue";

defineOptions({ name: "CategoryTree" });

const dragOver = ref(false);

interface CategoryNode {
  id: number;
  name: string;
  parent_id: number | null;
  children: CategoryNode[];
}

const props = defineProps<{
  node: CategoryNode;
  selectedId: number | null;
}>();

const emit = defineEmits<{
  select: [id: number, name: string];
  delete: [id: number];
  contextmenu: [event: MouseEvent, node: CategoryNode];
  drop: [videoId: number, categoryId: number];
}>();

function onDragOver(event: DragEvent) {
  event.preventDefault();
  dragOver.value = true;
  if (event.dataTransfer) event.dataTransfer.dropEffect = "move";
}

function onDrop(event: DragEvent) {
  dragOver.value = false;
  const raw =
    event.dataTransfer?.getData("application/x-collectrix-video-id") ||
    event.dataTransfer?.getData("text/plain") ||
    event.dataTransfer?.getData("text");
  const videoId = Number(raw);
  if (Number.isFinite(videoId) && videoId > 0) {
    emit("drop", videoId, props.node.id);
  }
}
</script>

<style scoped>
.category-drop-target {
  background: rgba(var(--v-theme-primary), 0.12);
  outline: 2px dashed rgba(var(--v-theme-primary), 0.55);
  outline-offset: -2px;
  border-radius: 6px;
}
</style>
