<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import { useAppStore } from '@/stores/app'
import { getReleaseDetails, type DetailsTarget, type DetailsChannel, type ReleaseDetails } from './releaseDetails'
const props = defineProps<{ target: DetailsTarget; channel: DetailsChannel; version: string | null; updaterNotes?: string | null }>()
const app = useAppStore()
const value = ref<ReleaseDetails | null>(null)
const loading = ref(false)
const failed = ref(false)
const section = ref<HTMLElement | null>(null)
let epoch = 0
let disposed = false
const root = computed(() => app.dataRootConfig?.currentDataRoot ?? app.dataRootPath)
const frozen = computed(() => app.dataRootSwitching || app.dataRootSaving || app.dataRootPendingRestart)
const notes = computed(() => props.target === 'manager' && props.updaterNotes?.trim() ? props.updaterNotes : value.value?.notes)
const link = computed(() => {
  try {
    const raw = value.value?.releaseUrl
    if (!raw || /[\s\\]/.test(raw)) return null
    const url = new URL(raw)
    return url.protocol === 'https:' && url.hostname === 'github.com' && !url.username && !url.password
      && !url.search && !url.hash && url.pathname.endsWith('/releases/tag/v' + props.version) ? url.href : null
  } catch { return null }
})
const ariaLabel = computed(() => props.target === 'manager' ? '桌面管理器更新说明' : 'OpenCodex 面板更新说明')
async function load(force = false) {
  const focusOwner = force && section.value?.contains(document.activeElement) ? document.activeElement : null
  function keepFocus() {
    if (focusOwner && !focusOwner.isConnected && document.activeElement === document.body) {
      (section.value?.querySelector<HTMLElement>('pre, button') ?? section.value)?.focus()
    }
  }
  const own = ++epoch
  const target = props.target, channel = props.channel, version = props.version, currentRoot = root.value
  value.value = null; failed.value = false; loading.value = false
  if (!version || frozen.value) return
  loading.value = true
  await nextTick()
  if (disposed || own !== epoch) return
  keepFocus()
  try {
    const result = await getReleaseDetails(target, channel, version, currentRoot, force)
    if (disposed || own !== epoch) return
    value.value = result
    failed.value = result.notesState === 'failed'
  } catch { if (!disposed && own === epoch) failed.value = true }
  finally {
    if (!disposed && own === epoch) {
      loading.value = false
      await nextTick()
      // A successful retry removes its focused button; preserve modal ownership.
      // A user who moved to another control while awaiting must keep that focus.
      keepFocus()
    }
  }
}
async function open() {
  if (link.value && !await app.openExternalLink(link.value)) app.showToast('网页详情打开失败，请稍后重试。')
}
watch(() => [props.target, props.channel, props.version, root.value, frozen.value], () => void load(), { immediate: true })
onBeforeUnmount(() => { disposed = true; epoch++ })
</script>
<template>
  <section ref="section" class="release-notes" tabindex="-1" :aria-busy="loading">
    <div class="release-notes-head"><h4>更新内容</h4><a v-if="link" :href="link" @click.prevent="open">{{ target === 'manager' ? '查看官方发布记录' : '查看官方包页面' }} ↗</a></div>
    <pre v-if="notes" class="release-notes-body" tabindex="0" :aria-label="ariaLabel">{{ notes }}</pre>
    <p v-if="loading" role="status">正在加载本版本更新说明…</p>
    <p v-else-if="frozen" role="status">数据目录变更待完成，请重启后查看说明。</p>
    <p v-else-if="!version">尚无可核验的更新候选。</p>
    <p v-else-if="failed" role="status">更新说明获取失败；版本检查和安装状态不受影响。</p>
    <p v-else-if="!notes">发布者尚未提供本版本更新说明。</p>
    <div class="release-notes-actions">
      <UiButton v-if="failed" variant="ghost" @click="load(true)">重试说明</UiButton>
    </div>
  </section>
</template>
<style scoped>
.release-notes-head a {color:var(--accent);font-size:var(--text-label);text-underline-offset:3px}
.release-notes-head { display: flex; align-items: baseline; justify-content: space-between; gap: var(--space-3); }
h4 { margin: 0 0 var(--space-3); font-size: var(--text-control); }
time, p { color: var(--muted); font-size: var(--text-label); line-height: 1.6; }
.release-notes-body { max-height: 220px; overflow: auto; white-space: pre-wrap; overflow-wrap: anywhere; margin: 0 0 var(--space-3); padding: var(--space-4); font: inherit; font-size: var(--text-control); line-height: 1.7; background: var(--raised); border: 1px solid var(--border-soft); border-radius: var(--radius-sm); }
.release-notes-actions { display: flex; flex-wrap: wrap; gap: var(--space-2); }
</style>
