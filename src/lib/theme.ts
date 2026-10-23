/**
 * 主色预设，对应 main.css 里的 [data-accent=…] 变量组。
 * swatch 只用于设置面板的色块显示，和 CSS 里的 primary 保持一致。
 */
export const ACCENTS = [
  { id: 'zinc', label: '锌', swatch: 'oklch(0.21 0.006 285.885)' },
  { id: 'slate', label: '板岩', swatch: 'oklch(0.208 0.042 265.755)' },
  { id: 'stone', label: '石头', swatch: 'oklch(0.216 0.006 56.043)' },
  { id: 'gray', label: '灰色', swatch: 'oklch(0.21 0.034 264.665)' },
  { id: 'neutral', label: '中性', swatch: 'oklch(0.205 0 0)' },
  { id: 'red', label: '红色', swatch: 'oklch(0.577 0.245 27.325)' },
  { id: 'rose', label: '玫瑰', swatch: 'oklch(0.586 0.253 17.585)' },
  { id: 'orange', label: '橙色', swatch: 'oklch(0.646 0.222 41.116)' },
  { id: 'green', label: '绿色', swatch: 'oklch(0.627 0.194 149.214)' },
  { id: 'blue', label: '蓝色', swatch: 'oklch(0.546 0.245 262.881)' },
  { id: 'yellow', label: '黄色', swatch: 'oklch(0.681 0.162 75.834)' },
  { id: 'violet', label: '紫罗兰', swatch: 'oklch(0.541 0.281 293.009)' },
] as const;

export type AccentId = (typeof ACCENTS)[number]['id'];
