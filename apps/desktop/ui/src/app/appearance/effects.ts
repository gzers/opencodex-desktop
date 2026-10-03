// 三档特效外观策略（UI规范 §26.2 / IMP-05 §3）。
//
// 这是**唯一**把「用户档位 + 系统减少动态 + 页面可见性」折算成有效表现并写到
// `document.documentElement` 的 `data-effects` 的地方。业务组件只读有效策略，
// 不各自监听媒体查询、不各自决定档位。
//
// 规则（§26.2）：
// - 主题与三档正交：主题只改颜色值。
// - 系统减少动态开启时，高档有效表现降为中档；低档仍为低档。
// - 用户保存值与有效表现分开；解除系统限制后恢复用户选择。
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

/** 用户档位 + 系统减少动态 → 有效档位。 */
export function resolveEffectiveEffects(
  setting: VisualEffectsSetting,
  reducedMotion: boolean,
): EffectiveEffects {
  if (setting === 'high' && reducedMotion) return 'mid'
  return setting
}

export interface EffectsStrategy {
  /** 用户保存值（与有效表现分开）。 */
  setting: VisualEffectsSetting
  /** 实际生效档位（已折算系统减少动态）。 */
  effective: EffectiveEffects
  reducedMotion: boolean
  visible: boolean
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
  reducedMotion: boolean,
  visible: boolean,
): EffectsStrategy {
  const effective = resolveEffectiveEffects(setting, reducedMotion)
  return {
    setting,
    effective,
    reducedMotion,
    visible,
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

const REDUCED_MOTION_QUERY = '(prefers-reduced-motion: reduce)'

export const useEffectsStore = defineStore('effects', () => {
  const setting = ref<VisualEffectsSetting>(DEFAULT_VISUAL_EFFECTS)
  const reducedMotion = ref(false)
  const visible = ref(true)

  const strategy = computed(() => effectsStrategy(setting.value, reducedMotion.value, visible.value))
  const effective = computed(() => strategy.value.effective)

  function sync() {
    applyEffectsAttribute(strategy.value.effective)
  }

  function setSetting(next: unknown) {
    setting.value = clampVisualEffects(next)
    sync()
  }

  function setReducedMotion(next: boolean) {
    reducedMotion.value = next
    sync()
  }

  function setVisible(next: boolean) {
    visible.value = next
    sync()
  }

  return {
    setting,
    reducedMotion,
    visible,
    strategy,
    effective,
    setSetting,
    setReducedMotion,
    setVisible,
    sync,
  }
})

let installed = false

/**
 * 装配唯一的系统监听：媒体查询（减少动态）与可见性。
 * 只装一次；在应用启动（`src/main.ts`）调用。测试或非浏览器环境不调用。
 */
export function installEffectsRuntime(): void {
  if (installed || typeof window === 'undefined') return
  installed = true
  const store = useEffectsStore()
  const media = window.matchMedia(REDUCED_MOTION_QUERY)
  store.setReducedMotion(media.matches)
  store.setVisible(document.visibilityState !== 'hidden')
  media.addEventListener('change', (event) => store.setReducedMotion(event.matches))
  document.addEventListener('visibilitychange', () => store.setVisible(document.visibilityState !== 'hidden'))
  store.sync()
}
