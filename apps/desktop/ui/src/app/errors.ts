// 运行期异常兜底（IMP-04 §14.3）：Vue 错误边界、未处理 rejection、启动 catch。
// 责任位置：`useAppStore().reportUiFault`（记录事件 + 置位界面故障提示）；恢复方式：界面故障条提供
// 「重新加载界面」与「关闭提示」。捕获不吞异常——故障事实会进入最近事件与界面提示，测试不被变绿。
import type { App } from 'vue'
import type { useAppStore } from '@/stores/app'

type AppStore = ReturnType<typeof useAppStore>

// 脱敏：遮蔽带凭据 URL 与疑似长令牌，避免把敏感值带进界面/日志（§14.3）。
export function sanitizeFaultText(text: string): string {
  return text
    .replace(/([a-z][a-z0-9+.-]*:\/\/)[^/@\s]+@/gi, '$1***@')
    .replace(/\b[A-Za-z0-9_-]{32,}\b/g, '***')
    .slice(0, 300)
}

export function describeFault(scope: string, error: unknown): string {
  const detail =
    error instanceof Error
      ? `${error.name}: ${error.message}`
      : typeof error === 'string'
        ? error
        : (() => {
            try {
              return JSON.stringify(error)
            } catch {
              return String(error)
            }
          })()
  return `${scope}：${sanitizeFaultText(detail || '未知错误')}`
}

// Vue 传给 `errorHandler` 的第三参数是「出错位置」标识，取值随构建环境而变：
//   开发构建 → `ErrorTypeStrings[type]`（形如 `render function`、`native event handler`）
//   生产构建 → 错误文档链接 `https://vuejs.org/error-reference/#runtime-N`
//   （见 `@vue/runtime-core` 的 `handleError`）
// 两者都是**英文技术串**，直接拼进兜底条违反 UI规范 §10（用户可见文案不得夹带英文），
// 因此这里做一次归一：链接丢弃、已知 Vue 标识映射中文、其余含拉丁字母的标识一律丢弃。
const VUE_FAULT_INFO_LABELS: Record<string, string> = {
  'setup function': '初始化',
  'render function': '渲染',
  'component render function': '渲染',
  'scheduler flush': '调度',
  'watcher callback': '侦听器',
  'watcher getter': '侦听器',
  'lifecycle hook': '生命周期',
  directive: '指令',
  component: '组件',
  'v-on handler': '事件处理',
  'native event handler': '事件处理',
  'component event handler': '事件处理',
  'async component loader': '异步组件',
  'app errorHandler': '错误边界',
}

export function readableFaultInfo(info: string | null | undefined): string | undefined {
  const trimmed = (info ?? '').trim()
  if (!trimmed) return undefined
  if (/^https?:\/\//i.test(trimmed)) return undefined
  const label = VUE_FAULT_INFO_LABELS[trimmed]
  if (label) return label
  // 未知且含拉丁字母：不进用户可见文案（英文技术串）。
  if (/[A-Za-z]/.test(trimmed)) return undefined
  return trimmed.slice(0, 60)
}

/** 安装全局兜底：Vue 错误边界 + 未处理 rejection。幂等（重复安装先移除旧监听）。 */
export function installErrorHandlers(appInstance: App, store: AppStore): void {
  appInstance.config.errorHandler = (error, _instance, info) => {
    store.reportUiFault(describeFault('界面异常', error), readableFaultInfo(info))
  }

  const previous = (window as unknown as { __ocxFaultRejectionHandler?: (e: PromiseRejectionEvent) => void })
    .__ocxFaultRejectionHandler
  if (previous) window.removeEventListener('unhandledrejection', previous)
  const handler = (event: PromiseRejectionEvent) => {
    store.reportUiFault(describeFault('未处理异步失败', event.reason))
  }
  ;(window as unknown as { __ocxFaultRejectionHandler?: (e: PromiseRejectionEvent) => void })
    .__ocxFaultRejectionHandler = handler
  window.addEventListener('unhandledrejection', handler)
}
