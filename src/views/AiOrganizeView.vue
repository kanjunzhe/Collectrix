<template>
  <div style="max-width: 1080px">
    <div class="text-h5 font-weight-bold mb-4">{{ t("AI 智能整理") }}</div>

    <!-- 连接状态 -->
    <v-card variant="outlined" class="mb-4">
      <v-card-text>
        <div class="d-flex align-center flex-wrap" style="gap: 12px">
          <v-chip
            :color="providerStatus.ok ? 'success' : 'warning'"
            variant="flat"
          >
            <v-icon start size="16">
              {{ providerStatus.ok ? "mdi-check-circle" : "mdi-alert-circle" }}
            </v-icon>
            {{ t(providerStatus.label) }}
          </v-chip>
          <span class="text-caption text-medium-emphasis flex-grow-1">
            {{ t(providerStatus.detail) }}
          </span>
          <v-btn
            size="small"
            variant="tonal"
            prepend-icon="mdi-connection"
            :loading="pingLoading"
            @click="onPing"
          >
            {{ t("检测连接") }}
          </v-btn>
          <v-btn
            v-if="settings?.ai_provider === 'ollama'"
            size="small"
            variant="tonal"
            prepend-icon="mdi-play-circle"
            :loading="launching"
            @click="onLaunchOllama"
          >
            {{ t("启动 Ollama") }}
          </v-btn>
        </div>
        <div class="text-caption text-medium-emphasis mt-2">
          {{
            t("模型：{model} · Base URL：{url}", {
              model: settings?.ai_model || "—",
              url: settings?.openai_base_url || "—",
            })
          }}
        </div>
      </v-card-text>
    </v-card>

    <!-- 操作栏 -->
    <div class="d-flex align-center mb-3 flex-wrap" style="gap: 8px">
      <div class="text-subtitle-1 font-weight-medium">
        {{ t("待处理建议") }}
        <span class="text-caption text-medium-emphasis">{{
          t("（{n} 条）", { n: suggestions.length })
        }}</span>
      </div>
      <v-spacer />
      <v-text-field
        v-model.number="batchSize"
        type="number"
        :label="t('本次数量')"
        density="compact"
        variant="outlined"
        hide-details
        style="max-width: 120px"
        :min="1"
        :max="50"
      />
      <v-btn
        variant="tonal"
        prepend-icon="mdi-format-list-bulleted"
        :loading="listingModels"
        @click="onListModels"
      >
        {{ t("查看可用模型") }}
      </v-btn>
      <v-btn
        color="primary"
        prepend-icon="mdi-robot"
        :loading="analyzing"
        :disabled="analyzing"
        @click="onAnalyze"
      >
        {{ analyzing ? t("分析中…") : t("开始分析") }}
      </v-btn>
      <v-btn
        v-if="analyzing"
        variant="tonal"
        color="error"
        prepend-icon="mdi-stop"
        @click="onCancel"
      >
        {{ t("停止") }}
      </v-btn>
      <v-btn
        v-if="suggestions.length > 0"
        variant="tonal"
        color="success"
        prepend-icon="mdi-check-all"
        @click="onAcceptAll"
      >
        {{ t("全部接受") }}
      </v-btn>
    </div>

    <div
      v-if="analyzing"
      class="d-flex align-center text-caption text-medium-emphasis mb-3"
      style="gap: 8px"
    >
      <v-progress-circular indeterminate size="14" width="2" />
      <span>{{ taskProgress || t("正在分析…") }}</span>
    </div>

    <!-- 日志面板 -->
    <v-expansion-panels v-model="logPanel" class="mb-4">
      <v-expansion-panel>
        <v-expansion-panel-title>
          <v-icon size="18" class="mr-2">mdi-text-box-outline</v-icon>
          <span class="text-body-2 font-weight-medium">{{ t("操作日志") }}</span>
          <v-chip size="x-small" class="ml-2" variant="text">{{
            t("{n} 条", { n: logs.length })
          }}</v-chip>
          <v-spacer />
          <v-btn
            size="x-small"
            variant="text"
            :disabled="logs.length === 0"
            @click.stop="clearLogs"
          >
            {{ t("清空") }}
          </v-btn>
        </v-expansion-panel-title>
        <v-expansion-panel-text>
          <div ref="logsEl" class="logs-box">
            <div
              v-if="logs.length === 0"
              class="text-caption text-medium-emphasis pa-2"
            >
              {{ t("暂无日志") }}
            </div>
            <div
              v-for="(l, i) in logs"
              :key="i"
              class="log-line"
              :class="`log-${l.level}`"
            >
              <span class="log-time">{{ l.time }}</span>
              <span class="log-level">[{{ l.level }}]</span>
              <span class="log-msg">{{ l.message }}</span>
            </div>
          </div>
        </v-expansion-panel-text>
      </v-expansion-panel>
    </v-expansion-panels>

    <!-- 空态 -->
    <div
      v-if="suggestions.length === 0 && !analyzing"
      class="pa-6 text-center text-medium-emphasis"
    >
      <v-icon size="48" opacity="0.3">mdi-robot-outline</v-icon>
      <div class="mt-2">
        {{ t("点击「开始分析」让 AI 对未分类视频给出建议。") }}
      </div>
      <div class="text-caption mt-1">
        {{ t("如果点击后没有反应，请展开上方「操作日志」查看具体错误。") }}
      </div>
    </div>

    <!-- 建议卡片 -->
    <v-card
      v-for="(s, idx) in suggestions"
      :key="s.video_id"
      variant="outlined"
      class="mb-3"
    >
      <v-card-text>
        <div class="d-flex align-start" style="gap: 12px">
          <!-- 缩略图 -->
          <div class="thumb" @click="openPreview(s)">
            <img
              v-if="s.cover_url"
              :src="s.cover_url"
              referrerpolicy="no-referrer"
              alt="cover"
            />
            <div v-else class="thumb-placeholder">
              <v-icon size="28" opacity="0.3">mdi-video-off-outline</v-icon>
            </div>
            <div class="thumb-play">
              <v-icon color="white" size="28">mdi-play-circle</v-icon>
            </div>
          </div>

          <!-- 标题 / 概要 -->
          <div class="flex-grow-1 min-w-0">
            <div
              class="text-subtitle-2 font-weight-medium text-truncate"
              :title="s.title"
            >
              {{ s.title }}
            </div>
            <div
              v-if="!isEditing(s.video_id)"
              class="text-caption text-medium-emphasis mt-1"
              style="white-space: pre-wrap; word-break: break-word"
            >
              {{ s.summary || t("（无概要）") }}
            </div>
          </div>

          <v-chip
            :color="
              s.confidence > 0.7
                ? 'success'
                : s.confidence > 0.4
                  ? 'warning'
                  : 'error'
            "
            size="small"
            variant="flat"
          >
            {{ (s.confidence * 100).toFixed(0) }}%
          </v-chip>
        </div>

        <v-divider class="my-3" />

        <!-- 只读模式 -->
        <div
          v-if="!isEditing(s.video_id)"
          class="d-flex flex-wrap align-center"
          style="gap: 8px"
        >
          <span class="text-caption text-medium-emphasis">{{ t("分类：") }}</span>
          <v-chip
            size="small"
            :color="resolveCategoryChipColor(s)"
            variant="flat"
          >
            <v-icon start size="12">{{ resolveCategoryChipIcon(s) }}</v-icon>
            {{ resolveCategoryDisplay(s) }}
          </v-chip>

          <span class="text-caption text-medium-emphasis ml-3">{{ t("标签：") }}</span>
          <v-chip
            v-for="tag in s.suggested_tags"
            :key="tag"
            size="small"
            :color="isExistingTag(tag) ? undefined : 'success'"
            :variant="isExistingTag(tag) ? 'tonal' : 'flat'"
          >
            <v-icon v-if="!isExistingTag(tag)" start size="12"
              >mdi-tag-plus</v-icon
            >
            {{ tag }}
          </v-chip>
          <span
            v-if="s.suggested_tags.length === 0"
            class="text-caption text-medium-emphasis"
          >
            {{ t("（无标签）") }}
          </span>
        </div>

        <!-- 编辑模式 -->
        <div v-else class="d-flex flex-column" style="gap: 12px">
          <!-- 父分类 -->
          <v-select
            v-model="draftParentId"
            :items="parentCategoryItems"
            item-title="title"
            item-value="value"
            :label="t('父分类（可留空 = 顶级分类）')"
            density="compact"
            variant="outlined"
            hide-details
            clearable
          />

          <!-- 分类名 -->
          <v-combobox
            v-model="draftCategory"
            :items="categoryItems"
            item-title="title"
            item-subtitle="subtitle"
            item-value="value"
            :label="t('分类（可从下拉选已有，或输入新名字创建）')"
            density="compact"
            variant="outlined"
            hide-details
            return-object
            clearable
          />
          <div class="text-caption text-medium-emphasis">
            {{
              t(
                "可输入「父/子/孙」一次性创建多级分类；选择父分类后会创建在它下面",
              )
            }}
          </div>

          <!-- 标签 -->
          <v-combobox
            v-model="draftTags"
            :items="existingTagList"
            :label="t('标签（可多选，回车添加新标签）')"
            multiple
            chips
            closable-chips
            density="compact"
            variant="outlined"
            hide-details
          />

          <!-- 概要 -->
          <v-textarea
            v-model="draftSummary"
            :label="t('概要')"
            rows="2"
            auto-grow
            density="compact"
            variant="outlined"
            hide-details
          />
        </div>
      </v-card-text>
      <v-divider />
      <v-card-actions>
        <v-spacer />
        <template v-if="isEditing(s.video_id)">
          <v-btn size="small" variant="text" @click="cancelEdit">{{
            t("取消")
          }}</v-btn>
          <v-btn
            size="small"
            color="primary"
            variant="tonal"
            prepend-icon="mdi-content-save"
            @click="saveEdit(idx)"
          >
            {{ t("保存修改") }}
          </v-btn>
        </template>
        <template v-else>
          <v-btn
            size="small"
            variant="text"
            prepend-icon="mdi-play-circle-outline"
            @click="openPreview(s)"
          >
            {{ t("预览") }}
          </v-btn>
          <v-btn
            size="small"
            variant="text"
            color="error"
            @click="onReject(idx)"
          >
            {{ t("忽略") }}
          </v-btn>
          <v-btn
            size="small"
            variant="text"
            prepend-icon="mdi-pencil"
            @click="startEdit(s)"
          >
            {{ t("编辑") }}
          </v-btn>
          <v-btn
            size="small"
            color="success"
            variant="tonal"
            prepend-icon="mdi-check"
            @click="onAccept(idx)"
          >
            {{ t("接受") }}
          </v-btn>
        </template>
      </v-card-actions>
    </v-card>

    <!-- 可用模型弹窗 -->
    <v-dialog v-model="showModelsDialog" max-width="560">
      <v-card>
        <v-card-title class="d-flex align-center">
          <v-icon size="20" class="mr-2">mdi-format-list-bulleted</v-icon>
          {{ t("可用模型（点击直接使用）") }}
          <v-spacer />
          <v-btn
            icon
            size="small"
            variant="text"
            :loading="listingModels"
            @click="onListModels"
          >
            <v-icon>mdi-refresh</v-icon>
          </v-btn>
        </v-card-title>
        <v-divider />
        <v-card-text style="max-height: 420px; overflow-y: auto">
          <div
            v-if="availableModels.length === 0"
            class="pa-4 text-center text-medium-emphasis"
          >
            <v-icon size="40" opacity="0.3">mdi-server-off</v-icon>
            <div class="mt-2 text-caption">
              {{ t("没有获取到模型，请检查 Base URL 和 API Key。") }}
            </div>
          </div>
          <v-list v-else density="compact">
            <v-list-item
              v-for="m in availableModels"
              :key="m"
              :active="m === settings?.ai_model"
              @click="useModel(m)"
            >
              <v-list-item-title class="text-body-2">{{ m }}</v-list-item-title>
              <template #append>
                <v-chip
                  v-if="m === settings?.ai_model"
                  size="x-small"
                  color="primary"
                  variant="flat"
                >
                  {{ t("当前") }}
                </v-chip>
              </template>
            </v-list-item>
          </v-list>
        </v-card-text>
        <v-card-actions>
          <span class="text-caption text-medium-emphasis ml-2">
            {{ t("共 {n} 个模型", { n: availableModels.length }) }}
          </span>
          <v-spacer />
          <v-btn @click="showModelsDialog = false">{{ t("关闭") }}</v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>

    <VideoPreviewDialog v-model="showPreview" :video="previewVideo" />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  AiSuggestion,
  AppSettings,
  CategoryFlat,
  CategoryNode,
  TagInfo,
  VideoRow,
} from "../types";
import { notify, confirmDialog } from "../composables/useUI";
import VideoPreviewDialog from "../components/VideoPreviewDialog.vue";
import { t } from "../i18n";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

let unlistenAiTask: UnlistenFn | null = null;

const emit = defineEmits<{ "videos-changed": [] }>();

type LogLevel = "info" | "success" | "warn" | "error";
interface LogEntry {
  time: string;
  level: LogLevel;
  message: string;
}

const settings = ref<AppSettings | null>(null);
const suggestions = ref<AiSuggestion[]>([]);
const previewVideo = ref<VideoRow | null>(null);
const showPreview = ref(false);
const batchSize = ref(15);
const analyzing = ref(false);
const canceled = ref(false);
const taskProgress = ref("");
const launching = ref(false);
const pingLoading = ref(false);

const providerStatus = ref({
  ok: false,
  label: "未检测",
  detail: "点击「检测连接」检查 AI 服务是否可用",
});

/** 后端进度是中文文本，这里转换成当前语言。 */
function localizedProgress(progress: string): string {
  if (progress === "正在读取待处理视频…") {
    return t("正在读取待处理视频…");
  }
  if (progress === "分析失败") return t("分析失败");

  let match = progress.match(/^正在分析 (\d+)\/(\d+)（本批 (\d+) 个）…$/);
  if (match) {
    return t("正在分析 {done}/{total}（本批 {n} 个）…", {
      done: match[1],
      total: match[2],
      n: match[3],
    });
  }
  match = progress.match(/^已分析 (\d+)\/(\d+)，当前共 (\d+) 条建议$/);
  if (match) {
    return t("已分析 {done}/{total}，当前共 {n} 条建议", {
      done: match[1],
      total: match[2],
      n: match[3],
    });
  }
  match = progress.match(/^分析完成，共 (\d+) 条建议$/);
  if (match) {
    return t("分析完成，共 {n} 条建议", { n: match[1] });
  }
  match = progress.match(/^还剩 (\d+) 条建议$/);
  if (match) {
    return t("还剩 {n} 条建议", { n: match[1] });
  }
  return progress;
}

const STORAGE_KEY = "aiOrganize.state.v1";

interface PersistedState {
  suggestions: AiSuggestion[];
  logs: LogEntry[];
  editingVideoId: number | null;
  draftParentId: number | null;
  draftCategory: any;
  draftTags: string[];
  draftSummary: string;
  batchSize: number;
}

// ============ 模型列表 ============

const listingModels = ref(false);
const availableModels = ref<string[]>([]);
const showModelsDialog = ref(false);

async function onListModels() {
  if (!settings.value?.openai_base_url) {
    notify.error(t("请先在设置页配置 Base URL"));
    return;
  }
  listingModels.value = true;
  log(
    "info",
    t("正在获取模型列表（{url}）", {
      url: settings.value.openai_base_url,
    }),
  );
  try {
    availableModels.value = await invoke<string[]>("list_ai_models");
    log(
      "success",
      t("获取到 {n} 个可用模型", { n: availableModels.value.length }),
    );
    if (availableModels.value.length === 0) {
      notify.warning(t("服务端没有返回任何模型"));
    } else {
      showModelsDialog.value = true;
    }
  } catch (e) {
    log("error", t("获取模型列表失败：{e}", { e: String(e) }));
    notify.error(t("获取模型列表失败：{e}", { e: String(e) }));
  } finally {
    listingModels.value = false;
  }
}

async function useModel(modelId: string) {
  if (!settings.value) return;
  try {
    settings.value.ai_model = modelId;
    await invoke("set_settings", { settings: settings.value });
    log("success", t("已切换模型：{model}", { model: modelId }));
    notify.success(t("已切换模型：{model}", { model: modelId }));
    showModelsDialog.value = false;
  } catch (e) {
    log("error", t("保存模型失败：{e}", { e: String(e) }));
    notify.error(t("保存失败：{e}", { e: String(e) }));
  }
}

// ============ 分类树 / 标签列表 ============

const flatCategories = ref<CategoryFlat[]>([]);
const existingTagNames = ref<Set<string>>(new Set());

const existingTagList = computed(() =>
  Array.from(existingTagNames.value).sort(),
);

async function loadCategoryTree() {
  try {
    const tree = await invoke<CategoryNode[]>("get_category_tree");
    const flat: CategoryFlat[] = [];
    const walk = (
      nodes: CategoryNode[],
      parentPath: string[],
      depth: number,
    ) => {
      for (const n of nodes) {
        const pathParts = [...parentPath, n.name];
        flat.push({
          id: n.id,
          name: n.name,
          parent_id: n.parent_id,
          path: pathParts.join(" / "),
          depth,
        });
        walk(n.children, pathParts, depth + 1);
      }
    };
    walk(tree, [], 0);
    flatCategories.value = flat;
  } catch (e) {
    log("warn", t("加载分类树失败：{e}", { e: String(e) }));
  }
}

async function loadTagList() {
  try {
    const tags = await invoke<TagInfo[]>("list_tags");
    existingTagNames.value = new Set(tags.map((t) => t.name));
  } catch (e) {
    log("warn", t("加载标签列表失败：{e}", { e: String(e) }));
  }
}

async function loadExistingNames() {
  await Promise.all([loadCategoryTree(), loadTagList()]);
}

function isExistingTag(name: string) {
  return existingTagNames.value.has(name);
}

// 父分类下拉：所有分类 + "顶级"
const parentCategoryItems = computed(() => [
  { title: t("（顶级分类）"), subtitle: "", value: null as number | null },
  ...flatCategories.value.map((c) => ({
    title: "　".repeat(c.depth) + c.name,
    subtitle: c.path,
    value: c.id as number | null,
  })),
]);

// 分类下拉：根据当前父分类动态生成
const categoryItems = computed(() => {
  const pid = draftParentId.value;
  if (pid === null || pid === undefined) {
    return flatCategories.value
      .filter((c) => c.parent_id === null)
      .map((c) => ({
        title: c.name,
        subtitle: c.path,
        value: c.id,
      }));
  }
  return flatCategories.value
    .filter((c) => c.parent_id === pid)
    .map((c) => ({
      title: c.name,
      subtitle: c.path,
      value: c.id,
    }));
});

// ============ 编辑状态 ============

const editingVideoId = ref<number | null>(null);
const draftParentId = ref<number | null>(null);
const draftCategory = ref<any>(null);
const draftTags = ref<string[]>([]);
const draftSummary = ref("");

function isEditing(videoId: number) {
  return editingVideoId.value === videoId;
}

function startEdit(s: AiSuggestion) {
  draftParentId.value = s.parent_category_id ?? null;

  if (s.category_path && s.category_path.length > 0) {
    draftParentId.value = null;
    draftCategory.value = s.category_path.join(" / ");
  } else if (s.suggested_category_id) {
    // 已选中某个已有分类 → 父分类取其真实 parent_id
    const existing = flatCategories.value.find(
      (c) => c.id === s.suggested_category_id,
    );
    draftParentId.value = existing?.parent_id ?? s.parent_category_id ?? null;
    draftCategory.value = existing
      ? { title: existing.name, subtitle: existing.path, value: existing.id }
      : s.suggested_category;
  } else {
    draftCategory.value = s.suggested_category;
  }

  draftTags.value = [...s.suggested_tags];
  draftSummary.value = s.summary;
  editingVideoId.value = s.video_id;
}

function cancelEdit() {
  editingVideoId.value = null;
  draftParentId.value = null;
  draftCategory.value = null;
  draftTags.value = [];
  draftSummary.value = "";
}

// 切换父分类时，如果分类当前是对象（已有项），转成字符串保留名字
watch(draftParentId, () => {
  if (draftCategory.value && typeof draftCategory.value === "object") {
    draftCategory.value = draftCategory.value.title;
  }
});

function saveEdit(idx: number) {
  const s = suggestions.value[idx];
  if (!s) return;

  let categoryId: number | null = null;
  let categoryName = "";
  let categoryPath: string[] | null = null;

  if (draftCategory.value && typeof draftCategory.value === "object") {
    categoryId = draftCategory.value.value;
    categoryName = draftCategory.value.title;
  } else if (typeof draftCategory.value === "string") {
    categoryName = draftCategory.value.trim();
    const typedParts = categoryName
      .split("/")
      .map((part) => part.trim())
      .filter((part) => part.length > 0);
    if (typedParts.length > 1) {
      const parentPath = draftParentId.value
        ? (flatCategories.value.find((c) => c.id === draftParentId.value)?.path ??
            "")
            .split(" / ")
            .filter((part) => part.length > 0)
        : [];
      categoryPath = [...parentPath, ...typedParts];
      categoryName = typedParts[typedParts.length - 1];
    }
  }

  if (!categoryId && !categoryName) {
    notify.warning(
      t("请指定分类（可以从下拉里选，也可以输入新名字）"),
    );
    return;
  }

  const originalCat =
    s.category_path && s.category_path.length > 0
      ? s.category_path.join(" / ")
      : s.suggested_category;
  const originalTags = [...s.suggested_tags];

  s.suggested_category = categoryName;
  s.suggested_category_id = categoryId;
  s.category_path = categoryPath;
  s.parent_category_id = categoryId || categoryPath ? null : draftParentId.value;
  s.suggested_tags = draftTags.value
    .map((t) => t.trim())
    .filter((t) => t.length > 0);
  s.summary = draftSummary.value.trim();

  const changes: string[] = [];
  if (categoryId) {
    const target = flatCategories.value.find((c) => c.id === categoryId);
    changes.push(
      t("分类 → {category}", { category: target?.path ?? categoryName }),
    );
  } else if (categoryPath) {
    changes.push(
      t("分类 → {category}", { category: categoryPath.join(" / ") }),
    );
  } else if (categoryName !== originalCat) {
    const parentLabel = draftParentId.value
      ? (flatCategories.value.find((c) => c.id === draftParentId.value)?.path ??
        "?")
      : t("顶级");
    changes.push(
      t("分类 → {parent} / {name}（新建）", {
        parent: parentLabel,
        name: categoryName,
      }),
    );
  }
  const removed = originalTags.filter((t) => !s.suggested_tags.includes(t));
  const added = s.suggested_tags.filter((t) => !originalTags.includes(t));
  if (removed.length)
    changes.push(t("移除标签 [{tags}]", { tags: removed.join(",") }));
  if (added.length)
    changes.push(t("新增标签 [{tags}]", { tags: added.join(",") }));

  log(
    "info",
    changes.length === 0
      ? t("已保存（无实际修改）：{title}", { title: s.title })
      : t("已修改建议：{title} | {changes}", {
          title: s.title,
          changes: changes.join("; "),
        }),
  );

  cancelEdit();
}

// ============ 分类展示 ============

function resolveCategoryDisplay(s: AiSuggestion): string {
  if (s.category_path && s.category_path.length > 0) {
    return s.category_path.join(" / ");
  }
  if (s.suggested_category_id) {
    const target = flatCategories.value.find(
      (c) => c.id === s.suggested_category_id,
    );
    return target?.path ?? s.suggested_category;
  }
  if (s.parent_category_id) {
    const parent = flatCategories.value.find(
      (c) => c.id === s.parent_category_id,
    );
    return t("{parent} / {name}（新建）", {
      parent: parent?.path ?? "?",
      name: s.suggested_category,
    });
  }
  const exists = flatCategories.value.some(
    (c) => c.parent_id === null && c.name === s.suggested_category,
  );
  return exists
    ? s.suggested_category
    : t("{name}（新建）", { name: s.suggested_category });
}

function resolveCategoryChipColor(s: AiSuggestion): string {
  if (s.category_path && s.category_path.length > 0) {
    const path = s.category_path.join(" / ");
    return flatCategories.value.some((category) => category.path === path)
      ? "primary"
      : "success";
  }
  if (s.suggested_category_id) return "primary";
  if (s.parent_category_id) {
    const exists = flatCategories.value.some(
      (c) =>
        c.parent_id === s.parent_category_id && c.name === s.suggested_category,
    );
    return exists ? "primary" : "success";
  }
  const exists = flatCategories.value.some(
    (c) => c.parent_id === null && c.name === s.suggested_category,
  );
  return exists ? "primary" : "success";
}

function resolveCategoryChipIcon(s: AiSuggestion): string {
  return resolveCategoryChipColor(s) === "success"
    ? "mdi-folder-plus"
    : "mdi-folder";
}

// ============ 日志 ============

const logs = ref<LogEntry[]>([]);
const logsEl = ref<HTMLElement | null>(null);
const logPanel = ref<number | undefined>(undefined);

function log(level: LogLevel, message: string) {
  const time = new Date().toLocaleTimeString();
  logs.value.push({ time, level, message });
  if (logs.value.length > 200) {
    logs.value.splice(0, logs.value.length - 200);
  }
  nextTick(() => {
    if (logsEl.value) logsEl.value.scrollTop = logsEl.value.scrollHeight;
  });
}

function clearLogs() {
  logs.value = [];
}

// ============ 设置加载 ============

async function loadSettings() {
  try {
    settings.value = await invoke<AppSettings>("get_settings");
    log(
      "info",
      t("已加载设置：provider={provider}, model={model}, base={base}", {
        provider: settings.value.ai_provider,
        model: settings.value.ai_model,
        base: settings.value.openai_base_url,
      }),
    );
    if (!settings.value.openai_base_url)
      log("warn", t("Base URL 为空，请在设置页填写"));
    if (!settings.value.ai_model) log("warn", t("模型名为空，请在设置页填写"));
  } catch (e) {
    log("error", t("加载设置失败：{e}", { e: String(e) }));
    notify.error(t("加载设置失败：{e}", { e: String(e) }));
  }
}

async function onPing() {
  log("info", t("开始检测 AI 连接"));
  pingLoading.value = true;
  try {
    const msg = await invoke<string>("ping_ai_provider");
    providerStatus.value = {
      ok: true,
      label: "连接正常",
      detail: msg,
    };
    log("success", t("连接成功：{msg}", { msg }));
    notify.success(t("AI 服务连接正常"));
  } catch (e) {
    providerStatus.value = {
      ok: false,
      label: "连接失败",
      detail: String(e),
    };
    log("error", t("连接失败：{e}", { e: String(e) }));
    notify.error(t("连接失败：{e}", { e: String(e) }));
  } finally {
    pingLoading.value = false;
  }
}

async function onLaunchOllama() {
  log("info", t("正在启动 Ollama"));
  launching.value = true;
  try {
    const msg = await invoke<string>("launch_ollama");
    providerStatus.value = {
      ok: true,
      label: "Ollama 已就绪",
      detail: msg,
    };
    log("success", msg);
    notify.success(msg);
  } catch (e) {
    log("error", t("启动失败：{e}", { e: String(e) }));
    notify.error(t("启动失败：{e}", { e: String(e) }));
  } finally {
    launching.value = false;
  }
}

// ============ 分析 ============

let analyzeToken = 0;

async function onAnalyze() {
  if (!batchSize.value || batchSize.value < 1) {
    notify.warning(t("本次数量至少为 1"));
    return;
  }

  analyzing.value = true;
  canceled.value = false;
  taskProgress.value = t("正在提交 AI 任务…");
  suggestions.value = [];
  cancelEdit();
  log("info", t("发起 AI 分析，本次数量 {n}", { n: batchSize.value }));

  try {
    await invoke("start_ai_organize", { limit: batchSize.value });
    log("info", t("任务已提交到后台，可离开本页，稍后回来查看结果"));
    // 结果通过 ai-task-update 事件推送
  } catch (e) {
    analyzing.value = false;
    taskProgress.value = "";
    log("error", t("启动失败：{e}", { e: String(e) }));
    notify.error(t("启动失败：{e}", { e: String(e) }));
  }
}

function onCancel() {
  if (!analyzing.value) return;
  analyzeToken++;
  canceled.value = true;
  analyzing.value = false;
  taskProgress.value = "";
  log("warn", t("用户已停止等待（后端请求仍在后台继续）"));
  notify.warning(t("已停止等待"));
}

async function openPreview(s: AiSuggestion) {
  try {
    previewVideo.value = await invoke<VideoRow>("get_video", {
      id: s.video_id,
    });
    showPreview.value = true;
  } catch (e) {
    notify.error(t("加载视频预览失败：{e}", { e: String(e) }));
  }
}

async function onAccept(idx: number) {
  const s = suggestions.value[idx];
  try {
    await invoke("ai_apply_suggestion", { suggestion: s });
    suggestions.value.splice(idx, 1);
    saveState();
    log(
      "success",
      t("已接受：{title}（分类={category}，标签={tags}）", {
        title: s.title,
        category: resolveCategoryDisplay(s),
        tags: s.suggested_tags.join("/"),
      }),
    );
    notify.success(t("已应用：{title}", { title: s.title }));
    emit("videos-changed");
    await loadExistingNames();
  } catch (e) {
    log("error", t("应用失败（{title}）：{e}", { title: s.title, e: String(e) }));
    notify.error(t("应用失败（{title}）：{e}", { title: s.title, e: String(e) }));
  }
}

async function onReject(idx: number) {
  const s = suggestions.value[idx];
  try {
    await invoke("ai_reject_suggestion", { videoId: s.video_id });
    suggestions.value.splice(idx, 1);
    saveState();
    log("info", t("已忽略：{title}", { title: s.title }));
  } catch (e) {
    log("error", t("忽略失败：{e}", { e: String(e) }));
    notify.error(t("忽略失败：{e}", { e: String(e) }));
  }
}

async function onAcceptAll() {
  const ok = await confirmDialog({
    title: t("批量接受"),
    message: t("确定接受全部 {n} 条建议？\n这会自动创建新分类和新标签。", {
      n: suggestions.value.length,
    }),
  });
  if (!ok) return;

  log(
    "info",
    t("开始批量接受 {n} 条建议", { n: suggestions.value.length }),
  );
  const list = [...suggestions.value];
  let success = 0;
  for (const s of list) {
    try {
      await invoke("ai_apply_suggestion", { suggestion: s });
      suggestions.value = suggestions.value.filter(
        (x) => x.video_id !== s.video_id,
      );
      success++;
    } catch (e) {
      log(
        "error",
        t("批量应用失败（{title}）：{e}", { title: s.title, e: String(e) }),
      );
    }
  }
  saveState();
  log(
    "success",
    t("批量接受完成：成功 {ok}/{total}", { ok: success, total: list.length }),
  );
  notify.success(
    t("批量接受完成：成功 {ok}/{total}", { ok: success, total: list.length }),
  );
  emit("videos-changed");
  await loadExistingNames();
}

function saveState() {
  const data: PersistedState = {
    suggestions: suggestions.value,
    logs: logs.value,
    editingVideoId: editingVideoId.value,
    draftParentId: draftParentId.value,
    draftCategory: draftCategory.value,
    draftTags: draftTags.value,
    draftSummary: draftSummary.value,
    batchSize: batchSize.value,
  };
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  } catch (e) {
    console.warn("保存 AI 整理状态失败", e);
  }
}

function restoreState() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return;
    const data: PersistedState = JSON.parse(raw);
    suggestions.value = data.suggestions ?? [];
    logs.value = data.logs ?? [];
    editingVideoId.value = data.editingVideoId ?? null;
    draftParentId.value = data.draftParentId ?? null;
    draftCategory.value = data.draftCategory ?? null;
    draftTags.value = data.draftTags ?? [];
    draftSummary.value = data.draftSummary ?? "";
    batchSize.value = data.batchSize ?? 15;
    log("info", t("已从本地恢复上次的 AI 整理状态"));
  } catch (e) {
    console.warn("恢复 AI 整理状态失败", e);
  }
}

watch(
  [
    suggestions,
    logs,
    editingVideoId,
    draftParentId,
    draftCategory,
    draftTags,
    draftSummary,
    batchSize,
  ],
  saveState,
  { deep: true },
);

onMounted(async () => {
  await loadSettings();
  await loadExistingNames();

  restoreState();

  // 查询当前后台任务状态
  try {
    const taskState = await invoke<{
      status: string;
      progress: string;
      suggestions: AiSuggestion[];
      error: string | null;
    }>("get_ai_task_state");

    if (taskState.status === "running") {
      analyzing.value = true;
      canceled.value = false;
      taskProgress.value = localizedProgress(taskState.progress);
      if (taskState.suggestions.length > 0 || suggestions.value.length === 0) {
        suggestions.value = taskState.suggestions;
      }
      log(
        "info",
        t("后台 AI 任务正在运行：{progress}", {
          progress: localizedProgress(taskState.progress),
        }),
      );
    } else if (taskState.status === "done") {
      // 后台状态有结果时以它为准；为空时保留本地恢复的建议，避免离开页面后建议消失。
      if (taskState.suggestions.length > 0) {
        suggestions.value = taskState.suggestions;
        saveState();
        log(
          "success",
          t("已恢复后台完成的 {n} 条建议", {
            n: taskState.suggestions.length,
          }),
        );
      }
    } else if (taskState.status === "error") {
      log(
        "error",
        t("后台任务失败：{error}", { error: taskState.error ?? "" }),
      );
    }
  } catch (e) {
    console.warn("查询后台任务状态失败", e);
  }

  // 监听后台 AI 任务进度
  unlistenAiTask = await listen<{
    status: string;
    progress: string;
    suggestions: AiSuggestion[];
    error: string | null;
  }>("ai-task-update", (event) => {
    const ts = event.payload;
    if (ts.progress)
      log("info", t("[后台] {progress}", {
        progress: localizedProgress(ts.progress),
      }));

    if (ts.status === "running") {
      if (!canceled.value) {
        analyzing.value = true;
        taskProgress.value = localizedProgress(ts.progress);
        suggestions.value = ts.suggestions ?? [];
      }
      return;
    }

    if (ts.status === "done") {
      canceled.value = false;
      analyzing.value = false;
      taskProgress.value = "";
      suggestions.value = ts.suggestions;
      saveState();
      log(
        "success",
        t("后台分析完成，共 {n} 条建议", { n: ts.suggestions.length }),
      );
      notify.success(
        t("AI 分析完成，共 {n} 条建议", { n: ts.suggestions.length }),
      );
    } else if (ts.status === "error") {
      canceled.value = false;
      analyzing.value = false;
      taskProgress.value = "";
      log("error", t("后台分析失败：{error}", { error: ts.error ?? "" }));
      notify.error(t("分析失败：{error}", { error: ts.error ?? "" }));
    }
  });

  await loadSettings();
  await loadExistingNames();
});

onUnmounted(() => {
  saveState(); // 离开时保存
  unlistenAiTask?.(); // 解除监听（组件重新挂载时会重新 listen）
});
</script>

<style scoped>
.min-w-0 {
  min-width: 0;
}

/* 缩略图 */
.thumb {
  width: 120px;
  height: 68px;
  flex-shrink: 0;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.06);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  cursor: pointer;
}
.thumb-play {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.25);
  opacity: 0;
  transition: opacity 0.15s;
}
.thumb:hover .thumb-play {
  opacity: 1;
}
.thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.thumb-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
}

/* 日志 */
.logs-box {
  max-height: 240px;
  overflow-y: auto;
  background: rgba(0, 0, 0, 0.03);
  border-radius: 6px;
  padding: 8px;
  font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
  font-size: 12px;
}
.log-line {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 2px 0;
  border-bottom: 1px dashed rgba(128, 128, 128, 0.1);
}
.log-line:last-child {
  border-bottom: none;
}
.log-time {
  opacity: 0.55;
  flex-shrink: 0;
}
.log-level {
  flex-shrink: 0;
  font-weight: 600;
}
.log-msg {
  word-break: break-word;
  white-space: pre-wrap;
}
.log-info .log-level {
  color: #1976d2;
}
.log-success .log-level {
  color: #2e7d32;
}
.log-warn .log-level {
  color: #ed6c02;
}
.log-error .log-level {
  color: #d32f2f;
}
.log-error .log-msg {
  color: #d32f2f;
}
</style>
