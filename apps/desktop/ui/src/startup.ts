// 首次落地页（首次打开应用时展示哪个页面）。
//
// 规则（2026-09-24 用户澄清）：**没有显式入口时，看「面板当前是否已启动」**——
// 面板已启动（OpenCodex 运行中、端口可用且非 unhealthy）→ 面板；未启动 → 概览。
// 不再是「启动后自动打开面板」这类偏好开关：落地页跟随面板的真实运行状态。
// 有显式入口（深链 hash、托盘菜单动作）时一律不覆盖——托盘「打开主界面 / 打开扩展面板」等自己决定去哪。
//
// 判定放在应用挂载**之前**：状态快照是异步取回的，若等它返回再跳页，首屏会先渲染概览再跳面板。
import type { RuntimeState, StatusSnapshot } from '@/contracts/runtimeStatus'

export type StartupRoute = '#panel' | '#overview'

/** 运行时仍在过渡态：此期间的快照不足以判断面板是否已启动。 */
const TRANSIENT_RUNTIME: RuntimeState[] = ['loading', 'starting', 'stopping']

/** 运行时是否已定档（可据快照判断面板状态）。 */
export function runtimeSettled(runtime: RuntimeState | undefined | null): boolean {
  return runtime != null && !TRANSIENT_RUNTIME.includes(runtime)
}

/** 面板是否「已启动且可用」：与面板内嵌判定（`loadPanelUrl`）保持同一口径。 */
export function panelReady(snapshot: StatusSnapshot | null): boolean {
  if (!snapshot) return false
  return (
    snapshot.matrix.runtime === 'running' &&
    snapshot.port !== null &&
    snapshot.facts.health !== 'unhealthy'
  )
}

/** 返回应当落地的路由；已有显式 hash 时返回 `null`（表示不要改写）。 */
export function startupRouteHash(
  snapshot: StatusSnapshot | null,
  currentHash: string,
): StartupRoute | null {
  const explicit = currentHash.replace(/^#/, '').trim()
  if (explicit) return null
  return panelReady(snapshot) ? '#panel' : '#overview'
}

/** 在挂载前写入落地页 hash；`readHash()` 随后据此初始化路由。 */
export function applyStartupRoute(
  snapshot: StatusSnapshot | null,
  location: Pick<Location, 'hash'> = window.location,
) {
  const target = startupRouteHash(snapshot, location.hash)
  if (!target || location.hash === target) return
  location.hash = target
}

export interface SettledSnapshotDeps {
  /** 读取当前（可能被 store 缓存的）快照。 */
  read: () => StatusSnapshot | null
  /** 主动取一次快照；失败不抛出。 */
  refresh: () => Promise<unknown>
  /** 等待一小段；默认 `setTimeout`。 */
  delay?: (ms: number) => Promise<void>
  timeoutMs?: number
  intervalMs?: number
}

/**
 * 让 `promise` 最多占用 `ms`：到时或结束都返回，晚到的结果被丢弃。
 *
 * 起点必须在 `await refresh()` **之前** 设限，否则单次 `refresh()` 永不返回时，
 * 一旦 await 住就没有机会再检查 deadline（IMP-04 §13.3 A04）。
 */
async function withinDeadline(promise: Promise<unknown>, ms: number): Promise<'done' | 'timeout'> {
  let timer: ReturnType<typeof setTimeout> | null = null
  const timeout = new Promise<'timeout'>(resolve => {
    timer = setTimeout(() => resolve('timeout'), Math.max(0, ms))
  })
  try {
    const settled = await Promise.race([
      promise.then(() => 'done' as const).catch(() => 'done' as const),
      timeout,
    ])
    return settled
  } finally {
    if (timer !== null) clearTimeout(timer)
  }
}

/**
 * 取到第一份「运行时已定档」的快照；超时则返回最后一次读到的快照（可能仍为 `null`）。
 *
 * 启动时状态采集是异步的：采集器初始为 `not_found`，必须等到首次真实采集落地才能
 * 判断面板是否已启动，否则会把「运行中的面板」误判成概览。等待有上限，不阻塞启动。
 * 上限对**每次** refresh 生效，单次 refresh 挂起也不会无限阻塞（A04）。
 */
export async function waitForSettledSnapshot(
  deps: SettledSnapshotDeps,
): Promise<StatusSnapshot | null> {
  const timeoutMs = deps.timeoutMs ?? 1500
  const intervalMs = deps.intervalMs ?? 60
  const delay = deps.delay ?? ((ms: number) => new Promise<void>(resolve => setTimeout(resolve, ms)))
  const deadline = Date.now() + timeoutMs
  for (;;) {
    const remaining = deadline - Date.now()
    // 每次 refresh 都套上限：挂起的请求不会让循环越过 deadline（A04）。
    // 仍至少发起一次 refresh（`remaining` 为 0 时上限即 0，但微任务里的 refresh 不会被立刻截断）。
    const outcome = await withinDeadline(Promise.resolve(deps.refresh()), Math.max(remaining, 0))
    const snapshot = deps.read()
    if (runtimeSettled(snapshot?.matrix.runtime)) return snapshot
    if (outcome === 'timeout' || Date.now() >= deadline) return snapshot
    await delay(intervalMs)
  }
}

/**
 * 有上限地等待启动前必需的偏好读取；超时按「未读到」继续，不阻塞首帧（A04）。
 */
export async function waitForPreferences(
  load: () => Promise<unknown>,
  timeoutMs = 1500,
): Promise<boolean> {
  return (await withinDeadline(Promise.resolve(load()).catch(() => {}), timeoutMs)) === 'done'
}
