import { defineStore } from "pinia"
import { getStatusSnapshot, type StatusSnapshot } from "@/contracts/runtimeStatus"
import { useFeedbackStore } from "@/app/feedback/store"
import { createCountdown, type Countdown } from "@/app/feedback/countdown"
import { cancelLeave, leave } from "@/app/feedback/motion"
import { runProcessAction } from "@/features/runtime/processAction"
import { runtimeLabel } from "@/lib/labels"
import {
  runtimeAnnouncements,
  transitionalRuntimeStates,
} from "@/features/runtime/scenarios"
import type {
  LifecycleAction,
  ProcessLifecycleState,
  RuntimeState,
} from "@/types/ui"
import {
  LIFECYCLE_FALLBACK_MS,
  LIFECYCLE_LATE_OBSERVE_MS,
  LIFECYCLE_NOTICE_START_MS,
  LIFECYCLE_NOTICE_STOP_MS,
  STATUS_SAMPLE_MS,
} from "@/config/runtimeDefaults"

const statusRequests = new WeakMap<object, { promise: Promise<void>; cancel: () => void }>()
const STATUS_BUSY_RETRY_MS = 200

function isStatusSamplerBusy(error: unknown): boolean {
  if (!error || typeof error !== 'object') return false
  return 'code' in error && error.code === 14 &&
    'message' in error && typeof error.message === 'string' &&
    error.message.includes('status_sample_in_progress')
}

async function sampleStatus(isCurrent: () => boolean): Promise<StatusSnapshot | null> {
  while (isCurrent()) {
    try {
      return await getStatusSnapshot()
    } catch (error) {
      if (!isStatusSamplerBusy(error)) return null
      await new Promise(resolve => window.setTimeout(resolve, STATUS_BUSY_RETRY_MS))
    }
  }
  return null
}

/** 启停成功卡停留 2600 ms（FZ-43.3）；倒计时对象不可序列化，按模块持有。 */
export const PROCESS_SUCCESS_MS = 2600
export const PROCESS_NOTIFICATION_KEY = "process"
let processCountdown: Countdown | null = null
export function currentProcessCountdown(): Countdown | null { return processCountdown }
function cancelProcessCountdown() {
  processCountdown?.cancel()
  processCountdown = null
  cancelLeave(PROCESS_NOTIFICATION_KEY)
}

/**
 * 运行生命周期与状态域（IMP-04 §19.4 E 切片十五）。
 *
 * 收敛「桌面与后端边界」里最核心的一对事实来源：后端状态快照（事件流 + 显式拉取）
 * 与本机启停/重启进程动作（含进度卡与定时器）。跨功能播报（Toast）经界面反馈域
 * （`app/feedback/store.ts`）完成；本 store **不反向依赖根 store**。
 *
 * 方案把「长任务进度」定位为需要跨页面持续显示时的 feature store 归属（§19.4），
 * 进程进度与状态快照由此处统一承载。定时器/事件流的清理见对应动作。
 */
export const useLifecycleStore = defineStore("lifecycle", {
  state: () => ({
    runtimeState: "loading" as RuntimeState,
    // 启动后的第一份快照不算「状态迁移」，避免一打开应用就弹说明句 Toast。
    runtimeAnnouncePrimed: false,
    statusLoading: false,
    statusObserved: false,
    statusNeedsCurrentSample: false,
    statusEventRevision: 0,
    statusError: false,
    statusSnapshot: null as StatusSnapshot | null,
    stopStatusEventStream: null as (() => void) | null,
    lifecycleState: "stopped" as ProcessLifecycleState,
    lastStartFailed: false,
    processActionBusy: false,
    processRequestPending: false,
    processAction: null as LifecycleAction | null,
    processProgressAction: null as LifecycleAction | null,
    processProgressOpen: false,
    processProgressError: "",
    processProgressTimedOut: false,
    processProgressSawTransition: false,
    // 动作开始时的进程身份：重启后 PID 变化是「真的换了进程」的硬证据，
    // 避免只靠中间态采样（真实重启可能快到采不到 stopping/starting）。
    processProgressStartPid: null as string | null,
    processProgressCompleted: false,
    // 超时后仍继续观察真实状态：迟到的成功要能收口成「已完成」，
    // 不能把「还没等到」长期显示成「未完成」。
    processProgressLateWatch: false,
    processProgressTimer: null as number | null,
    processProgressGiveUpTimer: null as number | null,
    processActionPoll: null as number | null,
    processAutoDismissTimer: null as number | null,
  }),
  actions: {
    processActionBlocked(action: LifecycleAction) {
      return this.processRequestPending || this.statusError ||
        Boolean(this.statusSnapshot?.status_failure) || this.statusSnapshot?.[`can_${action}`] !== true
    },
    applyStatusObservation(snapshot: StatusSnapshot | null) {
      this.statusObserved = true
      const confirmed = snapshot?.matrix && !snapshot.status_failure ? snapshot : null
      this.statusError = !confirmed
      this.setStatusSnapshot(confirmed)
      if (!confirmed) this.runtimeState = 'unreachable'
      else if (!['loading', 'starting', 'stopping'].includes(confirmed.matrix.runtime)) {
        this.runtimeState = this.statusSnapshot?.matrix.runtime ?? confirmed.matrix.runtime
      }
    },
    // 状态快照只在真实观测到运行状态迁移时登记一条事件，不臆测结论。
    setStatusSnapshot(snapshot: StatusSnapshot | null) {
      // A late/stale no-session snapshot cannot discharge an action receipt.
      // Explicit completed recovery/action results own that transition.
      if (snapshot?.matrix.runtime === 'running') this.lastStartFailed=false
      if (snapshot?.matrix.runtime === 'stopped' && this.lastStartFailed) {
        snapshot={...snapshot,matrix:{...snapshot.matrix,runtime:'starting_failed'}}
      }
      const previous = this.statusSnapshot?.matrix.runtime
      this.statusSnapshot = snapshot
      const next = snapshot?.matrix.runtime
      if (next && next !== previous) {
        useFeedbackStore().recordEvent(`运行状态：${runtimeLabel(next)}`)
        this.announceRuntimeTransition(next)
      }
      this.syncProcessProgress(snapshot)
    },
    /**
     * 概览状态卡不再承载说明句（UI规范 §19），说明句按性质分流。
     * 这里只负责「稳定事实」：迁移到该状态时播报**一次** info Toast。
     * 需关注的状态由后端发布持久通知（不在这里重复播报）；瞬时进度不写。
     * 启停 / 重启已由任务卡与结果反馈承载，同样不重复播报。
     */
    announceRuntimeTransition(next: RuntimeState) {
      if (transitionalRuntimeStates.includes(next)) return
      // 第一份「已定档」的快照只用来起锚：应用启动不该播报说明句。
      if (!this.runtimeAnnouncePrimed) {
        this.runtimeAnnouncePrimed = true
        return
      }
      if (this.processAction !== null) return
      const message = runtimeAnnouncements[next]
      if (message) useFeedbackStore().showToast(message)
    },
    clearProcessProgressTimer() {
      if (this.processProgressTimer !== null) {
        window.clearTimeout(this.processProgressTimer)
        this.processProgressTimer = null
      }
      if (this.processProgressGiveUpTimer !== null) {
        window.clearTimeout(this.processProgressGiveUpTimer)
        this.processProgressGiveUpTimer = null
      }
      this.stopProcessActionPoll()
    },
    startProcessActionPoll() {
      this.stopProcessActionPoll()
      // FZ-08：状态主渠道是后端事件流；这里只是兜底轮询，保证事件通道不可用时
      // 任务卡也能在真实状态迁移后收口，而不是一直停在「等待官方状态确认」。
      this.processActionPoll = window.setInterval(() => {
        void this.loadStatusSnapshot()
      }, LIFECYCLE_FALLBACK_MS)
    },
    stopProcessActionPoll() {
      if (this.processActionPoll !== null) {
        window.clearInterval(this.processActionPoll)
        this.processActionPoll = null
      }
    },
    scheduleProcessAutoDismiss() {
      if (this.processAutoDismissTimer !== null) {
        window.clearTimeout(this.processAutoDismissTimer)
      }
      cancelProcessCountdown()
      // 成功后自动退出通知；失败/超时保留，等待用户处理（FZ-43.3）。
      // 倒计时可被悬停 / 焦点暂停，到点后收起到铃铛（S18-09）。
      const countdown = createCountdown({
        durationMs: PROCESS_SUCCESS_MS,
        tone: "success",
        onElapsed: () => {
          if (processCountdown !== countdown) return
          leave(PROCESS_NOTIFICATION_KEY, () => {
            if (processCountdown !== countdown) return
            processCountdown = null
            this.processAutoDismissTimer = null
            this.processProgressOpen = false
          })
        },
      })
      processCountdown = countdown
      this.processAutoDismissTimer = null
    },
    finishProcessAction() {
      this.processActionBusy = false
      this.processProgressLateWatch = false
      this.processAction = null
      this.processProgressCompleted =
        !this.processProgressError && !this.processProgressTimedOut
      this.clearProcessProgressTimer()
      if (this.processProgressCompleted) this.scheduleProcessAutoDismiss()
    },
    syncProcessProgress(snapshot: StatusSnapshot | null) {
      if (
        (!this.processActionBusy && !this.processProgressLateWatch) ||
        !this.processAction ||
        !snapshot?.matrix || snapshot.status_failure
      ) {
        return
      }
      const runtime = snapshot.matrix.runtime
      if (runtime === "starting" || runtime === "pending" || runtime === "stopping") {
        this.processProgressSawTransition = true
      }
      // Readiness can arrive before the lifecycle command returns and releases
      // its backend mutation lease. Keep the original action busy until that
      // exact request settles; the observed snapshot still updates normally.
      if (this.processRequestPending) return
      const action = this.processAction
      const pidChanged = Boolean(
        this.processProgressStartPid &&
          snapshot.pid &&
          snapshot.pid !== this.processProgressStartPid,
      )
      const complete =
        action === "stop"
          ? runtime === "stopped" && this.processProgressSawTransition
          : runtime === "running" &&
            (action === "start" ||
              this.processProgressSawTransition ||
              (action === "restart" && pidChanged))
      // 只有官方明确报告的启动失败才立即判失败（S18-02）。启动/重启后代理端口
      // 尚未监听时会短暂读到 `unreachable`，这是过渡观测而非启动结果：继续等待
      // 运行态或超时，避免「已启动、网页能打开」却被报成未能进入可用状态。
      const failed =
        runtime === "starting_failed" ||
        (action === "stop" && runtime === "unreachable")
      if (complete || failed) {
        if (failed) {
          this.processProgressError =
            "OpenCodex 未能进入可用状态；可以查看日志或重试启动。"
        } else if (this.processProgressLateWatch) {
          // 超时之后才观测到的真实完成：撤销超时提示，按完成收口。
          this.processProgressError = ""
          this.processProgressTimedOut = false
        }
        this.finishProcessAction()
      }
    },
    dismissProcessProgress() {
      if (this.processAutoDismissTimer !== null) {
        window.clearTimeout(this.processAutoDismissTimer)
        this.processAutoDismissTimer = null
      }
      cancelProcessCountdown()
      this.processProgressOpen = false
    },
    /** 手动收起：停掉倒计时，走收起到铃铛动效后再关闭；后台动作不取消。 */
    collapseProcessProgress() {
      processCountdown?.cancel()
      leave(PROCESS_NOTIFICATION_KEY, () => this.dismissProcessProgress())
    },
    reopenProcessProgress() {
      if (this.processProgressAction) this.processProgressOpen = true
    },
    async requestProcessAction(action: LifecycleAction) {
      if (this.processActionBusy || this.processRequestPending || this.processActionBlocked(action)) return undefined
      this.processRequestPending = true
      this.processActionBusy = true
      this.processAction = action
      this.processProgressAction = action
      this.processProgressOpen = true
      this.processProgressError = ""
      this.processProgressTimedOut = false
      this.processProgressSawTransition = false
      this.processProgressStartPid = this.statusSnapshot?.pid ?? null
      this.processProgressCompleted = false
      this.processProgressLateWatch = false
      if (this.processAutoDismissTimer !== null) {
        window.clearTimeout(this.processAutoDismissTimer)
        this.processAutoDismissTimer = null
      }
      cancelProcessCountdown()
      this.clearProcessProgressTimer()
      this.startProcessActionPoll()
      this.processProgressTimer = window.setTimeout(() => {
        if (this.processAction !== action) return
        this.processProgressTimedOut = true
        this.processProgressError =
          "等待时间较长，后台仍会继续观察状态；可以先关闭此窗口。"
        // 不再直接收口：保留状态轮询与动作上下文，真实状态稍后收敛时把任务卡
        // 收口为「已完成」，避免已成功却一直显示「未完成」。
        this.processActionBusy = false
        this.processProgressLateWatch = true
        this.processProgressTimer = null
        this.processProgressGiveUpTimer = window.setTimeout(() => {
          // 迟到的完成也等不到时，才彻底收口并保留超时提示。
          this.processProgressGiveUpTimer = null
          if (this.processAction === action) this.finishProcessAction()
        }, LIFECYCLE_LATE_OBSERVE_MS)
      }, action === "stop" ? LIFECYCLE_NOTICE_STOP_MS : LIFECYCLE_NOTICE_START_MS)
      try {
        const result = await runProcessAction(action)
        this.processRequestPending = false
        this.lifecycleState = result.lifecycleState
        if (result.result === 'started' || result.result === 'stopped') this.lastStartFailed = false
        else if (result.failure && action !== 'stop') this.lastStartFailed=true
        if (result.result==='started') this.lastStartFailed=false
        if (
          result.result === null ||
          result.result === "failed" ||
          result.result === "cancelled"
        ) {
          this.processProgressError = result.failure?.message ?? (
            action === "stop"
              ? "停止指令未完成；当前状态未被桌面管理器臆测修改。"
              : "启动指令未完成；当前状态未被桌面管理器臆测修改。")
          this.finishProcessAction()
        } else if (action === "stop" && result.result === "stopped") {
          this.finishProcessAction()
        } else {
          this.syncProcessProgress(this.statusSnapshot)
        }
        return result
      } catch (error) {
        this.processRequestPending = false
        this.processProgressError =
          action === "stop"
            ? "停止请求失败；已保留当前状态。"
            : "启动请求失败；已保留当前状态。"
        this.finishProcessAction()
        throw error
      } finally {
        this.invalidateStatusObservation()
        void this.loadStatusSnapshot()
      }
    },
    async startStatusEventStream() {
      if (this.stopStatusEventStream) return this.stopStatusEventStream
      try {
        const { listen } = await import("@tauri-apps/api/event")
        const unlisten = await listen<StatusSnapshot>(
          "status-snapshot-changed",
          (event) => {
            if (this.statusNeedsCurrentSample) return
            this.statusEventRevision += 1
            this.applyStatusObservation(event.payload)
          },
        )
        this.stopStatusEventStream = () => {
          void unlisten()
          this.stopStatusEventStream = null
        }
        return this.stopStatusEventStream
      } catch {
        return null
      }
    },
    invalidateStatusObservation() {
      statusRequests.get(this)?.cancel()
      statusRequests.delete(this)
      this.statusNeedsCurrentSample = true
      this.statusLoading = false
      this.statusObserved = false
      this.statusError = false
      this.setStatusSnapshot(null)
      this.runtimeState = 'loading'
    },
    async loadStatusSnapshot() {
      const inFlight = statusRequests.get(this)
      if (inFlight) return inFlight.promise
      this.statusLoading = true
      const revision = this.statusEventRevision
      let active = true
      let timer = 0
      let cancel = () => {}
      const deadline = new Promise<null>(resolve => {
        timer = window.setTimeout(() => resolve(null), STATUS_SAMPLE_MS)
        cancel = () => {
          active = false
          resolve(null)
        }
      })
      const isCurrent = () => active && revision === this.statusEventRevision
      const request = Promise.race([sampleStatus(isCurrent), deadline]).then(snapshot => {
        if (isCurrent()) {
          this.statusNeedsCurrentSample = false
          this.applyStatusObservation(snapshot)
        }
      }).finally(() => {
        active = false
        window.clearTimeout(timer)
        if (statusRequests.get(this)?.promise !== request) return
        statusRequests.delete(this)
        this.statusLoading = false
      })
      statusRequests.set(this, { promise: request, cancel })
      return request
    },
    setRuntimeState(next: RuntimeState) {
      this.runtimeState = next
    },
  },
})
