export type LinkService =
  | "github"
  | "baidu"
  | "quark"
  | "aliyun"
  | "pan123"
  | "lanzou"
  | "google";

export interface LinkCard {
  url: string;
  urlWithPassword: string;
  service: LinkService;
  serviceLabel: string;
  title: string;
  password?: string;
}

const URL_RE =
  /((?:https?:\/\/)?(?:www\.)?(?:github\.com|pan\.baidu\.com|pan\.quark\.cn|aliyundrive\.com|alipan\.com|123pan\.com|123684\.com|lanzou[a-z0-9]*\.com|drive\.google\.com)\/[^\s]+|https?:\/\/[^\s]+)/gi;

const PASSWORD_RE =
  /(?:提取码|提取密码|访问码|密码|pwd|password|code)\s*[:：=]?\s*([A-Za-z0-9]{3,12})/i;

function normalizeUrl(raw: string): string {
  const trimmed = raw.replace(/[),.;!?，。；！？]+$/, "");
  return /^https?:\/\//i.test(trimmed) ? trimmed : `https://${trimmed}`;
}

function serviceFor(url: string): LinkService | null {
  try {
    const host = new URL(url).hostname.toLowerCase();
    if (host.endsWith("github.com")) return "github";
    if (host.endsWith("pan.baidu.com")) return "baidu";
    if (host.endsWith("pan.quark.cn")) return "quark";
    if (host.endsWith("aliyundrive.com") || host.endsWith("alipan.com"))
      return "aliyun";
    if (host.endsWith("123pan.com") || host.endsWith("123684.com"))
      return "pan123";
    if (host.includes("lanzou")) return "lanzou";
    if (host.endsWith("drive.google.com")) return "google";
    return null;
  } catch {
    return null;
  }
}

function serviceLabel(service: LinkService): string {
  switch (service) {
    case "github":
      return "GitHub";
    case "baidu":
      return "百度网盘";
    case "quark":
      return "夸克网盘";
    case "aliyun":
      return "阿里云盘";
    case "pan123":
      return "123 网盘";
    case "lanzou":
      return "蓝奏云";
    default:
      return "Google Drive";
  }
}

function passwordFromText(text: string): string | undefined {
  const match = text.match(PASSWORD_RE);
  return match?.[1];
}

function passwordFromUrl(url: string): string | undefined {
  try {
    const parsed = new URL(url);
    return (
      parsed.searchParams.get("pwd") ??
      parsed.searchParams.get("password") ??
      parsed.searchParams.get("code") ??
      undefined
    );
  } catch {
    return undefined;
  }
}

function withPassword(url: string, password?: string): string {
  if (!password) return url;
  try {
    const parsed = new URL(url);
    if (!parsed.searchParams.get("pwd")) {
      parsed.searchParams.set("pwd", password);
    }
    return parsed.toString();
  } catch {
    return `${url}${url.includes("?") ? "&" : "?"}pwd=${encodeURIComponent(password)}`;
  }
}

function cardTitle(url: string, service: LinkService): string {
  try {
    const parsed = new URL(url);
    const parts = parsed.pathname.split("/").filter(Boolean);
    if (service === "github") {
      return parts.slice(0, 2).join("/").replace(/\.git$/, "") || "github.com";
    }
    return parts[parts.length - 1] ?? parsed.hostname;
  } catch {
    return url;
  }
}

export function linkCardsFromText(text: string): LinkCard[] {
  if (!text) return [];
  const matches = Array.from(text.matchAll(URL_RE));
  if (matches.length === 0) return [];

  const cards: LinkCard[] = [];
  const seen = new Set<string>();

  for (let index = 0; index < matches.length; index++) {
    const match = matches[index];
    const url = normalizeUrl(match[0]);
    const service = serviceFor(url);
    if (!service) continue;

    const dedupeKey = url.replace(/[?#].*$/, "");
    if (seen.has(dedupeKey)) continue;
    seen.add(dedupeKey);

    const start = match.index ?? 0;
    const end = start + match[0].length;
    const previousEnd =
      index === 0
        ? 0
        : (matches[index - 1].index ?? 0) + matches[index - 1][0].length;
    const nextStart =
      index + 1 < matches.length
        ? (matches[index + 1].index ?? text.length)
        : text.length;
    const tail = text.slice(end, nextStart);
    const head = text.slice(previousEnd, start);

    const password =
      passwordFromUrl(url) ??
      passwordFromText(tail) ??
      passwordFromText(head) ??
      passwordFromText(text);

    cards.push({
      url,
      urlWithPassword: withPassword(url, password),
      service,
      serviceLabel: serviceLabel(service),
      title: cardTitle(url, service),
      password,
    });
  }

  return cards;
}
