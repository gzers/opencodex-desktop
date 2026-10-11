<script setup>
import { ref, nextTick, onMounted, onBeforeUnmount, watch } from 'vue'
import RuntimeMotionMark from '../src/features/runtime/components/RuntimeMotionMark.vue'
import { useEffectsStore } from '../src/app/appearance/effects'
import { useGlowRenderStore, detectWebglSupport } from '../src/app/appearance/glowRender'
import { useThemeStore } from '../src/app/appearance/theme'
const props = defineProps(['metrics'])
const size = ref('standard'), mode = ref('ready'), theme = ref('light'), quality = ref('mid'), foreground = ref(true)
const effects = useEffectsStore(), glow = useGlowRenderStore(), themes = useThemeStore()
effects.setSetting('mid'); glow.setWebglSupported(detectWebglSupport()); glow.setSetting('mesh'); themes.apply('light', { persistCache: false })
watch(size, value => document.documentElement.setAttribute('data-overview-size', value))
watch(theme, value => themes.apply(value, { persistCache: false }))
watch(quality, value => effects.setSetting(value))
watch(foreground, value => effects.setForeground(value))
const report = ref(''), draws = ref(0)
let timer
function sample() {
  const host = document.querySelector('.motion-mesh-host'), svg = document.querySelector('.motion-hero svg'), backdrop = document.querySelector('.motion-backdrop'), main = document.querySelector('main'), canvas = document.querySelector('canvas.motion-mesh')
  if (!host || !svg || !canvas) return
  const rect = el => { const r = el.getBoundingClientRect(); return { x:r.x, y:r.y, width:r.width, height:r.height } }
  const h = rect(host), s = rect(svg), b = rect(backdrop), m = rect(main)
  const expected = (s.y + s.height / 2 - h.y) / h.height
  draws.value = props.metrics.draws
  report.value = JSON.stringify({ size:size.value, mode:mode.value, theme:theme.value, quality:quality.value, foreground:foreground.value, webgl:glow.mode, host:h, logo:s, backdrop:b, main:m, cy:props.metrics.cy, expected, centerErrorPx:(props.metrics.cy-expected)*h.height, drawingBuffer:[canvas.width,canvas.height], dpr:window.devicePixelRatio, draws:props.metrics.draws, topCoverage:b.y<=m.y+1, leftCoverage:b.x<=m.x+1, horizontalOverflow:main.scrollWidth>main.clientWidth, teleported:backdrop.parentElement.matches('.route-section.ov') }, null, 2)
}
async function capture() {
  await nextTick()
  await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))
  sample()
}
onMounted(() => { timer = setInterval(sample, 250) })
onBeforeUnmount(() => clearInterval(timer))
</script>
<template>
  <div class="qa-controls">
    <label>布局<select v-model="size"><option value="standard">标准</option><option value="compact">紧凑</option></select></label>
    <label>模式<select v-model="mode"><option value="ready">就绪</option><option value="setup">准备</option></select></label>
    <label>主题<select v-model="theme"><option value="light">浅色</option><option value="dark">深色</option></select></label>
    <label>画质<select v-model="quality"><option value="mid">中</option><option value="high">高</option><option value="low">低</option></select></label>
    <label><input v-model="foreground" type="checkbox">模拟前台</label>
    <span>绘制 {{ draws }}</span>
    <button class="btn" @click="capture">采样当前布局</button>
  </div>
  <div class="qa-shell"><main class="main"><section class="route-section active ov" :data-mode="mode">
    <header class="topbar"><div class="page-title"><h1>概览</h1><p>生产组件与真实 WebGL：光场中心定位</p></div></header>
    <section class="motion-overview"><div class="motion-fixed"><div class="motion-stage"><RuntimeMotionMark state="running" :active="true" /></div><div class="motion-identity"><strong>{{ mode==='ready'?'就绪':'准备' }}</strong></div><div class="motion-actions"><button class="btn primary">启动 OpenCodex</button></div></div></section>
    <article class="card"><h2>四种布局验证</h2><p>只验证光场定位与覆盖，不计为原生性能或安装验收。</p></article>
  </section></main></div>
  <pre id="mesh-center-report">{{ report }}</pre>
</template>
<style>
.qa-controls { display:flex; gap:12px; padding:12px; align-items:center; }
.qa-controls label { display:flex; gap:8px; align-items:center; }
.qa-shell { height:760px; width:min(889px,100vw); position:relative; isolation:isolate; }
#mesh-center-report { font-size:12px; white-space:pre-wrap; padding:12px; }
</style>
