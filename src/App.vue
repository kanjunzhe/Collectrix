<!--
  改动点：
  1. viewMode 状态提升到 App.vue，localStorage 持久化（key: videoVault.viewMode）
  2. LibraryView 通过 v-model:view-mode 双向绑定，不再内部管理状态
  3. 其余逻辑（主题同步、全局编辑弹窗、导航、标签快捷筛选）保持不变
-->
<template>
  <GlobalNotifications />
  <div v-if="!lockReady" class="app-loading">
    <div class="app-loading-orbit">
      <CurveLoader :size="108" :particles="52" color="#1976D2" />
      <img
        src="/collectrix-logo-flat.svg"
        alt="Collectrix"
        class="app-loading-logo"
      />
    </div>
    <div class="app-loading-title">Collectrix</div>
    <div class="app-loading-caption">{{ t(startupStage) }}</div>
  </div>
  <AppLockScreen v-else-if="locked" @unlocked="locked = false" />
  <v-app v-else :class="{ 'app-safe-area': !isPlayerWindow }">
    <v-app-bar
      v-if="!isPlayerWindow"
      flat
      color="transparent"
      height="72"
      class="px-4"
    >
      <v-btn
        icon
        size="large"
        :title="isMobile ? t('展开侧栏') : railMode ? t('展开侧栏') : t('收起侧栏')"
        @click="onDrawerToggle"
      >
        <v-icon>
          {{
            isMobile
              ? "mdi-menu"
              : railMode
                ? "mdi-menu"
                : "mdi-chevron-left"
          }}
        </v-icon>
      </v-btn>
      <div class="d-flex align-center">
        <img
          src="/collectrix-logo-flat.svg"
          alt="Collectrix"
          class="app-logo-img mr-3"
        />
        <span class="text-h3 font-weight-bold app-title">Collectrix</span>
      </div>
      <v-spacer />
      <v-btn
        icon
        size="large"
        :title="t('语言')"
        @click="toggleLanguage"
      >
        <v-icon size="26">mdi-translate</v-icon>
      </v-btn>
      <v-btn
        icon
        size="large"
        :title="t('切换主题')"
        @click="toggleTheme"
      >
        <v-icon size="28" :color="isDark ? 'amber' : 'indigo'">
          {{ isDark ? "mdi-weather-sunny" : "mdi-weather-night" }}
        </v-icon>
      </v-btn>
    </v-app-bar>

    <v-navigation-drawer
      v-if="!isPlayerWindow"
      v-model="drawerOpen"
      :permanent="!isMobile"
      :rail="railMode"
      width="260"
    >
      <div class="drawer-scroll">
        <v-list nav>
          <v-list-item
            prepend-icon="mdi-view-dashboard-outline"
          :title="t('开始')"
          :active="currentView === 'dashboard'"
          @click="navigate('dashboard')"
        />
        <v-list-item
          prepend-icon="mdi-folder-multiple-outline"
          :title="t('视频库')"
          :active="currentView === 'library'"
          @click="navigate('library')"
        />
        <v-list-item
          prepend-icon="mdi-chart-box-outline"
          :title="t('统计')"
          :active="currentView === 'stats'"
          @click="navigate('stats')"
        />
        <v-list-item
          prepend-icon="mdi-graph-outline"
          :title="t('思维导图')"
          :active="currentView === 'mindmap'"
          @click="navigate('mindmap')"
        />
        <v-list-item
          prepend-icon="mdi-tag-multiple-outline"
          :title="t('标签管理')"
          :active="currentView === 'tags'"
          @click="navigate('tags')"
        />
        <v-list-item
          prepend-icon="mdi-bookmark-multiple-outline"
          :title="t('主题')"
          :active="currentView === 'topics'"
          @click="navigate('topics')"
        />
        <v-list-item
          prepend-icon="mdi-notebook-outline"
          :title="t('记事本')"
          :active="currentView === 'notebook'"
          @click="navigate('notebook')"
        />
        <v-list-item
          v-if="aiEnabled"
          prepend-icon="mdi-robot-outline"
          :title="t('AI 整理')"
          :active="currentView === 'ai'"
          @click="navigate('ai')"
        />
        <v-list-item
          v-if="aiEnabled"
          prepend-icon="mdi-filter-outline"
          :title="t('AI 筛选')"
          :active="currentView === 'aifilter'"
          @click="navigate('aifilter')"
        />
        <v-list-item
          v-if="aiEnabled"
          prepend-icon="mdi-chat-processing-outline"
          :title="t('AI 对话')"
          :active="currentView === 'aichat'"
          @click="navigate('aichat')"
        />
        <v-list-item
          prepend-icon="mdi-information-outline"
          :title="t('关于')"
          :active="currentView === 'about'"
          @click="navigate('about')"
        />
        <v-list-item
          prepend-icon="mdi-cog-outline"
          :title="t('设置')"
          :active="currentView === 'settings'"
          @click="navigate('settings')"
        />
        </v-list>
      </div>

      <div class="pa-2 drawer-footer">
        <v-btn
          block
          variant="text"
          :prepend-icon="railMode ? 'mdi-chevron-right' : 'mdi-chevron-left'"
          :title="railMode ? t('展开侧栏') : t('收起侧栏')"
          @click="onDrawerFooterClick"
        >
          <span v-if="!railMode">{{ t("收起侧栏") }}</span>
        </v-btn>
      </div>
    </v-navigation-drawer>

    <v-main>
      <v-container fluid class="pa-4">
        <DashboardView
          v-if="currentView === 'dashboard'"
          :tags="allTags"
          @navigate="navigate"
          @play="openPlayer"
          @tag="viewByTag"
          @data-changed="onExternalDataChanged"
        />
        <LibraryView
          v-else-if="currentView === 'library'"
          ref="libraryRef"
          v-model:view-mode="viewMode"
          @play="openPlayer"
          @tags-changed="loadTags"
        />
        <PlayerView
          v-else-if="currentView === 'player' && selectedVideo"
          :video="selectedVideo"
          @back="onPlayerBack"
          @edit="openGlobalEdit"
        />
        <LoadingView
          v-else-if="currentView === 'player'"
          :message="t('正在加载播放器…')"
        />
        <TagManagerView
          v-else-if="currentView === 'tags'"
          @tags-changed="loadTags"
        />
        <TopicManagerView
          v-else-if="currentView === 'topics'"
          @play="openPlayer"
          @edit="openGlobalEdit"
        />
        <AiOrganizeView
          v-else-if="currentView === 'ai'"
          @videos-changed="onExternalDataChanged"
        />
        <StatsView v-else-if="currentView === 'stats'" />
        <MindMapView
          v-else-if="currentView === 'mindmap'"
          @play="openPlayer"
        />
        <NotebookView
          v-else-if="currentView === 'notebook'"
          @play="openPlayer"
        />
        <AiFilterView v-else-if="currentView === 'aifilter'" />
        <AiChatView
          v-else-if="currentView === 'aichat'"
          @play="openPlayer"
          @edit="openGlobalEdit"
        />
        <AboutView v-else-if="currentView === 'about'" />
        <SettingsView v-else-if="currentView === 'settings'" />
      </v-container>
    </v-main>

    <!-- 全局编辑弹窗：给 TopicManagerView 等外部视图使用 -->
    <VideoEditDialog
      v-model="globalEditVisible"
      :video="globalEditingVideo"
      :categories="globalCategories"
      @saved="onExternalDataChanged"
    />

    <OnboardingDialog
      v-model="showOnboarding"
      @finish="completeOnboarding"
    />
  </v-app>
</template>

<script setup lang="ts">
import {
  computed,
  defineAsyncComponent,
  nextTick,
  onMounted,
  onUnmounted,
  ref,
  watch,
} from "vue";
import { useTheme } from "vuetify";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import AppLockScreen from "./components/AppLockScreen.vue";
import CurveLoader from "./components/CurveLoader.vue";
import GlobalNotifications from "./components/GlobalNotifications.vue";
import LoadingView from "./components/LoadingView.vue";
import OnboardingDialog from "./components/OnboardingDialog.vue";
import VideoEditDialog from "./components/VideoEditDialog.vue";

// 视图按需加载，避免启动时一次性解析全部页面；播放窗口只加载 PlayerView。
function lazyView(
  loader: () => Promise<{ default: import("vue").Component }>,
) {
  return defineAsyncComponent({
    loader,
    loadingComponent: LoadingView,
    delay: 120,
    timeout: 30_000,
  });
}

const DashboardView = lazyView(() => import("./views/DashboardView.vue"));
const LibraryView = lazyView(() => import("./views/LibraryView.vue"));
const PlayerView = lazyView(() => import("./views/PlayerView.vue"));
const TagManagerView = lazyView(() => import("./views/TagManagerView.vue"));
const TopicManagerView = lazyView(() => import("./views/TopicManagerView.vue"));
const AiOrganizeView = lazyView(() => import("./views/AiOrganizeView.vue"));
const AiChatView = lazyView(() => import("./views/AiChatView.vue"));
const AiFilterView = lazyView(() => import("./views/AiFilterView.vue"));
const StatsView = lazyView(() => import("./views/StatsView.vue"));
const MindMapView = lazyView(() => import("./views/MindMapView.vue"));
const NotebookView = lazyView(() => import("./views/NotebookView.vue"));
const AboutView = lazyView(() => import("./views/AboutView.vue"));
const SettingsView = lazyView(() => import("./views/SettingsView.vue"));
import type { AppSettings, CategoryNode, TagInfo, VideoRow } from "./types";
import { locale, setLocale, t } from "./i18n";
import { lastLightTheme, rememberTheme, resolveTheme } from "./themes";

// ============ 视图模式：全局唯一数据源 + localStorage 持久化 ============

type ViewMode = "grid" | "list";
const VIEW_MODE_STORAGE_KEY = "videoVault.viewMode";

function loadViewMode(): ViewMode {
  try {
    const v = localStorage.getItem(VIEW_MODE_STORAGE_KEY);
    return v === "list" ? "list" : "grid";
  } catch {
    return "grid";
  }
}

const viewMode = ref<ViewMode>(loadViewMode());

// 视图模式变化时立即持久化
watch(viewMode, (v) => {
  try {
    localStorage.setItem(VIEW_MODE_STORAGE_KEY, v);
  } catch (e) {
    console.warn("保存视图模式失败", e);
  }
});

// ============ 主题 ============

const theme = useTheme();
const isDark = ref(false);

type ViewName =
  | "dashboard"
  | "library"
  | "stats"
  | "mindmap"
  | "notebook"
  | "player"
  | "tags"
  | "topics"
  | "ai"
  | "aifilter"
  | "aichat"
  | "about"
  | "settings";

const playerWindowId = Number(
  new URLSearchParams(window.location.search).get("player_id") ??
    (
      window as unknown as { __COLLECTRIX_PLAYER_ID__?: number }
    ).__COLLECTRIX_PLAYER_ID__ ??
    0,
);
const isPlayerWindow = ref(playerWindowId > 0);
const currentView = ref<ViewName>(playerWindowId ? "player" : "dashboard");
// 设置读取完成前先不渲染 AI 入口，避免启动时闪出并提前初始化 AI 相关 UI。
const aiEnabled = ref(false);
const lockReady = ref(false);
const locked = ref(false);
const startupStage = ref("正在读取设置…");
const isMobile = ref(window.innerWidth < 760);
const drawerOpen = ref(!isMobile.value);
const drawerRail = ref(
  (() => {
    try {
      return localStorage.getItem("collectrix.drawerRail") === "1";
    } catch {
      return false;
    }
  })(),
);
const railMode = computed(() => drawerRail.value && !isMobile.value);

function toggleDrawer() {
  drawerRail.value = !drawerRail.value;
  try {
    localStorage.setItem("collectrix.drawerRail", drawerRail.value ? "1" : "0");
  } catch {
    // ignore storage errors
  }
}

function onDrawerToggle() {
  if (isMobile.value) {
    drawerOpen.value = !drawerOpen.value;
    return;
  }
  toggleDrawer();
}

function onDrawerFooterClick() {
  if (isMobile.value) {
    drawerOpen.value = false;
    return;
  }
  toggleDrawer();
}

function maybeShowOnboarding() {
  if (isPlayerWindow.value || !lockReady.value || locked.value) return;
  try {
    if (localStorage.getItem(ONBOARDING_KEY) !== "1") {
      showOnboarding.value = true;
    }
  } catch {
    // localStorage may be unavailable in private mode.
  }
}

function completeOnboarding() {
  try {
    localStorage.setItem(ONBOARDING_KEY, "1");
  } catch {
    // ignore storage errors
  }
  showOnboarding.value = false;
}

function openOnboarding() {
  showOnboarding.value = true;
}

watch([lockReady, locked], maybeShowOnboarding);

function onResize() {
  isMobile.value = window.innerWidth < 760;
  if (isMobile.value) {
    drawerOpen.value = false;
  } else {
    drawerOpen.value = true;
  }
}
const allTags = ref<TagInfo[]>([]);
const selectedVideo = ref<VideoRow | null>(null);
const ONBOARDING_KEY = "collectrix.onboardingDone";
const showOnboarding = ref(false);
const libraryRef = ref<{ setTagFilter: (tagId: number) => void } | null>(null);
const pendingTagId = ref<number | null>(null);

// LibraryView 是异步组件；如果用户从侧栏点标签时它还没加载完，等组件挂载后再应用筛选。
watch(libraryRef, (view) => {
  if (view && pendingTagId.value !== null) {
    view.setTagFilter(pendingTagId.value);
    pendingTagId.value = null;
  }
});

// 全局编辑弹窗
const globalEditVisible = ref(false);
const globalEditingVideo = ref<VideoRow | null>(null);
const globalCategories = ref<{ id: number; label: string }[]>([]);

/** 让 isDark 从 Vuetify 主题派生，任何来源变化都同步图标 */
watch(
  () => theme.global.name.value,
  (v) => {
    isDark.value = v.startsWith("dark");
  },
  { immediate: true },
);

async function toggleLanguage() {
  const next = locale.value === "zh" ? "en" : "zh";
  setLocale(next);
  try {
    const s = await invoke<AppSettings>("get_settings");
    s.language = next;
    await invoke("set_settings", { settings: s });
  } catch (e) {
    console.error("保存语言失败", e);
  }
}

async function toggleTheme() {
  const next = isDark.value ? lastLightTheme() : "dark";
  theme.global.name.value = next;
  rememberTheme(next);
  try {
    await invoke("set_theme", { theme: next });
  } catch (e) {
    console.error("保存主题失败", e);
  }
}

async function loadSettingsAndTheme() {
  startupStage.value = "正在读取设置…";
  try {
    const s = await invoke<AppSettings>("get_settings");
    setLocale(s.language);
    theme.global.name.value = resolveTheme(s.theme);
    aiEnabled.value = s.ai_enabled !== false;
    locked.value = s.lock_enabled === true;
    startupStage.value = "正在准备资料库…";
    if (
      !aiEnabled.value &&
      (currentView.value === "ai" ||
        currentView.value === "aifilter" ||
        currentView.value === "aichat")
    ) {
      navigate("dashboard");
    }
  } finally {
    lockReady.value = true;
  }
}

async function refreshSettingsChanged() {
  try {
    const s = await invoke<AppSettings>("get_settings");
    aiEnabled.value = s.ai_enabled !== false;
    if (
      !aiEnabled.value &&
      (currentView.value === "ai" ||
        currentView.value === "aifilter" ||
        currentView.value === "aichat")
    ) {
      navigate("dashboard");
    }
  } catch {
    // ignore
  }
}

// ============ 导航 ============

function navigate(view: string) {
  currentView.value = view as ViewName;
  if (isMobile.value) drawerOpen.value = false;
}

async function loadTags() {
  try {
    allTags.value = await invoke<TagInfo[]>("list_tags");
  } catch (e) {
    console.warn("加载标签失败", e);
  }
}

function scheduleTagLoad() {
  const run = () => {
    void loadTags();
  };
  if ("requestIdleCallback" in window) {
    (
      window as Window & {
        requestIdleCallback: (
          callback: () => void,
          options?: { timeout: number },
        ) => number;
      }
    ).requestIdleCallback(run, { timeout: 1500 });
    return;
  }
  globalThis.setTimeout(run, 300);
}

async function openPlayer(video: VideoRow) {
  // 默认在主窗口内播放，避免每次播放都创建新的 WebView2 窗口导致卡死。
  // 需要独立窗口时，可以在播放页点击“在新窗口播放”。
  selectedVideo.value = video;
  currentView.value = "player";
  await nextTick();
}

async function onPlayerBack() {
  if (isPlayerWindow.value) {
    try {
      await getCurrentWindow().close();
    } catch (e) {
      console.warn("关闭窗口失败", e);
    }
    return;
  }
  navigate("library");
}

async function initPlayerWindow() {
  if (!playerWindowId) return;
  try {
    const video = await invoke<VideoRow>("get_video", { id: playerWindowId });
    selectedVideo.value = video;
    currentView.value = "player";
    await getCurrentWindow().setTitle(`Collectrix - ${video.title}`);
  } catch (e) {
    console.error("加载播放窗口视频失败", e);
  }
}

async function viewByTag(tagId: number) {
  pendingTagId.value = tagId;
  if (currentView.value !== "library") {
    currentView.value = "library";
    await nextTick();
  }
  await nextTick();
  if (libraryRef.value) {
    libraryRef.value.setTagFilter(tagId);
    pendingTagId.value = null;
  }
}

// ============ 全局编辑弹窗 ============

async function loadGlobalCategories() {
  const tree = await invoke<CategoryNode[]>("get_category_tree");
  const flat: { id: number; label: string }[] = [];
  const walk = (nodes: CategoryNode[], depth: number) => {
    for (const n of nodes) {
      flat.push({ id: n.id, label: "　".repeat(depth) + n.name });
      walk(n.children, depth + 1);
    }
  };
  walk(tree, 0);
  globalCategories.value = flat;
}

async function openGlobalEdit(video: VideoRow) {
  await loadGlobalCategories();
  globalEditingVideo.value = video;
  globalEditVisible.value = true;
}

async function onExternalDataChanged() {
  await loadTags();
  if (selectedVideo.value) {
    try {
      selectedVideo.value = await invoke<VideoRow>("get_video", {
        id: selectedVideo.value.id,
      });
    } catch (e) {
      console.warn("刷新视频详情失败", e);
    }
  }
}

onMounted(async () => {
  const startupTasks: Promise<unknown>[] = [loadSettingsAndTheme()];
  if (isPlayerWindow.value) {
    // 播放窗口不需要侧栏标签，也不应该和主窗口争抢加载资源。
    startupTasks.push(initPlayerWindow());
  } else {
    // 侧栏标签不影响首屏，放到浏览器空闲时再查，减少启动时的数据库争用。
    scheduleTagLoad();
  }
  try {
    await Promise.all(startupTasks);
  } catch (e) {
    console.error("应用初始化失败", e);
  }
  window.addEventListener(
    "collectrix:settings-changed",
    refreshSettingsChanged,
  );
  window.addEventListener("collectrix:open-onboarding", openOnboarding);
  window.addEventListener("resize", onResize);
  maybeShowOnboarding();
});

onUnmounted(() => {
  window.removeEventListener(
    "collectrix:settings-changed",
    refreshSettingsChanged,
  );
  window.removeEventListener(
    "collectrix:open-onboarding",
    openOnboarding,
  );
  window.removeEventListener("resize", onResize);
});
</script>

<style>
.gap-2 {
  gap: 8px;
}
.app-loading {
  position: fixed;
  inset: 0;
  z-index: 100000;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  background: var(--boot-bg, #f5f7fa);
  color: var(--boot-fg, #1b1b1f);
}
.app-loading-orbit {
  position: relative;
  width: 108px;
  height: 108px;
  display: grid;
  place-items: center;
}
.app-loading-orbit .curve-loader {
  position: absolute;
  inset: 0;
}
.app-loading-logo {
  position: relative;
  width: 48px;
  height: 48px;
  border-radius: 13px;
  box-shadow: 0 10px 28px rgba(25, 118, 210, 0.22);
}
.app-loading-title {
  font-size: 1.45rem;
  font-weight: 700;
  letter-spacing: 0.02em;
}
.app-loading-caption {
  font-size: 12px;
  opacity: 0.6;
}
.app-logo-img {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  flex: 0 0 auto;
}
.v-application {
  background:
    radial-gradient(
      circle at 8% 0%,
      rgba(var(--v-theme-primary), 0.08),
      transparent 32%
    ),
    rgb(var(--v-theme-background));
}
.v-app-bar {
  background: rgba(var(--v-theme-surface), 0.78) !important;
  backdrop-filter: blur(16px);
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.07);
}
.v-navigation-drawer {
  background: rgb(var(--v-theme-surface)) !important;
  border-right: 1px solid rgba(var(--v-theme-on-surface), 0.07);
}
.v-navigation-drawer .v-list-item {
  border-radius: 10px;
}
.v-card {
  border-radius: 14px;
}
/* Android edge-to-edge: keep the app bar below the status bar and display cutout. */
.app-safe-area {
  --safe-top: env(safe-area-inset-top, 0px);
}
.app-safe-area .v-app-bar {
  height: calc(72px + var(--safe-top)) !important;
  padding-top: var(--safe-top);
}
.app-safe-area .v-app-bar .v-toolbar__content {
  height: 72px !important;
}
.app-safe-area .v-main {
  padding-top: calc(72px + var(--safe-top)) !important;
}
.app-safe-area .v-navigation-drawer {
  padding-top: 0;
}
.app-safe-area .v-navigation-drawer__content {
  padding-top: var(--safe-top);
}
.v-navigation-drawer__content {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  box-sizing: border-box;
}
.drawer-scroll {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
}
.drawer-footer {
  flex: 0 0 auto;
  border-top: 1px solid rgba(128, 128, 128, 0.12);
}
@media (max-width: 760px) {
  .app-title {
    font-size: 1.35rem !important;
    line-height: 1.2;
  }
  .app-logo-img {
    width: 32px;
    height: 32px;
  }
}
</style>
