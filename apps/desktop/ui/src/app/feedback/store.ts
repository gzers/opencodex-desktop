import { defineStore } from "pinia"
import { allowsFeedbackSignal } from "@/config/eventRegistry"
import type { RecentEvent } from "@/types/ui"

/**
 * 界面反馈域（IMP-04 §19.4 E 切片十四）。
 *
 * 方案要求「Toast、弹窗队列与后台持久通知分别管理，不混为同一个总 store」（§19.5）：
 * 弹窗队列在 `app/modal/store.ts`、后台通知在 `features/notifications/store.ts`，
 * Toast 与运行期异常故障条归这里。它也持有「最近事件」登记（状态迁移与界面异常的
 * 本机真实观测），供概览展示。
 *
 * 本 store 不依赖其它 app store，供根壳与各功能调用，避免循环依赖。
 */
/** 原型口径（index.html:7137）：普通 2200 ms、错误 5200 ms。 */
export const TOAST_INFO_MS = 2200
export const TOAST_ERROR_MS = 5200

export const useFeedbackStore = defineStore("feedback", {
  state: () => ({
    toast: "",
    toastRevision: 0,
    // 运行期异常兜底（IMP-04 §14.3）：非空时界面顶部显示故障条，提供恢复动作。
    uiFault: "",
    // 「最近事件」只登记本机真实观测到的状态迁移，不再使用原型固定样例。
    recentEvents: [] as RecentEvent[],
  }),
  actions: {
    /**
     * 瞬时提示。停留时长与原型一致：普通 2200 ms、错误 5200 ms，并允许调用点覆盖。
     * 这一层不持久、不进列表；要留痕的内容必须另写通知实体。
     */
    showToast(message: string, options?: { kind?: 'info' | 'error'; durationMs?: number }) {
      if (!allowsFeedbackSignal("ui-toast")) return
      this.toast = message
      const revision = ++this.toastRevision
      const duration = options?.durationMs ?? (options?.kind === 'error' ? TOAST_ERROR_MS : TOAST_INFO_MS)
      window.setTimeout(() => {
        if (this.toastRevision === revision) this.toast = ""
      }, duration)
    },
    clearToast() {
      this.toastRevision += 1
      this.toast = ""
    },
    recordEvent(message: string) {
      if (!allowsFeedbackSignal("ui-recent-event")) return
      const time = new Date().toLocaleTimeString("zh-CN", { hour12: false })
      this.recentEvents = [{ time, message }, ...this.recentEvents].slice(0, 8)
    },
    // 运行期异常兜底的责任位置：记录事实并置位界面故障条（不吞异常）。
    reportUiFault(message: string, info?: string) {
      if (!allowsFeedbackSignal("ui-fault")) return
      this.uiFault = info ? `${message}（${info}）` : message
      this.recordEvent(`界面异常：${message}`)
    },
    clearUiFault() {
      this.uiFault = ""
    },
  },
})
