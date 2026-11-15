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
        <v-card-subtitle v-if="article?.author || video.author" class="mt-1">
          {{ article?.author ?? video.author }}
          <span v-if="article?.publish_time"> · {{ article?.publish_time }}</span>
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
          <div class="text-caption text-medium-emphasis mt-2">
            {{ t("正在解析项目详情…") }}
          </div>
        </v-card-text>
        <v-card-text v-else>
          <v-alert
            v-if="error"
            type="warning"
            variant="tonal"
            density="compact"
            class="mb-3"
          >
            {{ t("加载项目详情失败：{e}", { e: error }) }}
          </v-alert>
          <div
            v-if="sanitizedHtml"
            class="article-content"
            v-html="sanitizedHtml"
          />
          <div
            v-else-if="video.description"
            class="fallback-description"
          >
            {{ video.description }}
          </div>
          <div v-else class="text-medium-emphasis">
            {{ t("暂无详细内容，请打开原网页查看。") }}
          </div>
          <v-btn
            v-if="articleUrl"
            class="mt-4"
            variant="tonal"
            prepend-icon="mdi-open-in-new"
            @click="openLink(articleUrl)"
          >
            {{ t("打开原网页") }}
          </v-btn>
        </v-card-text>
      </v-card>
    </v-col>

    <v-col cols="12" lg="4">
      <v-card v-if="github" variant="outlined" class="mb-3">
        <v-card-title class="text-subtitle-1">{{ t("项目信息") }}</v-card-title>
        <v-card-text>
          <div class="text-subtitle-2 font-weight-bold text-break">
            {{ github.full_name }}
          </div>
          <div
            v-if="github.description"
            class="text-body-2 text-medium-emphasis mt-1"
          >
            {{ github.description }}
          </div>

          <div class="github-stats mt-3">
            <v-chip size="small" variant="tonal" color="warning">
              <v-icon start size="14">mdi-star</v-icon>
              {{ github.stars }} Stars
            </v-chip>
            <v-chip size="small" variant="tonal">
              <v-icon start size="14">mdi-source-fork</v-icon>
              {{ github.forks }} Forks
            </v-chip>
            <v-chip size="small" variant="tonal">
              <v-icon start size="14">mdi-eye-outline</v-icon>
              {{ github.watchers }} Watchers
            </v-chip>
            <v-chip size="small" variant="tonal">
              <v-icon start size="14">mdi-alert-circle-outline</v-icon>
              {{ github.issues }} Issues
            </v-chip>
          </div>

          <div class="meta-grid mt-3">
            <span class="meta-label">{{ t("语言") }}</span>
            <span class="meta-value">{{ github.language || "—" }}</span>
            <span class="meta-label">{{ t("许可证") }}</span>
            <span class="meta-value">{{ github.license || "—" }}</span>
            <span class="meta-label">{{ t("默认分支") }}</span>
            <span class="meta-value">{{ github.default_branch }}</span>
            <span class="meta-label">{{ t("创建时间") }}</span>
            <span class="meta-value">{{ formatDate(github.created_at) }}</span>
            <span class="meta-label">{{ t("最近更新") }}</span>
            <span class="meta-value">{{ formatDate(github.updated_at) }}</span>
          </div>

          <v-alert
            v-if="github.archived"
            type="warning"
            variant="tonal"
            density="compact"
            class="mt-3"
          >
            {{ t("该仓库已归档") }}
          </v-alert>

          <div v-if="github.topics.length" class="mt-3">
            <div class="meta-label mb-1">{{ t("Topics") }}</div>
            <div class="d-flex flex-wrap" style="gap: 4px">
              <v-chip
                v-for="topic in github.topics"
                :key="topic"
                size="x-small"
                variant="outlined"
              >
                {{ topic }}
              </v-chip>
            </div>
          </div>

          <div class="d-flex flex-wrap mt-4" style="gap: 8px">
            <v-btn
              v-if="github.homepage"
              size="small"
              variant="tonal"
              prepend-icon="mdi-home-outline"
              @click="openLink(github.homepage)"
            >
              {{ t("主页") }}
            </v-btn>
            <v-btn
              size="small"
              variant="tonal"
              prepend-icon="mdi-github"
              @click="openLink(github.html_url)"
            >
              {{ t("打开项目") }}
            </v-btn>
          </div>
        </v-card-text>
      </v-card>

      <v-card variant="outlined" class="mb-3">
        <v-card-title class="text-subtitle-1">{{ t("收藏信息") }}</v-card-title>
        <v-card-text>
          <div class="meta-grid">
            <span class="meta-label">{{ t("平台") }}</span>
            <span class="meta-value">{{ t(platform.name) }}</span>
            <span class="meta-label">{{ t("作者") }}</span>
            <span class="meta-value">{{ video.author || t("未知作者") }}</span>
          </div>
          <div class="mt-3">
            <div class="meta-label mb-1">{{ t("标签") }}</div>
            <div class="d-flex flex-wrap" style="gap: 4px">
              <v-chip
                v-for="tag in video.tags"
                :key="tag.id"
                size="x-small"
                :color="tag.color"
                variant="flat"
              >
                {{ tag.name }}
              </v-chip>
              <span
                v-if="video.tags.length === 0"
                class="text-caption text-medium-emphasis"
              >
                {{ t("（无标签）") }}
              </span>
            </div>
          </div>
        </v-card-text>
      </v-card>

      <v-card variant="outlined">
        <v-card-title class="text-subtitle-1">{{ t("描述") }}</v-card-title>
        <v-card-text class="desc-box">
          {{ video.description || t("（无描述）") }}
        </v-card-text>
      </v-card>
    </v-col>
  </v-row>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import DOMPurify from "dompurify";
import type { VideoRow } from "../types";
import { getPlatform } from "../platforms";
import { t } from "../i18n";

interface GithubInfo {
  full_name: string;
  owner: string | null;
  avatar_url: string | null;
  description: string | null;
  language: string | null;
  stars: number;
  forks: number;
  watchers: number;
  issues: number;
  license: string | null;
  homepage: string | null;
  default_branch: string;
  archived: boolean;
  topics: string[];
  created_at: string | null;
  updated_at: string | null;
  pushed_at: string | null;
  html_url: string;
}

interface ArticleData {
  title: string;
  author: string | null;
  cover_url: string | null;
  description?: string | null;
  content_html: string;
  publish_time: string | null;
  site_name?: string;
  github?: GithubInfo | null;
}

const props = defineProps<{ video: VideoRow }>();

const platform = computed(() => getPlatform(props.video.platform));
const article = ref<ArticleData | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

const articleUrl = computed(() => {
  const original = props.video.original_url?.trim();
  if (original) {
    return /^https?:\/\//i.test(original) ? original : `https://${original}`;
  }
  if (props.video.platform === "github") {
    return `https://github.com/${props.video.platform_id}`;
  }
  if (
    props.video.platform === "blog" &&
    /^https?:\/\//i.test(props.video.platform_id)
  ) {
    return props.video.platform_id;
  }
  return "";
});

const coverUrl = computed(
  () => article.value?.cover_url ?? props.video.cover_url,
);

const github = computed(() => article.value?.github ?? null);

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
          "table",
          "thead",
          "tbody",
          "tr",
          "th",
          "td",
          "hr",
          "del",
          "s",
          "details",
          "summary",
          "sup",
          "sub",
          "div",
          "span",
          "article",
          "section",
          "figure",
          "figcaption",
        ],
        ALLOWED_ATTR: ["href", "src", "alt", "title"],
      })
    : "",
);

function formatDate(value: string | null): string {
  if (!value) return "—";
  return value.replace("T", " ").replace("Z", "").slice(0, 16);
}

async function openLink(url: string | null) {
  if (!url) return;
  const target = /^https?:\/\//i.test(url) ? url : `https://${url}`;
  try {
    await openUrl(target);
  } catch (e) {
    console.error("打开链接失败", e);
  }
}

async function loadArticle() {
  article.value = null;
  error.value = null;

  if (!articleUrl.value) {
    loading.value = false;
    return;
  }

  loading.value = true;
  try {
    if (props.video.platform === "bilibili_article") {
      article.value = await invoke<ArticleData>("fetch_bili_article_content", {
        cvId: props.video.platform_id,
      });
    } else {
      article.value = await invoke<ArticleData>("fetch_web_article", {
        url: articleUrl.value,
      });
    }
  } catch (e) {
    error.value = String(e);
    console.error("加载文章内容失败", e);
  } finally {
    loading.value = false;
  }
}

watch(
  () => [props.video.id, articleUrl.value],
  () => {
    void loadArticle();
  },
  { immediate: true },
);
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
.article-content :deep(table) {
  width: 100%;
  border-collapse: collapse;
  margin: 12px 0;
  font-size: 13px;
}
.article-content :deep(th),
.article-content :deep(td) {
  border: 1px solid rgba(128, 128, 128, 0.25);
  padding: 6px 8px;
  text-align: left;
}
.article-content :deep(th) {
  background: rgba(var(--v-theme-primary), 0.08);
}
.article-content :deep(details) {
  margin: 10px 0;
}
.article-content :deep(summary) {
  cursor: pointer;
  font-weight: 600;
}
.fallback-description {
  white-space: pre-wrap;
  line-height: 1.8;
  font-size: 15px;
}
.github-stats {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.meta-grid {
  display: grid;
  grid-template-columns: 92px minmax(0, 1fr);
  gap: 6px 10px;
  font-size: 13px;
}
.meta-label {
  color: rgba(var(--v-theme-on-surface), 0.6);
}
.meta-value {
  min-width: 0;
  overflow-wrap: anywhere;
}
.desc-box {
  white-space: pre-wrap;
  line-height: 1.6;
}
</style>
