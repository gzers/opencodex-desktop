<script setup lang="ts">
// 拓展 →「客户端」分区（N-EX-1）。结构、文案与列宽节奏对齐原型
// extension-manager.js 的 renderClients() / renderClientDetail()。
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectMenu from '@/components/ui/UiSelectMenu.vue'
import UiStatus from '@/components/ui/UiStatus.vue'
import { computed, nextTick, ref, watch } from 'vue'
import { useAppStore } from '@/stores/app'
import type { ExtensionClientId } from '@/features/extensions/api'
import {
  clientAdaptation,
  clientCapabilityLabel,
  clientDiscoveryLabel,
  clientStatusTag,
  filterClients,
  type ClientDirectoryEntry,
  type ClientStatusFilter,
} from '@/features/extensions/clients'

const props = defineProps<{
  entries: ClientDirectoryEntry[]
  busy?: boolean
  /** 重新发现（Skills / MCP / 落点 / 启用矩阵）；由路由注入，等结果回来再报数。 */
  discover: () => Promise<void>
}>()

const emit = defineEmits<{
  (event: 'open-resources', kind: 'skill' | 'mcp', client: ExtensionClientId): void
  (event: 'open-item', kind: 'skill' | 'mcp', id: string): void
  (event: 'manage-item', kind: 'skill' | 'mcp', name: string): void
}>()

const app = useAppStore()
const query = ref('')
const status = ref<ClientStatusFilter>('all')
const selectedId = ref<ExtensionClientId | null>(null)
const notice = ref('')
const statusOpen = ref(false)

const statusOptions: ReadonlyArray<{ value: ClientStatusFilter; label: string }> = [
  { value: 'all', label: '全部' },
  { value: 'installed', label: '已发现' },
  { value: 'issues', label: '有异常' },
  { value: 'unknown', label: '待核实适配' },
]

const statusLabel = computed(() => {
  const option = statusOptions.find(item => item.value === status.value)
  const label = option?.label ?? '全部'
  return status.value === 'all' ? `${label} ${props.entries.length} 项` : label
})

function chooseStatus(value: ClientStatusFilter) {
  status.value = value
  statusOpen.value = false
}

const visible = computed(() => filterClients(props.entries, query.value, status.value))
const selected = computed(() => props.entries.find(entry => entry.id === selectedId.value) ?? null)

watch(() => props.entries, entries => {
  if (selectedId.value && !entries.some(entry => entry.id === selectedId.value)) selectedId.value = null
})

function openClient(entry: ClientDirectoryEntry) {
  selectedId.value = entry.id
  notice.value = ''
}

function back() {
  selectedId.value = null
}

async function runDiscover() {
  // 等本轮重新发现与 props 更新都完成，再按当前结果报数，避免报出上一轮计数。
  await props.discover()
  await nextTick()
  const found = props.entries.filter(entry => entry.installed).length
  notice.value = '已发现 ' + found + ' 个客户端；没有自动新增资源分发。'
}

function kindLabel(kind: 'skill' | 'mcp') {
  return kind === 'skill' ? 'Skills' : 'MCP'
}

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

// 配置说明：原型 client-info 的只读信息区，走应用级弹窗。
function openConfig(entry: ClientDirectoryEntry) {
  app.openModal({
    title: entry.name + ' · 配置说明',
    body:
      '<dl class="ext-client-facts">' +
      '<dt>能力依据</dt><dd>' + escapeHtml(entry.evidence) + '</dd>' +
      '<dt>Skills</dt><dd>' + escapeHtml(entry.skillsPath ?? '待核实') + '</dd>' +
      '<dt>MCP</dt><dd>' + escapeHtml(entry.mcpPath ?? '待核实') + '</dd>' +
      '<dt>范围</dt><dd>' + escapeHtml(entry.scope) + '</dd>' +
      '</dl>' +
      '<p class="ext-client-facts-note">这里只展示本机已核实的落点；客户端专属设置复用设置页数据，生产适配尚未完成。</p>',
    cancelLabel: '关闭',
    wide: true,
  })
}
</script>

<template>
  <div class="ext-clients">
    <template v-if="!selected">
      <div class="ext-clients-head">
        <div>
          <h2>客户端</h2>
          <p>查看平台能力、配置落点和已分发的资源。</p>
        </div>
        <UiButton class="btn ghost" :disabled="busy" @click="runDiscover">发现客户端</UiButton>
      </div>
      <div class="ext-clients-toolbar">
        <input
          v-model="query"
          class="input ext-clients-search"
          type="search"
          aria-label="搜索客户端"
          placeholder="搜索客户端名称"
        >
        <div class="select ext-clients-filter" :class="{ open: statusOpen }">
          <button
            type="button"
            class="select-trigger"
            :aria-expanded="statusOpen ? 'true' : 'false'"
            aria-haspopup="listbox"
            aria-label="客户端状态筛选"
            @click="statusOpen = !statusOpen"
          ><span class="select-value">{{ statusLabel }}</span></button>
          <UiSelectMenu :open="statusOpen" role="listbox" aria-label="客户端状态筛选" @close="statusOpen = false">
            <button
              v-for="option in statusOptions"
              :key="option.value"
              type="button"
              class="select-option"
              :class="{ selected: status === option.value }"
              role="option"
              :aria-selected="status === option.value ? 'true' : 'false'"
              @click="chooseStatus(option.value)"
            >{{ option.value === 'all' ? `全部 ${entries.length} 项` : option.label }}</button>
          </UiSelectMenu>
        </div>
      </div>
      <p v-if="notice" class="ext-clients-note" role="status">{{ notice }}</p>
      <div class="ext-client-head" aria-hidden="true">
        <span>客户端</span>
        <span>扩展能力</span>
        <span class="ext-client-count">受管资源</span>
        <span>状态</span>
      </div>
      <div class="ext-client-list">
        <article v-for="entry in visible" :key="entry.id" class="ext-client-row">
          <div class="ext-client-name">
            <span class="ext-monogram" aria-hidden="true">{{ entry.monogram }}</span>
            <div>
              <button type="button" class="ext-client-link" @click="openClient(entry)">{{ entry.name }}</button>
              <div class="ext-client-meta">{{ entry.scope }}</div>
            </div>
          </div>
          <div class="ext-client-cap">Skills · {{ clientCapabilityLabel(entry.skillsReady) }}<br>MCP · {{ clientCapabilityLabel(entry.mcpReady) }}</div>
          <span class="ext-client-count">{{ entry.managed.length }} 项</span>
          <div class="ext-client-state">
            <UiStatus :tone="clientStatusTag(entry).tone" :label="clientStatusTag(entry).label" />
            <span class="ext-client-meta">{{ clientDiscoveryLabel(entry) }}</span>
          </div>
        </article>
        <div v-if="visible.length === 0" class="ext-clients-empty">没有匹配的客户端。</div>
      </div>
    </template>

    <div v-else class="ext-clients-detail">
      <UiButton class="btn ghost ext-client-back" @click="back">← 返回客户端</UiButton>
      <div class="ext-clients-head">
        <div>
          <h2>{{ selected.name }}</h2>
          <p>{{ selected.evidence }} · {{ clientDiscoveryLabel(selected) }}</p>
        </div>
        <UiButton class="btn ghost" @click="openConfig(selected)">配置说明</UiButton>
      </div>
      <dl class="ext-client-facts">
        <dt>范围</dt><dd>{{ selected.scope }}</dd>
        <dt>Skills</dt><dd>{{ selected.skillsPath ?? '搜索目录待核实' }}</dd>
        <dt>MCP</dt><dd>{{ selected.mcpPath ?? '配置节点待核实' }}</dd>
        <dt>适配状态</dt><dd>{{ clientAdaptation(selected) }}</dd>
      </dl>
      <div class="ext-clients-actions">
        <UiButton class="btn ghost" @click="emit('open-resources', 'skill', selected.id)">查看 Skills</UiButton>
        <UiButton class="btn ghost" @click="emit('open-resources', 'mcp', selected.id)">查看 MCP</UiButton>
      </div>
      <p class="ext-clients-muted">受管资源 {{ selected.managed.length }} 项</p>
      <div class="ext-client-list">
        <div v-for="resource in selected.managed" :key="resource.key" class="ext-client-row ext-client-row-detail">
          <button type="button" class="ext-client-link" @click="emit('open-item', resource.kind, resource.id)">{{ resource.name }}</button>
          <span class="ext-client-cap">{{ kindLabel(resource.kind) }} · {{ resource.status }}</span>
          <UiButton class="btn ghost" @click="emit('manage-item', resource.kind, resource.name)">管理目标</UiButton>
        </div>
        <div v-if="selected.managed.length === 0" class="ext-clients-empty">没有受管资源。</div>
      </div>
    </div>
  </div>
</template>

<style>
/* 配置说明弹窗正文由 v-html 渲染，无法使用 scoped；类名只在本页产生。 */
.ext-client-facts {
  display: grid;
  grid-template-columns: 80px minmax(0, 1fr);
  gap: 10px;
  margin: 0;
  font-size: var(--text-caption);
}
.ext-client-facts dt { color: var(--muted); }
.ext-client-facts dd { margin: 0; overflow-wrap: anywhere; }
.modal-body .ext-client-facts-note { margin: 14px 0 0; color: var(--muted); font-size: var(--text-caption); line-height: 1.6; }
</style>

<style scoped>
.ext-clients { display: grid; gap: 16px; }
.ext-clients-head { display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap; }
.ext-clients-head h2 { margin: 0; font-size: var(--text-title); }
.ext-clients-head p { margin: 4px 0 0; color: var(--muted); font-size: var(--text-caption); line-height: 1.5; }
.ext-clients-toolbar { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.ext-clients-search { flex: 1; min-width: 150px; }
.ext-clients-filter { flex: none; width: auto; min-width: 150px; max-width: 220px; }
.ext-clients-note { margin: 0; color: var(--accent); font-size: var(--text-caption); }
/* 四列节奏与原型 .xp-client-head/.xp-client-row 一致；所有单元格垂直居中。 */
.ext-client-head,
.ext-client-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 130px 100px 120px;
  gap: 14px;
  align-items: center;
}
.ext-client-head {
  padding: 5px 10px;
  color: var(--muted);
  font-size: var(--text-caption);
}
.ext-client-list { display: grid; border-top: 1px solid var(--border); }
.ext-client-row { padding: 13px 10px; border-bottom: 1px solid var(--border); font-size: var(--text-caption); }
.ext-client-row:hover { background: var(--hover); }
.ext-client-name { display: flex; gap: 10px; align-items: center; min-width: 0; }
.ext-monogram {
  width: 28px;
  height: 28px;
  flex: none;
  display: grid;
  place-items: center;
  border-radius: var(--radius-xs);
  background: var(--rail);
  color: var(--muted);
  font-size: var(--text-micro);
}
.ext-client-link {
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--text);
  font: inherit;
  font-weight: 650;
  text-align: left;
  cursor: pointer;
}
.ext-client-link:hover { text-decoration: underline; }
.ext-client-meta { color: var(--muted); font-size: var(--text-micro); }
.ext-client-cap { color: var(--muted); font-size: var(--text-micro); line-height: 1.6; }
.ext-client-count { font-variant-numeric: tabular-nums; }
.ext-client-state { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; }
/* 状态沿用软件既有 .tag 状态标签；tone 与原型同源（可管理 / 待核实 / 有异常）。 */
/* 状态标签外形由公共 .state-pill 提供；此处只保留紧凑内距。 */
.ext-client-state .state-pill{min-height:20px;padding:1px 6px}
.ext-clients-empty { padding: 40px 16px; border-bottom: 1px solid var(--border); color: var(--muted); text-align: center; }
.ext-clients-detail { display: grid; gap: 14px; }
.ext-client-back { justify-self: stretch; color: var(--muted); }
.ext-client-row-detail { grid-template-columns: minmax(0, 1fr) 130px 100px; }
.ext-clients-actions { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
.ext-clients-muted { margin: 0; color: var(--muted); font-size: var(--text-caption); }
@media (max-width: 1100px) {
  .ext-client-head,
  .ext-client-row { grid-template-columns: minmax(0, 1fr) 106px 84px 100px; gap: 8px; }
  .ext-client-row-detail { grid-template-columns: minmax(0, 1fr) 106px 100px; }
}
@media (max-width: 900px) {
  .ext-client-head,
  .ext-client-row { grid-template-columns: minmax(0, 1fr) 100px 96px; }
  .ext-client-head .ext-client-count,
  .ext-client-row .ext-client-count { display: none; }
  .ext-client-row-detail { grid-template-columns: minmax(0, 1fr) 100px 96px; }
}
</style>
