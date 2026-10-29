import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import App from '@/App.vue'
import { useUpdatesStore } from '@/features/updates/store'
import { useRouteStore } from '@/stores/routes'
import { managerUpdateStatus } from '@/features/updates/update'

const invoke = vi.fn()

vi.mock('@/features/about/api', async original => ({...await original<typeof import('@/features/about/api')>(),getOfficialProjectFacts:async()=>({displayName:'OpenCodex',version:'2.50.0',rawVersion:'2.50.0',truncated:false})}))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: query === '(prefers-color-scheme: dark)',
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

const mounted: ReturnType<typeof mount>[] = []
afterEach(() => { for (const wrapper of mounted.splice(0)) wrapper.unmount() })

async function settle(wrapper: ReturnType<typeof mount>) {
  await wrapper.vm.$nextTick()
  await new Promise(resolve => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()
}

function mountOverview() {
  const pinia = createPinia(); setActivePinia(pinia)
  const wrapper = mount(App, { global: { plugins: [pinia] } })
  mounted.push(wrapper)
  useRouteStore(pinia).go('overview')
  return wrapper
}

// 概览就绪形态需要环境就绪；发现调用返回一份真实形状的就绪报告（不连真实端点）。
function readyEnvironment() {
  const found = { found: true, path: '/fixtures/bin', version: null }
  return { node: { ...found }, npm: { ...found }, ocx: { ...found }, gate: 'ready', shortCircuited: false, brewFound: true }
}

describe('overview module cards', () => {
  beforeEach(() => {
    Object.assign(HTMLDialogElement.prototype,{showModal(this:HTMLDialogElement){this.setAttribute('open','')},close(this:HTMLDialogElement){this.removeAttribute('open')}})
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockImplementation((command: string) =>
      command === 'discover_environment' ? Promise.resolve(readyEnvironment()) : command === 'official_project_facts' ? Promise.resolve({displayName:'OpenCodex',version:'2.50.0',rawVersion:'2.50.0',truncated:false}) : command === 'official_remote_latest' ? Promise.resolve({tag:'latest',version:'2.51.0',integrity:null}) : Promise.resolve(undefined),
    )
  })

  it('uses the header ⋯ as the settings entry instead of a third action button', async () => {
    const wrapper = mountOverview()
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()

    const mods = wrapper.findAll('.mods .mod')
    expect(mods.length).toBe(3)

    for (const mod of mods) {
      // 右上角 ⋯ 是真实按钮，带可读名称。
      const go = mod.find('button.mod-go')
      expect(go.exists()).toBe(true)
      expect(go.attributes('aria-label')).toBeTruthy()
      expect(go.attributes('aria-hidden')).toBeUndefined()
      // 动作区不再重复放「…设置」按钮（原型用 [data-settings-section] 隐藏）。
      const labels = mod.findAll('.mod-actions .btn').map(btn => btn.text())
      expect(labels.some(label => label.endsWith('设置'))).toBe(false)
    }
  })

  it('navigates to the matching settings section from the header ⋯', async () => {
    const wrapper = mountOverview()
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()

    const routes = useRouteStore(wrapper.vm.$pinia)
    const upgrade = wrapper.findAll('.mods .mod')[0]
    expect(upgrade.find('.mod-head h3').text()).toBe('版本升级')

    await upgrade.find('button.mod-go').trigger('click')

    expect(routes.current).toBe('settings')
    expect(routes.settingsSection).toBe('upgrade')
  })

  it('opens import in the overview with a disabled read action before a file is selected', async () => {
    const wrapper = mountOverview(); await settle(wrapper); await flushPromises()
    const route=useRouteStore(wrapper.vm.$pinia)
    const importer=wrapper.findAll('button').find(b=>b.text()==='导入配置')!
    await importer.trigger('click'); await flushPromises()
    expect(route.current).toBe('overview')
    const modal=document.querySelector('dialog[open]')
    expect(modal?.textContent).toContain('尚未选择文件')
    const read=[...modal!.querySelectorAll('button')].find(b=>b.textContent==='读取并比较')!
    expect(read.disabled).toBe(true)
    const cancel=[...modal!.querySelectorAll('button')].find(b=>b.textContent==='取消')!
    cancel.click();await flushPromises()
    expect(route.current).toBe('overview')
    expect(document.querySelector('dialog[open]')).toBeNull()
  })

  // OV-4 / N-OV-2：版本行按原型「管理器 / 面板 + 版本 + 阶段 + ›」三段呈现（index.html:1877-1878）。
  it('renders each upgrade row as label, version, phase and chevron', async () => {
    managerUpdateStatus.value = {
      channel: 'stable', availableVersion: '0.2.9', currentVersion: '0.2.8', lastCheckedAt: '2026-10-25T00:00:00Z',
      signatureVerified: true, error: null,
    }
    try {
      const wrapper = mountOverview()
      await settle(wrapper)
      await useUpdatesStore(wrapper.vm.$pinia).loadProject();await useUpdatesStore(wrapper.vm.$pinia).loadRemoteLatest();await flushPromises()
      const upgrade = wrapper.findAll('.mods .mod')[0]
      const rows = upgrade.findAll('.mnt-version-row')
      await vi.waitFor(() => expect(rows.map(row => Array.from(row.element.children).map(node => node.textContent?.trim()))).toEqual([
        ['管理器', '0.2.8', '有可用更新', '›'],
        ['面板', '2.50.0', '有可用更新', '›'],
      ]))
      expect(rows[0].attributes('aria-label')).toBe('查看桌面管理器更新详情')
      expect(rows[1].attributes('aria-label')).toBe('查看 OpenCodex 面板更新详情')
    } finally {
      managerUpdateStatus.value = null
    }
  })

  // 统一要求：列表行内元素垂直居中，不互相挤压。
  it('keeps version rows vertically centered without wrapping', () => {
    const route = readFileSync(resolve(__dirname, '../src/routes/OverviewRoute.vue'), 'utf8')
    expect(route).toMatch(/\.mnt-version-row \{[^}]*align-items: center/)
    expect(route).toMatch(/\.mnt-version-row \{[^}]*white-space: nowrap/)
  })

  // 检查完成且无候选版本：阶段读作「已是最新」，而不是破折号。
  it('reads a completed check without a candidate as up to date', async () => {
    managerUpdateStatus.value = {
      channel: 'beta', availableVersion: null, currentVersion: '0.2.8', lastCheckedAt: '2026-10-25T00:00:00Z',
      signatureVerified: null, error: null,
    }
    try {
      const wrapper = mountOverview()
      await settle(wrapper)
      const row = wrapper.findAll('.mnt-version-row')[0]
      expect(Array.from(row.element.children).map(node => node.textContent?.trim())).toEqual(['管理器', '0.2.8', '已是最新', '›'])
    } finally {
      managerUpdateStatus.value = null
    }
  })
})
