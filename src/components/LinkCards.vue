<template>
  <div v-if="cards.length" class="link-cards">
    <v-card
      v-for="card in cards"
      :key="card.url"
      variant="outlined"
      class="link-card"
      :class="`link-card--${card.service}`"
      @click="open(card)"
    >
      <div class="link-icon">
        <v-icon size="24">{{ iconFor(card.service) }}</v-icon>
      </div>
      <div class="link-body">
        <div class="d-flex align-center flex-wrap" style="gap: 6px">
          <span class="link-service">{{ t(card.serviceLabel) }}</span>
          <v-chip
            v-if="card.password"
            size="x-small"
            color="warning"
            variant="flat"
          >
            {{ t("提取码：{code}", { code: card.password }) }}
          </v-chip>
        </div>
        <div class="link-title" :title="card.url">{{ card.title }}</div>
      </div>
      <v-btn
        icon
        size="small"
        variant="text"
        :title="
          card.password ? t('复制链接和提取码') : t('复制链接')
        "
        @click.stop="copy(card)"
      >
        <v-icon>mdi-content-copy</v-icon>
      </v-btn>
      <v-btn
        icon
        size="small"
        variant="text"
        :title="t('在浏览器打开')"
        @click.stop="open(card)"
      >
        <v-icon>mdi-open-in-new</v-icon>
      </v-btn>
    </v-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { linkCardsFromText, type LinkCard } from "../utils/linkCards";
import { notify } from "../composables/useUI";
import { t } from "../i18n";

const props = defineProps<{ text: string }>();

const cards = computed(() => linkCardsFromText(props.text));

function iconFor(service: LinkCard["service"]): string {
  switch (service) {
    case "github":
      return "mdi-github";
    case "google":
      return "mdi-google-drive";
    default:
      return "mdi-cloud-outline";
  }
}

async function copy(card: LinkCard) {
  const text = card.password
    ? `${card.urlWithPassword}\n${t("提取码：{code}", { code: card.password })}`
    : card.urlWithPassword;
  try {
    await navigator.clipboard.writeText(text);
    notify.success(t("已复制链接"));
  } catch (e) {
    notify.error(t("复制失败：{e}", { e: String(e) }));
  }
}

async function open(card: LinkCard) {
  try {
    await openUrl(card.urlWithPassword);
  } catch (e) {
    notify.error(t("打开链接失败：{e}", { e: String(e) }));
  }
}
</script>

<style scoped>
.link-cards {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 3px;
  margin: -3px -3px 9px;
  overflow: visible;
}

.link-card {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  cursor: pointer;
  overflow: visible;
  position: relative;
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease;
}

.link-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 18px rgba(0, 0, 0, 0.12);
  z-index: 2;
}

.link-card--github {
  background: linear-gradient(
    135deg,
    rgba(36, 41, 46, 0.06),
    rgba(88, 166, 255, 0.12)
  );
  border-color: rgba(88, 166, 255, 0.35);
}

.link-icon {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: rgba(var(--v-theme-primary), 0.1);
  color: rgb(var(--v-theme-primary));
}

.link-card--github .link-icon {
  background: #24292f;
  color: #fff;
}

.link-card--baidu .link-icon {
  background: rgba(41, 98, 255, 0.12);
  color: #2962ff;
}

.link-card--quark .link-icon {
  background: rgba(108, 92, 231, 0.12);
  color: #6c5ce7;
}

.link-body {
  min-width: 0;
  flex: 1 1 auto;
}

.link-service {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.75;
}

.link-title {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 1px;
}
</style>
