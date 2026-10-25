<script setup lang="ts">
/**
 * 保存预览（UI 规范 §38）
 *
 * 保存是唯一提交入口，也是预览入口：展示当前草稿的完整选择器顺序与激活预算，
 * 渠道与型号可在此快速启停，修改直接进入同一份草稿。预览固定当前草稿，没有来源切换。
 * 0.2.8 决策：标题统一为「保存预览」（原型 model-config.js:556 同名），不再叫「选择器预览」。
 */
import { computed, ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import type { DraftProjection, ModelPolicy } from './api'

interface PreviewChannel { id: string; label: string; enabled: boolean; protected?: boolean }
interface OtherChange { label: string; before: unknown; after: unknown }

const props = defineProps<{
  open: boolean
  projection: DraftProjection | null
  channels: PreviewChannel[]
  saved: ModelPolicy | null
  otherChanges?: OtherChange[]
  /** 系统占用（原型 model-config.js:518「系统占用 只读」分组）：官方目录里已存在但选择器隐藏的条目。 */
  systemRows?: { id: string; label: string }[]
  busy?: boolean
  saving?: boolean
  error?: string
}>()
const emit = defineEmits<{
  close: []
  confirm: []
  'toggle-channel': [id: string]
  'toggle-model': [channelId: string, modelId: string]
}>()

const rows = computed(() => props.projection?.rows ?? [])
const fastCount = computed(() => rows.value.filter(row => row.rules.fast.mode === 'custom' && row.rules.fast.value).length)
const active = computed(() => props.projection?.knownActiveModels ?? rows.value.filter(row => row.selected).length)
const over = computed(() => active.value > 100)
const catalog = computed(() => props.projection?.exactCatalogCount)
/** 系统占用条目：只读展示，不计入可编辑型号，也不参与启停。 */
const systemRows = computed(() => props.systemRows ?? [])

function savedChannel(id: string) { return props.saved?.channels.find(channel => channel.id === id) }
function savedActive(id: string, modelId: string) {
  const channel = savedChannel(id)
  if (!channel) return false
  return channel.selection.mode === 'all' ? channel.discovered.some(model => model.id === modelId) : channel.selection.mode === 'whitelist' && channel.selection.models.includes(modelId)
}
function channelDelta(channel: PreviewChannel) {
  const before = savedChannel(channel.id)
  if (!before) return { total: 0, text: '新渠道' }
  const changes: string[] = []
  if (before.connection.enabled !== channel.enabled) changes.push(channel.enabled ? '本次启用渠道' : '本次停用渠道')
  let added = 0, removed = 0
  for (const row of rows.value.filter(item => item.channelId === channel.id)) {
    const was = savedActive(channel.id, row.modelId)
    if (row.selected && !was) added++
    else if (!row.selected && was) removed++
  }
  if (added) changes.push('新启用 ' + added + ' 个型号')
  if (removed) changes.push('停用 ' + removed + ' 个型号')
  return { total: changes.length ? added + removed + (before.connection.enabled !== channel.enabled ? 1 : 0) : 0, text: changes.join('；') }
}
function modelState(channelId: string, modelId: string, selected: boolean) {
  const was = savedActive(channelId, modelId)
  if (was === selected) return ''
  return selected ? 'is-added' : 'is-removed'
}
const groups = computed(() => props.channels.map(channel => ({
  channel,
  delta: channelDelta(channel),
  items: rows.value.filter(row => row.channelId === channel.id).map((row, index) => ({ row, order: index + 1, change: modelState(channel.id, row.modelId, row.selected) })),
})))
const notices = computed(() => props.projection?.notices?.filter(notice => !notice.includes('当前为草稿基础模型投影')) ?? [])
/**
 * 摘要标题按内容取词：生产环境这里只会有编译器发的「未在当前清单发现，保留选择待核对」，
 * 但 DEV 夹具与导入路径也会写说明性 notice，不能一律叫「待核对」，否则等于给用户派任务。
 */
const noticesLabel = computed(() => notices.value.every(notice => notice.includes('保留选择待核对'))
  ? notices.value.length + ' 项待核对提示'
  : notices.value.length + ' 项说明')
const others = computed(() => (props.otherChanges ?? []).slice(0, 8))
const otherTotal = computed(() => props.otherChanges?.length ?? 0)
const note = ref('')

/**
 * 触屏与无 hover 环境用长按查看说明（§38.3.1）：
 * 500ms 计时命中后只展示说明，随后抬手产生的 click 必须被吞掉，避免“看说明顺手改了配置”。
 * 鼠标仍走 hover / focus-visible，不参与长按。
 */
const pressNote = ref('')
let pressTimer: ReturnType<typeof setTimeout> | undefined
let swallowClick = false
function clearPress() { if (pressTimer) { clearTimeout(pressTimer); pressTimer = undefined } }
function pressStart(channel: PreviewChannel, event: PointerEvent) {
  if (props.busy || props.saving || channel.protected || event.pointerType === 'mouse') return
  clearPress()
  const changed = pressChannelDelta(channel)
  if (!changed.total) return
  pressTimer = setTimeout(() => {
    pressTimer = undefined
    // 同一次手势只用于查看：命中后抬手不再启停。
    swallowClick = true
    pressNote.value = pressNote.value === channel.id ? '' : channel.id
  }, 500)
}
function pressChannelDelta(channel: PreviewChannel) { return channelDelta(channel) }
function pressEnd() { clearPress() }
function pressLeave() { clearPress(); }
/** §38.3.1：滚动开始也要清理计时器，否则滑动手势会被当成“长按看说明”。 */
function onOrderScroll() { if (pressTimer) { clearPress(); pressNote.value = '' } }
/** §38.3.1「收起」还要求点击空白处：命中标题行以外就收起，点击标题行本身交给启停逻辑。 */
function onSurfaceClick(event: MouseEvent) {
  if (!pressNote.value) return
  const target = event.target as HTMLElement | null
  if (target?.closest('.preview-group-title')) return
  pressNote.value = ''
}
function toggleChannel(channel: PreviewChannel) {
  if (swallowClick) { swallowClick = false; return }
  pressNote.value = ''
  if (props.busy || props.saving || channel.protected) return
  note.value = channel.enabled ? '已停用 ' + channel.label + '；修改保留在当前草稿' : '已恢复 ' + channel.label + '；修改保留在当前草稿'
  emit('toggle-channel', channel.id)
}
function toggleModel(channelId: string, modelId: string, selected: boolean) {
  if (props.busy || props.saving) return
  note.value = (selected ? '已停用 ' : '已启用 ') + modelId + '；修改保留在当前草稿'
  emit('toggle-model', channelId, modelId)
}
function confirm() { if (!props.saving) emit('confirm') }
</script>
<template>
 <UiDialog :open="open" title="保存预览" wide panel-class="save-preview-dialog" :busy="saving" @close="emit('close')">
  <div class="selector-preview" aria-label="保存预览" @click="onSurfaceClick">
   <div class="selector-preview-budget" :class="{over}" role="status">
    <strong>模型激活数 {{ active }}/100</strong>
    <span>基础模型 {{ rows.length }} · Fast {{ fastCount }} · 系统占用 {{ systemRows.length || (catalog ?? '未提供') }}</span>
    <span v-if="over" class="preview-alert">超出激活预算，请减少发布项后再保存。</span>
   </div>
   <p v-if="busy" class="preview-muted" role="status">正在解析当前草稿…</p>
   <p v-if="note" class="preview-muted" role="status">{{ note }}</p>
   <p v-if="error" class="preview-alert" role="alert">{{ error }}</p>
   <div class="selector-preview-order" aria-label="完整选择器顺序" @scroll.passive="onOrderScroll">
    <section v-for="group in groups" :key="group.channel.id" class="preview-group" :class="{off:!group.channel.enabled,locked:group.channel.protected}">
     <h3 v-if="group.channel.protected" class="preview-group-title is-locked">
      <span class="preview-eye" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/></svg></span>
      <strong>{{ group.channel.label }}</strong><small>官方管理 · 只读</small>
     </h3>
     <button v-else type="button" class="preview-group-title" :class="{off:!group.channel.enabled,changed:group.delta.total>0,'show-note':pressNote===group.channel.id}" :aria-pressed="!group.channel.enabled" :disabled="busy||saving" @pointerdown="pressStart(group.channel,$event)" @pointerup="pressEnd" @pointercancel="pressEnd" @pointerleave="pressLeave" @click="toggleChannel(group.channel)">
      <span class="preview-eye" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/></svg></span>
      <strong>{{ group.channel.label }}</strong>
      <span v-if="group.delta.total>0" class="preview-delta-tag" aria-hidden="true">改</span>
      <small v-if="!group.channel.enabled">已停用 · 点击恢复</small>
      <small v-else>启用 {{ group.items.filter(item => item.row.selected).length }} / {{ group.items.length }}</small>
      <span v-if="group.delta.total>0" class="preview-delta-note">{{ group.delta.text }}</span>
      <span class="preview-group-action">{{ group.channel.enabled?'停用':'恢复' }}</span>
     </button>
     <div v-if="group.channel.enabled" class="preview-items">
      <button v-for="item in group.items" :key="item.row.modelId" type="button" class="preview-item" :class="[item.row.selected?'is-on':'is-off',item.change]" :aria-pressed="item.row.selected" :title="(group.channel.protected?'官方管理 · 只读 · ':(item.row.selected?'点击停用':'点击启用')+' ')+item.row.modelId+(item.row.rules.fast.mode==='custom'&&item.row.rules.fast.value?' · 支持 Fast':'')+(item.change==='is-added'?' · 本次新启用':item.change==='is-removed'?' · 本次停用':'')" :disabled="busy||saving||group.channel.protected" @click="toggleModel(group.channel.id,item.row.modelId,item.row.selected)">
       <em v-if="!group.channel.protected">{{ String(item.order).padStart(2,'0') }}</em><span>{{ item.row.label || item.row.modelId }}</span>
       <em v-if="item.row.rules.fast.mode==='custom'&&item.row.rules.fast.value" class="preview-fast">Fast</em>
      </button>
      <p v-if="!group.items.length" class="preview-muted">该渠道暂无型号。</p>
     </div>
    </section>
    <!-- 原型 model-config.js:518：系统占用单独一段，标题带「只读」，条目不可点。 -->
    <section v-if="systemRows.length" class="preview-group preview-group-system">
     <h3 class="preview-group-title is-locked"><span class="preview-eye" aria-hidden="true"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Z"/><circle cx="12" cy="12" r="3"/></svg></span><strong>系统占用</strong><small>只读</small></h3>
     <div class="preview-items"><div v-for="row in systemRows" :key="row.id" class="preview-item hidden-item" :title="row.id"><span>{{ row.label }}</span></div></div>
    </section>
    <p v-if="!groups.length&&!systemRows.length" class="preview-muted">没有计划发布的目录条目。</p>
   </div>
   <details v-if="otherTotal" class="preview-other">
    <summary>另有 {{ otherTotal }} 项字段变化</summary>
    <dl><div v-for="change in others" :key="change.label"><dt>{{ change.label }}</dt><dd><span>{{ change.before === null || change.before === undefined ? '未设置' : typeof change.before === 'string' ? change.before : JSON.stringify(change.before) }}</span><em>→</em><span>{{ change.after === null || change.after === undefined ? '未设置' : typeof change.after === 'string' ? change.after : JSON.stringify(change.after) }}</span></dd></div></dl>
   </details>
   <!--
     待核对提示（后端投影 notices）：真实账号可能有几十条「未在当前清单发现」。
     曾把它们逐条平铺在弹窗底部，flex 列里每条都抢高度，把上面的渠道/型号区挤到 0，
     用户看到的就是「只有一堆提示、没有预览」——0.2.8 接报的真实缺陷。
     改为默认收起的一行摘要，展开后给独立滚动上限，与「另有 N 项字段变化」同一形态。
   -->
   <details v-if="notices.length" class="preview-notices">
    <summary>{{ noticesLabel }}</summary>
    <ul><li v-for="notice in notices" :key="notice">{{ notice }}</li></ul>
   </details>
  </div>
  <template #actions>
   <UiButton :disabled="saving" @click="emit('close')">取消</UiButton>
   <UiButton variant="primary" :loading="saving" @click="confirm">确认保存</UiButton>
  </template>
 </UiDialog>
</template>
<style scoped>
/* §38.1：预览弹窗宽度上限 980px（公共 wide 为 850px），窄窗仍按可用宽度收缩。
   弹窗被 Teleport 到 body，样式写在下方非 scoped 块里才能命中。 */
.selector-preview{display:flex;flex-direction:column;gap:var(--space-3);min-width:0;max-height:min(620px,calc(100vh / var(--ui-zoom) - 220px))}
.selector-preview-budget{flex:none;display:flex;align-items:baseline;gap:var(--space-2) var(--space-4);flex-wrap:wrap;padding:0 0 var(--space-2-5);border-bottom:1px solid var(--border);font-size:var(--text-label);color:var(--muted)}
.selector-preview-budget strong{color:var(--text);font-size:var(--text-control);font-variant-numeric:tabular-nums}
.selector-preview-budget.over strong,.selector-preview-budget.over .preview-alert{color:var(--red)}
/* min-height 兜底：任何附加区块都不能把渠道/型号区压成 0（0.2.8 缺陷复现条件）。 */
.selector-preview-order{flex:1 1 auto;min-height:180px;overflow:auto;overscroll-behavior:contain;display:grid;gap:var(--space-2-5);padding-right:var(--space-1)}
.preview-group{margin:0;padding:var(--space-1-5) var(--space-2) var(--space-2);border:1px solid var(--border);border-radius:var(--radius);background:color-mix(in srgb,var(--raised) 72%,transparent)}
.preview-group.off{background:transparent;border-style:dashed;border-color:var(--border-soft);opacity:.62}
.preview-group.locked{background:color-mix(in srgb,var(--raised) 55%,transparent)}
.preview-group-title{display:flex;align-items:center;gap:var(--space-2);width:100%;min-height:30px;margin:0 0 var(--space-1-5);padding:var(--space-1) var(--space-1-5);border:1px solid transparent;border-radius:var(--radius-xs);background:transparent;color:var(--text);font:inherit;font-size:var(--text-label);text-align:left;cursor:pointer}
.preview-group-title:hover:not(:disabled){border-color:var(--border-soft);background:var(--hover)}
.preview-group-title:disabled{cursor:default}
.preview-group-title.is-locked{cursor:default;margin-bottom:var(--space-1);font-weight:600}
.preview-group-title.changed{border-color:color-mix(in srgb,var(--amber) 45%,var(--border))}
.preview-group-title.off{color:var(--muted)}
.preview-eye{display:grid;place-items:center;flex:none;width:20px;height:20px;color:var(--muted)}
.preview-eye svg{width:14px;height:14px}
.preview-group-title:not(.off) .preview-eye{color:var(--accent)}
.preview-group-title strong{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.preview-group-title small{color:var(--muted);font-size:var(--text-caption);font-weight:400;white-space:nowrap}
.preview-group-title.is-locked{padding-left:0}
.preview-group-action{margin-left:auto;flex:none;color:var(--muted);font-size:var(--text-caption);white-space:nowrap}
.preview-group-title:hover .preview-group-action{color:var(--text)}
.preview-delta-tag{flex:none;padding:0 5px;border-radius:var(--radius-2xs);background:var(--amber-soft);color:var(--amber);font-size:var(--text-micro);line-height:16px;font-weight:600}
.preview-delta-note{display:none;flex:0 1 auto;min-width:0;margin-left:var(--space-2);color:var(--amber);font-size:var(--text-caption);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.preview-group-title:hover .preview-delta-note,.preview-group-title:focus-visible .preview-delta-note,.preview-group-title.show-note .preview-delta-note{display:block}
.preview-group-title{touch-action:manipulation;-webkit-touch-callout:none}
.preview-items{display:flex;flex-wrap:wrap;gap:var(--space-1-5)}
.preview-item{display:inline-flex;align-items:center;gap:var(--space-1-5);max-width:100%;padding:var(--space-1) var(--space-2);border:1px solid transparent;border-radius:var(--radius-2xs);background:var(--rail);color:var(--text);font:inherit;font-size:var(--text-caption);cursor:pointer}
.preview-item:hover:not(:disabled){border-color:var(--border);background:var(--hover)}
.preview-item:disabled{cursor:default}
.preview-item:focus-visible,.preview-group-title:focus-visible{outline:2px solid var(--accent);outline-offset:2px}
.preview-item em{color:var(--muted);font-style:normal;font-variant-numeric:tabular-nums}
.preview-item span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.preview-item.is-on{border-color:color-mix(in srgb,var(--accent) 30%,var(--border));background:color-mix(in srgb,var(--accent) 9%,var(--rail))}
.preview-item.is-off{color:var(--muted);background:transparent;border-color:var(--border-soft);text-decoration:line-through;text-decoration-color:color-mix(in srgb,var(--muted) 50%,transparent)}
.preview-item.is-added{border-color:color-mix(in srgb,var(--green) 55%,var(--border));text-decoration:none}
.preview-item.is-added::after{content:"新";flex:none;color:var(--green);font-size:var(--text-micro);font-weight:600}
.preview-item.is-removed{border-style:dashed;border-color:color-mix(in srgb,var(--red) 42%,var(--border-soft))}
.preview-item.is-removed::after{content:"停";flex:none;color:var(--red);font-size:var(--text-micro);font-weight:600}
.preview-fast{flex:none;padding:0 4px;border-radius:var(--radius-2xs);background:var(--accent-soft);color:var(--accent);font-size:var(--text-micro);text-decoration:none}
.preview-group-system{background:color-mix(in srgb,var(--rail) 60%,transparent)}
.preview-item.hidden-item{color:var(--muted);border:1px dashed var(--border);cursor:default}
.preview-other{flex:none;padding:var(--space-2) var(--space-2-5);border:1px solid var(--border-soft);border-radius:var(--radius-sm);font-size:var(--text-caption)}
.preview-other summary{cursor:pointer;color:var(--muted)}
.preview-other dl{display:grid;gap:var(--space-1);margin:var(--space-2) 0 0}
.preview-other dl>div{display:grid;grid-template-columns:minmax(90px,1fr) 2fr;gap:var(--space-2);align-items:baseline;min-width:0}
.preview-other dt{color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.preview-other dd{display:flex;align-items:baseline;gap:var(--space-1-5);margin:0;min-width:0}
.preview-other dd span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.preview-other dd em{color:var(--muted);font-style:normal;flex:none}
.preview-muted{margin:0;color:var(--muted);font-size:var(--text-label)}
/* 待核对提示：默认收起，展开后自身滚动，不参与上方列表的高度分配。 */
.preview-notices{flex:none;padding:var(--space-2) var(--space-2-5);border:1px solid var(--border-soft);border-radius:var(--radius-sm);font-size:var(--text-caption);color:var(--muted)}
.preview-notices>summary{display:flex;align-items:center;gap:6px;cursor:pointer;list-style:none;color:var(--muted)}
.preview-notices>summary:hover{color:var(--text)}
.preview-notices>summary::-webkit-details-marker{display:none}
.preview-notices>summary::before{content:"▸";flex:none;font-size:var(--text-micro)}
.preview-notices[open]>summary::before{content:"▾"}
.preview-notices ul{display:grid;gap:var(--space-1);max-height:112px;overflow:auto;overscroll-behavior:contain;margin:var(--space-2) 0 0;padding:0;list-style:none}
.preview-notices li{overflow-wrap:anywhere;line-height:1.6}
.preview-alert{margin:0;color:var(--red);font-size:var(--text-label);overflow-wrap:anywhere}
</style>

<style>
/* 弹窗被 Teleport 到 body，只有全局样式能命中它。 */
body dialog.ui-dialog.save-preview-dialog.wide{width:min(980px,calc(100vw / var(--ui-zoom) - 32px))}
</style>
