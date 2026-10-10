import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRouteStore } from '@/stores/routes'
import { useRuntimeStore } from '@/features/runtime/store'
import type { RuntimeProtectionStatus } from '@/features/runtime/api'

const { invoke, open } = vi.hoisted(() => ({ invoke: vi.fn(), open: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open }))

const none: RuntimeProtectionStatus = { state: 'none', version: null, backupId: null }
const pending: RuntimeProtectionStatus = { state: 'pending', version: '2.51.0', backupId: 'bk_exact' }
const unreadable: RuntimeProtectionStatus = { state: 'unreadable', version: null, backupId: null }
function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>(done => { resolve = done })
  return { promise, resolve }
}
function setup(status: RuntimeProtectionStatus = pending) {
  const pinia = createPinia()
  setActivePinia(pinia)
  const runtime = useRuntimeStore()
  runtime.source = {
    kind: 'managed', path: '/data/runtime/bin/ocx', version: '2.51.0', resolvedAt: null,
    insideDataRoot: true, managedEntry: '/data/runtime/bin/ocx',
    managedPrefix: '/data/runtime/opencodex', defaultPrefix: '/data/runtime/opencodex',
    explicitPath: '/selected/bin/ocx', history: [],
  }
  invoke.mockImplementation(async (command: string) => command === 'runtime_protection_status' ? status : null)
  useRouteStore().go('settings', { section: 'installation' })
  return { runtime, wrapper: mount(SettingsRoute, { global: { plugins: [pinia] } }) }
}

beforeEach(() => {
  setActivePinia(createPinia())
  invoke.mockReset()
  open.mockReset()
})

describe('local panel protection status and retry', () => {
  it.each([pending, { ...pending, state: 'verified_pending' as const }, unreadable])('keeps $state visible and disables installation, uninstall and directory changes', async status => {
    const { wrapper } = setup(status)
    await flushPromises()
    const warning = wrapper.get('[data-testid="runtime-protection"]')
    expect(warning.text()).toContain('面板更新保护核对')
    expect(warning.text()).toContain(status.state === 'unreadable' ? '记录与备份均保留' : '备份')
    for (const testid of ['runtime-install', 'runtime-import-offline', 'runtime-uninstall', 'data-root-migrate', 'runtime-change-source', 'runtime-restore-source']) {
      expect(wrapper.get('[data-testid="' + testid + '"]').attributes('disabled')).toBeDefined()
    }
    if (status.state === 'unreadable') expect(warning.text()).not.toContain('bk_exact')
    const diagnosis = warning.findAll('button').find(button => button.text() === '查看诊断')!
    await diagnosis.trigger('click')
    expect(useRouteStore().current).toBe('logs')
    expect(useRouteStore().diagnosticsTab).toBe('logs')
    wrapper.unmount()
  })

  it('blocks source changes until a clean protection read finishes', async () => {
    const { runtime, wrapper } = setup(none)
    runtime.protection = null
    runtime.protectionLoading = true
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[data-testid="runtime-change-source"]').attributes('disabled')).toBeDefined()
    expect(wrapper.get('[data-testid="runtime-restore-source"]').attributes('disabled')).toBeDefined()
    await flushPromises()
    expect(wrapper.get('[data-testid="runtime-change-source"]').attributes('disabled')).toBeUndefined()
    expect(wrapper.get('[data-testid="runtime-restore-source"]').attributes('disabled')).toBeUndefined()
    wrapper.unmount()
  })

  it('rechecks protection when the source file dialog returns', async () => {
    const { runtime, wrapper } = setup(none)
    await flushPromises()
    const selected = deferred<string>()
    open.mockReturnValue(selected.promise)
    await wrapper.get('[data-testid="runtime-change-source"]').trigger('click')
    expect(open).toHaveBeenCalledTimes(1)
    runtime.protection = pending
    selected.resolve('/selected/new/ocx')
    await flushPromises()
    expect(invoke.mock.calls.map(call => call[0])).not.toContain('set_runtime_source')
    expect(runtime.source?.explicitPath).toBe('/selected/bin/ocx')
    expect(wrapper.get('[data-testid="runtime-change-source"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('allows a selected source when protection stays clean', async () => {
    const { runtime, wrapper } = setup(none)
    await flushPromises()
    open.mockResolvedValue('/selected/new/ocx')
    invoke.mockImplementation(async command => command === 'set_runtime_source'
      ? { ...runtime.source, explicitPath: '/selected/new/ocx' } : null)
    await wrapper.get('[data-testid="runtime-change-source"]').trigger('click')
    await flushPromises()
    expect(invoke).toHaveBeenCalledWith('set_runtime_source', { path: '/selected/new/ocx' })
    wrapper.unmount()
  })

  it('shows transport failures as unknown rather than a clean protection state', async () => {
    const { runtime, wrapper } = setup()
    await flushPromises()
    invoke.mockRejectedValueOnce({ message: '保护记录读取失败' })
    await runtime.loadProtection()
    expect(runtime.protection).toEqual(pending)
    expect(wrapper.get('[role="alert"]').text()).toContain('保护记录读取失败')
    expect(wrapper.get('[data-testid="runtime-install"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('owns a single explicit retry and clears the warning only after success', async () => {
    const { runtime, wrapper } = setup()
    await flushPromises()
    const result = deferred<RuntimeProtectionStatus>()
    invoke.mockImplementation(command => command === 'retry_runtime_protection' ? result.promise : Promise.resolve(none))
    const retry = runtime.retryProtection()
    await runtime.retryProtection()
    await flushPromises()
    expect(wrapper.get('[data-testid="runtime-protection-retry"]').text()).toBe('核对中…')
    expect(wrapper.get('[data-testid="runtime-protection-retry"]').attributes('disabled')).toBeDefined()
    expect(invoke.mock.calls.filter(call => call[0] === 'retry_runtime_protection')).toHaveLength(1)
    result.resolve(none)
    await retry
    await flushPromises()
    expect(runtime.protection).toEqual(none)
    expect(wrapper.get('[data-testid="runtime-protection"]').text()).toContain('本地核对完成')
    expect(wrapper.get('[data-testid="runtime-install"]').attributes('disabled')).toBeUndefined()
    expect(invoke.mock.calls.map(call => call[0])).not.toContain('install_runtime')
    expect(invoke.mock.calls.map(call => call[0])).not.toContain('stop_opencodex')
    wrapper.unmount()
  })

  it('retains the readable error and warning after a rejected retry', async () => {
    const { runtime, wrapper } = setup(unreadable)
    await flushPromises()
    invoke.mockImplementation(async command => {
      if (command === 'retry_runtime_protection') throw { message: '安装记录尚未完成核验' }
      return unreadable
    })
    await runtime.retryProtection()
    await flushPromises()
    expect(runtime.protection).toEqual(unreadable)
    expect(wrapper.get('[data-testid="runtime-protection"]').text()).toContain('安装记录尚未完成核验')
    expect(wrapper.get('[data-testid="runtime-uninstall"]').attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('ignores a stale read that finishes after successful retry', async () => {
    const runtime = useRuntimeStore()
    const read = deferred<RuntimeProtectionStatus>()
    invoke.mockImplementation(command => command === 'runtime_protection_status' ? read.promise : Promise.resolve(none))
    const loading = runtime.loadProtection()
    await flushPromises()
    await runtime.retryProtection()
    read.resolve(pending)
    await loading
    expect(runtime.protection).toEqual(none)
    expect(runtime.protectionLoading).toBe(false)
    expect(runtime.protectionRetrying).toBe(false)
  })

  it('deduplicates simultaneous reads without opening a mutation', async () => {
    const runtime = useRuntimeStore()
    const read = deferred<RuntimeProtectionStatus>()
    invoke.mockReturnValue(read.promise)
    const loading = runtime.loadProtection()
    await flushPromises()
    await runtime.loadProtection()
    expect(invoke).toHaveBeenCalledTimes(1)
    expect(invoke).toHaveBeenCalledWith('runtime_protection_status')
    read.resolve(none)
    await loading
    expect(runtime.protection).toEqual(none)
  })

  it('continues showing protection when local retry returns a pending association', async () => {
    const runtime = useRuntimeStore()
    invoke.mockResolvedValue(pending)
    await runtime.retryProtection()
    expect(runtime.protection).toEqual(pending)
    expect(runtime.protectionFeedback).toContain('备份继续保留')
  })
})
