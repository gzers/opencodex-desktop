<script setup lang="ts">
// 概览状态形象：云形轮廓 + 内部聚合结构（A 完整内核 / B 分离三球 / C 柔性连接）+ 多光源背景。
// 纯装饰层（aria-hidden），主线文字在父页；只消费投影状态，不采集状态、不推定操作成功。
//
// 档位与生命周期只经由 `app/appearance/effects.ts` 的**唯一外观策略**消费；
// 背景光渲染方式（WEBGL 网格渐变＋颗粒 / 纯 CSS 极光）只经由 `app/appearance/glowRender.ts`。
// - 高档且可见 → 连续动画；中档 → 静态目标（状态改变重绘）；低档 → 静态目标且光场由 CSS 关闭。
// - 选 WEBGL 且设备支持 → 网格着色器层；否则回退纯 CSS 极光（同一套投影配色与运动模型）。
// 组件不再各自监听系统媒体查询，也不各自探测设备能力，避免覆盖与重复实现。
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { MotionStateId } from '../motion/projection'
import { createMotionScene, type MotionSceneHandle } from '../motion/scene'
import { createMeshGlow, type MeshGlowHandle } from '../motion/meshGlow'
import { useEffectsStore } from '@/app/appearance/effects'
import { useGlowRenderStore } from '@/app/appearance/glowRender'
import { useThemeStore } from '@/app/appearance/theme'

const props = defineProps<{ state: MotionStateId; active?: boolean }>()
const effects = useEffectsStore()
const glowRender = useGlowRenderStore()
const theme = useThemeStore()

const ambientEl = ref<HTMLElement | null>(null)
const markEl = ref<HTMLElement | null>(null)
const meshHostEl = ref<HTMLElement | null>(null)
let scene: MotionSceneHandle | null = null
let mesh: MeshGlowHandle | null = null
let lastGlowColors: readonly string[] = []

// 中/低档为静态目标；自由运行只在高档且可见时开启。
const reduced = computed(() => effects.strategy.effective !== 'high')
const animating = computed(() => effects.strategy.animated && (props.active ?? true))
const lightActive = computed(() => effects.strategy.lightAllowed && (props.active ?? true))
// 低档无光场，不创建着色器层（与 CSS 侧 `html[data-effects="low"] .motion-ambient{display:none}` 对齐）。
const meshEnabled = computed(() => glowRender.mode === 'mesh' && effects.strategy.effective !== 'low')

function mountMesh(): void {
  if (mesh || !meshHostEl.value) return
  const handle = createMeshGlow(meshHostEl.value, {
    getDark: () => document.documentElement.getAttribute('data-theme') === 'dark',
  })
  if (!handle) {
    // 创建失败：把能力探测收回为不支持，交给统一策略回退 CSS。
    glowRender.setWebglSupported(false)
    return
  }
  mesh = handle
  if (lastGlowColors.length) mesh.setColors(lastGlowColors)
  mesh.setReduced(reduced.value)
  mesh.setActive(lightActive.value)
}

function unmountMesh(): void {
  mesh?.destroy()
  mesh = null
}

onMounted(() => {
  if (!ambientEl.value || !markEl.value) return
  scene = createMotionScene({
    ambient: ambientEl.value,
    markHost: markEl.value,
    onGlow: (colors) => {
      lastGlowColors = colors
      mesh?.setColors(colors)
    },
  })
  scene.setReduced(reduced.value)
  scene.setState(props.state, true)
  scene.setActive(animating.value)
  if (meshEnabled.value) mountMesh()
})

watch(
  () => props.state,
  (state) => scene?.setState(state),
)
watch(
  animating,
  (active) => scene?.setActive(active),
)
watch(
  reduced,
  (value) => {
    scene?.setReduced(value)
    mesh?.setReduced(value)
  },
)
watch(
  lightActive,
  (value) => mesh?.setActive(value),
)
watch(meshEnabled, (enabled) => (enabled ? mountMesh() : unmountMesh()))
// 主题只改颜色与增益：重算一次着色器外观，不重建图层。
watch(
  () => theme.resolved,
  () => mesh?.setColors(lastGlowColors),
)

onBeforeUnmount(() => {
  unmountMesh()
  scene?.destroy()
  scene = null
})
</script>

<template>
  <div class="motion-mark" aria-hidden="true">
    <div ref="meshHostEl" class="motion-mesh-host"></div>
    <div ref="ambientEl" class="motion-ambient">
      <i class="energy"></i>
      <i class="cloud cloud-a"></i>
      <i class="cloud cloud-b"></i>
      <i class="cloud cloud-c"></i>
      <i class="light-floor"></i>
    </div>
    <div ref="markEl" class="motion-hero"></div>
  </div>
</template>
