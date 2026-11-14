const TAG_PALETTE = [
  "#1976D2",
  "#00897B",
  "#43A047",
  "#F57C00",
  "#E53935",
  "#8E24AA",
  "#5C6BC0",
  "#00838F",
  "#6D4C41",
  "#D81B60",
  "#7CB342",
  "#039BE5",
];

/** 根据标签名稳定生成一个易读的颜色，同名标签始终得到同一颜色。 */
export function smartTagColor(name: string): string {
  let hash = 0;
  for (const char of name) {
    hash = (hash * 31 + char.codePointAt(0)!) >>> 0;
  }
  return TAG_PALETTE[hash % TAG_PALETTE.length];
}
