import { beforeEach,describe,expect,it,vi } from 'vitest'
import { mount,flushPromises } from '@vue/test-utils'
import ImportedConnections from '@/features/sync/ImportedConnections.vue'
import {listImportedConnections,activateImportedConnection} from '@/features/sync/api'
vi.mock('@/features/sync/api',()=>({listImportedConnections:vi.fn(),activateImportedConnection:vi.fn()}))
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
const fixture=()=>({baseline:'frozen-active',items:[{id:'candidate',endpoint:{id:'source-default',url:'https://fixture.example.test',remotePath:'portable',username:'fixture',conflictPolicy:'ask'},hasPassword:true}]})
beforeEach(()=>{vi.clearAllMocks();vi.mocked(listImportedConnections).mockResolvedValue(fixture())})
describe('inactive imported connections',()=>{
 it('only activates after showing the exact server and confirming against the captured baseline',async()=>{
  vi.mocked(activateImportedConnection).mockResolvedValue(undefined);const wrapper=mount(ImportedConnections,{global:{stubs:{UiDialog:dialog}}});await flushPromises();expect(activateImportedConnection).not.toHaveBeenCalled()
  await wrapper.findAll('button').find(b=>b.text()==='启用此连接')!.trigger('click');expect(wrapper.text()).toContain('https://fixture.example.test / portable');expect(activateImportedConnection).not.toHaveBeenCalled()
  await wrapper.findAll('button').find(b=>b.text()==='确认启用')!.trigger('click');await flushPromises();expect(activateImportedConnection).toHaveBeenCalledExactlyOnceWith('candidate','frozen-active');expect(wrapper.emitted('activated')).toHaveLength(1);wrapper.unmount()
 })
 it('does not activate a connection without a saved password or silently retry a rejected baseline',async()=>{
  const data=fixture();data.items[0]!.hasPassword=false;vi.mocked(listImportedConnections).mockResolvedValueOnce(data);const missing=mount(ImportedConnections,{global:{stubs:{UiDialog:dialog}}});await flushPromises();expect(missing.findAll('button').find(b=>b.text()==='启用此连接')!.attributes('disabled')).toBeDefined();missing.unmount()
  vi.mocked(activateImportedConnection).mockRejectedValue({message:'活动连接已变化，请重新比较'});const wrapper=mount(ImportedConnections,{global:{stubs:{UiDialog:dialog}}});await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='启用此连接')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='确认启用')!.trigger('click');await flushPromises();expect(wrapper.text()).toContain('活动连接已变化');expect(activateImportedConnection).toHaveBeenCalledTimes(1);expect(wrapper.emitted('activated')).toBeUndefined();wrapper.unmount()
 })
})
