// 三档特效外观策略（UI规范 §26.2 / IMP-05 §3）。
//
// 这是**唯一**把「用户档位 + 页面/原生窗口可见性」折算成绘制策略并写到
// `document.documentElement` 的 `data-effects` 的地方。业务组件只读有效策略，
// 不各自监听媒体查询、不各自决定档位。
//
// 规则（§26.2）：
// - 主题与三档正交：主题只改颜色值。
// - 高/中/低是唯一画质选择，不读取系统减少动态，不增加独立动态开关。
// - 前后台暂停不改变用户档位；能力回退由 glowRender 独立说明。
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const VISUAL_EFFECTS = ['high', 'mid', 'low'] as const
export type VisualEffectsSetting = (typeof VISUAL_EFFECTS)[number]
export type EffectiveEffects = VisualEffectsSetting

export const DEFAULT_VISUAL_EFFECTS: VisualEffectsSetting = 'high'

/** 归一化偏好入参：非法值回退默认高档，不抛出、不臆造档位。 */
export function clampVisualEffects(value: unknown): VisualEffectsSetting {
  return VISUAL_EFFECTS.includes(value as VisualEffectsSetting)
    ? (value as VisualEffectsSetting)
    : DEFAULT_VISUAL_EFFECTS
}

/** 用户档位就是有效档位；生命周期只控制是否调度。 */
export function resolveEffectiveEffects(
  setting: VisualEffectsSetting,
): EffectiveEffects {
  return setting
}

export interface EffectsStrategy {
  /** 用户保存值（与有效表现分开）。 */
  setting: VisualEffectsSetting
  /** 实际生效档位，与用户选择一致。 */
  effective: EffectiveEffects
  visible: boolean
  /** 前台高/中档均可画光场；中档按需静态绘制。 */
  lightAllowed: boolean
  /** 连续装饰动画（rAF 循环）仅在高档且可见时允许。 */
  ambientAllowed: boolean
  /** 是否允许绘制动效（中/低档为静态目标，仍随状态改变重绘）。 */
  animated: boolean
  /** 低档：统一实底、无光场、无 backdrop-filter。 */
  solid: boolean
}

/** 纯函数策略：便于测试与在非组件环境复用。 */
export function effectsStrategy(
  setting: VisualEffectsSetting,
  visible: boolean,
): EffectsStrategy {
  const effective = resolveEffectiveEffects(setting)
  return {
    setting,
    effective,
    visible,
    lightAllowed: effective !== 'low' && visible,
    ambientAllowed: effective === 'high' && visible,
    animated: effective === 'high' && visible,
    solid: effective === 'low',
  }
}

/** 把有效档位写到根元素；这是档位属性的唯一写入点。 */
export function applyEffectsAttribute(
  effective: EffectiveEffects,
  root: HTMLElement | null = typeof document === 'undefined' ? null : document.documentElement,
): void {
  root?.setAttribute('data-effects', effective)
}

export const useEffectsStore = defineStore('effects', () => {
  const setting = ref<VisualEffectsSetting>(DEFAULT_VISUAL_EFFECTS)
  const visible = ref(true)
  const foreground = ref(true)

  const strategy = computed(() => effectsStrategy(setting.value, visible.value && foreground.value))
  const effective = computed(() => strategy.value.effective)

  function sync() {
    applyEffectsAttribute(strategy.value.effective)
  }

  function setSetting(next: unknown) {
    setting.value = clampVisualEffects(next)
    sync()
  }

  function setVisible(next: boolean) {
    visible.value = next
    sync()
  }

  function setForeground(next: boolean) { foreground.value = next }

  return {
    setting,
    visible,
    foreground,
    setForeground,
    strategy,
    effective,
    setSetting,
    setVisible,
    sync,
  }
})

let installed = false

/**
 * 装配唯一的生命周期监听；系统减少动态不参与画质。
 * 只装一次；在应用启动（`src/main.ts`）调用。测试或非浏览器环境不调用。
 */
export function installEffectsRuntime(): void {
  if (installed || typeof window === 'undefined') return
  installed = true
  const store = useEffectsStore()
  store.setVisible(document.visibilityState !== 'hidden')
  document.addEventListener('visibilitychange', () => store.setVisible(document.visibilityState !== 'hidden'))
  store.sync()
  // Native activation covers switching apps and hidden/minimized windows; document
  // visibility alone remains "visible" in WKWebView when another app is in front.
  if ('__TAURI_INTERNALS__' in window) {
    store.setForeground(false)
    void (async () => {
      const { listen } = await import('@tauri-apps/api/event')
      const { invoke } = await import('@tauri-apps/api/core')
      let revision = 0
      await listen<boolean>('app-foreground-changed', event => {
        revision++
        store.setForeground(event.payload)
      })
      const before = revision
      const current = await invoke<boolean>('app_foreground')
      if (before === revision) store.setForeground(current)
    })().catch(() => {
      const syncFocus = () => store.setForeground(document.hasFocus())
      window.addEventListener('focus', syncFocus)
      window.addEventListener('blur', syncFocus)
      syncFocus()
    })
  }
}
