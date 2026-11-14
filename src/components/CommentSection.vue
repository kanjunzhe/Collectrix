<template>
  <v-card variant="flat">
    <v-card-title class="d-flex align-center">
      <span>{{ t("评论区") }}</span>
      <v-spacer />
      <v-btn size="small" variant="text" :loading="loading" @click="load(true)">
        {{ t("刷新") }}
      </v-btn>
    </v-card-title>
    <v-divider />

    <div v-if="!aid" class="pa-4 text-medium-emphasis text-center">
      {{ t("缺少视频 aid，无法加载评论。") }}
    </div>

    <div v-else class="comments-scroll">
      <div v-for="reply in replies" :key="reply.rpid" class="comment-row">
        <v-avatar size="36" class="comment-avatar">
          <v-img :src="reply.member.avatar" referrerpolicy="no-referrer" />
        </v-avatar>

        <div class="comment-body">
          <div class="comment-name">{{ reply.member.uname }}</div>

          <!-- 评论正文：pre-wrap 保留换行，支持 B 站表情 -->
          <div class="comment-text">
            <CommentContent
              :message="reply.content.message"
              :emotes="reply.content.emote"
            />
          </div>

          <div class="comment-meta">
            <span class="text-caption text-medium-emphasis"
              >👍 {{ reply.like }}</span
            >
            <v-btn
              v-if="reply.rcount > 0"
              size="x-small"
              variant="text"
              color="primary"
              @click="toggleSub(reply)"
            >
              {{
                expanded[reply.rpid]
                  ? t("收起回复")
                  : t("{n} 条回复", { n: reply.rcount })
              }}
            </v-btn>
          </div>

          <!-- 子回复：只在第二列里出现，不影响父级头像位置 -->
          <div v-if="expanded[reply.rpid]" class="sub-replies">
            <div v-if="subLoading[reply.rpid]" class="text-caption pa-2">
              {{ t("加载中…") }}
            </div>
            <div
              v-for="sub in subReplies[reply.rpid] || []"
              :key="sub.rpid"
              class="sub-row"
            >
              <v-avatar size="24" class="sub-avatar">
                <v-img :src="sub.member.avatar" referrerpolicy="no-referrer" />
              </v-avatar>
              <div class="sub-body">
                <div class="sub-name">{{ sub.member.uname }}</div>
                <div class="comment-text sub-text">
                  <CommentContent
                    :message="sub.content.message"
                    :emotes="sub.content.emote"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="pa-3 text-center">
        <v-btn
          v-if="nextOffset && !isEnd"
          variant="tonal"
          :loading="loading"
          @click="load(false)"
        >
          {{ t("加载更多") }}
        </v-btn>
        <span
          v-else-if="replies.length > 0"
          class="text-caption text-medium-emphasis"
        >
          {{ t("没有更多评论了") }}
        </span>
      </div>
    </div>
  </v-card>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import CommentContent from "./CommentContent.vue";
import { t } from "../i18n";

interface Reply {
  rpid: number;
  content: {
    message: string;
    emote?: Record<
      string,
      { url: string; width?: number | null; height?: number | null }
    > | null;
  };
  member: { uname: string; avatar: string };
  like: number;
  rcount: number;
}

const props = defineProps<{ aid: number | null }>();

const replies = ref<Reply[]>([]);
const nextOffset = ref<string | null>(null);
const isEnd = ref(false);
const loading = ref(false);

const expanded = reactive<Record<number, boolean>>({});
const subLoading = reactive<Record<number, boolean>>({});
const subReplies = reactive<Record<number, Reply[]>>({});

async function load(reset: boolean) {
  if (!props.aid || loading.value) return;
  loading.value = true;
  try {
    const page = await invoke<{
      replies: Reply[];
      next_offset: string | null;
      is_end: boolean;
    }>("get_comments", {
      aid: props.aid,
      offset: reset ? null : nextOffset.value,
    });
    if (reset) {
      replies.value = page.replies;
      // 重置子回复状态
      for (const k of Object.keys(expanded)) delete expanded[+k];
      for (const k of Object.keys(subReplies)) delete subReplies[+k];
    } else {
      replies.value.push(...page.replies);
    }
    nextOffset.value = page.next_offset;
    isEnd.value = page.is_end;
  } catch (e) {
    console.error(e);
  } finally {
    loading.value = false;
  }
}

async function toggleSub(reply: Reply) {
  if (expanded[reply.rpid]) {
    expanded[reply.rpid] = false;
    return;
  }
  expanded[reply.rpid] = true;
  if (subReplies[reply.rpid]) return;

  subLoading[reply.rpid] = true;
  try {
    subReplies[reply.rpid] = await invoke<Reply[]>("get_sub_replies", {
      aid: props.aid,
      rootRpid: reply.rpid,
    });
  } catch (e) {
    console.error(e);
  } finally {
    subLoading[reply.rpid] = false;
  }
}

watch(
  () => props.aid,
  (aid) => {
    if (aid) {
      replies.value = [];
      nextOffset.value = null;
      isEnd.value = false;
      load(true);
    }
  },
);

onMounted(() => load(true));
</script>

<style scoped>
.comments-scroll {
  max-height: 720px;
  overflow-y: auto;
}

.comment-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.12);
}

/* 关键：avatar 用 flex-shrink:0 + align-self:flex-start 固定左上角 */
.comment-avatar {
  flex-shrink: 0;
  align-self: flex-start;
}

.comment-body {
  flex: 1 1 auto;
  min-width: 0; /* 关键：允许内容在 flex 里正确换行 */
}

.comment-name {
  font-size: 13px;
  font-weight: 600;
  opacity: 0.85;
}

/* 评论正文：完整多行显示 */
.comment-text {
  font-size: 13px;
  line-height: 1.55;
  margin-top: 2px;
  white-space: pre-wrap; /* 保留换行 */
  word-break: break-word; /* 长英文/URL 可断行 */
  overflow-wrap: anywhere;
  color: rgba(var(--v-theme-on-surface), 0.87);
}

.comment-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.sub-replies {
  margin-top: 8px;
  padding-left: 8px;
  border-left: 2px solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.sub-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 6px 0;
}
.sub-avatar {
  flex-shrink: 0;
  align-self: flex-start;
}
.sub-body {
  flex: 1 1 auto;
  min-width: 0;
}
.sub-name {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.85;
}
.sub-text {
  font-size: 12px;
}
</style>
