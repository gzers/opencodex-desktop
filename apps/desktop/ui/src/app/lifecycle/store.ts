import { defineStore } from "pinia"
import { getStatusSnapshot, type StatusSnapshot } from "@/contracts/runtimeStatus"
import { useFeedbackStore } from "@/app/feedback/store"
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
    // 在途时再来的刷新请求：不丢弃，记一笔「还欠一次补拉」，当前请求结束后立刻再拉一次。
    statusQueued: false,
    statusError: false,
    statusSnapshot: null as StatusSnapshot | null,
    stopStatusEventStream: null as (() => void) | null,
    lifecycleState: "stopped" as ProcessLifecycleState,
    processActionBusy: false,
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
    // 状态快照只在真实观测到运行状态迁移时登记一条事件，不臆测结论。
    setStatusSnapshot(snapshot: StatusSnapshot | null) {
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
      }, 1500)
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
      // 成功后自动退出通知；失败/超时保留，等待用户处理（FZ-43.3）。
      this.processAutoDismissTimer = window.setTimeout(() => {
        this.processAutoDismissTimer = null
        this.processProgressOpen = false
      }, 2600)
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
        !snapshot?.matrix
      ) {
        return
      }
      const runtime = snapshot.matrix.runtime
      if (runtime === "starting" || runtime === "pending" || runtime === "stopping") {
        this.processProgressSawTransition = true
      }
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
      const failed = ["starting_failed", "unreachable"].includes(runtime)
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
      this.processProgressOpen = false
    },
    reopenProcessProgress() {
      if (this.processProgressAction) this.processProgressOpen = true
    },
    async requestProcessAction(action: LifecycleAction) {
      if (this.processActionBusy) return undefined
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
        }, 180_000)
      }, action === "stop" ? 15_000 : 35_000)
      try {
        const result = await runProcessAction(action)
        this.lifecycleState = result.lifecycleState
        if (
          result.result === null ||
          result.result === "failed" ||
          result.result === "cancelled"
        ) {
          this.processProgressError =
            action === "stop"
              ? "停止指令未完成；当前状态未被桌面管理器臆测修改。"
              : "启动指令未完成；当前状态未被桌面管理器臆测修改。"
          this.finishProcessAction()
        } else if (action === "stop" && result.result === "stopped") {
          this.finishProcessAction()
        }
        void this.loadStatusSnapshot()
        return result
      } catch (error) {
        this.processProgressError =
          action === "stop"
            ? "停止请求失败；已保留当前状态。"
            : "启动请求失败；已保留当前状态。"
        this.finishProcessAction()
        throw error
      }
    },
    async startStatusEventStream() {
      if (this.stopStatusEventStream) return this.stopStatusEventStream
      try {
        const { listen } = await import("@tauri-apps/api/event")
        const unlisten = await listen<StatusSnapshot>(
          "status-snapshot-changed",
          (event) => {
            this.setStatusSnapshot(event.payload)
            if (
              !["loading", "starting", "stopping"].includes(
                event.payload.matrix.runtime,
              )
            ) {
              this.runtimeState = event.payload.matrix.runtime
            }
            this.statusError = false
            this.statusLoading = false
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
    async loadStatusSnapshot() {
      // 在途时不再「直接丢弃」：记一次待补拉，等当前请求收口后立刻再拉一次（用户 2026-10-02 决策）。
      if (this.statusLoading) {
        this.statusQueued = true
        return
      }
      this.statusLoading = true
      try {
        this.setStatusSnapshot(await getStatusSnapshot())
        if (
          this.statusSnapshot &&
          !["loading", "starting", "stopping"].includes(
            this.statusSnapshot.matrix.runtime,
          )
        ) {
          this.runtimeState = this.statusSnapshot.matrix.runtime
        }
        this.statusError = false
      } catch {
        // 保留上一次快照与 UI 展示；不猜测真实运行状态。
        this.statusError = true
      } finally {
        this.statusLoading = false
        if (this.statusQueued) {
          this.statusQueued = false
          void this.loadStatusSnapshot()
        }
      }
    },
    setRuntimeState(next: RuntimeState) {
      this.runtimeState = next
    },
  },
})
