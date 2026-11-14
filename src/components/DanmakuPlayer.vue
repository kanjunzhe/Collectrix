<template>
  <div class="danmaku-player" ref="containerRef">
    <video
      ref="videoRef"
      :src="videoSrc"
      controls
      class="local-video"
      @timeupdate="onTimeUpdate"
      @seeked="onSeeked"
      @seeking="onSeeking"
      @loadedmetadata="checkVideoTrack"
      @play="onPlay"
      @playing="onPlay"
      @pause="onPause"
      @waiting="onPause"
      @ended="onPause"
      @error="$emit('invalid')"
    />

    <!-- 显示/隐藏控制 -->
    <div class="player-toggles">
      <v-btn
        v-if="allDanmaku.length > 0"
        icon
        size="small"
        variant="tonal"
        :title="danmakuVisible ? t('隐藏弹幕') : t('显示弹幕')"
        @click="toggleDanmaku"
      >
        <v-icon size="18">
          {{ danmakuVisible ? "mdi-comment-text-outline" : "mdi-comment-off-outline" }}
        </v-icon>
      </v-btn>
      <v-btn
        v-if="allSubtitles.length > 0"
        icon
        size="small"
        variant="tonal"
        :title="subtitleVisible ? t('隐藏字幕') : t('显示字幕')"
        @click="subtitleVisible = !subtitleVisible"
      >
        <v-icon size="18">
          {{ subtitleVisible ? "mdi-subtitles-outline" : "mdi-subtitles-off-outline" }}
        </v-icon>
      </v-btn>
    </div>

    <!-- 弹幕层 -->
    <div
      v-if="danmakuVisible"
      class="danmaku-layer"
      :class="{ 'danmaku-paused': danmakuPaused }"
    >
      <div
        v-for="d in activeDanmus"
        :key="d.id"
        class="danmaku-item"
        :class="{
          'danmaku-scroll': d.mode !== 4 && d.mode !== 5,
          'danmaku-top': d.mode === 5,
          'danmaku-bottom': d.mode === 4,
        }"
        :style="danmuStyle(d)"
        @animationend="removeDanmu(d.id)"
      >
        {{ d.text }}
      </div>
    </div>

    <!-- 字幕层 -->
    <div v-if="subtitleVisible && currentSubtitle" class="subtitle-overlay">
      {{ currentSubtitle }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import type { CSSProperties } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { t } from "../i18n";

interface RawDanmaku {
  time: number;
  mode: number;
  fontsize: number;
  color: number;
  text: string;
}

interface RawSubtitle {
  from: number;
  to: number;
  content: string;
}

interface ActiveDanmu {
  id: number;
  text: string;
  color: string;
  mode: number;
  fontsize: number;
  track: number;
  duration: number;
  distance: number;
  /** 顶部/底部弹幕按视频时间过期；滚动弹幕由 animationend 移除。 */
  endTime?: number;
}

const props = defineProps<{
  videoSrc: string;
  bvid: string;
  cid: number;
  localSubtitlePaths?: string[];
}>();

const emit = defineEmits<{ invalid: [] }>();

const videoRef = ref<HTMLVideoElement>();
const containerRef = ref<HTMLElement>();

function checkVideoTrack() {
  const video = videoRef.value;
  if (video && video.readyState >= 1 && video.videoWidth === 0) {
    // 文件可能只有音轨，或编码不被 WebView 支持。
    emit("invalid");
  }
}

// ============ 弹幕状态 ============

const allDanmaku = ref<RawDanmaku[]>([]);
const activeDanmus = ref<ActiveDanmu[]>([]);
const danmakuVisible = ref(true);
const danmakuPaused = ref(false);
const subtitleVisible = ref(true);
const currentTime = ref(0);

let lastIndex = 0;
let lastTime = 0;
let nextId = 1;
let danmakuToken = 0;
let subtitleToken = 0;
let dataLoaded = false;
let loadTimer: number | undefined;

const IS_NARROW_SCREEN = window.innerWidth < 760;
const TRACK_COUNT = IS_NARROW_SCREEN ? 4 : 6;
const MAX_ACTIVE_DANMAKU = IS_NARROW_SCREEN ? 120 : 300;
const TRACK_HEIGHT = 32;
const SCROLL_SPEED = 120; // 像素/秒
const FIXED_DURATION = 5; // 顶部/底部弹幕显示时长（秒，视频时间）

// 轨道占用记录：tracks[i] = 该轨道上一条弹幕的结束时间（video 时间）
const tracks: number[] = new Array(TRACK_COUNT).fill(0);

// ============ 字幕状态 ============

const allSubtitles = ref<RawSubtitle[]>([]);

const currentSubtitle = computed(() => {
  const now = currentTime.value;
  return (
    allSubtitles.value.find((cue) => now >= cue.from && now < cue.to)?.content ??
    ""
  );
});

// ============ 弹幕渲染 ============

function danmuStyle(d: ActiveDanmu): CSSProperties {
  const style: Record<string, string> = {
    color: d.color,
    fontSize: `${d.fontsize}px`,
    lineHeight: `${TRACK_HEIGHT}px`,
  };
  if (d.mode === 5) {
    // 顶部弹幕：按 track 排
    style.top = `${10 + d.track * TRACK_HEIGHT}px`;
  } else if (d.mode === 4) {
    // 底部弹幕：从底部往上排
    style.bottom = `${60 + d.track * TRACK_HEIGHT}px`;
  } else {
    // 滚动弹幕（含 B 站的高级/特殊模式，统一按滚动处理）
    style.top = `${10 + d.track * TRACK_HEIGHT}px`;
    style.animationDuration = `${d.duration}s`;
    style["--danmaku-distance"] = `${d.distance}px`;
  }
  return style as CSSProperties;
}

function removeDanmu(id: number) {
  const idx = activeDanmus.value.findIndex((d) => d.id === id);
  if (idx >= 0) activeDanmus.value.splice(idx, 1);
}

function pickTrack(currentTime: number): number {
  // 找一条已经空出的轨道
  for (let i = 0; i < TRACK_COUNT; i++) {
    if (tracks[i] < currentTime) {
      return i;
    }
  }
  // 全占满就选最早空出的
  let min = 0;
  for (let i = 1; i < TRACK_COUNT; i++) {
    if (tracks[i] < tracks[min]) min = i;
  }
  return min;
}

function spawnDanmu(raw: RawDanmaku) {
  if (!danmakuVisible.value) return;
  const containerWidth = containerRef.value?.clientWidth ?? 800;
  const fontsize = raw.fontsize || 25;
  // 估算宽度：中文约 fontsize px，英文约 0.6 * fontsize
  let estimatedWidth = 0;
  for (const ch of raw.text) {
    estimatedWidth += /[\u4e00-\u9fa5]/.test(ch) ? fontsize : fontsize * 0.6;
  }
  estimatedWidth = Math.max(estimatedWidth, 40);

  const isFixed = raw.mode === 4 || raw.mode === 5;
  const duration = isFixed
    ? FIXED_DURATION
    : (containerWidth + estimatedWidth) / SCROLL_SPEED;
  const track = pickTrack(raw.time);
  // 顶部/底部弹幕的轨道占用时间应为固定显示时长，而不是滚动距离用时。
  tracks[track] = raw.time + duration;

  const colorHex = `#${raw.color.toString(16).padStart(6, "0")}`;

  activeDanmus.value.push({
    id: nextId++,
    text: raw.text,
    color: colorHex,
    mode: raw.mode,
    fontsize,
    track,
    duration,
    distance: containerWidth + estimatedWidth,
    endTime: isFixed ? raw.time + FIXED_DURATION : undefined,
  });

  // 防止渲染数组无限增长；移动端保留更少的 DOM 弹幕以保证播放流畅。
  if (activeDanmus.value.length > MAX_ACTIVE_DANMAKU) {
    activeDanmus.value.splice(
      0,
      activeDanmus.value.length - MAX_ACTIVE_DANMAKU,
    );
  }
}

function syncIndex(time: number) {
  lastIndex = allDanmaku.value.findIndex((d) => d.time >= time - 0.3);
  if (lastIndex < 0) lastIndex = allDanmaku.value.length;
  lastTime = time;
  currentTime.value = time;
}

function onTimeUpdate() {
  const time = videoRef.value?.currentTime ?? 0;
  currentTime.value = time;

  // 循环播放、用户回退或某些 WebView 未触发 seeked 时，重新定位弹幕索引。
  if (time + 0.5 < lastTime) {
    activeDanmus.value = [];
    tracks.fill(0);
    syncIndex(time);
  }
  lastTime = time;

  // 顶部/底部弹幕按视频时间过期；暂停时 currentTime 不前进，自然也会暂停。
  if (activeDanmus.value.some((d) => d.endTime !== undefined)) {
    activeDanmus.value = activeDanmus.value.filter(
      (d) => d.endTime === undefined || d.endTime > time,
    );
  }

  if (allDanmaku.value.length === 0) return;

  // 处理时间窗口内的弹幕：[t - 0.3, t + 0.05]
  // 0.3 秒的回看容差是为了避免 timeupdate 间隔过大（低帧率视频）漏弹幕
  const windowEnd = time + 0.05;

  // 快进时，lastIndex 之前的弹幕跳过了
  while (
    lastIndex < allDanmaku.value.length &&
    allDanmaku.value[lastIndex].time < time - 0.3
  ) {
    lastIndex++;
  }

  while (
    lastIndex < allDanmaku.value.length &&
    allDanmaku.value[lastIndex].time <= windowEnd
  ) {
    spawnDanmu(allDanmaku.value[lastIndex]);
    lastIndex++;
  }
}

function onSeeking() {
  // 拖动进度条期间先清空，避免旧弹幕在画面上“漂移”。
  activeDanmus.value = [];
  tracks.fill(0);
}

function onSeeked() {
  const time = videoRef.value?.currentTime ?? 0;
  activeDanmus.value = [];
  tracks.fill(0);
  syncIndex(time);
}

function onPlay() {
  danmakuPaused.value = false;
  // 先让视频开始播放，再在空闲时加载弹幕/字幕，避免打开播放器时同时解析大量数据。
  ensureDataLoaded();
}

function onPause() {
  danmakuPaused.value = true;
}

function toggleDanmaku() {
  if (danmakuVisible.value) {
    danmakuVisible.value = false;
    // v-if 会销毁弹幕层；不清空的话重新显示时旧动画会从头播放。
    activeDanmus.value = [];
    tracks.fill(0);
    return;
  }

  danmakuVisible.value = true;
  syncIndex(videoRef.value?.currentTime ?? 0);
}

// ============ 数据加载 ============

function ensureDataLoaded() {
  if (dataLoaded || loadTimer !== undefined) return;
  loadTimer = window.setTimeout(() => {
    loadTimer = undefined;
    if (dataLoaded) return;
    dataLoaded = true;
    loadDanmaku();
    loadSubtitle();
  }, 250);
}

function resetDanmaku() {
  danmakuToken++;
  subtitleToken++;
  dataLoaded = false;
  if (loadTimer !== undefined) {
    window.clearTimeout(loadTimer);
    loadTimer = undefined;
  }
  allDanmaku.value = [];
  allSubtitles.value = [];
  activeDanmus.value = [];
  tracks.fill(0);
  lastIndex = 0;
  lastTime = 0;
  currentTime.value = 0;
}

async function loadDanmaku() {
  const cid = props.cid;
  if (!cid) {
    allDanmaku.value = [];
    return;
  }
  const token = ++danmakuToken;
  try {
    const raw = await invoke<RawDanmaku[]>("get_danmaku", { cid });
    if (token !== danmakuToken) return;
    // 按时间升序排序（后端返回顺序依赖 XML，通常已是升序，保险起见再排一次）
    raw.sort((a, b) => a.time - b.time);
    allDanmaku.value = raw;
    syncIndex(videoRef.value?.currentTime ?? 0);
  } catch (e) {
    if (token === danmakuToken) {
      console.warn("加载弹幕失败", e);
    }
    // 不弹通知：弹幕加载失败不影响视频播放
  }
}

async function loadSubtitle() {
  const { cid, bvid } = props;
  const localPaths = props.localSubtitlePaths ?? [];
  if (!cid && !bvid && localPaths.length === 0) {
    allSubtitles.value = [];
    return;
  }
  const token = ++subtitleToken;

  // 本地 sidecar 字幕优先，避免重复请求 B 站接口。
  if (localPaths.length > 0) {
    const localCues: RawSubtitle[] = [];
    for (const path of localPaths) {
      try {
        localCues.push(
          ...(await invoke<RawSubtitle[]>("get_local_subtitle", { path })),
        );
      } catch (e) {
        console.warn("加载本地字幕失败", path, e);
      }
    }
    if (token !== subtitleToken) return;
    if (localCues.length > 0) {
      localCues.sort((a, b) => a.from - b.from);
      allSubtitles.value = localCues;
      return;
    }
  }

  if (!cid || !bvid) {
    allSubtitles.value = [];
    return;
  }

  try {
    const list = await invoke<{ lan: string; lan_doc: string; url: string }[]>(
      "get_subtitle_list",
      { bvid, cid },
    );
    if (token !== subtitleToken || list.length === 0) return;

    // 优先中文，其次任意
    const zh = list.find((s) => s.lan.startsWith("zh")) ?? list[0];
    const cues = await invoke<RawSubtitle[]>("get_subtitle_content", {
      url: zh.url,
    });
    if (token !== subtitleToken) return;
    cues.sort((a, b) => a.from - b.from);
    allSubtitles.value = cues;
  } catch (e) {
    if (token === subtitleToken) {
      console.warn("加载字幕失败", e);
    }
  }
}

watch(
  () => [
    props.cid,
    props.bvid,
    props.videoSrc,
    (props.localSubtitlePaths ?? []).join("|"),
  ],
  () => {
    resetDanmaku();
    if (videoRef.value && !videoRef.value.paused) ensureDataLoaded();
  },
);

onMounted(() => {
  // 进入播放器后就预取弹幕/字幕，避免用户开始播放后还要等一个额外延迟。
  // 仍保留 250ms 的轻量延迟，给视频首帧让出时间。
  ensureDataLoaded();
});

onUnmounted(() => {
  resetDanmaku();
});
</script>

<style scoped>
.danmaku-player {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  background: #000;
  overflow: hidden;
}
.player-toggles {
  position: absolute;
  top: 10px;
  right: 10px;
  z-index: 30;
  display: flex;
  gap: 6px;
}

.local-video {
  width: 100%;
  height: 100%;
  display: block;
}

.danmaku-layer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  overflow: hidden;
  z-index: 10;
}

.danmaku-item {
  position: absolute;
  white-space: nowrap;
  font-weight: 600;
  height: 32px;
  text-shadow:
    1px 0 2px rgba(0, 0, 0, 0.8),
    -1px 0 2px rgba(0, 0, 0, 0.8),
    0 1px 2px rgba(0, 0, 0, 0.8),
    0 -1px 2px rgba(0, 0, 0, 0.8);
  pointer-events: none;
  will-change: transform;
}

/* 滚动弹幕：从容器右侧进入，完全离开左侧后由 animationend 移除 */
.danmaku-scroll {
  left: 100%;
  animation-name: danmaku-scroll;
  animation-timing-function: linear;
  animation-fill-mode: forwards;
}

@keyframes danmaku-scroll {
  from {
    transform: translateX(0);
  }
  to {
    transform: translateX(calc(-1 * var(--danmaku-distance, 100vw)));
  }
}

/* 视频暂停/缓冲时冻结滚动动画，恢复播放时继续。 */
.danmaku-layer.danmaku-paused .danmaku-scroll {
  animation-play-state: paused;
}

/* 顶部弹幕 */
.danmaku-top {
  left: 50%;
  transform: translateX(-50%);
}

/* 底部弹幕 */
.danmaku-bottom {
  left: 50%;
  transform: translateX(-50%);
}

/* 字幕 */
.subtitle-overlay {
  position: absolute;
  bottom: 60px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.65);
  color: #fff;
  padding: 4px 16px;
  border-radius: 4px;
  font-size: 18px;
  line-height: 1.5;
  text-align: center;
  max-width: 90%;
  pointer-events: none;
  z-index: 20;
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
