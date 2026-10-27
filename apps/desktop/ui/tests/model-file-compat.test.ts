import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {defineComponent,watch} from 'vue'
import ModelSaveHistory from '@/features/backup/ModelSaveHistory.vue'
import ChannelsView from '@/features/models/ChannelsView.vue'
import SelectorPreviewDialog from '@/features/models/SelectorPreviewDialog.vue'
import {emptyRules,type ChannelPolicy,type CredentialView,type ModelPolicy,type ModelSnapshot,type SavePlan} from '@/features/models/api'
import {useModelsStore} from '@/features/models/store'
import {useModelSaveFeedback} from '@/app/feedback/modelSave'
import {useFeedbackStore} from '@/app/feedback/store'
import {useModalStore} from '@/app/modal/store'

const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
vi.mock('@/stores/routes',()=>({useRouteStore:()=>({go:vi.fn()})}))
const dialog=defineComponent({props:['open','title'],template:'<section v-if="open" :data-dialog-title="title"><slot/><slot name="actions"/></section>'})
const clone=<T,>(value:T):T=>JSON.parse(JSON.stringify(value))
function channel():ChannelPolicy{return {id:'source',label:'Actual file channel',templateRefs:[],modelRefs:{},connection:{providerId:'custom',protocol:'openai-chat',authMode:'key',nodes:[{id:'node',name:'Default',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['a']},modelOrder:['a'],discovered:[{id:'a',label:'a',rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{}}}
function file():ModelSnapshot{return {library:{schemaVersion:1,templates:[]},libraryDigest:'library',policy:{schemaVersion:1,templateRefs:[],channels:[channel()]},policyDigest:'actual-config-file',workspaceDigest:'empty-workspace',workspace:null}}
function legacy():ModelSnapshot{const s=file();s.policy.channels[0]!.label='Old manager channel';return {...s,policyDigest:'legacy-policy',workspaceDigest:'legacy-workspace'}}
function recovered():ModelSnapshot{const s=legacy(),policy=clone(s.policy);policy.recoveryId='recovery-id';policy.channels[0]!.label='Historical channel';return {...s,libraryDigest:'accepted-library',workspaceDigest:'accepted-workspace',workspace:{basePolicy:'legacy-policy',policy}}}
function restoredFile():ModelSnapshot{const s=file();s.libraryDigest='accepted-library';s.policy.channels[0]!.label='Historical channel';return {...s,policyDigest:'restored-config-file'}}
function plan(phase:string,error:string|null=null):SavePlan{return {id:'runtime-recovery-plan',phase,error,catalogCount:1,currentCatalogCount:1,changedChannels:['Historical channel'],historySource:'successful-save'}}
const savedKey=():CredentialView=>({configured:true,viewable:true,value:'fixture-saved-key',source:'config-file'})
let currentFile:ModelSnapshot
let accepted:ModelSnapshot
let overrides:Record<string,((args:Record<string,unknown>)=>unknown)|undefined>
let wrappers:VueWrapper[]
const calls=(command:string)=>ipc.mock.calls.filter(call=>call[0]===command)
function render(component:typeof ModelSaveHistory|typeof ChannelsView){const w=mount(component,{global:{stubs:{UiDialog:dialog,UiSelectField:true,ChannelEditor:true,ChannelNodeSelect:true}}});wrappers.push(w);return w}
function button(w:VueWrapper,text:string){const result=w.findAll('button').find(node=>node.text()===text);expect(result,'button: '+text).toBeDefined();return result!}
async function compareHistory(){const w=render(ModelSaveHistory);await flushPromises();await button(w,'生成恢复草稿').trigger('click');await button(w,'比较恢复差异').trigger('click');await flushPromises();return w}
async function acceptHistory(){const w=await compareHistory();await button(w,'放入恢复草稿').trigger('click');await flushPromises();expect(useModelsStore().historicalRecovery?.id).toBe('recovery-id');return w}
async function previewSave(w:VueWrapper){await flushPromises();await w.get('[data-testid="policy-save"]').trigger('click');await flushPromises();expect(w.findAll('[data-dialog-title="保存预览"]')).toHaveLength(1);expect(w.findAll('.channels-view > p[role="status"]')).toHaveLength(0);expect(w.text()).not.toContain('保存历史恢复')}
async function confirmSave(w:VueWrapper){await previewSave(w);await button(w,'确认保存').trigger('click');await flushPromises()}
async function openCopy(w:VueWrapper,reuse=true){await button(w,'复制').trigger('click');await w.get('input[maxlength="256"]').setValue('My copy');const checkbox=w.findAll('label').find(label=>label.text()==='副本复用此本机渠道Key')!.get('input');await checkbox.setValue(reuse)}
function copied(){return useModelsStore().channelDraft!.channels.find(c=>c.id!=='source')!}

beforeEach(async()=>{
  setActivePinia(createPinia());vi.clearAllMocks();currentFile=file();accepted=recovered();overrides={};wrappers=[]
  ipc.mockImplementation(async(command:string,args:Record<string,unknown>={})=>{
    if(overrides[command])return overrides[command]!(args)
    if(command==='model_file_snapshot')return clone(currentFile)
    if(command==='model_config_snapshot')return legacy()
    if(command==='model_policy_history')return [{id:'successful-save',createdAt:'2026-10-19T00:00:00Z',phase:'committed',channelLabels:['Historical channel'],recoverable:true}]
    if(command==='model_policy_recovery_prepare')return {id:'recovery-id',sourceId:'successful-save',side:'before',scope:'affected',restoreDefaults:false,requiresRuntimeReview:true,changes:[],defaultChanges:[],channelLabels:['Historical channel'],notices:[]}
    if(command==='model_policy_recovery_accept')return clone(accepted)
    if(command==='model_policy_preview')return {rows:[],knownActiveModels:1,exactCatalogCount:1,notices:[]}
    if(command==='model_agent_directory')return {channels:[],rows:[]}
    if(command==='model_policy_prepare')return plan('prepared')
    if(command==='model_policy_apply'||command==='model_policy_result'){currentFile=restoredFile();return plan('committed')}
    if(command==='model_file_save'){const request=args.request as {policy:ModelPolicy};currentFile={...currentFile,policy:clone(request.policy),policyDigest:'ordinary-saved-file'};return {changed:true,snapshot:clone(currentFile)}}
    if(command==='model_file_credential')return savedKey()
    if(command==='model_policy_save_blocked')return undefined
    if(command==='model_policy_latest')throw {code:'plan_incomplete',message:'Old plan must not block file saves'}
    throw new Error('Unexpected IPC: '+command)
  })
  await useModelsStore().load()
})
afterEach(()=>{for(const w of wrappers)w.unmount();vi.restoreAllMocks()})

describe('explicit history adapter using the actual file models UI',()=>{
  it('isolates all legacy baselines through compare, accept, runtime save and file reload, then uses only file saves',async()=>{
    const models=useModelsStore(),seen:string[]=[]
    const stop=watch(()=>models.snapshot?.policyDigest,value=>{if(value)seen.push(value)},{flush:'sync'})
    await acceptHistory()
    expect(ipc).toHaveBeenCalledWith('model_policy_recovery_prepare',{request:{sourceId:'successful-save',side:'before',scope:'affected',restoreDefaults:false,expectedLibrary:'library',expectedPolicy:'legacy-policy',expectedWorkspace:'legacy-workspace'}})
    expect(models.snapshot?.policy).toEqual(file().policy);expect(models.snapshot?.workspace).toBeNull();expect(models.snapshot?.policyDigest).toBe('actual-config-file');expect(models.channelBaseline).toBe('actual-config-file')
    expect(models.historicalRecovery).toMatchObject({libraryDigest:'accepted-library',policyDigest:'legacy-policy',workspaceDigest:'accepted-workspace'})
    await models.load();expect(models.channelDraft).toEqual(accepted.workspace!.policy)
    const w=render(ChannelsView);await previewSave(w)
    const before=clone(models.channelDraft)
    w.getComponent(SelectorPreviewDialog).vm.$emit('toggle-channel','source');w.getComponent(SelectorPreviewDialog).vm.$emit('toggle-model','source','a')
    expect(models.channelDraft).toEqual(before)
    expect(useModelSaveFeedback().open).toBe(false)
    await button(w,'确认保存').trigger('click');await flushPromises()
    expect(ipc).toHaveBeenCalledWith('model_policy_prepare',{policy:accepted.workspace!.policy,expectedLibrary:'accepted-library',expectedPolicy:'legacy-policy',expectedWorkspace:'accepted-workspace'})
    expect(ipc).toHaveBeenCalledWith('model_policy_apply',{planId:'runtime-recovery-plan'})
    expect(models.snapshot).toEqual(restoredFile());expect(models.channelBaseline).toBe('restored-config-file');expect(models.historicalRecovery).toBeNull();expect(models.channelDirty).toBe(false);expect(models.channelDraft?.recoveryId).toBeUndefined();expect(useModelSaveFeedback().revision).toBe(1)
    expect(calls('model_file_save')).toHaveLength(0);expect(calls('model_policy_stage')).toHaveLength(0);expect(seen).not.toContain('legacy-policy')
    models.channelDraft!.channels[0]!.label='Ordinary new edit';await confirmSave(w)
    expect(calls('model_file_save')).toHaveLength(1);expect(calls('model_file_save')[0]![1].request.expectedFile).toBe('restored-config-file')
    expect(calls('model_config_snapshot')).toHaveLength(1);expect(calls('model_policy_prepare')).toHaveLength(1);expect(calls('model_policy_latest')).toHaveLength(0);expect(calls('model_policy_result')).toHaveLength(0);expect(useModelSaveFeedback().revision).toBe(2);stop()
  })

  it('saves an authorization-only historical target even when the visible channel policy is unchanged',async()=>{
    accepted.workspace!.policy={...clone(file().policy),recoveryId:'recovery-id'}
    await acceptHistory();const models=useModelsStore();expect(models.channelDirty).toBe(true)
    const w=render(ChannelsView);await confirmSave(w)
    expect(calls('model_policy_prepare')[0]![1].policy.recoveryId).toBe('recovery-id');expect(calls('model_policy_apply')).toHaveLength(1);expect(calls('model_file_save')).toHaveLength(0);expect(useModelSaveFeedback().open).toBe(true)
  })

  it('retains a failed recovery draft, then allows an ordinary file save after explicit discard despite old plans',async()=>{
    overrides.model_policy_prepare=()=>{throw {code:'plan_incomplete',message:'Historical plan needs review'}}
    await acceptHistory();const models=useModelsStore(),draft=clone(models.channelDraft),w=render(ChannelsView);await confirmSave(w)
    expect(models.channelDraft).toEqual(draft);expect(models.snapshot?.policyDigest).toBe('actual-config-file');expect(models.historicalRecovery).not.toBeNull();expect(useModelSaveFeedback().open).toBe(false);expect(w.text()).toContain('Historical plan needs review');expect(calls('model_file_save')).toHaveLength(0)
    await button(w,'还原修改').trigger('click');useModalStore().resolve('confirm');await flushPromises()
    expect(models.historicalRecovery).toBeNull();models.channelDraft!.channels[0]!.label='ordinary';await confirmSave(w)
    expect(calls('model_file_save')).toHaveLength(1);expect(calls('model_policy_prepare')).toHaveLength(1);expect(calls('model_policy_latest')).toHaveLength(0);expect(useModelSaveFeedback().revision).toBe(1)
  })

  it.each(['partial','unknown'])('retains the original plan after a %s result and rechecks that plan on explicit retry',async outcome=>{
    overrides.model_policy_apply=()=>{if(outcome==='unknown')throw {message:'History transport interrupted'};return plan('history_authorizations','management_unknown')}
    await acceptHistory();const models=useModelsStore(),draft=clone(models.channelDraft),w=render(ChannelsView);await confirmSave(w)
    expect(models.channelDraft).toEqual(draft);expect(models.historicalRecovery?.plan?.id).toBe('runtime-recovery-plan');expect(models.snapshot?.policyDigest).toBe('actual-config-file');expect(useModelSaveFeedback().open).toBe(false)
    await confirmSave(w)
    expect(ipc).toHaveBeenCalledWith('model_policy_result',{planId:'runtime-recovery-plan'});expect(calls('model_policy_prepare')).toHaveLength(1);expect(calls('model_policy_apply')).toHaveLength(1);expect(calls('model_file_save')).toHaveLength(0);expect(models.snapshot).toEqual(restoredFile());expect(models.historicalRecovery).toBeNull();expect(useModelSaveFeedback().revision).toBe(1)
  })

  it('retains a committed recovery until file reload succeeds and never repeats the runtime write',async()=>{
    await acceptHistory();const models=useModelsStore(),w=render(ChannelsView)
    overrides.model_file_snapshot=()=>{throw {message:'read failed'}}
    await confirmSave(w)
    expect(models.historicalRecovery?.plan?.phase).toBe('committed');expect(models.snapshot?.policyDigest).toBe('actual-config-file');expect(useModelSaveFeedback().open).toBe(false);expect(useFeedbackStore().toast).toContain('历史恢复已提交');expect(useFeedbackStore().toast).not.toContain('未写入')
    delete overrides.model_file_snapshot;await confirmSave(w)
    expect(models.snapshot).toEqual(restoredFile());expect(models.historicalRecovery).toBeNull();expect(calls('model_policy_prepare')).toHaveLength(1);expect(calls('model_policy_apply')).toHaveLength(1);expect(calls('model_policy_result')).toHaveLength(0);expect(useModelSaveFeedback().revision).toBe(1)
  })

  it('retains Key input created during recovery acceptance without installing the legacy response',async()=>{
    let finish!:(value:ModelSnapshot)=>void
    overrides.model_policy_recovery_accept=()=>new Promise<ModelSnapshot>(resolve=>{finish=resolve})
    const w=await compareHistory();await button(w,'放入恢复草稿').trigger('click');await flushPromises()
    const models=useModelsStore();models.editCredential('source','fixture-new-key','')
    finish(recovered());await flushPromises()
    expect(models.credentialEdits.source?.value).toBe('fixture-new-key');expect(models.historicalRecovery).toBeNull();expect(models.snapshot?.policy).toEqual(file().policy);expect(w.text()).toContain('当前输入已保留')
  })

  it.each(['channel','key'])('rejects %s edits mixed into an accepted historical target without losing input or using either writer',async kind=>{
    await acceptHistory();const models=useModelsStore()
    if(kind==='key')models.editCredential('source','fixture-new-key','')
    else models.channelDraft!.channels[0]!.label='new unsaved label'
    const draft=clone(models.channelDraft),keys=clone(models.credentialEdits),w=render(ChannelsView);await confirmSave(w)
    expect(models.channelDraft).toEqual(draft);expect(models.credentialEdits).toEqual(keys);expect(calls('model_policy_prepare')).toHaveLength(0);expect(calls('model_file_save')).toHaveLength(0);expect(useModelSaveFeedback().open).toBe(false);expect(w.text()).toContain('当前输入已保留')
  })
})

describe('channel copy with actual-file credentials',()=>{
  it('reads a saved Key once before adding the copy, stages it only in memory and writes it only on explicit Save',async()=>{
    let finish!:(value:CredentialView)=>void
    overrides.model_file_credential=()=>new Promise<CredentialView>(resolve=>{finish=resolve})
    const models=useModelsStore(),w=render(ChannelsView);await openCopy(w)
    await button(w,'建立副本草稿').trigger('click');await button(w,'建立副本草稿').trigger('click');await flushPromises()
    expect(calls('model_file_credential')).toHaveLength(1);expect(calls('model_file_credential')[0]![1]).toEqual({providerId:'custom'});expect(models.channelDraft!.channels).toHaveLength(1);expect(w.get('input[maxlength="256"]').element).toHaveProperty('value','My copy')
    finish(savedKey());await flushPromises()
    const copy=copied();expect(copy.connection.credentialRef).toBeNull();expect(models.credentialEdits[copy.id]).toEqual({channelId:copy.id,action:'replace',value:'fixture-saved-key'});expect(calls('model_file_save')).toHaveLength(0);expect(calls('model_policy_stage')).toHaveLength(0);expect(w.text()).not.toContain('fixture-saved-key')
    models.view.channelEditor='';await flushPromises();await confirmSave(w)
    expect(calls('model_file_save')[0]![1].request.credentials).toEqual([{channelId:copy.id,action:'replace',value:'fixture-saved-key'}]);expect(calls('model_policy_prepare')).toHaveLength(0)
  })

  it.each(['replace','clear'])('uses the source pending %s without reading an older saved Key',async action=>{
    const models=useModelsStore()
    if(action==='replace')models.editCredential('source','fixture-replacement','')
    else models.clearCredential('source')
    const original=clone(models.credentialEdits.source),w=render(ChannelsView);await openCopy(w);await button(w,'建立副本草稿').trigger('click');await flushPromises()
    const copy=copied();expect(copy).toBeDefined();expect(calls('model_file_credential')).toHaveLength(0);expect(models.credentialEdits.source).toEqual(original)
    if(action==='replace')expect(models.credentialEdits[copy.id]?.value).toBe('fixture-replacement')
    else expect(models.credentialEdits[copy.id]).toBeUndefined()
    expect(calls('model_file_save')).toHaveLength(0)
  })

  it.each(['local','none'])('copies %s authorization without fetching a stale Key',async authMode=>{
    const models=useModelsStore();models.channelDraft!.channels[0]!.connection.authMode=authMode
    const w=render(ChannelsView);await openCopy(w);await button(w,'建立副本草稿').trigger('click');await flushPromises()
    expect(copied().connection.authMode).toBe(authMode);expect(copied().connection.credentialRef).toBeNull();expect(models.credentialEdits).toEqual({});expect(calls('model_file_credential')).toHaveLength(0);expect(calls('model_file_save')).toHaveLength(0)
  })

  it.each(['unreadable','failed'])('retains copy input on %s credentials and displays only a fixed error',async failure=>{
    overrides.model_file_credential=()=>{if(failure==='failed')throw {message:'untrusted credential payload'};return {...savedKey(),viewable:false,value:''}}
    const models=useModelsStore(),w=render(ChannelsView);await openCopy(w);await button(w,'建立副本草稿').trigger('click');await flushPromises()
    expect(models.channelDraft!.channels).toHaveLength(1);expect(models.credentialEdits).toEqual({});expect(w.get('input[maxlength="256"]').element).toHaveProperty('value','My copy');expect(w.get('input[maxlength="256"]').attributes('disabled')).toBeUndefined();expect(w.text()).toContain('副本尚未建立');expect(w.text()).not.toContain('untrusted credential payload');expect(calls('model_file_save')).toHaveLength(0)
  })

  it('copies without a Key when reuse is declined, even with a pending replacement',async()=>{
    const models=useModelsStore();models.editCredential('source','fixture-replacement','');const w=render(ChannelsView);await openCopy(w,false);await button(w,'建立副本草稿').trigger('click');await flushPromises()
    expect(copied().connection.credentialRef).toBeNull();expect(models.credentialEdits[copied().id]).toBeUndefined();expect(models.credentialEdits.source?.value).toBe('fixture-replacement');expect(calls('model_file_credential')).toHaveLength(0);expect(calls('model_file_save')).toHaveLength(0)
  })
})
