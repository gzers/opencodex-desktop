// 诊断中心控制器（IMP-04 §19.4 E：从 useAppController 拆出）。
// 日志分类读取（应用日志/调用日志）与 Doctor 只读摘要。
import { computed, ref } from 'vue'
import { useAppStore } from '@/stores/app'
import { readLogs, type LogKind, type LogsDto } from './logs'
import { doctorReportText as buildDoctorReportText } from './presentation'

export function useDiagnosticsController() {
  const app = useAppStore()

  // 日志分类：应用日志（app.log）/ 调用日志（audit.log）。分类显示名口径见《日志分类展示方案》。
  const logCategory = ref<Extract<LogKind, 'app' | 'audit'>>('app')
  const currentLogKind = computed<LogKind>(() => logCategory.value)
  const logsState = ref<LogsDto | null>(null)
  const logsLoading = ref(false)
  const logsError = ref(false)
  const logs = computed(() => logsState.value?.lines ?? [])

  // 请求序号：快速切换分类时只应用最新一次结果，晚到的旧响应不得覆盖新界面（IMP-04 §13.3 A03）。
  let logsRequestId = 0
  async function loadLogs() {
    const requestId = ++logsRequestId
    const kind = currentLogKind.value
    logsLoading.value = true
    try {
      const result = await readLogs(kind)
      if (requestId !== logsRequestId) return
      logsState.value = result
      logsError.value = false
    } catch {
      if (requestId !== logsRequestId) return
      logsError.value = true
    } finally {
      if (requestId === logsRequestId) logsLoading.value = false
    }
  }

  function setLogCategory(category: Extract<LogKind, 'app' | 'audit'>) {
    if (logCategory.value === category) return
    logCategory.value = category
    void loadLogs()
  }

  const doctorShown = ref(false)
  // 纯文本渲染：doctor 输出是外部进程内容，按文本绑定，不做 HTML 注入。
  const doctorReportText = computed(() => buildDoctorReportText(app.doctorReport, doctorShown.value))

  async function runDoctor() {
    if (app.doctorLoading) return
    doctorShown.value = false
    await app.loadDoctorReport()
    if (!app.doctorError) {
      doctorShown.value = true
      app.showToast('Doctor 只读摘要已更新。')
    } else {
      app.showToast('Doctor 暂不可用；已保留上次结果。')
    }
  }

  return {
    logCategory,
    currentLogKind,
    setLogCategory,
    logs,
    logsState,
    logsLoading,
    logsError,
    loadLogs,
    doctorShown,
    doctorReportText,
    doctorLoading: computed(() => app.doctorLoading),
    doctorError: computed(() => app.doctorError),
    runDoctor,
  }
}
