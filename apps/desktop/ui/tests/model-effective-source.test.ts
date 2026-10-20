import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {reactive} from 'vue'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import ModelRuleFields from '@/features/models/ModelRuleFields.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {emptyContent,emptyRules,modelApi,type ChannelPolicy,type ModelProjection,type Template} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{project:vi.fn(),discover:vi.fn(),forgetCredential:vi.fn()}}))
let pinia:ReturnType<typeof createPinia>
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);vi.mocked(modelApi.project).mockResolvedValue({rows:[],knownActiveModels:0,exactCatalogCount:null,notices:[]})})
afterEach(()=>pinia._s.forEach(store=>store.$dispose()))
function fixture(){
 const channel=reactive<ChannelPolicy>({id:'channel',label:'测试渠道',templateRefs:[{templateId:'family',revisionId:'version'}],modelRefs:{},connection:{providerId:'custom',protocol:'openai-chat',nodes:[{id:'node',name:'主站',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['raw/a','raw/b']},modelOrder:['raw/a','raw/b'],discovered:['raw/a','raw/b'].map(id=>({id,label:id,rules:emptyRules()})),defaults:emptyRules(),overrides:{},familyOverrides:{}})
 const template:Template={id:'family',label:'自建族',source:'user',defaultRevision:'version',revisions:[{id:'version',savedLabel:'自建族',createdAt:'2026-10-20',digest:'fixture',content:emptyContent(),baseRevision:null,changes:[]}]}
 const rows:ModelProjection[]=channel.discovered.map(m=>({channelId:'channel',providerId:'custom',modelId:m.id,label:m.id,enabled:true,selected:true,discovered:true,templateRef:m.id==='raw/a'?{templateId:'family',revisionId:'version'}:null,rules:{...emptyRules(),fast:{mode:'custom',value:false}},source:{fast:'单模型覆盖'}}))
 return {channel,template,rows}
}
describe('model family and effective sources',()=>{
 it('accepts alias on blur through the common validator while raw identity and other model settings remain stable',async()=>{
  const {channel,template,rows}=fixture();const wrapper=mount(ChannelEditor,{props:{channel,templates:[template],projection:rows},global:{stubs:{NodePicker:true,ModelRuleFields:true,UiSelectField:true}}})
  const input=wrapper.get('input[aria-label="raw/a路由别名"]');await input.setValue('short-a');expect(channel.naming?.['raw/a']).toBeUndefined();await input.trigger('blur');await flushPromises()
  expect(channel.naming?.['raw/a']).toEqual({alias:'short-a'});expect(channel.discovered[0]!.id).toBe('raw/a');expect(channel.selection).toEqual({mode:'whitelist',models:['raw/a','raw/b']});expect(channel.naming?.['raw/b']).toBeUndefined();expect(modelApi.project).toHaveBeenCalled();wrapper.unmount()
 })
 it('uses the resolved adopted family for filtering and keeps rows while a new projection is pending',async()=>{
  const {channel,template,rows}=fixture();const wrapper=mount(ChannelEditor,{props:{channel,templates:[template],projection:rows},global:{stubs:{NodePicker:true,ModelRuleFields:true,UiSelectField:true}}})
  const selector=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='渠道内型号族筛选')!
  selector.vm.$emit('update:modelValue','family');await wrapper.vm.$nextTick();expect(wrapper.findAll('tr[data-model]')).toHaveLength(1)
  expect(wrapper.get('button[aria-label="raw/a下移"]').attributes('disabled')).toBeDefined()
  const before=wrapper.get('tr[data-model] input').element
  await wrapper.setProps({projectionBusy:true});expect(wrapper.get('tr[data-model] input').element).toBe(before)
  await wrapper.setProps({projectionBusy:false,projectionError:'conflict'});expect(wrapper.findAll('tr[data-model]')).toHaveLength(1);expect(wrapper.text()).toContain('有效值暂不可确定')
  const familyFields=wrapper.findAllComponents(ModelRuleFields)[1]!
  familyFields.vm.$emit('update:modelValue',{...emptyRules(),fast:{mode:'custom',value:false}});await wrapper.vm.$nextTick();expect(channel.familyOverrides.family?.fast).toEqual({mode:'custom',value:false})
  wrapper.unmount()
 })
 it('shows inheritance, unknown and explicit false against only server-provided effective sources',()=>{
  const {rows}=fixture();const rules={...emptyRules(),contextWindow:{mode:'custom' as const,value:4096}}
  const wrapper=mount(ModelRuleFields,{props:{modelValue:rules,effective:rows[0]},global:{stubs:{UiSelectField:true}}})
  const table=wrapper.get('table');expect(table.text()).toContain('当前草稿有效值与来源');expect(table.text()).toContain('未知（无声明或覆盖）');expect(table.text()).toContain('不支持');expect(table.text()).toContain('单模型覆盖');expect(table.text()).toContain('4096');wrapper.unmount()
 })
 it('adds a manual exact ID only after confirmation and keeps its input DOM stable',async()=>{
  const {channel,template,rows}=fixture();const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
  const wrapper=mount(ChannelEditor,{props:{channel,templates:[template],projection:rows},global:{stubs:{NodePicker:true,ModelRuleFields:true,UiSelectField:true,UiDialog:dialog}}})
  await wrapper.findAll('button').find(b=>b.text()==='手动添加ID')!.trigger('click');const input=wrapper.findAll('label').find(l=>l.text()==='原始模型ID')!.get('input');const element=input.element
  await input.setValue('new/raw');expect(input.element).toBe(element);expect(channel.discovered).toHaveLength(2)
  await wrapper.findAll('button').find(b=>b.text()==='添加型号')!.trigger('click');expect(channel.discovered.at(-1)?.id).toBe('new/raw');expect(channel.selection).toEqual({mode:'whitelist',models:['raw/a','raw/b']});wrapper.unmount()
 })
})
