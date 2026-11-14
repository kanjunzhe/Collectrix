<template>
  <div
    class="curve-loader"
    :style="{ width: `${size}px`, height: `${size}px` }"
    role="status"
    :aria-label="label"
  >
    <svg viewBox="0 0 100 100" fill="none" aria-hidden="true">
      <g ref="groupRef"></g>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

const props = withDefaults(
  defineProps<{
    size?: number;
    particles?: number;
    color?: string;
    label?: string;
  }>(),
  {
    size: 72,
    particles: 42,
    color: "",
    label: "Loading",
  },
);

const groupRef = ref<SVGGElement>();
let animationFrame = 0;
let startedAt = 0;
let trail: SVGCircleElement[] = [];

const SVG_NS = "http://www.w3.org/2000/svg";
const DURATION = 4600;
const ROTATION_DURATION = 28000;
const PULSE_DURATION = 4200;
const TRAIL_SPAN = 0.38;
const BASE_RADIUS = 7;
const DETAIL_AMPLITUDE = 3;
const PETAL_COUNT = 7;
const CURVE_SCALE = 3.9;

function normalize(value: number): number {
  return ((value % 1) + 1) % 1;
}

function curvePoint(progress: number, detailScale: number) {
  const angle = progress * Math.PI * 2;
  const x =
    BASE_RADIUS * Math.cos(angle) -
    DETAIL_AMPLITUDE * detailScale * Math.cos(PETAL_COUNT * angle);
  const y =
    BASE_RADIUS * Math.sin(angle) -
    DETAIL_AMPLITUDE * detailScale * Math.sin(PETAL_COUNT * angle);
  return {
    x: 50 + x * CURVE_SCALE,
    y: 50 + y * CURVE_SCALE,
  };
}

function buildGuidePath(): string {
  return Array.from({ length: 241 }, (_, index) => {
    const point = curvePoint(index / 240, 0.72);
    return `${index === 0 ? "M" : "L"} ${point.x.toFixed(2)} ${point.y.toFixed(2)}`;
  }).join(" ");
}

function detailScaleAt(now: number): number {
  const pulse = ((now % PULSE_DURATION) / PULSE_DURATION) * Math.PI * 2;
  return 0.52 + ((Math.sin(pulse + 0.55) + 1) / 2) * 0.48;
}

function drawFrame(now: number) {
  const group = groupRef.value;
  if (!group) return;
  if (!startedAt) startedAt = now;
  const elapsed = now - startedAt;
  const detailScale = detailScaleAt(elapsed);
  const rotation = -((elapsed % ROTATION_DURATION) / ROTATION_DURATION) * 360;
  const head = (elapsed % DURATION) / DURATION;

  group.setAttribute("transform", `rotate(${rotation.toFixed(2)} 50 50)`);

  trail.forEach((circle, index) => {
    const tailOffset = trail.length <= 1 ? 0 : index / (trail.length - 1);
    const point = curvePoint(
      normalize(head - tailOffset * TRAIL_SPAN),
      detailScale,
    );
    const fade = Math.pow(1 - tailOffset, 0.56);
    circle.setAttribute("cx", point.x.toFixed(2));
    circle.setAttribute("cy", point.y.toFixed(2));
    circle.setAttribute("r", (0.9 + fade * 2.7).toFixed(2));
    circle.setAttribute("opacity", (0.04 + fade * 0.96).toFixed(2));
  });

  animationFrame = requestAnimationFrame(drawFrame);
}

onMounted(() => {
  const group = groupRef.value;
  if (!group) return;

  const guide = document.createElementNS(SVG_NS, "path");
  guide.setAttribute("d", buildGuidePath());
  guide.setAttribute("fill", "none");
  guide.setAttribute("stroke", "currentColor");
  guide.setAttribute("stroke-width", "1.2");
  guide.setAttribute("opacity", "0.12");
  guide.setAttribute("stroke-linecap", "round");
  group.appendChild(guide);

  trail = Array.from({ length: Math.max(12, props.particles) }, () => {
    const circle = document.createElementNS(SVG_NS, "circle");
    circle.setAttribute("fill", props.color || "currentColor");
    group.appendChild(circle);
    return circle;
  });

  const reduceMotion = window.matchMedia?.(
    "(prefers-reduced-motion: reduce)",
  ).matches;
  if (reduceMotion) {
    drawFrame(performance.now());
    cancelAnimationFrame(animationFrame);
    return;
  }
  animationFrame = requestAnimationFrame(drawFrame);
});

onUnmounted(() => cancelAnimationFrame(animationFrame));
</script>

<style scoped>
.curve-loader {
  color: rgb(var(--v-theme-primary, 25, 118, 210));
  flex: 0 0 auto;
}
.curve-loader svg {
  display: block;
  width: 100%;
  height: 100%;
  overflow: visible;
}
</style>
