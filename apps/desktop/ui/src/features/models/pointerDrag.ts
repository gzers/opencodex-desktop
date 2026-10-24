/**
 * 行拖拽排序（指针事件版）。
 *
 * 为什么不用 HTML5 Drag and Drop：
 * 在 macOS WKWebView 里，表格滚动容器内的 `dragstart` 能触发，但 `drop` 常常不触发，
 * 排序静默失败；同时浏览器会顺手把经过的文字框选出来（0.2.7 真机复核复现）。
 * 指针事件在各引擎与触屏上行为一致，且能显式压住选择。
 *
 * 约定：
 * - 按下位置在按钮 / 输入 / 下拉 / 链接 / 可编辑区上时不接管，正常交互优先。
 * - 移动超过阈值才进入拖拽，未过阈值按普通点击处理（不吞点击）。
 * - 进入拖拽后立刻清空选择，并给根节点加 `ocx-pointer-drag`（CSS 统一禁用选择）。
 * - Esc / 指针取消都能干净收尾，不留下半透明行。
 */
import { onBeforeUnmount } from 'vue'

const ACTIVE_CLASS = 'ocx-pointer-drag'
const THRESHOLD = 5

export interface PointerDragOptions {
  /** 行选择器，例如 'tr[data-model]' */
  rowSelector: string
  /** 行上承载标识的属性名，例如 'data-model' */
  itemAttr: string
  /** 当前是否允许排序；false 时不接管 */
  enabled: () => boolean
  /** 落到目标行时回调（起止 id 不同才回调） */
  onReorder: (id: string, targetId: string) => void
  /** 拖拽中的行 id（null 表示结束），用于行高亮 */
  onActive?: (id: string | null) => void
  /** 悬停的目标行 id（null 表示无） */
  onOver?: (id: string | null) => void
}

export function usePointerDrag(options: PointerDragOptions) {
  let armed: { id: string; x: number; y: number; pointerId: number } | null = null
  let active = false
  let overId: string | null = null
  let listening = false

  function rowIdAt(x: number, y: number): string | null {
    const el = document.elementFromPoint?.(x, y) as HTMLElement | null
    const row = el?.closest?.(options.rowSelector) as HTMLElement | null
    return row?.getAttribute(options.itemAttr) ?? null
  }

  function setOver(next: string | null) {
    if (next === overId) return
    overId = next
    options.onOver?.(next)
  }

  function cleanup() {
    if (active) {
      document.documentElement.classList.remove(ACTIVE_CLASS)
      options.onActive?.(null)
    }
    setOver(null)
    armed = null
    active = false
    if (listening) {
      window.removeEventListener('pointermove', onMove, true)
      window.removeEventListener('pointerup', onUp, true)
      window.removeEventListener('pointercancel', onCancel, true)
      window.removeEventListener('keydown', onKey, true)
      listening = false
    }
  }

  function beginActive() {
    active = true
    document.documentElement.classList.add(ACTIVE_CLASS)
    // 清掉按下瞬间可能已经开始的选择，避免“拖一下框住一片字”。
    window.getSelection?.()?.removeAllRanges()
    options.onActive?.(armed!.id)
  }

  function onMove(event: PointerEvent) {
    if (!armed) return
    if (!active) {
      if (Math.abs(event.clientX - armed.x) < THRESHOLD && Math.abs(event.clientY - armed.y) < THRESHOLD) return
      if (!options.enabled()) { cleanup(); return }
      beginActive()
    }
    event.preventDefault()
    const target = rowIdAt(event.clientX, event.clientY)
    setOver(target && target !== armed.id ? target : null)
  }

  function onUp() {
    const source = armed?.id
    const target = overId
    const wasActive = active
    cleanup()
    if (wasActive && source && target && source !== target && options.enabled()) options.onReorder(source, target)
  }

  function onCancel() { cleanup() }

  function onKey(event: KeyboardEvent) {
    if (event.key === 'Escape' && armed) { event.preventDefault(); cleanup() }
  }

  /** 绑定到行的 pointerdown。 */
  function start(event: PointerEvent, id: string) {
    if (!options.enabled() || event.button !== 0) return
    // 触屏上「滑动列表」与「拖动行」是同一个手势，接管会直接让人滚不动列表。
    // 触屏保留原生滚动，排序走行内 ↑↓ 按钮（原型同样只在鼠标下可拖）。
    if (event.pointerType === 'touch') return
    const target = event.target as HTMLElement | null
    if (target?.closest('button,input,textarea,select,a,label,[contenteditable="true"]')) return
    cleanup()
    armed = { id, x: event.clientX, y: event.clientY, pointerId: event.pointerId }
    if (!listening) {
      window.addEventListener('pointermove', onMove, true)
      window.addEventListener('pointerup', onUp, true)
      window.addEventListener('pointercancel', onCancel, true)
      window.addEventListener('keydown', onKey, true)
      listening = true
    }
  }

  onBeforeUnmount(cleanup)
  return { start, cancel: cleanup }
}

