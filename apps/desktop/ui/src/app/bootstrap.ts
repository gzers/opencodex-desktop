// 应用启动数据装配（IMP-04 §5/§13.7）：把「进入应用时先要拉一轮的功能数据」集中到一处，
// 供 useAppController 的初始化调用。仅做编排，不含展示逻辑；各功能仍由自己的 store 方法负责取数。
import type { useAppStore } from '@/stores/app'

type AppStore = ReturnType<typeof useAppStore>

/**
 * 启动 / 重新初始化时拉取各功能的初始数据。
 * 不等待完成（事件流与状态快照会随后自行收敛）；失败仍由各功能方法内部收敛，不在此吞错或改写状态。
 */
export function primeAppData(app: AppStore) {
  void app.loadDataRootConfig()
  void app.loadOfficialProject()
  void app.refreshEnvironment()
  // 运行来源（IMP FZ-48）：卡片事实 + 安装进度事件，与状态流并列。
  void app.loadRuntimeSource()
  void app.loadStatusSnapshot()
  void app.loadNotifications()
  void app.loadExtensionConfig()
  void app.loadSyncConfig()
  void app.loadExtensions()
}
