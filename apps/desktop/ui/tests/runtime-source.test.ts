import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import type { RuntimeSourceDto } from '@/features/runtime/api'
import type { EnvironmentReport } from '@/features/environment/api'
import EnvironmentGate from '@/features/environment/components/EnvironmentGate.vue'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

const eventBus = vi.hoisted(() => ({
  listeners: {} as Record<string, (event: { payload: unknown }) => void>,
}))
vi.mock('@tauri-apps/api/event', () => ({
  listen: async (name: string, handler: (event: { payload: unknown }) => void) => {
    eventBus.listeners[name] = handler
    return () => {
      delete eventBus.listeners[name]
    }
  },
}))

function sourceDto(overrides: Partial<RuntimeSourceDto> = {}): RuntimeSourceDto {
  return {
    kind: 'managed',
    path: '/data/runtime/bin/ocx',
    version: '2.50.0',
    resolvedAt: '2026-09-25T00:00:00Z',
    insideDataRoot: true,
    managedEntry: '/data/runtime/bin/ocx',
    managedPrefix: '/data/runtime/opencodex',
    defaultPrefix: '/data/runtime/opencodex',
    explicitPath: null,
    history: [],
    ...overrides,
  }
}

function report(overrides: Partial<EnvironmentReport> = {}): EnvironmentReport {
  const found = { found: true, path: '/fixtures/node', version: null }
  return {
    node: { ...found },
    npm: { ...found },
    ocx: { found: false, path: null, version: null },
    gate: 'missing_ocx',
    shortCircuited: false,
    brewFound: false,
    ...overrides,
  }
}

describe('runtime source store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    for (const key of Object.keys(eventBus.listeners)) delete eventBus.listeners[key]
  })

  it('loads the source fact from the backend and stores it verbatim', async () => {
    invoke.mockResolvedValueOnce(sourceDto())
    const app = useAppStore()
    expect(await app.loadRuntimeSource()).toBe(true)
    expect(invoke).toHaveBeenCalledWith('runtime_source', undefined)
    expect(app.runtimeSource?.kind).toBe('managed')
    expect(app.runtimeSource?.managedPrefix).toBe('/data/runtime/opencodex')
  })

  it('keeps the previous fact and records a readable error when the read fails', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce(sourceDto({ kind: 'discovered', path: '/usr/local/bin/ocx' }))
    await app.loadRuntimeSource()
    invoke.mockRejectedValueOnce({ code: 'runtime-managed', message: '读取失败' })
    expect(await app.loadRuntimeSource()).toBe(false)
    expect(app.runtimeSourceError).toBe('读取失败')
    // 读不到不等于「未解析」：上一次的真实事实必须保留。
    expect(app.runtimeSource?.kind).toBe('discovered')
  })

  it('sends the install request through and refreshes the source afterwards', async () => {
    const app = useAppStore()
    invoke.mockImplementation(async (command: string) => {
      if (command === 'install_runtime') {
        return {
          package: '@bitkyc08/opencodex',
          version: '2.50.0',
          target: '/data/runtime/opencodex',
          entry: '/data/runtime/bin/ocx',
          tarballSha256: 'a'.repeat(64),
          source: 'registry',
          scriptsEnabled: false,
          npmPath: '/usr/local/bin/npm',
          installedAt: '2026-09-25T00:00:00Z',
          proxyUsed: false,
          needsRestart: true,
        }
      }
      if (command === 'runtime_source') return sourceDto()
      if (command === 'official_project_facts') {
        return {
          displayName: 'OpenCodex',
          version: '2.50.0',
          rawVersion: 'OpenCodex 2.50.0',
          truncated: false,
        }
      }
      return null
    })
    const ok = await app.installRuntime({
      prefix: '/data/runtime/opencodex',
      source: 'registry',
      version: 'latest',
      offlinePath: null,
      proxyScheme: null,
      proxyHost: null,
      proxyUsername: null,
      proxySecret: null,
      allowScripts: false,
    })
    expect(ok).toBe(true)
    expect(app.runtimeInstallOutcome?.version).toBe('2.50.0')
    expect(app.runtimeInstall?.phase).toBe('done')
    expect(invoke).toHaveBeenCalledWith('install_runtime', {
      request: expect.objectContaining({ source: 'registry', allowScripts: false }),
    })
    // 终态后必须重新解析来源，否则卡片会停在旧事实。
    expect(invoke).toHaveBeenCalledWith('runtime_source', undefined)
    // 也要刷新官方版本事实：升级卡 / 关于读的是它，否则装完还显示「未发现」。
    expect(invoke).toHaveBeenCalledWith('official_project_facts')
    expect(app.officialProject?.version).toBe('2.50.0')
    expect(app.toast).toContain('重启后生效')
  })

  it('refuses to claim success when the source did not actually switch to managed', async () => {
    const app = useAppStore()
    invoke.mockImplementation(async (command: string) => {
      if (command === 'install_runtime') {
        return {
          package: '@bitkyc08/opencodex',
          version: '2.66.0',
          target: '/data/runtime/opencodex',
          entry: '/data/runtime/bin/ocx',
          tarballSha256: 'c'.repeat(64),
          source: 'registry',
          scriptsEnabled: false,
          npmPath: '/usr/local/bin/npm',
          installedAt: '2026-09-26T00:00:00Z',
          proxyUsed: false,
          needsRestart: false,
        }
      }
      // 来源仍是未解析：说明这次安装并没有真正落地。
      if (command === 'runtime_source') return sourceDto({ kind: 'unresolved', path: null, version: null })
      return null
    })
    const ok = await app.installRuntime({
      prefix: null,
      source: 'registry',
      version: 'latest',
      offlinePath: null,
      proxyScheme: null,
      proxyHost: null,
      proxyUsername: null,
      proxySecret: null,
      allowScripts: false,
    })
    expect(ok).toBe(false)
    expect(app.runtimeInstall?.phase).toBe('failed')
    expect(app.runtimeInstallError).toContain('没有切换到托管安装')
  })

  it('re-reads the version fact after switching source so the card is not stuck on 未知', async () => {
    // 真机回归：切到显式来源后卡片「版本」一直是「未知」——`FZ-48` 的
    // `resolved_version` 没有被回填。切换来源后必须读一次版本，再重读来源。
    const app = useAppStore()
    const calls: string[] = []
    invoke.mockImplementation(async (command: string) => {
      calls.push(command)
      if (command === 'set_runtime_source') {
        return sourceDto({ kind: 'explicit', path: '/usr/local/bin/ocx', version: null })
      }
      if (command === 'official_project_facts') {
        return {
          displayName: 'OpenCodex',
          version: '2.50.0',
          rawVersion: 'OpenCodex 2.50.0',
          truncated: false,
        }
      }
      if (command === 'runtime_source') {
        return sourceDto({ kind: 'explicit', path: '/usr/local/bin/ocx', version: '2.50.0' })
      }
      return null
    })
    expect(await app.setRuntimeSourcePath('/usr/local/bin/ocx')).toBe(true)
    expect(calls).toContain('official_project_facts')
    expect(calls.indexOf('official_project_facts')).toBeLessThan(calls.lastIndexOf('runtime_source'))
    expect(app.runtimeSource?.version).toBe('2.50.0')
  })

  it('restores automatic discovery and re-reads the version fact', async () => {
    // 回归：`恢复自动发现` 此前无用例覆盖（§17.1 入口 SettingsRoute#24）。
    // 恢复后必须：调用后端 restore、失败保留原值、成功后重读版本事实并给反馈。
    const app = useAppStore()
    const calls: string[] = []
    invoke.mockImplementation(async (command: string) => {
      calls.push(command)
      if (command === 'restore_discovered_runtime') {
        return sourceDto({ kind: 'discovered', path: '/usr/local/bin/ocx', explicitPath: null })
      }
      if (command === 'official_project_facts') {
        return {
          displayName: 'OpenCodex',
          version: '2.50.0',
          rawVersion: 'OpenCodex 2.50.0',
          truncated: false,
        }
      }
      if (command === 'get_status_snapshot') return null
      if (command === 'runtime_source') {
        return sourceDto({ kind: 'discovered', path: '/usr/local/bin/ocx', explicitPath: null })
      }
      return null
    })
    expect(await app.restoreDiscoveredRuntime()).toBe(true)
    expect(calls).toContain('restore_discovered_runtime')
    expect(calls).toContain('official_project_facts')
    expect(calls).toContain('get_status_snapshot')
    expect(app.runtimeSource?.kind).toBe('discovered')
    expect(app.toast).toBe('已恢复自动发现。')
  })

  it('keeps the explicit source and surfaces the reason when restore fails', async () => {
    const app = useAppStore()
    invoke.mockImplementation(async (command: string) => {
      if (command === 'restore_discovered_runtime') throw new Error('restore failed')
      return null
    })
    expect(await app.restoreDiscoveredRuntime()).toBe(false)
    expect(app.runtimeSourceError).toBe('restore failed')
  })

  it('refreshes the source fact after the version check so the card shows the version', async () => {
    const app = useAppStore()
    const calls: string[] = []
    invoke.mockImplementation(async (command: string) => {
      calls.push(command)
      if (command === 'official_project_facts') {
        return {
          displayName: 'OpenCodex',
          version: '2.50.0',
          rawVersion: 'OpenCodex 2.50.0',
          truncated: false,
        }
      }
      if (command === 'runtime_source') {
        return sourceDto({ kind: 'explicit', path: '/usr/local/bin/ocx', version: '2.50.0' })
      }
      return null
    })
    await app.loadOfficialProject()
    expect(calls).toEqual(['official_project_facts', 'runtime_source'])
    expect(app.runtimeSource?.version).toBe('2.50.0')
  })

  it('shares an in-flight source read so a later caller never sees a stale source', async () => {
    // 真机回归：安装完成后端先广播 `runtime-source-changed`（事件监听发起一次读取），
    // 安装流程紧接着自己也读一次。若后来者空手返回，安装流程就会拿切换前的旧来源，
    // 把一次成功安装误报成「来源没有切换到托管安装」。这里固定「后来者复用同一份结果」。
    const app = useAppStore()
    let resolveSource: (value: RuntimeSourceDto) => void = () => {}
    const pending = new Promise<RuntimeSourceDto>((resolve) => {
      resolveSource = resolve
    })
    invoke.mockImplementation(async (command: string) => {
      if (command === 'runtime_source') return pending
      return null
    })
    const eventRead = app.loadRuntimeSource()
    const installRead = app.loadRuntimeSource()
    resolveSource(sourceDto({ kind: 'managed', path: '/data/runtime/bin/ocx' }))
    expect(await installRead).toBe(true)
    expect(await eventRead).toBe(true)
    expect(app.runtimeSource?.kind).toBe('managed')
    // 后来者不能另起一次读取：要复用同一份在途结果。
    expect(invoke).toHaveBeenCalledTimes(1)
  })

  it('surfaces the install failure reason and does not claim success', async () => {
    const app = useAppStore()
    invoke.mockImplementation(async (command: string) => {
      if (command === 'install_runtime') {
        throw { code: 'scripts_required', message: '该包需要执行安装脚本' }
      }
      if (command === 'runtime_source') return sourceDto({ kind: 'unresolved', path: null })
      return null
    })
    const ok = await app.installRuntime({
      prefix: null,
      source: 'registry',
      version: 'latest',
      offlinePath: null,
      proxyScheme: null,
      proxyHost: null,
      proxyUsername: null,
      proxySecret: null,
      allowScripts: false,
    })
    expect(ok).toBe(false)
    expect(app.runtimeInstallError).toBe('该包需要执行安装脚本')
    expect(app.runtimeInstall?.phase).toBe('failed')
    expect(app.runtimeInstallOutcome).toBeNull()
  })

  it('carries the proxy only in the request and never persists it in state', async () => {
    const app = useAppStore()
    invoke.mockImplementation(async (command: string) => {
      if (command === 'install_runtime') {
        return {
          package: '@bitkyc08/opencodex',
          version: '2.50.0',
          target: '/data/runtime/opencodex',
          entry: '/data/runtime/bin/ocx',
          tarballSha256: 'b'.repeat(64),
          source: 'registry',
          scriptsEnabled: false,
          npmPath: '/usr/local/bin/npm',
          installedAt: '2026-09-25T00:00:00Z',
          proxyUsed: true,
          needsRestart: false,
        }
      }
      if (command === 'runtime_source') return sourceDto()
      return null
    })
    await app.installRuntime({
      prefix: null,
      source: 'registry',
      version: 'latest',
      offlinePath: null,
      proxyScheme: 'socks5h',
      proxyHost: '127.0.0.1:1080',
      proxyUsername: 'alice',
      proxySecret: 's3cret',
      allowScripts: false,
    })
    const payload = JSON.stringify(app.$state)
    expect(payload).not.toContain('s3cret')
    expect(payload).not.toContain('127.0.0.1:1080')
  })

  it('runs a full uninstall through the app and reports steps plus residue', async () => {
    const app = useAppStore()
    invoke.mockImplementation(async (command: string) => {
      if (command === 'uninstall_runtime') {
        return {
          scope: 'full',
          sourceKind: 'discovered',
          sourcePath: '/home/.local/bin/ocx',
          backupId: 'bk_20260925000000_deadbeef',
          backupDirectory: '/data/backups/2026/09/runtime-uninstall/bk_x',
          steps: [
            { name: '生成备份', status: 'ok', detail: 'bk bk_20260925000000_deadbeef' },
            { name: '移除包体与入口', status: 'ok', detail: 'npm uninstall -g @bitkyc08/opencodex' },
            { name: '执行官方 ocx uninstall', status: 'ok', detail: '官方已清理 service / shim / config' },
          ],
          residue: [{ path: '/home/.local/bin/ocx', status: 'cleared' }],
          officialOutput: ['✅ service removed'],
          needsRestart: false,
          message: '已按「完整卸载」卸载',
        }
      }
      if (command === 'official_uninstall_observation') return ['exit_code=0']
      if (command === 'runtime_source') return sourceDto()
      if (command === 'official_project_facts') {
        throw { code: 'not-configured', message: '运行来源未解析' }
      }
      return null
    })
    const ok = await app.uninstallRuntime({
      scope: 'full',
      autoBackup: true,
      cleanData: true,
      confirmation: 'confirmed',
    })
    expect(ok).toBe(true)
    expect(app.runtimeUninstallResult?.steps).toHaveLength(3)
    expect(app.runtimeUninstallResult?.steps[1]?.status).toBe('ok')
    expect(app.runtimeUninstallResult?.residue[0]?.status).toBe('cleared')
    expect(app.officialUninstallObservation).toEqual(['exit_code=0'])
    // 读不到时保留上次结果并标记错误，不谎称当前版本（FZ-48）。
    expect(app.officialProjectError).toBe(true)
    expect(invoke).toHaveBeenCalledWith('official_project_facts')
  })

  it('loads the read-only uninstall plan and keeps its refusal reason', async () => {
    const app = useAppStore()
    invoke.mockImplementation(async (command: string) => {
      if (command === 'plan_runtime_uninstall') {
        return {
          sourceKind: 'explicit',
          sourcePath: '/tmp/stray-ocx',
          managed: false,
          backupId: null,
          removable: false,
          reason: '无法确认 /tmp/stray-ocx 属于 @bitkyc08/opencodex 的包目录，已拒绝代删',
          removeObjects: [],
          runtimeObjects: ['service · 官方 launchd/systemd/WinSW 注册'],
          dataObjects: [],
          residueCandidates: [],
          officialCommand: 'ocx uninstall',
          externalCommand: null,
        }
      }
      return null
    })
    const ok = await app.loadUninstallPlan()
    expect(ok).toBe(true)
    expect(app.runtimeUninstallPlan?.removable).toBe(false)
    expect(app.runtimeUninstallPlan?.reason).toContain('已拒绝代删')
  })

  it('reports uninstall refusals instead of pretending success', async () => {
    const app = useAppStore()
    invoke.mockRejectedValueOnce({ code: 'uninstall_backup_failed', message: '备份失败，已中止卸载（未删除任何文件）' })
    const ok = await app.uninstallRuntime({
      scope: 'full',
      autoBackup: true,
      cleanData: false,
      confirmation: 'confirmed',
    })
    expect(ok).toBe(false)
    expect(app.runtimeUninstallError).toContain('备份失败')
    expect(app.runtimeUninstallResult).toBeNull()
  })

  it('feeds live install progress into the modal state and unsubscribes on demand', async () => {
    const app = useAppStore()
    const stop = await app.startRuntimeSourceEventStream()
    expect(stop).not.toBeNull()
    expect(typeof eventBus.listeners['runtime-install-progress']).toBe('function')
    expect(typeof eventBus.listeners['runtime-source-changed']).toBe('function')

    eventBus.listeners['runtime-install-progress']({ payload: { phase: 'installing', percent: 60, line: 'npm install line' } })
    expect(app.runtimeInstall?.phase).toBe('installing')
    expect(app.runtimeInstallLines).toEqual(['npm install line'])

    invoke.mockResolvedValueOnce(sourceDto({ kind: 'managed' }))
    eventBus.listeners['runtime-source-changed']({ payload: null })
    await new Promise(resolve => setTimeout(resolve, 0))
    expect(invoke).toHaveBeenCalledWith('runtime_source', undefined)

    stop?.()
    expect(eventBus.listeners['runtime-install-progress']).toBeUndefined()
  })

  it('hands the overview install request over exactly once', () => {
    const app = useAppStore()
    expect(app.consumeInstallModalRequest()).toBeNull()
    app.requestInstallModal('offline')
    expect(app.installModalRequest).toBe('offline')
    expect(app.consumeInstallModalRequest()).toBe('offline')
    expect(app.consumeInstallModalRequest()).toBeNull()
  })
})

describe('overview missing-ocx guidance', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('offers managed install and offline import instead of only a copyable command', async () => {
    const app = useAppStore()
    app.setEnvironment(report())
    const wrapper = mount(EnvironmentGate)
    const actions = wrapper.find('[data-testid="env-install-actions"]')
    expect(actions.exists()).toBe(true)
    const buttons = actions.findAll('button')
    expect(buttons.map(button => button.text())).toEqual(['安装 OpenCodex', '导入离线包', '重新检查'])

    await buttons[0].trigger('click')
    expect(app.installModalRequest).toBe('registry')
    await buttons[1].trigger('click')
    expect(app.installModalRequest).toBe('offline')
  })

  it('drops the PATH assumption from the guidance text', () => {
    const app = useAppStore()
    app.setEnvironment(report())
    const wrapper = mount(EnvironmentGate)
    const bound = wrapper.find('.env-bound').text()
    expect(bound).not.toContain('不执行 brew、npm 或 ocx 安装')
    expect(bound).toContain('不读取 PATH')
    expect(wrapper.text()).not.toContain('ocx --version')
    expect(wrapper.text()).toContain('npm install -g @bitkyc08/opencodex')
  })
})
