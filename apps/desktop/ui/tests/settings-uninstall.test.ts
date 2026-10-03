import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: async () => () => {},
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

const sourceDto = () => ({
  kind: 'discovered',
  path: '/home/.local/bin/ocx',
  version: '2.64.0',
  resolvedAt: '2026-09-28T00:00:00Z',
  insideDataRoot: false,
  managedEntry: '/data/runtime/bin/ocx',
  managedPrefix: '/data/runtime/opencodex',
  defaultPrefix: '/data/runtime/opencodex',
  explicitPath: null,
  history: [],
})

const planDto = (overrides: Record<string, unknown> = {}) => ({
  sourceKind: 'discovered',
  sourcePath: '/home/.local/bin/ocx',
  managed: false,
  backupId: null,
  removable: true,
  reason: null,
  removeObjects: ['npm 全局包 @bitkyc08/opencodex', '/home/.local/bin/ocx'],
  runtimeObjects: ['service · 官方 launchd/systemd/WinSW 注册'],
  dataObjects: ['/home/.opencodex/config.json'],
  residueCandidates: ['/home/.opencodex/routing-history.sqlite'],
  officialCommand: "'/home/.local/bin/ocx' uninstall",
  externalCommand: 'npm uninstall -g @bitkyc08/opencodex',
  ...overrides,
})

const resultDto = () => ({
  scope: 'full',
  sourceKind: 'unresolved',
  sourcePath: null,
  backupId: 'bk_20260928000000_deadbeef',
  backupDirectory: '/data/backups/2026/09/runtime-uninstall/bk_x',
  steps: [
    { name: '移除包体与入口', status: 'ok', detail: 'npm uninstall -g @bitkyc08/opencodex' },
    { name: '执行官方 ocx uninstall', status: 'ok', detail: '官方已清理' },
  ],
  residue: [{ path: '/home/.local/bin/ocx', status: 'cleared' }],
  officialOutput: ['✅ service removed'],
  needsRestart: false,
  message: '已按「完整卸载」卸载',
})

async function mountInstallation() {
  const pinia = createPinia()
  setActivePinia(pinia)
  useRouteStore().go('settings', { section: 'installation' })
  const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] } })
  await flushPromises()
  await nextTick()
  // 运行来源卡片的事实来自后端解析结果；测试里显式触发一次读取。
  await useAppStore().loadRuntimeSource()
  await flushPromises()
  await nextTick()
  return wrapper
}

async function openUninstall(wrapper: ReturnType<typeof mount>) {
  const button = wrapper.findAll('button').find(node => node.text().trim() === '卸载')
  expect(button, '运行来源卡片应有「卸载」按钮（无省略号）').toBeTruthy()
  await button!.trigger('click')
  await flushPromises()
  await nextTick()
  return button!
}

beforeEach(() => {
  invoke.mockReset()
})

describe('统一卸载弹窗（Revision 11）', () => {
  it('按来源渲染方案、备份只提醒，且未确认前主行动禁用并给出原因', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'runtime_source') return Promise.resolve(sourceDto())
      if (command === 'plan_runtime_uninstall') return Promise.resolve(planDto())
      return Promise.resolve(null)
    })

    const wrapper = await mountInstallation()
    await openUninstall(wrapper)

    const modal = wrapper.find('[data-testid="runtime-uninstall-modal"]')
    expect(modal.exists()).toBe(true)
    // 范围两级
    expect(modal.find('[data-testid="uninstall-scope-full"]').exists()).toBe(true)
    expect(modal.find('[data-testid="uninstall-scope-body"]').exists()).toBe(true)
    // 备份只提醒、不阻断
    expect(modal.find('[data-testid="uninstall-backup"]').text()).toContain('未检测到可恢复备份')
    const autoBackup = modal.find('[data-testid="uninstall-auto-backup"] input')
    expect((autoBackup.element as HTMLInputElement).checked).toBe(true)
    // 确认项常驻 + 禁用原因
    expect(modal.find('[data-testid="uninstall-ack"]').exists()).toBe(true)
    const confirm = modal.find('[data-testid="uninstall-confirm"]')
    expect(confirm.attributes('disabled')).toBeDefined()
    expect(modal.find('[data-testid="uninstall-ack-hint"]').text()).toContain('勾选')
    // 将移除的对象：外部包体给出应用将执行的固定命令
    expect(modal.text()).toContain('npm uninstall -g @bitkyc08/opencodex')
    await wrapper.unmount()
  })

  it('勾选确认后按完整卸载请求执行，并渲染真实步骤与残留核验', async () => {
    const requests: unknown[] = []
    const commands: string[] = []
    invoke.mockImplementation((command: string, args?: Record<string, unknown>) => {
      commands.push(command)
      if (command === 'runtime_source') return Promise.resolve(sourceDto())
      if (command === 'plan_runtime_uninstall') return Promise.resolve(planDto())
      if (command === 'uninstall_runtime') {
        requests.push(args?.request)
        return Promise.resolve(resultDto())
      }
      return Promise.resolve(null)
    })

    const wrapper = await mountInstallation()
    await openUninstall(wrapper)

    const modal = wrapper.find('[data-testid="runtime-uninstall-modal"]')
    await modal.find('[data-testid="uninstall-ack"]').setValue(true)
    await nextTick()
    expect(modal.find('[data-testid="uninstall-confirm"]').attributes('disabled')).toBeUndefined()

    await modal.find('[data-testid="uninstall-confirm"]').trigger('click')
    await flushPromises()
    await nextTick()

    expect(requests).toEqual([
      { scope: 'full', autoBackup: true, cleanData: true, confirmation: 'confirmed' },
    ])
    const view = wrapper.find('[data-testid="runtime-uninstall-modal"]')
    expect(view.find('[data-testid="uninstall-progress-title"]').text()).toContain('卸载完成')
    // 完成后换成后端返回的真实步骤
    const steps = view.find('[data-testid="uninstall-steps"]').text()
    expect(steps).toContain('执行官方 ocx uninstall')
    // 残留核验卡片 + 官方输出明细框
    expect(view.find('[data-testid="uninstall-residue"]').text()).toContain('已清除')
    expect(view.find('[data-testid="uninstall-console"]').text()).toContain('service removed')
    // 卸载后必须重跑环境发现：ocx 入口已移除，环境卡 / 概览「安装形态」/ 门禁
    // 都要立刻不再显示「已发现」，不能停在卸载前的事实。
    expect(commands).toContain('discover_environment')
    await wrapper.unmount()
  })

  it('形态无法确认时禁止确认并显示原因', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'runtime_source') return Promise.resolve(sourceDto())
      if (command === 'plan_runtime_uninstall')
        return Promise.resolve(planDto({ removable: false, reason: '无法确认 /x 属于官方包目录，已拒绝代删' }))
      return Promise.resolve(null)
    })

    const wrapper = await mountInstallation()
    await openUninstall(wrapper)

    const modal = wrapper.find('[data-testid="runtime-uninstall-modal"]')
    expect(modal.find('[data-testid="uninstall-not-removable"]').text()).toContain('已拒绝代删')
    await modal.find('[data-testid="uninstall-ack"]').setValue(true)
    await nextTick()
    expect(modal.find('[data-testid="uninstall-confirm"]').attributes('disabled')).toBeDefined()
    await wrapper.unmount()
  })

  // §26.1 禁止假成功：有失败步骤时不得报「卸载完成 · 100%」。
  it('有失败步骤时不显示 100% 成功，而是报出失败步数', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'runtime_source') return Promise.resolve(sourceDto())
      if (command === 'plan_runtime_uninstall') return Promise.resolve(planDto())
      if (command === 'uninstall_runtime') {
        return Promise.resolve({
          ...resultDto(),
          steps: [
            { name: '移除包体与入口', status: 'ok', detail: 'npm uninstall -g @bitkyc08/opencodex' },
            { name: '执行官方 ocx uninstall', status: 'failed', detail: 'exit 1' },
          ],
          message: '卸载完成但 1 步失败；请按步骤详情处理，未清理项已列入残留核验',
        })
      }
      return Promise.resolve(null)
    })

    const wrapper = await mountInstallation()
    await openUninstall(wrapper)

    const modal = wrapper.find('[data-testid="runtime-uninstall-modal"]')
    await modal.find('[data-testid="uninstall-ack"]').setValue(true)
    await nextTick()
    await modal.find('[data-testid="uninstall-confirm"]').trigger('click')
    await flushPromises()
    await nextTick()

    const view = wrapper.find('[data-testid="runtime-uninstall-modal"]')
    expect(view.find('[data-testid="uninstall-progress-title"]').text()).toBe('卸载完成但有 1 步失败')
    expect(view.find('[data-testid="uninstall-progress-title"]').text()).not.toContain('100%')
    expect(view.find('.wiz-umeta').text()).toContain('1 步失败')
    expect(view.find('.wiz-umeta').text()).not.toContain('100%')
    expect(view.find('.wiz-ufill').classes()).toContain('faulted')
    await wrapper.unmount()
  })
})
