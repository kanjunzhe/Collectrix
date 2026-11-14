<template>
  <v-menu location="top end" offset="8">
    <template #activator="{ props }">
      <v-btn
        v-bind="props"
        class="add-fab"
        color="primary"
        icon
        size="large"
        elevation="6"
        :title="t('添加')"
      >
        <v-icon size="28">mdi-plus</v-icon>
      </v-btn>
    </template>
    <v-list density="compact">
      <v-list-item @click="$emit('select', 'single')">
        <template #prepend><v-icon>mdi-video-plus</v-icon></template>
        <v-list-item-title>{{ t("单个视频") }}</v-list-item-title>
      </v-list-item>
      <v-list-item @click="$emit('select', 'local')">
        <template #prepend><v-icon>mdi-harddisk-plus</v-icon></template>
        <v-list-item-title>{{ t("导入本地视频") }}</v-list-item-title>
      </v-list-item>
      <v-list-item @click="$emit('select', 'collection')">
        <template #prepend><v-icon>mdi-playlist-plus</v-icon></template>
        <v-list-item-title>{{ t("合集") }}</v-list-item-title>
      </v-list-item>
      <v-list-item @click="$emit('select', 'favorites')">
        <template #prepend><v-icon>mdi-folder-star</v-icon></template>
        <v-list-item-title>{{ t("收藏夹") }}</v-list-item-title>
      </v-list-item>
      <v-list-item @click="$emit('select', 'watchLater')">
        <template #prepend><v-icon>mdi-clock-outline</v-icon></template>
        <v-list-item-title>{{ t("稍后再看") }}</v-list-item-title>
      </v-list-item>
    </v-list>
  </v-menu>
</template>

<script setup lang="ts">
import { t } from "../i18n";

defineEmits<{
  select: [mode: "single" | "local" | "collection" | "favorites" | "watchLater"];
}>();
</script>

<style scoped>
.add-fab {
  position: fixed;
  right: 28px;
  bottom: 28px;
  z-index: 10;
}
</style>
