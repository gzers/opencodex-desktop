// 0.2.24 uses admitted version directories and preserves config by default.
// Legacy discovered/global-npm removal is no longer a settings entry point.
import {installNativeDialogStub} from './helpers/nativeDialog'
import {createPinia,setActivePinia} from 'pinia'
import {DOMWrapper,flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import {useRuntimeStore} from '@/features/runtime/store'
import {useRouteStore} from '@/stores/routes'
const {invoke}=vi.hoisted(()=>({invoke:vi.fn()}))
vi.mock('@tauri-apps/api/core',()=>({invoke}))
vi.mock('@tauri-apps/api/event',()=>({listen:async()=>()=>{}}))
Object.defineProperty(window,'matchMedia',{writable:true,value:(query:string)=>({matches:false,media:query,addEventListener:()=>{},removeEventListener:()=>{}})})
const source=()=>({kind:'managed',path:'/fixture/program/runtime/bin/ocx-runtime',version:'2.99.0',resolvedAt:null,insideDataRoot:false,managedEntry:'/fixture/program/runtime/bin/ocx-runtime',managedPrefix:'/fixture/program/runtime/versions/2.99.0/opencodex',defaultPrefix:'/fixture/program/runtime/versions/2.99.0/opencodex',explicitPath:null,history:[]})
const review=()=>({id:'registered-review',resume:false,commandPath:'/fixture/.local/bin/ocx',shellPaths:['/fixture/.zshrc'],confirmationDigest:'review-hash',removal:{operationId:'registered-operation',versions:['2.99.0'],programRoot:'/fixture/program',preservedHome:'/fixture/home',files:12,bytes:512,confirmationDigest:'plan-hash'}})
let view:VueWrapper
async function render(noSource=false){
 const pinia=createPinia();setActivePinia(pinia);useRuntimeStore().protection={state:'none',version:null,backupId:null}
 invoke.mockImplementation(async(command:string)=>{
  if(command==='runtime_source')return noSource?null:source()
  if(command==='runtime_protection_status')return {state:'none',version:null,backupId:null}
  if(command==='runtime_removal_preview')return review()
  if(command==='runtime_removal_apply')return {operationId:'registered-operation',removedVersions:['2.99.0'],configurationPreserved:true}
  return null
 })
 useRouteStore().go('settings',{section:'installation'})
 view=mount(SettingsRoute,{global:{plugins:[pinia]},attachTo:document.body});await flushPromises();return view
}
const dialog=()=>new DOMWrapper(document.body).get('dialog[open]')
const action=(text:string)=>dialog().findAll('button').find(b=>b.text()===text)!
beforeEach(()=>{installNativeDialogStub();invoke.mockReset()})
afterEach(()=>{view?.unmount();document.body.innerHTML='';vi.restoreAllMocks()})
describe('versioned uninstall in installation settings',()=>{
 it.each([false,true])('keeps removal reachable for pending/unresolved source=%s and observes only on click',async noSource=>{
  await render(noSource)
  const trigger=view.get('[data-testid="runtime-program-removal"]');expect(trigger.attributes('disabled')).toBeUndefined()
  expect(invoke.mock.calls.some(c=>c[0]==='runtime_removal_preview')).toBe(false)
  await trigger.trigger('click');await flushPromises()
  expect(dialog().text()).toContain('配置与备份');expect(dialog().text()).toContain('保留');expect(dialog().text()).toContain('/fixture/.zshrc')
  expect(dialog().text()).not.toContain('清空 OPENCODEX_HOME');expect(dialog().text()).not.toContain('完整卸载（推荐）')
  await action('取消').trigger('click');await flushPromises()
  expect(invoke.mock.calls.some(c=>c[0]==='runtime_removal_apply'||c[0]==='uninstall_runtime'||c[0]==='plan_runtime_uninstall')).toBe(false)
 })
 it('executes only after explicit confirmation and sends the exact one-use review',async()=>{
  await render();await view.get('[data-testid="runtime-program-removal"]').trigger('click');await flushPromises()
  expect(invoke.mock.calls.some(c=>c[0]==='runtime_removal_apply')).toBe(false)
  await action('确认卸载程序').trigger('click');await flushPromises()
  expect(invoke.mock.calls.filter(c=>c[0]==='runtime_removal_apply')).toEqual([['runtime_removal_apply',{request:{id:'registered-review',confirmationDigest:'review-hash'}}]])
  expect(dialog().text()).toContain('用户配置和备份已保留')
 })
 it('failed source/inventory proof leaves no confirm action or fallback to old uninstall',async()=>{
  await render();const existing=invoke.getMockImplementation()!
  invoke.mockImplementation((command,...args)=>command==='runtime_removal_preview'?Promise.reject({message:'版本目录出现外部文件，未卸载'}):existing(command,...args))
  await view.get('[data-testid="runtime-program-removal"]').trigger('click');await flushPromises()
  expect(dialog().get('[role="alert"]').text()).toContain('外部文件');expect(action('确认卸载程序')).toBeUndefined()
  expect(invoke.mock.calls.some(c=>c[0]==='runtime_removal_apply'||c[0]==='uninstall_runtime')).toBe(false)
 })
 it('blocks a new removal while installation or update protection is active',async()=>{
  await render();useRuntimeStore().protection={state:'pending',version:'3.0.0',backupId:'protected'};await flushPromises()
  expect(view.get('[data-testid="runtime-program-removal"]').attributes('disabled')).toBeDefined()
  expect(invoke.mock.calls.some(c=>c[0]==='runtime_removal_preview')).toBe(false)
 })
})
