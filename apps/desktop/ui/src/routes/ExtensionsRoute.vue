<script setup lang="ts">
import { computed, nextTick, onMounted, onUpdated, ref, watch } from 'vue'
import AppTopbar from '@/components/AppTopbar.vue'
import ExtensionDetailModal from '@/features/extensions/components/ExtensionDetailModal.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { skillTargets } from '@/features/extensions/targets'
import {
  clientLabels,
  extensionChips,
  formatDateOnly,
  formatUpdatedAt,
  serverVersion,
  skillSourceLabels,
  skillVersion,
  targetEnabled,
} from '@/features/extensions/presentation'
import ClientIcon from '@/features/extensions/components/ClientIcon.vue'
import type {
  ExtensionClientId,
  ExtensionSkillDto,
  ExtensionServerDto,
} from '@/features/extensions/api'
import { readMcpDetail, readSkillDetail } from '@/features/extensions/api'

const routes = useRouteStore()
const app = useAppStore()
const tab = computed(() => routes.extensionTab)
const skillsSearch = ref('')
const mcpSearch = ref('')

const skills = computed<ExtensionSkillDto[]>(() => app.extensions?.skills ?? [])
const extensionBusy = computed(() => app.extensionToggleBusy || app.extensionConfigLoading)

async function toggleSkill(skill: ExtensionSkillDto, target: ExtensionClientId) {
  if (extensionBusy.value) return
  // 亮/灭读的是该 Skill 在这一个客户端上的实际落地状态；关掉 = 只断开这个客户端的链接。
  const linked = targetEnabled(skill.clients, target)
  const saved = linked
    ? await app.executeExtensionWrite({
        kind: 'unlink_skill',
        name: skill.name,
        client: target,
        confirm: true,
      })
    : await app.executeExtensionWrite({ kind: 'link_skill', name: skill.name, client: target })
  if (!saved) {
    app.showToast(failureText('Skill 同步失败'))
    return
  }
  // 以重新发现的结果为准：目标客户端已有同名真实目录时链接会被跳过（不覆盖）。
  const updated = app.extensions?.skills.find(item => item.name === skill.name)
  const nowLinked = updated ? targetEnabled(updated.clients, target) : false
  const label = clientLabels[target]
  if (linked) {
    app.showToast(
      nowLinked
        ? `${label} 已有同名目录，${skill.name} 未移除。`
        : `已断开 ${skill.name} 到 ${label} 的链接。`,
    )
  } else {
    app.showToast(
      nowLinked
        ? `已同步 ${skill.name} 到 ${label}。`
        : `${label} 已有同名目录，未覆盖 ${skill.name}。`,
    )
  }
}

async function toggleMcp(server: ExtensionServerDto, target: ExtensionClientId) {
  if (extensionBusy.value) return
  const configured = targetEnabled(server.clients, target)
  const saved = configured
    ? await app.executeExtensionWrite({
        kind: 'remove_mcp',
        name: server.name,
        client: target,
        confirm: true,
      })
    : await app.executeExtensionWrite({
        kind: 'write_mcp',
        name: server.name,
        client: target,
        confirm: true,
      })
  if (saved) {
    app.showToast(`已${configured ? '移除' : '写入'} ${server.name} 到 ${clientLabels[target]}。`)
  } else {
    app.showToast(failureText(`MCP ${configured ? '断开' : '写入'}失败`))
  }
}

const actionBusy = ref(false)

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll('\'', '&#39;')
}

async function runExtensionWrite(command: Parameters<typeof app.executeExtensionWrite>[0], successMessage: string, failureMessage: string) {
  if (actionBusy.value || extensionBusy.value) return
  actionBusy.value = true
  const result = await app.executeExtensionWrite(command)
  actionBusy.value = false
  app.showToast(result ? successMessage : failureText(failureMessage))
}

// 失败提示带上后端真实原因；拿不到原因时退回原有说法，不虚构。
function failureText(base: string) {
  const reason = app.extensionWriteError
  return reason ? `${base}：${reason}` : base
}

function importSkillArchive() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.zip,application/zip'
  input.onchange = () => {
    const file = input.files?.[0]
    if (!file) return
    const reader = new FileReader()
    reader.onload = () => {
      const payload = String(reader.result)
      const separator = payload.indexOf(',')
      if (separator < 0) {
        app.showToast('ZIP 导入失败；文件无法读取。')
        return
      }
      const archiveName = file.name
      void runExtensionWrite(
        { kind: 'import_skill_archive', archiveName, archivePayload: payload.slice(separator + 1) },
        `已导入 ${archiveName}。`,
        'ZIP 导入失败；压缩包超过限制或路径不安全。',
      )
    }
    reader.readAsDataURL(file)
  }
  input.click()
}

async function restoreSkill() {
  // WKWebView 不支持 window.prompt；改用应用内输入对话框，避免静默无反应。
  const values = await app.openInputModal({
    title: '恢复 Skill',
    body: '从扩展备份中恢复本机 Skills 源目录；只影响管理器自有域。',
    fields: [{ key: 'name', label: 'Skill 名称', placeholder: '输入需要恢复的 Skill 名称' }],
    confirmLabel: '恢复',
  })
  const value = values?.name?.trim()
  if (!value) return
  void runExtensionWrite(
    { kind: 'restore_skill', name: value },
    `已恢复 ${value}。`,
    'Skill 恢复失败；未找到可复验的备份。',
  )
}

function checkSkillUpdate(skill?: ExtensionSkillDto) {
  const name = skill?.name ?? skills.value[0]?.name ?? ''
  if (!name) {
    app.showToast('没有可检查的 Skill。')
    return
  }
  void app.loadExtensions()
  app.showToast(`已刷新 ${name} 的本机发现结果；更新请使用 ZIP 导入。`)
}

function uninstallSkill(skill: ExtensionSkillDto) {
  app.openModal({
    title: '卸载 Skill',
    body: `<p class="modal-lead">将断开 ${escapeHtml(skill.name)} 在各客户端的链接；若管理器源中存在副本，会先备份再移除。不会删除源目录中的内容。</p>`,
    confirmLabel: '卸载',
    primaryKind: 'danger',
    onConfirm: () => {
      void runExtensionWrite(
        { kind: 'uninstall_skill', name: skill.name, confirm: true },
        `已卸载 ${skill.name}；备份可恢复。`,
        'Skill 卸载失败；文件不可写或冲突未确认。',
      )
    },
  })
}

function importMcpDefinition() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json,.jsonc,.toml,.yaml,.yml'
  input.onchange = () => {
    const file = input.files?.[0]
    if (!file) return
    const reader = new FileReader()
    reader.onload = () => {
      try {
        const payload = JSON.parse(String(reader.result)) as Record<string, unknown>
        const name = typeof payload.name === 'string' ? payload.name : ''
        if (!name.trim()) {
          app.showToast('MCP 配置导入失败；缺少有效名称。')
          return
        }
        confirmMcpDefinition(name.trim(), payload, null)
      } catch {
        app.showToast('MCP 配置导入失败；文件不是有效 JSON。')
      }
    }
    reader.readAsText(file)
  }
  input.click()
}

async function addMcpDefinition() {
  const values = await app.openInputModal({
    title: '新增 MCP 服务器',
    body: '填写名称与启动命令；确认后只写入已启用客户端的 MCP 配置节点，原文件会备份。',
    fields: [
      { key: 'name', label: 'MCP 名称', placeholder: '例如 context7' },
      { key: 'command', label: 'MCP 启动命令', placeholder: '例如 npx -y @upstash/context7-mcp' },
    ],
    confirmLabel: '下一步',
  })
  const name = values?.name?.trim()
  const command = values?.command?.trim()
  if (!name || !command) return
  confirmMcpDefinition(name, { command, args: [] }, null)
}

async function editMcpDefinition(server: ExtensionServerDto) {
  const values = await app.openInputModal({
    title: `编辑 ${server.name}`,
    body: '修改启动命令；确认后只写入已启用客户端的 MCP 配置节点。',
    fields: [
      {
        key: 'command',
        label: 'MCP 启动命令',
        value: server.command ?? '',
        placeholder: '例如 npx -y @upstash/context7-mcp',
      },
    ],
    confirmLabel: '下一步',
  })
  const command = values?.command?.trim()
  if (!command) return
  confirmMcpDefinition(server.name, { command, args: server.args }, server.name)
}

function deleteMcpDefinition(server: ExtensionServerDto) {
  app.openModal({
    title: '删除 MCP 服务器',
    body: `<p class="modal-lead">将从启用的客户端配置移除 ${escapeHtml(server.name)}；原文件与统一配置会先备份，可恢复。</p>`,
    confirmLabel: '删除',
    primaryKind: 'danger',
    onConfirm: () => {
      void runExtensionWrite(
        { kind: 'remove_mcp', name: server.name, client: null, confirm: true },
        `已删除 ${server.name}；备份可恢复。`,
        'MCP 删除失败；目标冲突或配置不可写。',
      )
    },
  })
}

function confirmMcpDefinition(name: string, definition: Record<string, unknown>, previous: string | null) {
  const command = typeof definition.command === 'string' ? definition.command : ''
  const args = Array.isArray(definition.args)
    ? definition.args.filter((item): item is string => typeof item === 'string').join(' ')
    : ''
  const safeName = escapeHtml(name)
  const safeCommand = escapeHtml(`${command} ${args}`.trim())
  app.openModal({
    title: previous ? '编辑 MCP 服务器' : '新增 MCP 服务器',
    body: `<p class="modal-lead">本机执行边界确认</p><p><b>${safeName}</b><br><code>${safeCommand}</code></p><p>不会立即执行；只写入已启用客户端的 MCP 配置节点。原文件会备份并按 0600 原子替换。</p>`,
    confirmLabel: previous ? '保存' : '添加',
    onConfirm: () => {
      void runExtensionWrite(
        previous
          ? { kind: 'edit_mcp', name: previous, definition: { name, value: definition }, confirm: true }
          : { kind: 'add_mcp', definition: { name, value: definition } },
        `已${previous ? '保存' : '添加'} ${name}。`,
        'MCP 写入失败；目标冲突或配置不可写。',
      )
    },
  })
}

const mcpServers = computed<ExtensionServerDto[]>(() => app.extensions?.servers ?? [])
const extensionsError = computed(() => app.extensionsError)

const skillChips = computed(() => extensionChips('已安装', skills.value.length, skills.value))
const serverChips = computed(() => extensionChips(`已配置 ${mcpServers.value.length} 个`, mcpServers.value.length, mcpServers.value))

// 客户端 chip 与搜索都真实过滤；过滤后再分页，条件或数据变化时页码回到第一页。
const PAGE_SIZE = 8
const skillsClientFilter = ref<ExtensionClientId | null>(null)
const mcpClientFilter = ref<ExtensionClientId | null>(null)
const skillsPage = ref(1)
const mcpPage = ref(1)

const filteredSkills = computed(() => {
  const query = skillsSearch.value.trim().toLowerCase()
  return skills.value.filter(skill => {
    if (skillsClientFilter.value && !skill.clients.includes(skillsClientFilter.value)) return false
    if (!query) return true
    return `${skill.name} ${skill.description}`.toLowerCase().includes(query)
  })
})

const filteredServers = computed(() => {
  const query = mcpSearch.value.trim().toLowerCase()
  return mcpServers.value.filter(server => {
    if (mcpClientFilter.value && !server.clients.includes(mcpClientFilter.value)) return false
    if (!query) return true
    return `${server.name} ${server.description}`.toLowerCase().includes(query)
  })
})

function pageCount(total: number) {
  return Math.max(1, Math.ceil(total / PAGE_SIZE))
}
const skillsPageCount = computed(() => pageCount(filteredSkills.value.length))
const serversPageCount = computed(() => pageCount(filteredServers.value.length))
const pagedSkills = computed(() =>
  filteredSkills.value.slice((skillsPage.value - 1) * PAGE_SIZE, skillsPage.value * PAGE_SIZE),
)
const pagedServers = computed(() =>
  filteredServers.value.slice((mcpPage.value - 1) * PAGE_SIZE, mcpPage.value * PAGE_SIZE),
)

watch([skillsSearch, skillsClientFilter, () => skills.value.length], () => { skillsPage.value = 1 })
watch([mcpSearch, mcpClientFilter, () => mcpServers.value.length], () => { mcpPage.value = 1 })

function selectSkillsClient(key: ExtensionClientId | null) {
  skillsClientFilter.value = skillsClientFilter.value === key ? null : key
}
function selectMcpClient(key: ExtensionClientId | null) {
  mcpClientFilter.value = mcpClientFilter.value === key ? null : key
}
function goSkillsPage(page: number) {
  skillsPage.value = Math.min(Math.max(1, page), skillsPageCount.value)
}
function goServersPage(page: number) {
  mcpPage.value = Math.min(Math.max(1, page), serversPageCount.value)
}

// AC-11 / CAP-11.5：条目只读详情。列表不预读正文，打开时只读被打开的那一条。
/** `wide` = 整行（原型 `.span-2`）；`scroll` = 取值区自带内部滚动（长环境变量名单）。 */
interface DetailFact { label: string; value: string; wide?: boolean; scroll?: boolean }
interface DetailAction { key: string; title: string; icon: 'refresh' | 'trash' | 'edit'; danger?: boolean }
interface DetailState {
  kind: 'skill' | 'mcp'
  name: string
  title: string
  markdown: string | null
  markdownTruncated: boolean
  facts: DetailFact[]
  actions: DetailAction[]
}

const detail = ref<DetailState | null>(null)

function formatBytes(bytes: number | null) {
  if (bytes === null) return '未知'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function skillFacts(skill: ExtensionSkillDto): DetailFact[] {
  return [
    { label: '来源', value: skillSourceLabels[skill.source] },
    { label: '版本', value: skillVersion(skill) },
    // 标签已写明「更新时间」，取值只给日期，避免出现「更新时间 更新 2026-09-23」。
    { label: '更新时间', value: formatDateOnly(skill.updatedAt) },
  ]
}

async function openSkillDetail(skill: ExtensionSkillDto) {
  detail.value = {
    kind: 'skill',
    name: skill.name,
    title: skill.name,
    // 描述只出现在正文面板里（与原型一致）：面板 = 列表那段完整描述 + `SKILL.md` 正文。
    markdown: skill.description,
    markdownTruncated: false,
    facts: skillFacts(skill),
    actions: [
      { key: 'check', title: '检查 / 更新', icon: 'refresh' },
      { key: 'uninstall', title: '卸载', icon: 'trash', danger: true },
    ],
  }
  try {
    const info = await readSkillDetail(skill.name)
    if (detail.value?.kind !== 'skill' || detail.value.name !== skill.name) return
    detail.value = {
      ...detail.value,
      markdown: [skill.description, info.body].filter(part => part && part.trim()).join('\n\n'),
      markdownTruncated: info.bodyTruncated,
      facts: [
        ...skillFacts(skill),
        { label: '入口文件', value: info.entryFile },
        { label: '文件数', value: info.fileCount === null ? '未知' : `${info.fileCount} 个文件` },
        { label: '体量', value: formatBytes(info.totalBytes) },
        { label: '源目录', value: info.sourcePath, wide: true },
      ],
    }
  } catch {
    // 详情读取失败不阻塞弹窗：基础信息仍在，正文留空。
    app.showToast('详情读取失败；已保留列表中的基础信息。')
  }
}

async function openMcpDetail(server: ExtensionServerDto) {
  detail.value = {
    kind: 'mcp',
    name: server.name,
    title: server.name,
    markdown: server.description,
    markdownTruncated: false,
    facts: [
      { label: '类型', value: server.transport },
      { label: '版本', value: serverVersion(server) },
      { label: '更新时间', value: formatDateOnly(server.updatedAt) },
    ],
    actions: [
      { key: 'edit', title: '编辑', icon: 'edit' },
      { key: 'delete', title: '删除', icon: 'trash', danger: true },
    ],
  }
  try {
    const info = await readMcpDetail(server.name)
    if (detail.value?.kind !== 'mcp' || detail.value.name !== server.name) return
    const present = info.landings.filter(landing => landing.present)
    const sections = [info.description]
    if (info.configJson) sections.push('## 完整配置', '```json\n' + info.configJson + '\n```')
    detail.value = {
      ...detail.value,
      markdown: sections.filter(part => part && part.trim()).join('\n\n'),
      facts: [
        { label: '类型', value: info.transport },
        { label: '版本', value: serverVersion(server) },
        { label: '更新时间', value: formatDateOnly(info.updatedAt) },
        { label: '命令', value: info.command ?? '—' },
        { label: '参数', value: info.args.length ? info.args.join(' ') : '—' },
        // 环境变量名单可能很长：单独占整行并在取值框内滚动，避免把信息区撑高。
        {
          label: '环境变量',
          value: info.envKeys.length ? info.envKeys.join('、') : '—',
          wide: true,
          scroll: true,
        },
        {
          label: '配置落点',
          value: present.length
            ? present.map(landing => `${clientLabels[landing.client]}：${landing.configPath}`).join('；')
            : '—',
          wide: true,
        },
      ],
    }
  } catch {
    app.showToast('详情读取失败；已保留列表中的基础信息。')
  }
}

function closeDetail() {
  detail.value = null
}

const detailEnabledClients = computed<ExtensionClientId[]>(() => {
  const current = detail.value
  if (!current) return []
  const clients = current.kind === 'skill'
    ? skills.value.find(item => item.name === current.name)?.clients
    : mcpServers.value.find(item => item.name === current.name)?.clients
  if (!clients) return []
  // 只按实际落地状态展示：逐个客户端的连/断已经由统一配置的逐项意图决定。
  return skillTargets.filter(target => targetEnabled(clients, target))
})

async function detailToggleClient(client: ExtensionClientId) {
  const current = detail.value
  if (!current) return
  const skillsTab = current.kind === 'skill'
  const row: ExtensionSkillDto | ExtensionServerDto | undefined = skillsTab
    ? skills.value.find(item => item.name === current.name)
    : mcpServers.value.find(item => item.name === current.name)
  if (!row) return
  if (skillsTab) await toggleSkill(row as ExtensionSkillDto, client)
  else await toggleMcp(row as ExtensionServerDto, client)
}

function detailAction(key: string) {
  const current = detail.value
  if (!current) return
  if (current.kind === 'skill') {
    const skill = skills.value.find(item => item.name === current.name)
    if (!skill) return
    if (key === 'check') checkSkillUpdate(skill)
    else if (key === 'uninstall') uninstallSkill(skill)
  } else {
    const server = mcpServers.value.find(item => item.name === current.name)
    if (!server) return
    if (key === 'edit') editMcpDefinition(server)
    else if (key === 'delete') deleteMcpDefinition(server)
  }
  closeDetail()
}

// 描述真被两行截断时才加底部渐隐；只按实际高度判定，短描述保持干净。
const clampedIds = ref<Set<string>>(new Set())
function sameIds(left: Set<string>, right: Set<string>) {
  return left.size === right.size && [...left].every(value => right.has(value))
}
function markClampedDescriptions() {
  const next = new Set<string>()
  document.querySelectorAll<HTMLElement>('.skill-info[data-ext-detail]').forEach(node => {
    const id = node.getAttribute('data-detail-id')
    const paragraph = node.querySelector('p')
    if (id && paragraph && paragraph.scrollHeight > paragraph.clientHeight + 1) next.add(id)
  })
  if (!sameIds(clampedIds.value, next)) clampedIds.value = next
}

function detailOpen(kind: 'skill' | 'mcp', id: string) {
  if (kind === 'skill') {
    const skill = skills.value.find(item => item.id === id)
    if (skill) void openSkillDetail(skill)
  } else {
    const server = mcpServers.value.find(item => item.id === id)
    if (server) void openMcpDetail(server)
  }
}

function detailKeydown(kind: 'skill' | 'mcp', id: string, event: KeyboardEvent) {
  if (event.key !== 'Enter' && event.key !== ' ') return
  event.preventDefault()
  detailOpen(kind, id)
}

onMounted(() => {
  window.addEventListener('resize', markClampedDescriptions)
  void nextTick(markClampedDescriptions)
})
onUpdated(markClampedDescriptions)
</script>

<template>
  <section class="route-section active">
    <AppTopbar />
    <div class="ext-tabs" role="tablist">
      <button :class="{ active: tab === 'skills' }" @click="routes.go('extensions', { tab: 'skills' })">Skills</button>
      <button :class="{ active: tab === 'mcp' }" @click="routes.go('extensions', { tab: 'mcp' })">MCP</button>
    </div>

    <div v-if="tab === 'skills'" class="ext-view active">
      <UiCard>
        <UiCardHeader>
          <h2>Skills 管理</h2><p>发现本机 Skills 库，并把同一个 Skill 按需同步到各客户端配置。</p>
          <template #actions>
            <div class="toolbar">
            <button class="btn ghost" @click="checkSkillUpdate()">检查更新</button>
            <button class="btn ghost" @click="app.loadExtensions()">发现</button>
            <button class="btn ghost" @click="app.showToast('仓库管理暂未开放；只操作本机 Skills。')">仓库管理</button>
            <button class="btn ghost" @click="restoreSkill()">恢复</button>
            <button class="btn ghost" @click="importSkillArchive()">ZIP</button>
            <button class="btn ghost" @click="importMcpDefinition()">导入</button>
            <button class="btn" @click="routes.go('settings', { section: 'extensions' })">设置</button>
            </div>
          </template>
        </UiCardHeader>
        <div class="skills-metrics">
          <button
            v-for="chip in skillChips"
            :key="chip.label"
            type="button"
            class="skills-chip"
            :class="{ active: skillsClientFilter === chip.key }"
            :aria-pressed="skillsClientFilter === chip.key ? 'true' : 'false'"
            @click="selectSkillsClient(chip.key)"
          ><span>{{ chip.label }}</span><b>{{ chip.value }}</b></button>
        </div>
        <div class="skills-toolbar"><input v-model="skillsSearch" class="input skills-search" type="search" placeholder="搜索 Skills 名称或说明" aria-label="搜索 Skills"></div>
        <div class="skills-list-head">
          <span>Skills</span>
          <div class="skills-target-labels"><span>Claude</span><span>Codex</span><span>Gemini</span><span>Grok Build</span><span>OpenCode</span><span>Hermes</span></div>
          <span>操作</span>
        </div>
        <div class="skills-list">
          <article v-for="skill in pagedSkills" :key="skill.id" class="skill-row">
            <div
              class="skill-info"
              :class="{ 'is-clamped': clampedIds.has(skill.id) }"
              data-ext-detail
              :data-detail-id="skill.id"
              role="button"
              tabindex="0"
              :aria-label="`查看 ${skill.name} 完整信息`"
              @click="detailOpen('skill', skill.id)"
              @keydown="detailKeydown('skill', skill.id, $event)"
            >
              <div class="skill-title"><h3>{{ skill.name }}</h3><span class="tag">{{ skillSourceLabels[skill.source] }}</span><span class="skill-meta"><span>{{ skillVersion(skill) }}</span><span>{{ formatUpdatedAt(skill.updatedAt) }}</span></span></div>
              <p>{{ skill.description }}</p>
            </div>
            <div class="skill-targets">
              <label v-for="target in skillTargets" :key="target" class="target-cell">
                <button
                  class="target-icon"
                  :aria-checked="targetEnabled(skill.clients, target) ? 'true' : 'false'"
                  :disabled="extensionBusy"
                  :aria-label="`将 ${skill.name} 同步到 ${clientLabels[target]}`"
                  :title="clientLabels[target]"
                  @click="toggleSkill(skill, target)"
                >
                  <ClientIcon :client="target" />
                </button>
              </label>
            </div>
            <div class="row-actions">
              <button class="icon-btn" title="检查 / 更新" @click="checkSkillUpdate(skill)"><svg viewBox="0 0 24 24"><path d="M20 11a8 8 0 1 0-2.34 5.66"/><path d="M20 4v7h-7"/></svg></button>
              <button class="icon-btn danger" title="卸载" @click="uninstallSkill(skill)"><svg viewBox="0 0 24 24"><path d="M4 7h16"/><path d="M10 11v6"/><path d="M14 11v6"/><path d="M6 7l1 13h10l1-13"/><path d="M9 7V4h6v3"/></svg></button>
            </div>
          </article>
        </div>
        <div class="skills-empty" v-if="extensionsError"><b>扩展发现失败</b><span>已保留上次结果；请确认本机 Skills 或客户端配置可读。</span></div>
        <div class="skills-empty" v-else-if="skills.length === 0"><b>暂无 Skills</b><span>发现本机 Skills 源后会显示在这里。</span></div>
        <div class="ext-pagination">
          <span>{{ filteredSkills.length }} / {{ skills.length }} 条 · 第 {{ skillsPage }} / {{ skillsPageCount }} 页</span>
          <div class="ext-page-controls">
            <button class="btn ghost" :disabled="skillsPage <= 1" aria-label="首页" @click="goSkillsPage(1)">«</button>
            <button class="btn ghost" :disabled="skillsPage <= 1" aria-label="上一页" @click="goSkillsPage(skillsPage - 1)">‹</button>
            <span>{{ skillsPage }}</span>
            <button class="btn ghost" :disabled="skillsPage >= skillsPageCount" aria-label="下一页" @click="goSkillsPage(skillsPage + 1)">›</button>
            <button class="btn ghost" :disabled="skillsPage >= skillsPageCount" aria-label="末页" @click="goSkillsPage(skillsPageCount)">»</button>
          </div>
        </div>
      </UiCard>
    </div>

    <div v-else class="ext-view active">
      <UiCard>
        <UiCardHeader>
          <h2>MCP 服务器管理</h2><p>把同一个 MCP 服务器按需同步到各客户端配置；只写对应配置节点。</p>
          <template #actions>
            <div class="toolbar">
            <button class="btn ghost" @click="importMcpDefinition()">导入</button>
            <button class="btn" @click="addMcpDefinition()">添加</button>
            <button class="btn ghost" @click="routes.go('settings', { section: 'extensions' })">设置</button>
            </div>
          </template>
        </UiCardHeader>
        <div class="skills-metrics">
          <button
            v-for="chip in serverChips"
            :key="chip.label"
            type="button"
            class="skills-chip"
            :class="{ active: mcpClientFilter === chip.key }"
            :aria-pressed="mcpClientFilter === chip.key ? 'true' : 'false'"
            @click="selectMcpClient(chip.key)"
          ><span>{{ chip.label }}</span><b>{{ chip.value }}</b></button>
        </div>
        <div class="skills-toolbar"><input v-model="mcpSearch" class="input skills-search" type="search" placeholder="搜索 MCP 名称或说明" aria-label="搜索 MCP"></div>
        <div class="skills-list-head">
          <span>MCP 服务器</span>
          <div class="skills-target-labels"><span>Claude</span><span>Codex</span><span>Gemini</span><span>Grok Build</span><span>OpenCode</span><span>Hermes</span></div>
          <span>操作</span>
        </div>
        <div class="skills-list">
          <article v-for="server in pagedServers" :key="server.id" class="skill-row">
            <div
              class="skill-info"
              :class="{ 'is-clamped': clampedIds.has(server.id) }"
              data-ext-detail
              :data-detail-id="server.id"
              role="button"
              tabindex="0"
              :aria-label="`查看 ${server.name} 完整信息`"
              @click="detailOpen('mcp', server.id)"
              @keydown="detailKeydown('mcp', server.id, $event)"
            >
              <div class="skill-title"><h3>{{ server.name }}</h3><span class="tag">{{ server.transport }}</span><span class="skill-meta"><span>{{ serverVersion(server) }}</span><span>{{ formatUpdatedAt(server.updatedAt) }}</span></span></div>
              <p>{{ server.description }}</p>
            </div>
            <div class="skill-targets">
              <label v-for="target in skillTargets" :key="target" class="target-cell">
                <button
                  class="target-icon"
                  :aria-checked="targetEnabled(server.clients, target) ? 'true' : 'false'"
                  :disabled="extensionBusy"
                  :aria-label="`将 ${server.name} 写入 ${clientLabels[target]}`"
                  :title="clientLabels[target]"
                  @click="toggleMcp(server, target)"
                >
                  <ClientIcon :client="target" />
                </button>
              </label>
            </div>
            <div class="row-actions">
              <button class="icon-btn" title="编辑" @click="editMcpDefinition(server)"><svg viewBox="0 0 24 24"><path d="M4 20h4l10-10-4-4L4 16v4Z"/><path d="m14 6 4 4"/></svg></button>
              <button class="icon-btn danger" title="删除" @click="deleteMcpDefinition(server)"><svg viewBox="0 0 24 24"><path d="M4 7h16"/><path d="M10 11v6"/><path d="M14 11v6"/><path d="M6 7l1 13h10l1-13"/><path d="M9 7V4h6v3"/></svg></button>
            </div>
          </article>
        </div>
        <div class="skills-empty" v-if="extensionsError"><b>扩展发现失败</b><span>已保留上次结果；请确认客户端 MCP 配置可读。</span></div>
        <div class="skills-empty" v-else-if="mcpServers.length === 0"><b>暂无 MCP</b><span>发现客户端 MCP 配置后会显示在这里。</span></div>
        <div class="ext-pagination">
          <span>{{ filteredServers.length }} / {{ mcpServers.length }} 条 · 第 {{ mcpPage }} / {{ serversPageCount }} 页</span>
          <div class="ext-page-controls">
            <button class="btn ghost" :disabled="mcpPage <= 1" aria-label="首页" @click="goServersPage(1)">«</button>
            <button class="btn ghost" :disabled="mcpPage <= 1" aria-label="上一页" @click="goServersPage(mcpPage - 1)">‹</button>
            <span>{{ mcpPage }}</span>
            <button class="btn ghost" :disabled="mcpPage >= serversPageCount" aria-label="下一页" @click="goServersPage(mcpPage + 1)">›</button>
            <button class="btn ghost" :disabled="mcpPage >= serversPageCount" aria-label="末页" @click="goServersPage(serversPageCount)">»</button>
          </div>
        </div>
      </UiCard>
    </div>

    <ExtensionDetailModal
      v-if="detail"
      :title="detail.title"
      :markdown="detail.markdown"
      :markdown-truncated="detail.markdownTruncated"
      :facts="detail.facts"
      :enabled-clients="detailEnabledClients"
      :actions="detail.actions"
      :busy="extensionBusy"
      @close="closeDetail"
      @toggle-client="detailToggleClient"
      @action="detailAction"
    />
  </section>
</template>
