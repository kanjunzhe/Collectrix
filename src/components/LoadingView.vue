<template>
  <div class="loading-view">
    <div class="loading-orbit">
      <CurveLoader :size="96" :particles="46" />
      <img
        src="/collectrix-logo-flat.svg"
        alt="Collectrix"
        class="loading-logo"
      />
    </div>
    <div class="loading-title">Collectrix</div>
    <div class="loading-message">{{ message || t("正在加载…") }}</div>
  </div>
</template>

<script setup lang="ts">
import CurveLoader from "./CurveLoader.vue";
import { t } from "../i18n";

defineProps<{ message?: string }>();
</script>

<style scoped>
.loading-view {
  min-height: 320px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}
.loading-orbit {
  position: relative;
  width: 96px;
  height: 96px;
  display: grid;
  place-items: center;
}
.loading-orbit :deep(.curve-loader) {
  position: absolute;
  inset: 0;
}
.loading-logo {
  position: relative;
  width: 42px;
  height: 42px;
  border-radius: 11px;
  box-shadow: 0 8px 22px rgba(25, 118, 210, 0.2);
}
.loading-title {
  font-size: 1.25rem;
  font-weight: 700;
  letter-spacing: 0.02em;
}
.loading-message {
  font-size: 12px;
  color: rgb(var(--v-theme-on-surface));
  opacity: 0.6;
}
</style>
