import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import {useModelsStore} from '@/features/models/store'
import {emptyRules,modelApi,type ChannelPolicy} from '@/features/models/api'
import {channelRenameError,channelRenameIntent,renameChannelLabel,renameChannelProvider} from '@/features/models/channelRename'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{credential:vi.fn(),discover:vi.fn(),measureNodes:vi.fn(),stopNodeMeasurement:vi.fn(),forgetCredential:vi.fn()}}))
function channel(id='stable',provider='Krill-1'):ChannelPolicy{return {id,label:provider,templateRefs:[],modelRefs:{},connection:{providerId:provider,protocol:'openai-chat',authMode:'key',nodes:[{id:'stable-node',name:'主站',url:'https://fixture.invalid/v1'}],activeNodeId:'stable-node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:['vendor/a:8b'],discovered:[{id:'vendor/a:8b',label:'vendor/a:8b',rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{}}}
let wrappers:VueWrapper[]=[]
function render(){const store=useModelsStore();store.acceptTransfer({library:{schemaVersion:1,templates:[]},libraryDigest:'library',policyDigest:'file',policy:{schemaVersion:1,templateRefs:[],channels:[channel(),channel('neighbor','Krill-10')]},workspace:null,workspaceDigest:''});const wrapper=mount(ChannelEditor,{props:{channel:store.channelDraft!.channels[0]!,templates:[],existing:true,dirty:true},global:{stubs:{ChannelModelTable:true,NodePicker:true,ModelRuleFields:true,UiSelectField:true,UiDialog:true}}});wrappers.push(wrapper);return wrapper}
beforeEach(()=>{setActivePinia(createPinia());vi.resetAllMocks();vi.mocked(modelApi.credential).mockResolvedValue({configured:true,viewable:true,value:'fixture-saved',source:'配置文件'})})
afterEach(()=>{for(const w of wrappers)w.unmount();wrappers=[]})
describe('AC02 direct provider rename',()=>{
 it('updates name and provider together, previews full original model ID and retains credential origin before Save',async()=>{
  const w=render();await flushPromises();const s=useModelsStore(),before=JSON.parse(JSON.stringify(s.channelDraft!.channels[0]!));
  await w.get('input[maxlength="80"]').setValue('Krill');await flushPromises()
  expect(s.channelDraft!.channels[0]!.connection.providerId).toBe('Krill')
  expect(w.get('[data-channel-rename-preview]').text()).toContain('Krill-1 → Krill')
  expect(w.get('[data-channel-rename-preview]').text()).toContain('Krill/vendor/a:8b')
  expect(w.text()).toContain('保存后需重启 Codex 生效')
  expect(modelApi.credential).toHaveBeenCalledTimes(1);expect(modelApi.credential).toHaveBeenCalledWith('Krill-1');expect(s.credentialEdits).toEqual({})
  expect(s.channelDraft!.channels[0]!.id).toBe(before.id);expect(s.channelDraft!.channels[0]!.connection.nodes).toEqual(before.connection.nodes);expect(s.channelDraft!.channels[0]!.discovered).toEqual(before.discovered)
  await w.get('button[aria-label="保存渠道草稿"]').trigger('click');expect(w.emitted('save')).toEqual([[]])
 })
 it('keeps an explicit valid provider when the name needs other characters',async()=>{
  const w=render();await flushPromises();await w.get('input[maxlength="80"]').setValue('渠道名称')
  expect(w.get('#channel-rename-error').text()).toContain('运行标识仅支持')
  await w.get('input[aria-label="运行渠道标识"]').setValue('Krill')
  await w.get('input[maxlength="80"]').setValue('新的渠道名称')
  expect(useModelsStore().channelDraft!.channels[0]!.connection.providerId).toBe('Krill');expect(w.find('#channel-rename-error').exists()).toBe(false)
 })
 it.each(['bad/id','openai','Krill-10',''])('blocks invalid/conflicting provider %s inline without saving',async target=>{
  const w=render();await flushPromises();await w.get('input[aria-label="运行渠道标识"]').setValue(target)
  expect(w.get('#channel-rename-error').text()).not.toBe('');expect(w.get('input[aria-label="运行渠道标识"]').attributes('aria-invalid')).toBe('true')
  await w.get('button[aria-label="保存渠道草稿"]').trigger('click');expect(w.emitted('save')).toBeUndefined();expect(useModelsStore().snapshot!.policy.channels[0]!.connection.providerId).toBe('Krill-1')
 })
 it('rejects duplicate names, keeps saved provider slots occupied, and protects original managed sources',()=>{
  const saved=[channel(),channel('other','other-provider')],c=channel();c.label='other-provider';c.connection.providerId='free';expect(channelRenameError(c,saved)).toContain('名称已存在')
  c.label='free';c.connection.providerId='other-provider';expect(channelRenameError(c,saved,[c])).toContain('标识已存在')
  const protectedSaved=channel();protectedSaved.connection.authMode='oauth';c.connection.providerId='free';expect(channelRenameError(c,[protectedSaved])).toContain('不能直接改名')
 })
 it('keeps explicit provider intent when reopening a draft and performs no persistence on return',async()=>{
  const saved=channel(),draft=channel();draft.label='别名';draft.connection.providerId='Explicit';const intent=channelRenameIntent(draft,saved);renameChannelLabel(draft,'其它名称',intent);expect(draft.connection.providerId).toBe('Explicit');renameChannelProvider(draft,'Next',intent);expect(draft.connection.providerId).toBe('Next')
  const w=render();await flushPromises();await w.get('input[maxlength="80"]').setValue('Krill');await w.get('button[aria-label="返回渠道列表"]').trigger('click');expect(w.emitted('save')).toBeUndefined();expect(useModelsStore().snapshot!.policy.channels[0]!.label).toBe('Krill-1')
 })
})
