import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {beforeEach,describe,it,expect,vi} from 'vitest'
import SyncRoute from '@/routes/SyncRoute.vue'
import AppSidebar from '@/components/AppSidebar.vue'
import {useRouteStore} from '@/stores/routes'
import {settingsSections} from '@/navigation'
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn().mockResolvedValue(undefined)}))
const transfer={template:'<div><input aria-label="范围搜索样例" /></div>'}
beforeEach(()=>{window.location.hash='#overview';setActivePinia(createPinia())})
describe('one top-level configuration sync entry',()=>{
 it('redirects both old links and keeps each tab buffer while switching',async()=>{
  const routes=useRouteStore();routes.go('settings',{section:'migration'});expect(routes.current).toBe('sync');expect(routes.syncTab).toBe('file');expect(window.location.hash).toBe('#sync?tab=file')
  const wrapper=mount(SyncRoute,{global:{stubs:{AppTopbar:true,ModelTransfer:transfer,ImportedConnections:true,WebDavConnection:true}}});await flushPromises()
  const search=wrapper.findAll('input')[0]!;await search.setValue('保留的范围搜索')
  await wrapper.findAll('[role=tab]')[1]!.trigger('click');expect(routes.syncTab).toBe('webdav')
  await wrapper.findAll('[role=tab]')[0]!.trigger('click');expect((search.element as HTMLInputElement).value).toBe('保留的范围搜索')
  routes.go('settings',{section:'sync'});expect(routes.current).toBe('sync');expect(routes.syncTab).toBe('webdav')
  expect(settingsSections).not.toContain('sync');expect(settingsSections).not.toContain('migration');wrapper.unmount()
 })
 it('places the shared sync menu between extensions and diagnostics',()=>{
  const wrapper=mount(AppSidebar);const buttons=wrapper.findAll('nav button').map(b=>b.attributes('aria-label'));expect(buttons.indexOf('同步')).toBe(buttons.indexOf('拓展')+1);expect(buttons.indexOf('诊断')).toBe(buttons.indexOf('同步')+1);wrapper.unmount()
 })
})
