import { createApp } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import App from './App.vue'
import { useAppStore } from '@/stores/app'
import { applyStartupRoute, waitForPreferences, waitForSettledSnapshot } from '@/startup'
import { describeFault, installErrorHandlers } from '@/app/errors'
import {
  installEarlyFaultCapture,
  mountMinimalFaultNotice,
} from '@/app/errors'
import { installEffectsRuntime } from '@/app/appearance/effects'
import { installGlowRenderRuntime } from '@/app/appearance/glowRender'
import { installNativeAppearance } from '@/app/appearance/nativeWindow'
import './styles/tokens.css'
import './styles/base.css'
import './styles/materials.css'
import './styles/effects.css'

const pinia = createPinia()
setActivePinia(pinia)

// 启动早期兜底必须先于任何业务代码装配（F-05）：偏好探针、外观装配或首帧之前的
// 异常此前没有任何记录点。这里的 store 未挂载也能记录故障事实。
installEarlyFaultCapture(useAppStore())

// 首次落地页要先于首帧决定：偏好与状态快照都是异步的，等它们返回再跳页会先闪一下概览。
// 落地页判定依赖「面板是否已启动」，因此必须先等到第一份「运行时已定档」的状态快照；
// 等待有上限（1500 ms），取不到就按概览落地，不无限阻塞启动。
async function bootstrap() {
  const store = useAppStore()
  // 唯一用户三档策略先于首帧装配；系统减少动态不参与。
  installEffectsRuntime()
  // 背景光渲染方式同样先于首帧装配：设备能力只探测一次，用户选择随后由偏好落到 `data-glow-render`。
  installGlowRenderRuntime()
  try {
    // 偏好读取有上限：偏好端挂起时不阻塞首帧，按未读到继续（IMP-04 §13.3 A04）。
    await waitForPreferences(() => store.loadPreferences())
    void installNativeAppearance()
    const snapshot = await waitForSettledSnapshot({
      read: () => store.statusSnapshot,
      refresh: () => store.loadStatusSnapshot(),
    })
    applyStartupRoute(snapshot)
  } catch (error) {
    // 启动 catch：任何启动步骤异常都记录为界面故障，但仍挂载应用以便用户看到恢复入口。
    store.reportUiFault(describeFault('启动异常', error))
  }
  const appInstance = createApp(App)
  installErrorHandlers(appInstance, store)
  appInstance.use(pinia).mount('#app')
}

void bootstrap().catch(error => {
  // 连 Vue 都没起来时的最小恢复入口（F-05）：不依赖样式/模块，直接提供刷新动作。
  mountMinimalFaultNotice(describeFault('启动异常', error))
})

// 审计夹具只在开发环境按显式 `?__audit=1` 安装；正式构建不含入口。
if (import.meta.env.DEV) {
  void import('./dev/auditHarness').then(module => module.installAuditHarness())
}

// 覆盖式滚动条：scroll 不冒泡，用捕获阶段统一打点；停止 0.9s 后撤掉 .is-scrolling。
let scrollQuietTimer: number | null = null
document.addEventListener(
  'scroll',
  event => {
    const target = event.target
    if (!(target instanceof Element)) return
    document.querySelectorAll('.is-scrolling').forEach(node => {
      if (node !== target) node.classList.remove('is-scrolling')
    })
    target.classList.add('is-scrolling')
    if (scrollQuietTimer !== null) window.clearTimeout(scrollQuietTimer)
    scrollQuietTimer = window.setTimeout(() => target.classList.remove('is-scrolling'), 900)
  },
  true,
)
