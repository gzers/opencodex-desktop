import {beforeEach,describe,expect,it,vi} from 'vitest'
import {createPinia,setActivePinia} from 'pinia'
import {emptyRules,modelApi,type ChannelPolicy,type ModelPolicy,type ModelSnapshot} from '@/features/models/api'
import {createModelRowsState,rowFields} from '@/features/models/modelRows'
import {useModelsStore} from '@/features/models/store'

vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{acceptRecovery:vi.fn()}}))
const copy=<T>(value:T):T=>JSON.parse(JSON.stringify(value))
function channel(id:string):ChannelPolicy {
  return {id,label:id,templateRefs:[],modelRefs:{},connection:{providerId:'provider-'+id,protocol:'openai-chat',nodes:[{id:'node-'+id,name:'主站',url:'https://'+id+'.example.test'}],activeNodeId:'node-'+id,credentialRef:'key-'+id,enabled:true,allowPrivateNetwork:false,sourceReadId:'file-read'},selection:{mode:'all'},modelOrder:['model'],discovered:[{id:'model',label:'model',rules:emptyRules()}],defaults:emptyRules(),overrides:{model:emptyRules()},naming:{model:{}},familyOverrides:{}}
}
function actual():ModelSnapshot {
  return {library:{schemaVersion:1,templates:[]},libraryDigest:'library',policyDigest:'actual-file',workspaceDigest:'no-disk-draft',workspace:null,policy:{schemaVersion:1,templateRefs:[],channels:[channel('a'),channel('b')]}}
}
function adopted(policy:ModelPolicy):ModelSnapshot {
  return {...actual(),libraryDigest:'adopted-library',workspaceDigest:'incoming-workspace',workspace:{basePolicy:'actual-file',policy:copy(policy)}}
}
function plan(policy:ModelPolicy) {return {expectedPolicy:'actual-file',mergedPolicy:policy}}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();useModelsStore().acceptTransfer(actual())})

describe('transfer adoption into the current file draft',()=>{
  it('merges unrelated channels, nested fields and model rows while retaining every Key intent and raw receipt',async()=>{
    const store=useModelsStore(),incoming=actual().policy
    store.channelDraft!.channels[1]!.label='unsaved other channel'
    store.channelDraft!.channels[0]!.defaults.contextWindow={mode:'custom',value:32000}
    store.channelDraft!.channels[0]!.connection.nodes[0]!.name='unsaved node name'
    store.channelDraft!.channels[0]!.naming!.model={alias:'unsaved-alias'}
    store.channelDraft!.channels[1]!.connection.importedConfig={id:'earlier-import',digest:'earlier-material'}
    store.editCredential('a','unsaved-key','old-key');store.clearCredential('b')
    const credentials=store.credentialEdits
    const state=createModelRowsState(),fields=rowFields(store.channelDraft!.channels[1]!,'model')
    state.buffers.model={...copy(fields),base:copy(fields),error:''}
    state.buffers.model.rules.fast={mode:'custom',value:true};store.modelRows.b=state
    const rows=store.modelRows
    incoming.channels[0]!.defaults.protocol={mode:'custom',value:'anthropic'}
    incoming.channels[0]!.connection.nodes[0]!.url='https://incoming.example.test'
    incoming.channels[0]!.naming!.model={displayName:'Imported name'}
    incoming.channels[0]!.connection.importedConfig={id:'incoming',digest:'raw-material'}
    incoming.channels.push(channel('imported'))
    const accept=vi.fn().mockResolvedValue(adopted(incoming))
    await store.adoptTransfer(plan(incoming),accept)
    expect(accept).toHaveBeenCalledOnce()
    const a=store.channelDraft!.channels[0]!
    expect(a.defaults.contextWindow).toEqual({mode:'custom',value:32000})
    expect(a.defaults.protocol).toEqual({mode:'custom',value:'anthropic'})
    expect(a.connection.nodes[0]).toMatchObject({name:'unsaved node name',url:'https://incoming.example.test'})
    expect(a.naming!.model).toEqual({alias:'unsaved-alias',displayName:'Imported name'})
    expect(a.connection.importedConfig).toEqual({id:'incoming',digest:'raw-material'})
    expect(store.channelDraft!.channels[1]).toMatchObject({label:'unsaved other channel',connection:{importedConfig:{id:'earlier-import',digest:'earlier-material'}}})
    expect(store.channelDraft!.channels.map(c=>c.id)).toEqual(['a','b','imported'])
    expect(store.credentialEdits).toBe(credentials)
    expect(store.credentialEdits).toEqual({a:{channelId:'a',action:'replace',value:'unsaved-key'},b:{channelId:'b',action:'clear',value:null}})
    expect(store.modelRows).toBe(rows);expect(store.pendingRows).toEqual([{channel:'b',model:'model'}])
    expect(store.channelBaseline).toBe('actual-file');expect(store.snapshot!.policy).toEqual(actual().policy)
    expect(store.channelDirty).toBe(true)
  })

  it.each(['field','channel deletion','local deletion','Key-only deletion','pending row','raw receipt','exact revision'] as const)('blocks overlapping %s before adopting backend side effects',async kind=>{
    const store=useModelsStore(),incoming=actual().policy
    store.editCredential('a','unsaved-key','old-key')
    if (kind==='field') {store.channelDraft!.channels[0]!.label='local';incoming.channels[0]!.label='incoming'}
    if (kind==='channel deletion') {store.channelDraft!.channels[0]!.label='local';incoming.channels.shift()}
    if (kind==='local deletion') {store.channelDraft!.channels.shift();incoming.channels[0]!.label='incoming'}
    if (kind==='Key-only deletion') incoming.channels.shift()
    if (kind==='pending row') {
      const state=createModelRowsState(),fields=rowFields(store.channelDraft!.channels[0]!,'model')
      state.buffers.model={...copy(fields),base:copy(fields),error:''};state.buffers.model.rules.fast={mode:'custom',value:true};store.modelRows.a=state
      incoming.channels[0]!.overrides.model!.fast={mode:'custom',value:false}
    }
    if (kind==='raw receipt') {
      store.channelDraft!.channels[0]!.connection.importedConfig={id:'local-material',digest:'local-digest'}
      incoming.channels[0]!.connection.importedConfig={id:'incoming-material',digest:'incoming-digest'}
    }
    if (kind==='exact revision') {
      const base=actual();base.policy.channels[0]!.modelRefs.model={templateId:'template',revisionId:'original'};store.acceptTransfer(base)
      store.channelDraft!.channels[0]!.modelRefs.model={templateId:'other-template',revisionId:'original'}
      incoming.channels[0]!.modelRefs.model={templateId:'template',revisionId:'other-revision'}
      store.clearCredential('a')
    }
    const draft=copy(store.channelDraft),credentials=copy(store.credentialEdits),snapshot=store.snapshot,accept=vi.fn().mockResolvedValue(adopted(incoming))
    await expect(store.adoptTransfer(plan(incoming),accept)).rejects.toMatchObject({code:'transfer_draft_conflict'})
    expect(accept).not.toHaveBeenCalled();expect(store.channelDraft).toEqual(draft);expect(store.credentialEdits).toEqual(credentials)
    expect(store.snapshot).toBe(snapshot);expect(store.channelBaseline).toBe('actual-file');expect(store.busy).toBe(false)
  })

  it('accepts identical changes and independent channel additions in their existing gaps',async()=>{
    const store=useModelsStore(),incoming=actual().policy
    store.channelDraft!.channels[0]!.label='same edit';incoming.channels[0]!.label='same edit'
    store.channelDraft!.channels.splice(1,0,channel('local-added'));incoming.channels.splice(1,0,channel('import-added'))
    await store.adoptTransfer(plan(incoming),vi.fn().mockResolvedValue(adopted(incoming)))
    expect(store.channelDraft!.channels.map(c=>c.id)).toEqual(['a','local-added','import-added','b'])
    expect(store.channelDraft!.channels[0]!.label).toBe('same edit')
  })

  it('merges independent first edits to an absent model override and optional naming fields',async()=>{
    const store=useModelsStore(),base=actual()
    base.policy.channels[0]!.overrides={};delete base.policy.channels[0]!.naming;store.acceptTransfer(base)
    const incoming=copy(base.policy)
    store.channelDraft!.channels[0]!.overrides.model={...emptyRules(),fast:{mode:'custom',value:true}}
    store.channelDraft!.channels[0]!.naming={model:{alias:'local-alias'}}
    incoming.channels[0]!.overrides.model={...emptyRules(),contextWindow:{mode:'custom',value:128000}}
    incoming.channels[0]!.naming={model:{displayName:'Imported display name'}}
    await store.adoptTransfer(plan(incoming),vi.fn().mockResolvedValue({...adopted(incoming),policy:base.policy}))
    expect(store.channelDraft!.channels[0]!.overrides.model).toMatchObject({fast:{mode:'custom',value:true},contextWindow:{mode:'custom',value:128000}})
    expect(store.channelDraft!.channels[0]!.naming!.model).toEqual({alias:'local-alias',displayName:'Imported display name'})
  })

  it('merges repeated imports despite different read receipts for the same actual file digest',async()=>{
    const store=useModelsStore(),first=actual().policy
    first.channels[0]!.defaults.fast={mode:'custom',value:true};first.channels[0]!.connection.sourceReadId='first-comparison-read'
    const accepted=adopted(first);accepted.policy.channels[0]!.connection.sourceReadId='first-acceptance-read'
    await store.adoptTransfer(plan(first),vi.fn().mockResolvedValue(accepted))
    const second=actual().policy;second.channels[0]!.label='second import';second.channels[0]!.connection.sourceReadId='second-comparison-read'
    await store.adoptTransfer(plan(second),vi.fn().mockResolvedValue(adopted(second)))
    expect(store.channelDraft!.channels[0]).toMatchObject({label:'second import',defaults:{fast:{mode:'custom',value:true}}})
    expect(store.channelBaseline).toBe('actual-file')
  })
  it('treats sourceReadId as an exact model ID in naming, not a connection receipt',async()=>{
    const store=useModelsStore(),base=actual()
    base.policy.channels[0]!.discovered.push({id:'sourceReadId',label:'sourceReadId',rules:emptyRules()})
    base.policy.channels[0]!.naming={sourceReadId:{alias:'before'},model:{displayName:'before-model'}}
    store.acceptTransfer(base)
    store.channelDraft!.channels[0]!.naming!.sourceReadId!.alias='local-alias'
    const incoming=copy(base.policy)
    incoming.channels[0]!.naming!.model!.displayName='imported-model'
    await store.adoptTransfer(plan(incoming),vi.fn().mockResolvedValue({...adopted(incoming),policy:base.policy}))
    expect(store.channelDraft!.channels[0]!.naming!.sourceReadId!.alias).toBe('local-alias')
    expect(store.channelDraft!.channels[0]!.naming!.model!.displayName).toBe('imported-model')
  })
  it.each(['importedConfig','revisionId'])('merges independent naming edits for the exact model ID %s',async id=>{
    const store=useModelsStore(),base=actual()
    base.policy.channels[0]!.discovered.push({id,label:id,rules:emptyRules()})
    base.policy.channels[0]!.naming={[id]:{alias:'before',displayName:'before'}}
    store.acceptTransfer(base)
    store.channelDraft!.channels[0]!.naming![id]!.alias='local-alias'
    const incoming=copy(base.policy)
    incoming.channels[0]!.naming![id]!.displayName='imported display'
    await store.adoptTransfer(plan(incoming),vi.fn().mockResolvedValue({...adopted(incoming),policy:base.policy}))
    expect(store.channelDraft!.channels[0]!.naming![id]).toEqual({alias:'local-alias',displayName:'imported display'})
  })

  it('retains unrelated fields and replacement/clear Key input typed during IPC',async()=>{
    const store=useModelsStore(),incoming=actual().policy
    incoming.channels[0]!.defaults.fast={mode:'custom',value:true}
    let resolve!:(saved:ModelSnapshot)=>void
    const acceptance=store.adoptTransfer(plan(incoming),()=>new Promise(done=>{resolve=done}))
    store.channelDraft!.channels[0]!.label='typed while accepting'
    store.editCredential('a','newly-typed-key','old');store.clearCredential('b')
    resolve(adopted(incoming));await acceptance
    expect(store.channelDraft!.channels[0]).toMatchObject({label:'typed while accepting',defaults:{fast:{mode:'custom',value:true}}})
    expect(store.credentialEdits).toEqual({a:{channelId:'a',action:'replace',value:'newly-typed-key'},b:{channelId:'b',action:'clear',value:null}})
    expect(store.channelBaseline).toBe('actual-file');expect(store.busy).toBe(false)
  })

  it('leaves the whole live draft intact when late input overlaps the accepted workspace',async()=>{
    const store=useModelsStore(),incoming=actual().policy
    incoming.channels[0]!.label='incoming'
    let resolve!:(saved:ModelSnapshot)=>void
    const acceptance=store.adoptTransfer(plan(incoming),()=>new Promise(done=>{resolve=done}))
    store.channelDraft!.channels[0]!.label='late local input';store.clearCredential('a')
    const live=store.channelDraft
    resolve(adopted(incoming));await expect(acceptance).rejects.toMatchObject({code:'transfer_draft_conflict'})
    expect(store.channelDraft).toBe(live);expect(store.channelDraft!.channels[0]!.label).toBe('late local input')
    expect(store.credentialEdits.a?.action).toBe('clear');expect(store.channelBaseline).toBe('actual-file')
    expect(store.snapshot!.policy).toEqual(actual().policy);expect(store.snapshot!.libraryDigest).toBe('adopted-library')
  })

  it('updates only the library for template-only acceptance even if a legacy response carries a workspace',async()=>{
    const store=useModelsStore(),policy=store.snapshot!.policy,draft=store.channelDraft
    store.channelDraft!.channels[0]!.label='local';store.clearCredential('a');store.edit();store.draft!.label='manual template'
    const stale=adopted(actual().policy);stale.policyDigest='old-manager-policy';stale.policy.channels=[];stale.workspace!.basePolicy='old-manager-policy'
    await store.adoptTransfer({expectedPolicy:'actual-file',mergedPolicy:null},vi.fn().mockResolvedValue(stale))
    expect(store.snapshot!.policy).toBe(policy);expect(store.snapshot!.policyDigest).toBe('actual-file');expect(store.snapshot!.workspace).toBeNull()
    expect(store.snapshot!.workspaceDigest).toBe('no-disk-draft');expect(store.snapshot!.libraryDigest).toBe('adopted-library')
    expect(store.channelBaseline).toBe('actual-file');expect(store.channelDraft).toBe(draft);expect(store.channelDraft!.channels[0]!.label).toBe('local')
    expect(store.credentialEdits.a?.action).toBe('clear');expect(store.draft!.label).toBe('manual template');expect(store.channelDirty).toBe(true)
  })

  it('supports a template-only acceptance with no workspace and keeps a clean file draft clean',async()=>{
    const store=useModelsStore(),draft=store.channelDraft,policy=store.snapshot!.policy
    await store.adoptTransfer({expectedPolicy:'actual-file'},vi.fn().mockResolvedValue({...actual(),libraryDigest:'new-templates'}))
    expect(store.channelDraft).toBe(draft);expect(store.snapshot!.policy).toBe(policy);expect(store.channelDirty).toBe(false)
    expect(store.channelBaseline).toBe('actual-file');expect(store.snapshot!.libraryDigest).toBe('new-templates')
  })

  it('keeps raw-material-only adoption saveable until ordinary Save or Restore consumes it',async()=>{
    const store=useModelsStore(),incoming=actual().policy
    incoming.channels[0]!.connection.importedConfig={id:'raw-only',digest:'unknown-fields-changed'}
    await store.adoptTransfer(plan(incoming),vi.fn().mockResolvedValue(adopted(incoming)))
    expect(store.channelDraft!.channels[0]!.connection.importedConfig).toEqual({id:'raw-only',digest:'unknown-fields-changed'})
    expect(store.channelDirty).toBe(true);store.restoreChannels();expect(store.channelDirty).toBe(false)
    await store.adoptTransfer(plan(incoming),vi.fn().mockResolvedValue(adopted(incoming)))
    store.acceptTransfer(actual());expect(store.channelDirty).toBe(false)
  })

  it('blocks a stale preview before IPC and rejects a response in the legacy policy digest domain',async()=>{
    const store=useModelsStore(),incoming=actual().policy;incoming.channels[0]!.label='imported'
    const accept=vi.fn().mockResolvedValue(adopted(incoming))
    await expect(store.adoptTransfer({...plan(incoming),expectedPolicy:'old-file'},accept)).rejects.toMatchObject({message:expect.stringContaining('基准已变化')})
    expect(accept).not.toHaveBeenCalled()
    const legacy=adopted(incoming);legacy.workspace!.basePolicy='legacy-policy'
    await expect(store.adoptTransfer(plan(incoming),vi.fn().mockResolvedValue(legacy))).rejects.toMatchObject({message:expect.stringContaining('不符')})
    expect(store.channelDraft).toEqual(actual().policy);expect(store.channelBaseline).toBe('actual-file')
  })

  it('keeps historical recovery explicit and allows library-only changes without mixing channel candidates',async()=>{
    const store=useModelsStore(),recovery=actual().policy;recovery.recoveryId='recovery'
    vi.mocked(modelApi.acceptRecovery).mockResolvedValue({...adopted(recovery),policyDigest:'legacy-policy',workspace:{basePolicy:'legacy-policy',policy:recovery}})
    await store.acceptHistoricalRecovery('recovery','history-record')
    const historical=store.historicalRecovery,draft=store.channelDraft,accept=vi.fn().mockResolvedValue(adopted(actual().policy))
    await expect(store.adoptTransfer(plan(actual().policy),accept)).rejects.toMatchObject({message:expect.stringContaining('历史恢复')})
    expect(accept).not.toHaveBeenCalled();expect(store.historicalRecovery).toBe(historical);expect(store.channelDraft).toBe(draft)
    await store.adoptTransfer({expectedPolicy:'actual-file',mergedPolicy:null},vi.fn().mockResolvedValue({...actual(),libraryDigest:'templates-only'}))
    expect(store.historicalRecovery).toBe(historical);expect(store.channelDraft).toBe(draft);expect(store.channelBaseline).toBe('actual-file')
  })
})
