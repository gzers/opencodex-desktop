// 背景光渲染方式（画质卡片）：WEBGL 网格渐变＋颗粒着色器（默认） / 纯 CSS 极光（兜底）。
//
// 与三档特效正交：特效档决定「有没有光场、动还是静」，本项只决定「用哪种渲染」。
// 纯 CSS 极光不依赖 GPU，环境不支持 WEBGL 时由这里统一降级，组件不再各自探测。
// 这里是 `data-glow-render` 的**唯一**写入点。
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const GLOW_RENDERS = ['mesh', 'css'] as const
export type GlowRenderSetting = (typeof GLOW_RENDERS)[number]

export const DEFAULT_GLOW_RENDER: GlowRenderSetting = 'mesh'

/** 归一化偏好入参：非法值回退默认（网格着色器），不抛出、不臆造。 */
export function clampGlowRender(value: unknown): GlowRenderSetting {
  return GLOW_RENDERS.includes(value as GlowRenderSetting)
    ? (value as GlowRenderSetting)
    : DEFAULT_GLOW_RENDER
}

/** 探测当前环境是否可用 WEBGL；失败或异常一律视为不支持。 */
export function detectWebglSupport(): boolean {
  if (typeof document === 'undefined') return false
  try {
    const canvas = document.createElement('canvas')
    return !!(canvas.getContext('webgl') || canvas.getContext('experimental-webgl'))
  } catch {
    return false
  }
}

/** 用户选择 + 设备能力 → 实际渲染方式；选 WEBGL 但不支持时降级为 CSS。 */
export function resolveGlowRender(setting: GlowRenderSetting, webglSupported: boolean): GlowRenderSetting {
  if (setting === 'mesh' && !webglSupported) return 'css'
  return setting
}

/** 把实际渲染方式写到根元素；这是该属性的唯一写入点。 */
export function applyGlowRenderAttribute(
  mode: GlowRenderSetting,
  root: HTMLElement | null = typeof document === 'undefined' ? null : document.documentElement,
): void {
  root?.setAttribute('data-glow-render', mode)
}

export const useGlowRenderStore = defineStore('glowRender', () => {
  const setting = ref<GlowRenderSetting>(DEFAULT_GLOW_RENDER)
  const webglSupported = ref(false)

  const mode = computed(() => resolveGlowRender(setting.value, webglSupported.value))
  /** 用户选了 WEBGL 但设备不支持：设置页据此给出回退说明并禁用该项。 */
  const fellBack = computed(() => setting.value === 'mesh' && !webglSupported.value)

  function sync() {
    applyGlowRenderAttribute(mode.value)
  }

  function setSetting(next: unknown) {
    setting.value = clampGlowRender(next)
    sync()
  }

  function setWebglSupported(next: boolean) {
    webglSupported.value = next
    sync()
  }

  return { setting, webglSupported, mode, fellBack, setSetting, setWebglSupported, sync }
})

let installed = false

/** 装配设备能力探测与首次落属性；在应用启动（`src/main.ts`）调用一次。 */
export function installGlowRenderRuntime(): void {
  if (installed || typeof window === 'undefined') return
  installed = true
  const store = useGlowRenderStore()
  store.setWebglSupported(detectWebglSupport())
}
