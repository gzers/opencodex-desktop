import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import {defineComponent,h} from 'vue'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import {emptyRules,type ChannelPolicy} from '@/features/models/api'
import {channelRenameError,allocateChannelProvider,renameChannelLabel,channelNameErrorV2} from '@/features/models/channelRename'
import {workspaceBackend,workspaceId as id,copyWorkspace} from './helpers/modelWorkspaceV2'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
function channel(id='stable',provider='Krill-1'):ChannelPolicy{return {id,label:provider,templateRefs:[],modelRefs:{},connection:{providerId:provider,protocol:'openai-chat',authMode:'key',nodes:[{id:'stable-node',name:'主站',url:'https://fixture.invalid/v1'}],activeNodeId:'stable-node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:['vendor/a:8b'],discovered:[{id:'vendor/a:8b',label:'vendor/a:8b',rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{}}}
let backend:ReturnType<typeof workspaceBackend>,store:Awaited<ReturnType<ReturnType<typeof workspaceBackend>['load']>>,wrappers:VueWrapper[]=[]
function render(){const host=mount(defineComponent({setup:()=>()=>h(ChannelEditor,{channel:store.channels[0]!,channels:store.channels,dirty:store.channelDirty(id('a')),onIntent:store.queueChannel})}),{global:{stubs:{ChannelModelTable:true,NodePicker:true,ModelRuleFields:true,UiSelectField:true,UiDialog:true}}});wrappers.push(host);return host.getComponent(ChannelEditor)}
beforeEach(async()=>{backend=workspaceBackend(ipc);const fixture=copyWorkspace(backend.current);fixture.savedPolicy.channels[0]!.label='Krill-1';fixture.savedPolicy.channels[1]!.label='Krill-10';backend.current=fixture;store=await backend.load()})
afterEach(()=>{wrappers.forEach(w=>w.unmount());wrappers=[]})
describe('AC02 direct channel rename through v2 intent',()=>{
 it('edits the name while retaining backend-owned runtime key, raw identity, source and credential origin before Save',async()=>{
  const w=render();await flushPromises();const before=copyWorkspace(store.channels[0]!);await w.get('input[maxlength="80"]').setValue('Krill');await flushPromises()
  expect(store.channels[0]!.label).toBe('Krill');expect(store.channels[0]!.connection.providerId).toBe(before.connection.providerId);expect(w.find('input[aria-label="运行渠道标识"]').exists()).toBe(false);expect(w.text()).toContain('保存后需重启 Codex 生效');expect(w.text()).toContain('运行键及引用')
  expect(ipc.mock.calls.some(c=>c[0]==='model_v2_credential_view')).toBe(false);expect(store.keyBuffer(id('a'))).toBeNull();expect(store.channels[0]!.id).toBe(before.id);expect(store.channels[0]!.connection).toEqual(before.connection);expect(store.channels[0]!.models).toEqual(before.models);expect(store.file!.savedPolicy.channels[0]!.label).toBe('Krill-1')
  await w.get('button[aria-label="保存渠道草稿"]').trigger('click');expect(w.emitted('save')).toEqual([[]]);expect(ipc.mock.calls.some(c=>c[0]==='model_v2_file_save')).toBe(false);await store.saveChannels(id('a'));expect(ipc.mock.calls.find(c=>c[0]==='model_v2_file_save')![1].request.channelIntents).toEqual([{kind:'rename',channelId:id('a'),name:'Krill'}]);expect(store.file!.savedPolicy.channels[0]!.models).toEqual(before.models)
 })
 it.each(['渠道名称','Has Space','openai','bad/id','_bad','bad-','policy','constructor'])('accepts unique display name %s and delegates runtime allocation to the backend',async name=>{
  const w=render();await flushPromises();const before=copyWorkspace(store.channels[0]!);await w.get('input[maxlength="80"]').setValue(name);expect(channelNameErrorV2(name,id('a'),store.channels)).toBe('');expect(w.find('[role="alert"]').exists()).toBe(false);expect(store.channels[0]!.connection.providerId).toBe(before.connection.providerId);expect(store.channels[0]!.models).toEqual(before.models);expect(w.find('input[aria-label="运行渠道标识"]').exists()).toBe(false);await w.get('button[aria-label="保存渠道草稿"]').trigger('click');expect(w.emitted('save')).toEqual([[]])
 })
 it('rejects duplicate names, keeps saved provider slots occupied, and protects original managed sources',()=>{
  const saved=[channel(),channel('other','other-provider')],c=channel();c.label='other-provider';c.connection.providerId='free';expect(channelRenameError(c,saved)).toContain('名称已存在');c.label='free';c.connection.providerId=allocateChannelProvider('other-provider',c.id,saved);expect(c.connection.providerId).not.toBe('other-provider');expect(channelRenameError(c,saved,[c])).toBe('');const protectedSaved=channel();protectedSaved.connection.authMode='oauth';c.connection.providerId='free';expect(channelRenameError(c,[protectedSaved])).toContain('不能直接改名')
 })
 it('retains explicit rename intent when returning, without changing the saved file',async()=>{
  const saved=channel(),draft=channel();draft.label='别名';draft.connection.providerId='Explicit';renameChannelLabel(draft,'其它名称',[saved]);expect(draft.connection.providerId).toBe('channel-stable');const w=render();await flushPromises();await w.get('input[maxlength="80"]').setValue('Krill');await w.get('button[aria-label="返回渠道列表"]').trigger('click');expect(w.emitted('save')).toBeUndefined();expect(w.emitted('close')).toEqual([[]]);expect(store.file!.savedPolicy.channels[0]!.label).toBe('Krill-1');expect(store.channels[0]!.label).toBe('Krill');expect(ipc.mock.calls.some(c=>c[0]==='model_v2_file_save')).toBe(false)
 })
 it.each(['','Krill-10','badname'])('blocks invalid or duplicate v2 name %j before a Save event',async name=>{const w=render();await flushPromises();await w.get('input[maxlength="80"]').setValue(name);expect(w.get('[role="alert"]').text()).not.toBe('');expect(w.get('button[aria-label="保存渠道草稿"]').attributes('disabled')).toBeDefined();await w.get('button[aria-label="保存渠道草稿"]').trigger('click');expect(w.emitted('save')).toBeUndefined()})
})
