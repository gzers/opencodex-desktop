import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {nextTick,reactive} from 'vue'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import ChannelModelTable from '@/features/models/ChannelModelTable.vue'
import NodePicker from '@/features/models/NodePicker.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {useModalStore} from '@/app/modal/store'
import {useModelsStore} from '@/features/models/store'
import {copy,createModelRowsState,rowFields} from '@/features/models/modelRows'
import {emptyRules,modelApi,type ChannelPolicy,type CredentialView} from '@/features/models/api'

vi.mock('@/features/models/api',async original=>({
  ...await original<typeof import('@/features/models/api')>(),
  modelApi:{credential:vi.fn(),discover:vi.fn(),measureNodes:vi.fn(),stopNodeMeasurement:vi.fn(),forgetCredential:vi.fn()},
}))

function channel(id:string):ChannelPolicy{return {
  id,label:id,templateRefs:[],modelRefs:{},
  connection:{providerId:'provider-'+id,protocol:'openai-chat',authMode:'key',nodes:[{id:'node',name:'默认节点',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},
  selection:{mode:'all'},modelOrder:['model'],discovered:[{id:'model',label:'model',rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{},
}}
const savedKey=(value='fixture-saved-key'):CredentialView=>({configured:true,value,source:'配置文件',viewable:true})
function deferred<T>(){
  let resolve!:(value:T)=>void,reject!:(cause:unknown)=>void
  const promise=new Promise<T>((yes,no)=>{resolve=yes;reject=no})
  return {promise,resolve,reject}
}
let wrappers:VueWrapper[]
function render(props:Partial<InstanceType<typeof ChannelEditor>['$props']>={},realTable=false){
  const store=useModelsStore()
  const wrapper=mount(ChannelEditor,{
    props:{channel:store.channelDraft!.channels[0]!,templates:[],existing:true,dirty:true,saveStatus:'未保存修改',saveTone:'starting',...props},
    global:{stubs:{ChannelModelTable:!realTable,NodePicker:true,ModelRuleFields:true,UiSelectField:true,UiDialog:true}},
  })
  wrappers.push(wrapper)
  return wrapper
}
function button(wrapper:VueWrapper,label:string){return wrapper.findAll('button').find(node=>node.text()===label)!}
function selection(wrapper:VueWrapper){return wrapper.findAllComponents(UiSelectField).find(node=>node.props('label')==='模型公开范围')!}
function pendingRows(value:ChannelPolicy,alias:string){
  const state=reactive(createModelRowsState()),fields=rowFields(value,'model')
  state.buffers.model={...copy(fields),base:copy(fields),naming:{alias},error:''}
  return state
}

beforeEach(()=>{
  setActivePinia(createPinia());vi.resetAllMocks();wrappers=[]
  vi.mocked(modelApi.credential).mockResolvedValue(savedKey())
  vi.mocked(modelApi.discover).mockResolvedValue({models:[],credentialRef:null,elapsedMs:1})
  vi.mocked(modelApi.forgetCredential).mockResolvedValue()
  vi.mocked(modelApi.stopNodeMeasurement).mockResolvedValue()
  vi.mocked(modelApi.measureNodes).mockImplementation(async request=>({runId:request.runId,stopped:false,results:[]}))
  useModelsStore().acceptTransfer({library:{schemaVersion:1,templates:[]},libraryDigest:'library',policyDigest:'saved-file-1',policy:{schemaVersion:1,templateRefs:[],channels:[channel('first'),channel('second')]},workspace:null,workspaceDigest:''})
})
afterEach(()=>{for(const wrapper of wrappers)wrapper.unmount();vi.restoreAllMocks()})

describe('WP85 direct channel editing',()=>{
  it('keeps only Save in the sidebar and receives current-channel status without a duplicate status line',async()=>{
    const wrapper=render();await flushPromises()
    expect(wrapper.find('.detail-navigation-actions').exists()).toBe(false)
    expect(wrapper.findAll('.detail-navigation-primary button')).toHaveLength(1)
    expect(wrapper.find('.detail-navigation-primary [role="status"]').exists()).toBe(false)
    expect(wrapper.get('[data-status-scope="current-channel"]').text()).toBe('未保存修改')
    expect(wrapper.get('[data-status-scope="current-channel"]').attributes('title')).toBe('当前渠道的保存状态')
    expect(wrapper.find('[data-status-scope="all-channels"]').exists()).toBe(false)
    const save=wrapper.get('button[aria-label="保存渠道草稿"]')
    expect(save.attributes('title')).toBe('保存当前渠道，成功后返回渠道列表；其它渠道修改保留')
    expect(save.attributes('data-save-scope')).toBe('current-channel')
    await save.trigger('click');expect(wrapper.emitted('save')).toEqual([[]]);expect(wrapper.emitted('close')).toBeUndefined()
    await wrapper.setProps({dirty:false,saveStatus:'未修改',saveTone:'muted'})
    expect(save.attributes('disabled')).toBeDefined();expect(wrapper.get('[data-status-scope="current-channel"]').text()).toBe('未修改')
    await wrapper.setProps({dirty:true,busy:true});expect(save.attributes('disabled')).toBeDefined()
    await wrapper.setProps({busy:false,saveDisabled:true});expect(save.attributes('disabled')).toBeDefined()
    await wrapper.setProps({saveDisabled:false})
    await wrapper.get('button[aria-label="收起渠道导航"]').trigger('click')
    expect(wrapper.findAll('.detail-navigation-primary button')).toHaveLength(1)
    expect(wrapper.find('.detail-navigation-primary [role="status"]').exists()).toBe(false)
    expect(save.find('svg').exists()).toBe(true)
    await wrapper.get('button[aria-label="返回渠道列表"]').trigger('click')
    expect(wrapper.emitted('close')).toEqual([[]]);expect(wrapper.emitted('restore')).toBeUndefined()
  })

  it('edits the channel directly and leaves discovery and detection in their sections',async()=>{
    const wrapper=render();await flushPromises()
    const config=wrapper.get('#channel-config')
    for(const text of ['确认连接与模型','还原配置','有连接修改','连接配置与草稿一致'])expect(config.text()).not.toContain(text)
    expect(config.find('.channel-config-footer').exists()).toBe(false)
    expect(wrapper.findAll('button').filter(node=>node.text()==='获取模型列表')).toHaveLength(1)
    expect(wrapper.get('#channel-models').text()).toContain('获取模型列表')
    expect(wrapper.get('#channel-nodes').findComponent(NodePicker).exists()).toBe(true)
    await config.get('input[maxlength="80"]').setValue('直接编辑的名称')
    const protocol=wrapper.findAllComponents(UiSelectField).find(node=>node.props('label')==='默认接口协议')!
    protocol.vm.$emit('update:modelValue','anthropic')
    await wrapper.get('input[aria-label="API Key"]').setValue('fixture-replacement-key')
    const store=useModelsStore()
    expect(store.channelDraft!.channels[0]!.label).toBe('直接编辑的名称')
    expect(store.channelDraft!.channels[0]!.connection.protocol).toBe('anthropic')
    expect(store.credentialEdits.first?.action).toBe('replace')
    expect(modelApi.discover).not.toHaveBeenCalled();expect(modelApi.measureNodes).not.toHaveBeenCalled()
  })
})

describe('current-channel Key intent across file refreshes',()=>{
  it.each(['success','failure'] as const)('retains replacement and clear intents when a saved-file read ends in %s',async outcome=>{
    const store=useModelsStore(),wrapper=render();await flushPromises()
    store.editCredential('second','fixture-other-channel-key','')
    for(const action of ['replace','clear'] as const){
      if(action==='replace')await wrapper.get('input[aria-label="API Key"]').setValue('fixture-replacement-key')
      else await button(wrapper,'清空 Key').trigger('click')
      const intent=copy(store.credentialEdits),read=deferred<CredentialView>()
      vi.mocked(modelApi.credential).mockReturnValueOnce(read.promise)
      store.snapshot!.policyDigest+='-changed';await nextTick()
      const key=wrapper.get<HTMLInputElement>('input[aria-label="API Key"]')
      expect(key.element.value).toBe(action==='replace'?'fixture-replacement-key':'')
      expect(key.attributes('disabled')).toBeDefined();expect(store.credentialEdits).toEqual(intent)
      if(outcome==='success')read.resolve(savedKey('fixture-file-refreshed-key'))
      else read.reject({message:'读取失败'})
      await flushPromises()
      expect(key.element.value).toBe(action==='replace'?'fixture-replacement-key':'')
      expect(key.attributes('disabled')).toBeUndefined();expect(store.credentialEdits).toEqual(intent)
      expect(wrapper.text()).toContain(action==='replace'?'保存后替换 Key':'保存后清空 Key')
      await button(wrapper,'获取模型列表').trigger('click');await flushPromises()
      expect(modelApi.discover).toHaveBeenLastCalledWith(expect.objectContaining({apiKey:action==='replace'?'fixture-replacement-key':null,providerId:'provider-first'}))
      wrapper.getComponent(NodePicker).vm.$emit('measure',['node']);await flushPromises()
      expect(modelApi.measureNodes).toHaveBeenLastCalledWith(expect.objectContaining({apiKey:action==='replace'?'fixture-replacement-key':null,providerId:'provider-first'}))
    }
  })

  it.each(['success','failure'] as const)('restores the latest store intent, including edits made during a %s read',async outcome=>{
    const wrapper=render();await flushPromises()
    const store=useModelsStore(),read=deferred<CredentialView>()
    vi.mocked(modelApi.credential).mockReturnValueOnce(read.promise)
    store.snapshot!.policyDigest='saved-file-2';await nextTick()
    store.editCredential('first','fixture-edited-during-read','')
    if(outcome==='success')read.resolve(savedKey())
    else read.reject({message:'读取失败'})
    await flushPromises()
    expect(wrapper.get<HTMLInputElement>('input[aria-label="API Key"]').element.value).toBe('fixture-edited-during-read')
    await button(wrapper,'获取模型列表').trigger('click');await flushPromises()
    expect(modelApi.discover).toHaveBeenCalledWith(expect.objectContaining({apiKey:'fixture-edited-during-read'}))
  })

  it('uses the saved file digest rather than unrelated snapshot changes to refresh the Key',async()=>{
    const wrapper=render();await flushPromises()
    await wrapper.get('input[aria-label="API Key"]').setValue('fixture-replacement-key')
    const store=useModelsStore()
    store.snapshot={...store.snapshot!,libraryDigest:'library-2',workspaceDigest:'workspace-2'}
    await flushPromises()
    expect(modelApi.credential).toHaveBeenCalledTimes(1)
    expect(wrapper.get<HTMLInputElement>('input[aria-label="API Key"]').element.value).toBe('fixture-replacement-key')
  })

  it.each(['success','failure'] as const)('ignores an older credential %s after switching channels and refreshing again',async outcome=>{
    const first=deferred<CredentialView>(),second=deferred<CredentialView>(),latest=deferred<CredentialView>()
    vi.mocked(modelApi.credential).mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise).mockReturnValueOnce(latest.promise)
    const store=useModelsStore(),wrapper=render()
    store.editCredential('second','fixture-second-channel-key','')
    await wrapper.setProps({channel:store.channelDraft!.channels[1]!})
    store.snapshot!.policyDigest='saved-file-2';await nextTick()
    latest.resolve(savedKey('fixture-latest-file-key'));await flushPromises()
    if(outcome==='success'){first.resolve(savedKey('fixture-stale-first-key'));second.resolve(savedKey('fixture-stale-second-key'))}
    else {first.reject({message:'旧渠道读取失败'});second.reject({message:'旧文件读取失败'})}
    await flushPromises()
    const key=wrapper.get<HTMLInputElement>('input[aria-label="API Key"]')
    expect(key.element.value).toBe('fixture-second-channel-key');expect(key.attributes('disabled')).toBeUndefined()
    expect(wrapper.text()).not.toContain('读取失败')
    await button(wrapper,'获取模型列表').trigger('click');await flushPromises()
    expect(modelApi.discover).toHaveBeenCalledWith(expect.objectContaining({providerId:'provider-second',apiKey:'fixture-second-channel-key'}))
  })

  it('keeps revealed and emptied input behind origin validation while only explicit clear deletes the Key',async()=>{
    const store=useModelsStore(),wrapper=render();await flushPromises()
    const key=wrapper.get<HTMLInputElement>('input[aria-label="API Key"]')
    await wrapper.get('button[aria-label="显示 API Key"]').trigger('click')
    expect(key.attributes('type')).toBe('text');expect(store.credentialEdits).toEqual({})
    store.channelDraft!.channels[0]!.connection.nodes[0]!.url='https://changed.example.test'
    await button(wrapper,'获取模型列表').trigger('click');await flushPromises()
    expect(modelApi.discover).toHaveBeenCalledWith(expect.objectContaining({url:'https://changed.example.test',apiKey:null,providerId:'provider-first',credentialRef:null,allowPrivateNetwork:false}))
    wrapper.getComponent(NodePicker).vm.$emit('measure',['node']);await flushPromises()
    expect(modelApi.measureNodes).toHaveBeenCalledWith(expect.objectContaining({apiKey:null,providerId:'provider-first',nodes:[expect.objectContaining({url:'https://changed.example.test'})]}))
    await key.setValue('fixture-replacement-key');await key.setValue('')
    expect(store.credentialEdits.first).toBeUndefined();expect(wrapper.text()).toContain('保持原 Key')
    await button(wrapper,'清空 Key').trigger('click')
    expect(store.credentialEdits.first).toEqual({channelId:'first',action:'clear',value:null})
    expect(key.attributes('type')).toBe('password')
  })
})

describe('discovery cancellation and late responses',()=>{
  it.each(['success','failure'] as const)('keeps a cancelled %s from changing a later discovery',async outcome=>{
    const old=deferred<Awaited<ReturnType<typeof modelApi.discover>>>(),current=deferred<Awaited<ReturnType<typeof modelApi.discover>>>()
    vi.mocked(modelApi.discover).mockReturnValueOnce(old.promise).mockReturnValueOnce(current.promise)
    const store=useModelsStore(),wrapper=render();await flushPromises()
    await button(wrapper,'获取模型列表').trigger('click')
    expect(wrapper.get('#channel-models').text()).toContain('停止等待')
    expect(wrapper.get('#channel-config').text()).not.toContain('停止等待')
    await button(wrapper,'停止等待').trigger('click');await button(wrapper,'获取模型列表').trigger('click')
    if(outcome==='success')old.resolve({models:[{id:'stale',label:'stale',rules:emptyRules()}],credentialRef:'stale-reference',elapsedMs:1})
    else old.reject({message:'旧清单获取失败'})
    await flushPromises()
    expect(button(wrapper,'获取模型列表').attributes('disabled')).toBeDefined()
    expect(wrapper.text()).not.toContain('旧清单获取失败');expect(store.channelDraft!.channels[0]!.modelOrder).toEqual(['model'])
    if(outcome==='success')expect(modelApi.forgetCredential).toHaveBeenCalledWith('stale-reference')
    current.resolve({models:[{id:'current',label:'current',rules:emptyRules()}],credentialRef:null,elapsedMs:1});await flushPromises()
    expect(store.channelDraft!.channels[0]!.modelOrder).toEqual(['model','current'])
    expect(button(wrapper,'停止等待')).toBeUndefined()
  })

  it.each(['url','protocol','node','channel','unmount'] as const)('ignores discovery after its %s binding changes',async change=>{
    const read=deferred<Awaited<ReturnType<typeof modelApi.discover>>>()
    vi.mocked(modelApi.discover).mockReturnValueOnce(read.promise)
    const store=useModelsStore(),wrapper=render();await flushPromises()
    await button(wrapper,'获取模型列表').trigger('click')
    const first=store.channelDraft!.channels[0]!
    if(change==='url')first.connection.nodes[0]!.url='https://changed.example.test'
    if(change==='protocol')first.connection.protocol='anthropic'
    if(change==='node'){first.connection.nodes.push({id:'other',name:'备用',url:'https://other.example.test'});first.connection.activeNodeId='other'}
    if(change==='channel')await wrapper.setProps({channel:store.channelDraft!.channels[1]!})
    if(change==='unmount')wrapper.unmount()
    await nextTick()
    read.resolve({models:[{id:'stale',label:'stale',rules:emptyRules()}],credentialRef:'stale-reference',elapsedMs:1});await flushPromises()
    expect(store.channelDraft!.channels.map(value=>value.modelOrder)).toEqual([['model'],['model']])
    expect(modelApi.forgetCredential).toHaveBeenCalledWith('stale-reference')
  })
})

describe('current-channel row buffers and selection confirmation',()=>{
  it('uses replacement rowState in both the table and pending-row guard',async()=>{
    const value=useModelsStore().channelDraft!.channels[0]!,initial=reactive(createModelRowsState())
    const wrapper=render({rowState:initial},true);await flushPromises()
    const next=pendingRows(value,'pending-alias')
    await wrapper.setProps({rowState:next})
    expect(wrapper.getComponent(ChannelModelTable).props('state')).toBe(next)
    expect(wrapper.get<HTMLInputElement>('input[aria-label="model路由别名"]').element.value).toBe('pending-alias')
    selection(wrapper).vm.$emit('update:modelValue','none');await nextTick()
    expect(useModalStore().current).toBeNull();expect(wrapper.get('#channel-models').text()).toContain('请先处理未确认型号编辑')
    await wrapper.setProps({rowState:reactive(createModelRowsState())})
    expect(wrapper.get<HTMLInputElement>('input[aria-label="model路由别名"]').element.value).toBe('')
    selection(wrapper).vm.$emit('update:modelValue','none');await nextTick()
    expect(useModalStore().current?.title).toBe('调整模型公开范围')
  })

  it('keeps fallback row buffers with their channel when the same instance switches channels',async()=>{
    const store=useModelsStore(),first=store.channelDraft!.channels[0]!,second=store.channelDraft!.channels[1]!
    const wrapper=render({},true);await flushPromises()
    await wrapper.get('input[aria-label="model路由别名"]').setValue('first-pending-alias')
    await wrapper.setProps({channel:second})
    expect(wrapper.get<HTMLInputElement>('input[aria-label="model路由别名"]').element.value).toBe('')
    selection(wrapper).vm.$emit('update:modelValue','none');await nextTick()
    useModalStore().resolve('confirm');await nextTick();expect(second.selection.mode).toBe('none');expect(first.selection.mode).toBe('all')
    await wrapper.setProps({channel:first})
    expect(wrapper.get<HTMLInputElement>('input[aria-label="model路由别名"]').element.value).toBe('first-pending-alias')
    selection(wrapper).vm.$emit('update:modelValue','none');await nextTick();expect(useModalStore().current).toBeNull()
  })

  it.each(['channel','rowState','pending'] as const)('invalidates an open selection confirmation after a %s change',async change=>{
    const store=useModelsStore(),first=store.channelDraft!.channels[0]!,state=reactive(createModelRowsState())
    const wrapper=render({rowState:state});await flushPromises()
    selection(wrapper).vm.$emit('update:modelValue','none');await nextTick()
    expect(useModalStore().current).not.toBeNull()
    if(change==='channel')await wrapper.setProps({channel:store.channelDraft!.channels[1]!,rowState:reactive(createModelRowsState())})
    if(change==='rowState')await wrapper.setProps({rowState:reactive(createModelRowsState())})
    if(change==='pending')state.buffers=pendingRows(first,'pending-after-open').buffers
    useModalStore().resolve('confirm');await nextTick()
    expect(store.channelDraft!.channels.map(value=>value.selection.mode)).toEqual(['all','all'])
    expect(wrapper.get('#channel-models').text()).toContain('范围确认已过期')
  })
})
