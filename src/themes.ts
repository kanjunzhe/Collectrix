export const themeOptions = [
  { value: "light", label: "经典浅色", swatch: "#1976D2" },
  { value: "light-blue", label: "浅蓝", swatch: "#42A5F5" },
  { value: "light-green", label: "浅绿", swatch: "#66BB6A" },
  { value: "light-red", label: "浅红", swatch: "#EF5350" },
  { value: "light-purple", label: "浅紫", swatch: "#AB47BC" },
  { value: "light-amber", label: "浅橙", swatch: "#FFB300" },
  { value: "dark", label: "暗色", swatch: "#121212" },
  { value: "dark-blue", label: "暗蓝", swatch: "#1A2B4A" },
  { value: "dark-green", label: "暗绿", swatch: "#173326" },
  { value: "dark-red", label: "暗红", swatch: "#3A1B1E" },
  { value: "dark-purple", label: "暗紫", swatch: "#2B1A3A" },
] as const;

export type ThemeValue = (typeof themeOptions)[number]["value"] | "system";

export const themeValues: string[] = themeOptions.map((option) => option.value);

/** 上次实际生效的主题（已解析，不会是 system）。 */
export const THEME_CACHE_KEY = "collectrix.theme";

/** 启动时先按缓存的主题渲染，避免先亮后暗闪一下。 */
export function initialTheme(): string {
  try {
    const stored = localStorage.getItem(THEME_CACHE_KEY);
    if (stored && themeValues.includes(stored)) {
      return stored;
    }
  } catch {
    // ignore storage errors
  }
  return "light";
}

/** 记住解析后的主题，下次启动首屏直接用。 */
export function rememberTheme(resolved: string): void {
  try {
    localStorage.setItem(THEME_CACHE_KEY, resolved);
  } catch {
    // ignore storage errors
  }
}

/** 将设置值解析为 Vuetify 主题名；system 跟随操作系统。 */
export function resolveTheme(value: string): string {
  let resolved: string;
  if (value === "system") {
    resolved = window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  } else {
    resolved = themeValues.includes(value) ? value : "light";
  }
  rememberTheme(resolved);
  if (!resolved.startsWith("dark")) {
    try {
      localStorage.setItem("collectrix.lastLightTheme", resolved);
    } catch {
      // ignore storage errors
    }
  }
  return resolved;
}

/** 上次使用的浅色主题，供标题栏的深浅切换恢复。 */
export function lastLightTheme(): string {
  try {
    const stored = localStorage.getItem("collectrix.lastLightTheme");
    return stored &&
      !stored.startsWith("dark") &&
      themeValues.includes(stored)
      ? stored
      : "light";
  } catch {
    return "light";
  }
}
