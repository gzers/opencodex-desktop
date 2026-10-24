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
/** 距可滚动容器上下边缘多少像素内开始自动滚动。 */
const EDGE = 48
/** 每次自动滚动步进（约每帧）；越靠边缘越快。 */
const MAX_STEP = 14

/**
 * 指针靠近可视区上下边缘时每帧滚多少像素；0 表示不滚。
 *
 * 容器可能超出视口（页面 main 常比窗口高），所以只取它与视口的交集判定边缘。
 * 抽成纯函数是为了能直接测边界：jsdom 没有布局，靠 DOM 测不到这段。
 */
export function edgeStep(pointerY: number, rect: { top: number; bottom: number }, viewportHeight: number): number {
  const top = Math.max(rect.top, 0) + EDGE
  const bottom = Math.min(rect.bottom, viewportHeight) - EDGE
  if (bottom <= top) return 0
  if (pointerY < top) return -Math.ceil(MAX_STEP * Math.min(1, (top - pointerY) / EDGE))
  if (pointerY > bottom) return Math.ceil(MAX_STEP * Math.min(1, (pointerY - bottom) / EDGE))
  return 0
}

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
  let raf = 0
  let last: { x: number; y: number } | null = null
  let scroller: HTMLElement | null = null

  function rowIdAt(x: number, y: number): string | null {
    const el = document.elementFromPoint?.(x, y) as HTMLElement | null
    const row = el?.closest?.(options.rowSelector) as HTMLElement | null
    return row?.getAttribute(options.itemAttr) ?? null
  }

  /**
   * 按下时从行往上找可滚动祖先并记住它。
   *
   * 不在拖动过程中用 elementFromPoint 找：指针移出视口或滚到底时它返回 null，
   * 自动滚动会静默失效（实测过）。表格本身不是滚动容器，真正滚的常是页面 main。
   */
  function scrollerFrom(el: HTMLElement | null): HTMLElement | null {
    while (el && el !== document.body) {
      const style = getComputedStyle(el)
      // 只按 overflow 判定：真实环境里 scrollHeight 才可靠，且 tick 在「滚不动」时
      // 会自动换上一层，不需要在这里先做一次可滚性探测。
      if (/(auto|scroll)/.test(style.overflowY)) return el
      el = el.parentElement
    }
    return null
  }

  /** 拖到视口边缘时自动滚动，否则长列表里拖不过一屏就没法把行放到远处。 */
  function tick() {
    raf = 0
    if (!active || !last || !scroller) return
    const box = scroller
    if (box) {
      const rect = box.getBoundingClientRect()
      const step = edgeStep(last.y, rect, window.innerHeight)
      if (step) {
        const before = box.scrollTop
        box.scrollTop = before + step
        // 滚不动就换下一层可滚动祖先（有的页面表格自己滚，有的页面整体滚）。
        if (box.scrollTop === before) scroller = scrollerFrom(box.parentElement)
      }
    }
    raf = requestAnimationFrame(tick)
  }

  function setOver(next: string | null) {
    if (next === overId) return
    overId = next
    options.onOver?.(next)
  }

  function cleanup() {
    if (raf) { cancelAnimationFrame(raf); raf = 0 }
    last = null
    scroller = null
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
    if (!raf) raf = requestAnimationFrame(tick)
  }

  function onMove(event: PointerEvent) {
    if (!armed) return
    if (!active) {
      if (Math.abs(event.clientX - armed.x) < THRESHOLD && Math.abs(event.clientY - armed.y) < THRESHOLD) return
      if (!options.enabled()) { cleanup(); return }
      beginActive()
    }
    event.preventDefault()
    last = { x: event.clientX, y: event.clientY }
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
    scroller = scrollerFrom(target?.closest(options.rowSelector) as HTMLElement | null)
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

