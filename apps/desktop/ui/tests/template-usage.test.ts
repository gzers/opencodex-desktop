import {beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import TemplateUsageDialog from '@/features/models/TemplateUsageDialog.vue'
import {useModelsStore} from '@/features/models/store'
import {modelApi,type TemplateUsage} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{templateUsage:vi.fn()}}))
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
const report=(revisionId='one'):TemplateUsage=>({templateId:'family',revisionId,libraryDigest:'lib',policyDigest:'policy',workspaceDigest:'work',references:[{state:'工作区草稿',location:'A渠道 · 精确型号采用'}],historicalDependency:true,models:[{state:'已提交策略',channelId:'channel',channelLabel:'A渠道',modelId:'raw-match',matched:true,reason:'已匹配',adopted:false,resolutionError:null},{state:'工作区草稿',channelId:'channel',channelLabel:'A渠道',modelId:'raw-explicit',matched:false,reason:'未命中包含规则',adopted:true,resolutionError:null}]})
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();useModelsStore().snapshot={library:{schemaVersion:1,templates:[]},libraryDigest:'lib',policyDigest:'policy',workspaceDigest:'work',workspace:null,policy:{schemaVersion:1,templateRefs:[],channels:[]}};vi.mocked(modelApi.templateUsage).mockResolvedValue(report())})
describe('saved revision matching and usage',()=>{
 it('shows matching and real adoption in separate columns, retaining exact reference and historical reasons',async()=>{
  const wrapper=mount(TemplateUsageDialog,{props:{templateId:'family',revisionId:'one'},global:{stubs:{UiDialog:dialog}}});await flushPromises();expect(modelApi.templateUsage).toHaveBeenCalledWith('family','one','lib');const rows=wrapper.findAll('tbody tr');expect(rows[0]!.text()).toContain('已匹配');expect(rows[0]!.findAll('td')[2]!.text()).toBe('否');expect(rows[1]!.text()).toContain('未命中包含规则');expect(rows[1]!.findAll('td')[2]!.text()).toBe('是');expect(wrapper.text()).toContain('恢复记录依赖');useModelsStore().snapshot!.policyDigest='changed';await flushPromises();expect(wrapper.text()).toContain('上次查询结果');wrapper.unmount()
 })
 it('ignores a late response for another revision and keeps read failures explicit',async()=>{
  let finish!:(value:TemplateUsage)=>void;vi.mocked(modelApi.templateUsage).mockReturnValueOnce(new Promise(resolve=>{finish=resolve})).mockResolvedValueOnce(report('two'))
  const wrapper=mount(TemplateUsageDialog,{props:{templateId:'family',revisionId:'one'},global:{stubs:{UiDialog:dialog}}});await wrapper.setProps({revisionId:'two'});await flushPromises();finish({...report(),references:[{state:'过期',location:'不可展示'}]});await flushPromises();expect(wrapper.text()).not.toContain('不可展示');expect(wrapper.text()).toContain('two')
  vi.mocked(modelApi.templateUsage).mockRejectedValue({message:'精确版本已变化'});await wrapper.findAll('button').find(b=>b.text()==='重新读取')!.trigger('click');await flushPromises();expect(wrapper.text()).toContain('精确版本已变化');expect(wrapper.find('table').exists()).toBe(false);wrapper.unmount()
 })
})
