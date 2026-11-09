import { installNativeDialogStub } from './helpers/nativeDialog'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useDataRootStore } from '@/features/data-root/store'
import { useRuntimeStore } from '@/features/runtime/store'
import { useRouteStore } from '@/stores/routes'
import type { DataRootConfig, PathsSnapshotV2 } from '@/features/data-root/api'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
Object.defineProperty(window, 'matchMedia', { writable: true, value: (media: string) => ({ matches: false, media, addEventListener() {}, removeEventListener() {} }) })
const current: DataRootConfig = { aggregateRoot: '/fixtures/source-U', activeDataRoot: '/fixtures/source-D', currentDataRoot: '/fixtures/source-D',
  opencodexHomeMode: 'external', opencodexHome: '/fixtures/external-home', currentOpencodexHome: '/fixtures/external-home', runtimeActive: true }
const fact = (path: string) => ({ path, status: 'valid' as const })
const paths: PathsSnapshotV2 = { schemaVersion: 1, aggregateRoot: fact('/fixtures/source-U'), desktopRoot: fact('/fixtures/source-D'),
  opencodexHome: fact('/fixtures/external-home'), configPath: fact('/fixtures/external-config'), installationRoot: fact('/fixtures/actual-I'),
  runtimePrefix: fact('/fixtures/actual-prefix'), runtimeEntry: fact('/fixtures/entry'), managerExecutable: fact('/fixtures/manager'),
  program: { packageVersion: '23', manifestVersion: '23', sourceVersion: '23', consistency: 'consistent' },
  currentAggregateRoot: '/fixtures/source-U', pendingRestart: false }
const mounted: Array<ReturnType<typeof mount>> = []
const switches = () => invoke.mock.calls.filter(call => call[0] === 'switch_data_root')
const writes = () => invoke.mock.calls.filter(call => ['switch_data_root', 'initialize_data_root', 'set_opencodex_home_config'].includes(call[0]))
function read(command: string) {
  if (command === 'get_data_root_config') return { ...current }
  if (command === 'model_paths_v2_snapshot') return structuredClone(paths)
  if (command === 'validate_data_root_structure') return 'valid'
  if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
  return null
}
async function setup() {
  const pinia = createPinia(); setActivePinia(pinia)
  useRuntimeStore().protection = { state: 'none', version: null, backupId: null }
  useRouteStore().go('settings', { section: 'installation' })
  const page = mount(SettingsRoute, { global: { plugins: [pinia] }, attachTo: document.body }); mounted.push(page)
  await vi.dynamicImportSettled(); await flushPromises()
  return { page, store: useDataRootStore() }
}
// Replaces the obsolete copy wizard assertions, preserving cancel, side-effect and source-protection checks.
describe('data root reference-only selection and restart boundary (no migration)', () => {
  beforeEach(() => { installNativeDialogStub(); invoke.mockReset(); invoke.mockImplementation(async command => read(command)) })
  afterEach(() => { mounted.splice(0).forEach(page => page.unmount()); document.body.innerHTML = '' })

  it('uses backend identity for aliases and removes directory selection from the default UI',async()=>{
    invoke.mockImplementation(async command=>command==='get_data_root_config'?{...current,activeDataRoot:'/tmp/source-D',currentDataRoot:'/private/tmp/source-D'}:read(command))
    const{page,store}=await setup();expect(page.find('[data-testid="paths-pending"]').exists()).toBe(false)
    expect(page.find('[data-testid="paths-select"]').exists()).toBe(false);expect(page.find('[data-testid="data-root-migrate"]').exists()).toBe(false)
    expect(await store.switchTo('/fixtures/target-U',true)).toBe(false);expect(writes()).toEqual([])
  })
  it('default navigation and complete-path viewing do not initiate directory writes',async()=>{
    const{page}=await setup();await page.get('[data-testid="paths-details"]').trigger('click');await flushPromises()
    expect(writes()).toEqual([]);expect(page.text()).not.toContain('选择新版目录')
    expect(invoke.mock.calls.some(c=>c[0]==='validate_data_root_structure')).toBe(false)
  })
  it('retains explicit store reference-only contract and blocks concurrent/pending writes',async()=>{
    let finish!:(value:unknown)=>void;invoke.mockImplementation(command=>command==='switch_data_root'?new Promise(resolve=>{finish=resolve}):Promise.resolve(read(command)))
    const{page,store}=await setup();const pending=store.switchTo('/fixtures/confirmed-U');await flushPromises()
    expect(switches()).toEqual([['switch_data_root',{request:{targetPath:'/fixtures/confirmed-U',mode:'reference_only'}}]])
    expect(await store.switchTo('/second/U')).toBe(false)
    finish({status:'restart_required',blocked:null,reconciliationRequired:false,config:{...current,aggregateRoot:'/fixtures/confirmed-U',activeDataRoot:'/fixtures/confirmed-D',opencodexHome:'/target explicit HOME',runtimeActive:false}})
    await pending;await flushPromises()
    const view=page.get('[data-testid="paths-pending"]');for(const value of ['/fixtures/source-U','/fixtures/confirmed-U','/fixtures/source-D','/fixtures/confirmed-D','/fixtures/external-home','/target explicit HOME'])expect(view.text()).toContain(value)
    expect(await store.save('/another/U')).toBe(false);expect(await store.saveHome('inside')).toBe(false);expect(writes()).toHaveLength(1)
  })
  it('retains uncertain reference results as pending reconciliation',async()=>{
    invoke.mockImplementation(async command=>command==='switch_data_root'?{status:'restart_required',blocked:null,reconciliationRequired:true,config:{...current,aggregateRoot:'/fixtures/intended-U',runtimeActive:false}}:read(command))
    const{page,store}=await setup();await store.switchTo('/fixtures/intended-U');await flushPromises()
    expect(page.get('[data-testid="paths-pending"] [role="alert"]').text()).toContain('仅为预期目标');expect(store.reconciliationRequired).toBe(true)
    expect(await store.switchTo('/other/U')).toBe(false)
  })
  it('preserves source facts after explicit store rejection without fake-copy claims',async()=>{
    invoke.mockImplementation(async command=>{if(command==='switch_data_root')throw new Error('pointer denied');return read(command)})
    const{store}=await setup();expect(await store.switchTo('/fixtures/rejected-U')).toBe(false)
    expect(store.config).toEqual(current);expect(store.paths).toEqual(paths);expect(store.error).toContain('原目录内容未由本操作复制或删除')
    expect(switches()).toEqual([['switch_data_root',{request:{targetPath:'/fixtures/rejected-U',mode:'reference_only'}}]])
  })
  it('latches restart after a lost response to an explicit store reference change',async()=>{
    let changed=false;invoke.mockImplementation(async command=>{if(command==='switch_data_root'){changed=true;throw new Error('response lost')}if(changed&&command==='get_data_root_config')return{...current,aggregateRoot:'/saved/U',runtimeActive:false};return read(command)})
    const{store}=await setup();expect(await store.switchTo('/saved/U')).toBe(false);expect(store.pendingRestart).toBe(true);expect(await store.switchTo('/next/U')).toBe(false);expect(switches()).toHaveLength(1)
  })
})
