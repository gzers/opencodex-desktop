import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import NewChannelDialog from '@/features/models/NewChannelDialog.vue'
import {workspaceBackend,workspaceId as id} from './helpers/modelWorkspaceV2'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
let backend:ReturnType<typeof workspaceBackend>,s:Awaited<ReturnType<ReturnType<typeof workspaceBackend>['load']>>,wrappers:VueWrapper[]=[]
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions" :close="() => $emit(&quot;close&quot;)"/></section>'}
const calls=(name:string)=>ipc.mock.calls.filter(c=>c[0]===name)
const button=(w:Pick<VueWrapper,'findAll'>,text:string)=>w.findAll('button').find(b=>b.text()===text)!
function render(){const w=mount(ChannelEditor,{props:{channel:s.channels[0]!,effective:s.file!.effective},global:{stubs:{ChannelModelTable:true,UiSelectField:true,UiDialog:dialog}}});wrappers.push(w);return w}
beforeEach(async()=>{backend=workspaceBackend(ipc,['a']);s=await backend.load()})
afterEach(()=>{wrappers.forEach(w=>w.unmount());wrappers=[]})
describe('editing an existing channel Key',()=>{
 it('rebases an untouched B Key intention after this session saves A, so B can be saved explicitly afterwards',async()=>{
  backend=workspaceBackend(ipc,['a','b']);s=await backend.load();s.setKeyBuffer(id('b'),{action:'replace',value:'fixture-pending-b'});s.queueChannel({kind:'rename',channelId:id('a'),name:'A saved'});await s.saveChannels(id('a'));expect(s.fileError).toBe('');expect(backend.keys.get(id('b'))).toBe('fixture-original-b');expect(s.keyBuffer(id('b'))).toMatchObject({value:'fixture-pending-b'});await s.saveChannels(id('b'));expect(s.fileError).toBe('');expect(backend.keys.get(id('b'))).toBe('fixture-pending-b');expect(calls('model_v2_stage_credential')).toHaveLength(1);expect(calls('model_v2_stage_credential')[0]![1].request.channelId).toBe(id('b'))
 })
 it('does not rebind a pending replacement to changed node targets even if a source digest is unchanged',async()=>{
  s.setKeyBuffer(id('a'),{action:'replace',value:'fixture-original-target-key'});const next={...s.file!,draft:{...s.file!.draft!,policy:{...s.file!.draft!.policy,channels:s.file!.draft!.policy.channels.map(c=>({...c,connection:{...c.connection,nodes:c.connection.nodes.map((n,index)=>index===0?{...n,url:'https://changed.example.test'}:n)}}))}}};s.file=next;await s.saveChannels(id('a'));expect(calls('model_v2_stage_credential')).toHaveLength(0);expect(s.keyBuffer(id('a'))).toMatchObject({value:'fixture-original-target-key'});expect(s.fileError).toContain('来源基准已变化')
 })
 it('reads only on an explicit click, reveals and hides the stored value without changing configuration',async()=>{
  const w=render();await flushPromises();const key=w.get<HTMLInputElement>('input[aria-label="渠道替换API Key"]')
  expect(key.element.value).toBe('');expect(key.attributes('type')).toBe('password');expect(calls('model_v2_credential_view')).toHaveLength(0);expect(s.channelDirty(id('a'))).toBe(false)
  await button(w,'查看已保存 Key').trigger('click');await flushPromises();expect(w.get<HTMLInputElement>('input[aria-label="已保存渠道Key"]').element.value).toBe('fixture-original-a');expect(calls('model_v2_credential_view')).toHaveLength(1)
  await button(w,'隐藏 Key').trigger('click');expect(w.find('input[aria-label="已保存渠道Key"]').exists()).toBe(false);expect(s.keyBuffer(id('a'))).toBeNull();expect(s.channelDirty(id('a'))).toBe(false)
 })
 it('records replacing or clearing only the Key as a saveable edit without fetching a model list',async()=>{
  const w=render();await flushPromises();const key=w.get('input[aria-label="渠道替换API Key"]');await key.setValue('fixture-replacement-key')
  expect(s.channelDirty(id('a'))).toBe(true);expect(s.keyBuffer(id('a'))).toMatchObject({action:'replace',value:'fixture-replacement-key'});expect(calls('model_v2_discover')).toHaveLength(0)
  await key.setValue('');expect(s.keyBuffer(id('a'))).toBeNull();expect(s.channelDirty(id('a'))).toBe(false)
  await button(w,'明确清空 Key').trigger('click');expect(s.keyBuffer(id('a'))).toMatchObject({action:'clear'});await s.saveChannels(id('a'));expect(s.fileError).toBe('');expect(calls('model_v2_stage_credential')[0]![1].request).toEqual({expected:expect.any(Object),channelId:id('a'),action:'clear'});expect(s.channels[0]!.connection.authMode).toBe('key');expect(backend.keys.get(id('a'))).toBe('')
 })
 it('retains entered replacement and its save intent when discovery in the connection editor fails',async()=>{
  ipc.mockImplementation(async(command,args)=>{if(command==='model_v2_discover')throw {code:'offline'};return backend.handler(command,args)})
  const w=render();await flushPromises();await w.get('input[aria-label="渠道替换API Key"]').setValue('fixture-replacement-key');await button(w,'编辑渠道连接').trigger('click');await flushPromises();const form=w.getComponent(NewChannelDialog)
  await form.get('input[aria-label="API Key"]').setValue('fixture-connection-key');await button(form,'下一步').trigger('click');form.findAllComponents({name:'UiSelectField'}).find(c=>c.props('label')==='本次清单获取方式')!.vm.$emit('update:modelValue','compatible_models');await flushPromises();await button(form,'获取模型列表').trigger('click');await flushPromises()
  expect(w.get<HTMLInputElement>('input[aria-label="渠道替换API Key"]').element.value).toBe('fixture-replacement-key');expect(s.keyBuffer(id('a'))).toMatchObject({action:'replace',value:'fixture-replacement-key'});expect(form.text()).toContain('操作未完成');expect(calls('model_v2_discover')[0]![1].request).toMatchObject({url:'https://a.example.test/v1',apiKey:'fixture-connection-key',discoveryKind:'openai_list'});expect(calls('model_v2_file_save')).toHaveLength(0)
 })
 it('uses the newly saved Key as the baseline for a second edit without leaving the editor',async()=>{
  const w=render();await flushPromises();await w.get('input[aria-label="渠道替换API Key"]').setValue('fixture-second-key');await s.saveChannels(id('a'));await flushPromises();expect(backend.keys.get(id('a'))).toBe('fixture-second-key');expect(s.keyBuffer(id('a'))).toBeNull();expect(w.get<HTMLInputElement>('input[aria-label="渠道替换API Key"]').element.value).toBe('')
  await w.get('input[aria-label="渠道替换API Key"]').setValue('fixture-original-a');expect(s.keyBuffer(id('a'))).toMatchObject({action:'replace',value:'fixture-original-a'});await s.saveChannels(id('a'));expect(backend.keys.get(id('a'))).toBe('fixture-original-a')
 })
 it('never reuses a viewed saved Key for a changed URL: connection discovery requires a newly entered Key',async()=>{
  const w=render();await flushPromises();await button(w,'查看已保存 Key').trigger('click');await flushPromises();await button(w,'编辑渠道连接').trigger('click');await flushPromises();const form=w.getComponent(NewChannelDialog);await form.get('input[aria-label="主站编辑地址"]').setValue('https://other.example.test');await button(form,'下一步').trigger('click');await flushPromises();expect(form.text()).toContain('请输入有效 API Key');expect(calls('model_v2_discover')).toHaveLength(0);expect(form.get<HTMLInputElement>('input[aria-label="API Key"]').element.value).toBe('');expect(s.keyBuffer(id('a'))).toBeNull();expect(s.channels[0]!.connection.nodes[0]!.url).toBe('https://a.example.test/v1')
 })
})
