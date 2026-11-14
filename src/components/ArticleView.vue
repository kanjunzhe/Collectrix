<template>
  <v-row>
    <v-col cols="12" lg="8">
      <v-card variant="outlined">
        <v-img
          v-if="coverUrl"
          :src="coverUrl"
          height="240"
          cover
          referrerpolicy="no-referrer"
        />
        <v-card-title class="text-h5 mt-3">
          {{ article?.title ?? video.title }}
        </v-card-title>
        <v-card-subtitle v-if="article?.author" class="mt-1">
          {{ article?.author }} · {{ article?.publish_time }}
          <v-chip
            v-if="article?.site_name"
            size="x-small"
            variant="tonal"
            class="ml-2"
          >
            {{ article?.site_name }}
          </v-chip>
        </v-card-subtitle>
        <v-divider class="my-3" />
        <v-card-text v-if="loading" class="text-center py-8">
          <v-progress-circular indeterminate color="primary" />
        </v-card-text>
        <v-card-text v-else class="article-content" v-html="sanitizedHtml" />
      </v-card>
    </v-col>

    <v-col cols="12" lg="4">
      <!-- 复用 PlayerView 的「元数据」卡片，此处省略 -->
    </v-col>
  </v-row>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import DOMPurify from "dompurify";
import type { VideoRow } from "../types";

interface ArticleData {
  title: string;
  author: string | null;
  cover_url: string | null;
  content_html: string;
  publish_time: string | null;
  site_name?: string;
}

const props = defineProps<{ video: VideoRow }>();
const article = ref<ArticleData | null>(null);
const loading = ref(false);

const coverUrl = computed(
  () => article.value?.cover_url ?? props.video.cover_url,
);

/** 必须净化：专栏正文来自 B站，直接 v-html 有 XSS 风险 */
const sanitizedHtml = computed(() =>
  article.value
    ? DOMPurify.sanitize(article.value.content_html, {
        ALLOWED_TAGS: [
          "p",
          "br",
          "strong",
          "em",
          "a",
          "img",
          "h1",
          "h2",
          "h3",
          "h4",
          "ul",
          "ol",
          "li",
          "blockquote",
          "code",
          "pre",
        ],
        ALLOWED_ATTR: ["href", "src", "alt", "title"],
      })
    : "",
);

onMounted(async () => {
  loading.value = true;
  try {
    if (props.video.platform === "bilibili_article") {
      article.value = await invoke<ArticleData>("fetch_bili_article_content", {
        cvId: props.video.platform_id,
      });
    } else if (props.video.original_url) {
      article.value = await invoke<ArticleData>("fetch_web_article", {
        url: props.video.original_url,
      });
    }
  } catch (e) {
    console.error("加载文章内容失败", e);
  } finally {
    loading.value = false;
  }
});
</script>

<style scoped>
.article-content {
  line-height: 1.8;
  font-size: 15px;
}
.article-content :deep(img) {
  max-width: 100%;
  border-radius: 6px;
  margin: 8px 0;
}
.article-content :deep(blockquote) {
  border-left: 3px solid rgba(128, 128, 128, 0.3);
  padding-left: 12px;
  color: rgba(128, 128, 128, 0.9);
}
</style>
