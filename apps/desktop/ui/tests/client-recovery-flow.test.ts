import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import ClientRecovery from '@/features/runtime/ClientRecovery.vue'
import { clientRecoveryApi, type ClientRecoveryPreview } from '@/features/runtime/client-recovery'
import { useClientRecoveryStore } from '@/features/runtime/client-recovery-store'
import { useLifecycleStore } from '@/app/lifecycle/store'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'
const preview = (): ClientRecoveryPreview => ({ status: 'pending', phase: 'restore_prepared', revision: 6,
  attemptId: 'f394bd75-355b-4873-83cc-6818efeb547a', originalEvidence: 'verified', planEvidence: 'verified',
  completionEvidence: 'unregistered', phaseCount: 3, canContinue: false, canRollback: false,
  canFinalize: true, confirmationDigest: 'a'.repeat(64) })
const status = (required = false): StatusSnapshot => ({ source: 'live', matrix: { runtime: 'stopped', connection: 'unconfigured', operation: 'idle' },
  facts: { data_root: '/fixture/desktop', opencodex_home: '/fixture/home', client_recovery: required ? 'required' : 'not_required', health: 'unknown',
    runtime_label: 'bundled', protection: null, reboot_safe: true, service_present: null, shim_present: null, version_drift: null },
  pid: null, port: null, can_start: !required, can_stop: false, can_restart: false })
const result = () => ({ attemptId: preview().attemptId!, revision: 7, status: 'restored' as const })
function pending<T>() { let resolve!: (v: T) => void; const promise = new Promise<T>(done => { resolve = done }); return { promise, resolve } }
async function inspect() { const store = useClientRecoveryStore(); store.start(); await flushPromises(); return store }
let views: VueWrapper[] = []
function render() {
  const view = mount(ClientRecovery, { global: { stubs: { UiDialog: { props: ['open'], emits: ['close'], template: '<section v-if="open" role="dialog"><slot/><slot name="actions"/></section>' } } } })
  views.push(view); return { view, button: (name: string) => view.findAll('button').find(b => b.text() === name) }
}
beforeEach(() => {
  setActivePinia(createPinia())
  vi.spyOn(clientRecoveryApi, 'preview').mockResolvedValue(preview())
  vi.spyOn(clientRecoveryApi, 'finalize').mockResolvedValue(result())
  vi.spyOn(clientRecoveryApi, 'continue').mockResolvedValue(result())
  vi.spyOn(clientRecoveryApi, 'status').mockResolvedValue(status())
  const lifecycle = useLifecycleStore(); lifecycle.statusSnapshot = status(true); lifecycle.clientRecoveryRequired = true
})
afterEach(() => { views.forEach(v => v.unmount()); views = []; vi.restoreAllMocks() })
describe('explicit client recovery interaction', () => {
  it('continues an exact interrupted plan only after explicit confirmation, then reconciles fresh status', async () => {
    vi.mocked(clientRecoveryApi.preview).mockResolvedValue({ ...preview(),canFinalize:false,canContinue:true,completionEvidence:'not_recorded' })
    const {view,button}=render();await button('检查客户端恢复')!.trigger('click');await flushPromises()
    expect(view.text()).toContain('恢复到本次启动前的配置')
    expect(button('确认完成收尾')).toBeUndefined();expect(clientRecoveryApi.continue).not.toHaveBeenCalled()
    await button('确认继续恢复')!.trigger('click');await flushPromises()
    expect(clientRecoveryApi.continue).toHaveBeenCalledTimes(1);expect(clientRecoveryApi.finalize).not.toHaveBeenCalled()
    expect(useClientRecoveryStore().ready).toBe(true);expect(useLifecycleStore().clientRecoveryRequired).toBe(false)
  })
  it('retains a single continuation through close/reopen and cannot switch into finalization',async()=>{
    vi.mocked(clientRecoveryApi.preview).mockResolvedValue({...preview(),canFinalize:false,canContinue:true,completionEvidence:'not_recorded'})
    const action=pending<ReturnType<typeof result>>();vi.mocked(clientRecoveryApi.continue).mockReturnValue(action.promise)
    const store=await inspect();const operation=store.apply();await flushPromises();store.close();store.start()
    expect(await store.apply()).toBe(false);expect(store.continuing).toBe(true)
    action.resolve(result());expect(await operation).toBe(true);expect(clientRecoveryApi.continue).toHaveBeenCalledTimes(1)
    expect(clientRecoveryApi.finalize).not.toHaveBeenCalled()
  })
  it('requires independent partial-plan evidence and keeps the gate when continuation is refused',async()=>{
    for (const changed of [{completionEvidence:'unregistered' as const},{planEvidence:'unknown' as const},{confirmationDigest:null},{status:'active' as const},{canFinalize:true}]) {
      vi.mocked(clientRecoveryApi.preview).mockResolvedValue({...preview(),canFinalize:false,canContinue:true,completionEvidence:'not_recorded',...changed})
      const store=await inspect();expect(store.canContinue).toBe(false);store.close()
    }
    vi.mocked(clientRecoveryApi.preview).mockResolvedValue({...preview(),canFinalize:false,canContinue:true,completionEvidence:'not_recorded'})
    vi.mocked(clientRecoveryApi.continue).mockRejectedValue(new Error('现场已变化'))
    const store=await inspect();expect(await store.apply()).toBe(false);expect(store.preview).toBeNull()
    expect(useLifecycleStore().clientRecoveryRequired).toBe(true);expect(await store.apply()).toBe(false)
    expect(clientRecoveryApi.continue).toHaveBeenCalledTimes(1)
  })
  it('checks only on entry and cancellation never submits a mutation', async () => {
    const { button, view } = render(); expect(clientRecoveryApi.preview).not.toHaveBeenCalled()
    await button('检查客户端恢复')!.trigger('click'); await flushPromises()
    expect(view.text()).toContain('客户端配置已恢复'); expect(clientRecoveryApi.finalize).not.toHaveBeenCalled()
    await button('取消')!.trigger('click'); expect(useClientRecoveryStore().open).toBe(false)
    expect(clientRecoveryApi.finalize).not.toHaveBeenCalled()
  })
  it.each(['unknown', 'unsupported', 'active', 'not_required', 'restored'] as const)('never offers finalization for %s', async state => {
    vi.mocked(clientRecoveryApi.preview).mockResolvedValue({ ...preview(), status: state, canFinalize: true })
    const { view, button } = render(); await button('检查客户端恢复')!.trigger('click'); await flushPromises()
    expect(button('确认完成收尾')).toBeUndefined(); expect(await useClientRecoveryStore().apply()).toBe(false)
    expect(clientRecoveryApi.finalize).not.toHaveBeenCalled(); expect(view.text()).not.toContain('继续恢复')
  })
  it('requires bound evidence and a valid digest even if a partial DTO claims canFinalize', async () => {
    for (const p of [{ ...preview(), confirmationDigest: null }, { ...preview(), planEvidence: 'unknown' as const }, { ...preview(), completionEvidence: 'unknown' as const }, { ...preview(), revision: null }]) {
      vi.mocked(clientRecoveryApi.preview).mockResolvedValue(p); const store = await inspect()
      expect(store.canFinalize).toBe(false); expect(await store.apply()).toBe(false); store.close()
    }
    expect(clientRecoveryApi.finalize).not.toHaveBeenCalled()
  })
  it('only a matching response plus fresh scoped status clears the current UI recovery latch', async () => {
    const store = await inspect(); const lifecycle = useLifecycleStore()
    lifecycle.setStatusSnapshot(status()); expect(lifecycle.clientRecoveryRequired).toBe(true)
    expect(await store.apply()).toBe(true)
    expect(clientRecoveryApi.finalize).toHaveBeenCalledWith({ attemptId: preview().attemptId, revision: 6, confirmationDigest: 'a'.repeat(64) })
    expect(lifecycle.clientRecoveryRequired).toBe(false); expect(lifecycle.processActionBlocked('start')).toBe(false)
    expect(store.ready).toBe(true); expect(clientRecoveryApi.finalize).toHaveBeenCalledTimes(1)
  })
  it.each([{ ...result(), attemptId: 'another' }, { ...result(), revision: 9 }, { ...result(), status: 'pending' }])('refuses mismatched response %# and requires fresh inspection', async response => {
    vi.mocked(clientRecoveryApi.finalize).mockResolvedValue(response as ReturnType<typeof result>)
    const store = await inspect(); expect(await store.apply()).toBe(false)
    expect(store.error).toContain('回执'); expect(store.preview).toBeNull(); expect(useLifecycleStore().clientRecoveryRequired).toBe(true)
    expect(await store.apply()).toBe(false); expect(clientRecoveryApi.finalize).toHaveBeenCalledTimes(1)
  })
  it('retains the one in-flight confirmation through close/remount and ignores duplicate clicks', async () => {
    const action = pending<ReturnType<typeof result>>(); vi.mocked(clientRecoveryApi.finalize).mockReturnValue(action.promise)
    const first = render(); await first.button('检查客户端恢复')!.trigger('click'); await flushPromises()
    const store = useClientRecoveryStore(); const operation = store.apply(); await flushPromises()
    expect(store.executing).toBe(true); expect(await store.apply()).toBe(false)
    store.close(); first.view.unmount(); const second = render()
    await second.button('查看恢复进度')!.trigger('click'); await flushPromises()
    expect(second.view.text()).toContain('正在完成恢复收尾'); expect(clientRecoveryApi.preview).toHaveBeenCalledTimes(1)
    action.resolve(result()); expect(await operation).toBe(true); await flushPromises()
    expect(second.view.text()).toContain('服务保持停止，现在可以启动'); expect(clientRecoveryApi.finalize).toHaveBeenCalledTimes(1)
  })
  it('late observations from a closed query cannot resurrect its preview', async () => {
    const query = pending<ClientRecoveryPreview>(); vi.mocked(clientRecoveryApi.preview).mockReturnValueOnce(query.promise)
    const store = useClientRecoveryStore(); store.start(); store.close(); query.resolve(preview()); await flushPromises()
    expect(store.preview).toBeNull(); expect(store.open).toBe(false)
  })
  it('reopening a completed view inspects the current session instead of retaining an old success', async () => {
    const store = await inspect(); await store.apply(); expect(store.ready).toBe(true)
    store.close(); vi.mocked(clientRecoveryApi.preview).mockResolvedValue({ ...preview(), revision: 20, canFinalize: false, confirmationDigest: null })
    store.start(); await flushPromises(); expect(store.outcome).toBeNull(); expect(store.preview?.revision).toBe(20)
    expect(store.canFinalize).toBe(false); expect(clientRecoveryApi.finalize).toHaveBeenCalledTimes(1)
  })
  it('refresh failure keeps the gate, and refresh retries only status without submitting again', async () => {
    vi.mocked(clientRecoveryApi.status).mockResolvedValueOnce(status(true)).mockRejectedValueOnce(new Error('offline')).mockResolvedValueOnce(status())
    const store = await inspect(); expect(await store.apply()).toBe(true)
    expect(store.outcome?.status).toBe('restored'); expect(store.refreshError).toContain('读取失败')
    expect(useLifecycleStore().clientRecoveryRequired).toBe(true); store.close(); store.start(); await flushPromises()
    expect(clientRecoveryApi.preview).toHaveBeenCalledTimes(1); expect(await store.refresh()).toBe(true)
    expect(useLifecycleStore().clientRecoveryRequired).toBe(false); expect(clientRecoveryApi.finalize).toHaveBeenCalledTimes(1)
  })
  it.each(['scope', 'unknown', 'running', 'fixture', 'binding-changed'] as const)('keeps the gate on post-confirmation %s status', async scenario => {
    const after = status()
    if (scenario === 'scope') after.facts.opencodex_home = '/another/home'
    if (scenario === 'unknown') after.facts.client_recovery = 'unknown'
    if (scenario === 'running') { after.matrix.runtime = 'running'; after.pid = '42'; after.port = 1234 }
    if (scenario === 'fixture') after.source = 'fixture'
    vi.mocked(clientRecoveryApi.status).mockResolvedValueOnce(status(true)).mockImplementationOnce(async () => {
      if (scenario === 'binding-changed') useLifecycleStore().statusSnapshot = { ...status(true), facts: { ...status(true).facts, data_root: '/changed' } }
      return after
    })
    const store = await inspect(); expect(await store.apply()).toBe(true); expect(store.ready).toBe(false)
    expect(useLifecycleStore().clientRecoveryRequired).toBe(true); expect(store.refreshError).toContain('仍待核对')
  })
})
