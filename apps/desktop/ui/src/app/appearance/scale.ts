// 界面缩放：归一化与在真实界面上的应用。
// 与冻结原型 `applyInterfaceScale`（原型/index.html）保持一致：clamp 50–200、四舍五入、
// 空值或无法解析时回退 100。滑块与数字读数共用这一条规则，避免两处各写一套边界。

export const MIN_INTERFACE_SCALE = 50
export const MAX_INTERFACE_SCALE = 200
export const DEFAULT_INTERFACE_SCALE = 100

export function clampScale(value: unknown): number {
  return Math.min(
    MAX_INTERFACE_SCALE,
    Math.max(MIN_INTERFACE_SCALE, Math.round(Number(value) || DEFAULT_INTERFACE_SCALE)),
  )
}

// 缩放在界面上的唯一落点：根元素上的 `--ui-zoom`。
// 主壳 `.app-window`（见 styles/base.css）以 CSS `zoom` 消费它，文字、控件、侧栏、
// 设置、弹窗与通知同源缩放；内嵌官方面板由原生子 WebView 的 `set_zoom` 承担同一因子
// （见 apps/desktop/tauri/src/commands/panel.rs），两处不会叠加。
export function applyInterfaceScale(
  value: unknown,
  root: HTMLElement | null = typeof document === 'undefined' ? null : document.documentElement,
): number {
  const scale = clampScale(value)
  root?.style.setProperty('--ui-zoom', String(scale / 100))
  return scale
}
