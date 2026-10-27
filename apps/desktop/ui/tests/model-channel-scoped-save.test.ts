import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {emptyContent,emptyRules,modelApi,type ChannelPolicy,type CredentialEdit,type DraftProjection,type ModelPolicy,type ModelSnapshot,type Template} from '@/features/models/api'
import {createModelRowsState,rowFields} from '@/features/models/modelRows'
import {useModelsStore} from '@/features/models/store'
import ChannelsView from '@/features/models/ChannelsView.vue'

vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{writeFile:vi.fn(),project:vi.fn(),load:vi.fn(),acceptRecovery:vi.fn(),preparePolicy:vi.fn(),applyPolicy:vi.fn(),policyResult:vi.fn(),latestPolicy:vi.fn(),policySaveBlocked:vi.fn(),executionDirectory:vi.fn()}}))
const copy=<T>(value:T):T=>JSON.parse(JSON.stringify(value))
function deferred<T>() {
  let resolve!:(value:T)=>void, reject!:(cause:unknown)=>void
  const promise=new Promise<T>((done,fail)=>{resolve=done;reject=fail})
  return {promise,resolve,reject}
}
function channel(id:string):ChannelPolicy {
  return {id,label:id,templateRefs:[],modelRefs:{},connection:{providerId:'provider-'+id,protocol:'openai-chat',nodes:[{id:'node-'+id,name:'主站',url:'https://'+id+'.example.test'}],activeNodeId:'node-'+id,credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:['model'],discovered:[{id:'model',label:'model',rules:emptyRules()}],defaults:emptyRules(),overrides:{},naming:{model:{}},familyOverrides:{}}
}
function template(id:string):Template {
  return {id,label:id,source:'user',defaultRevision:'revision',revisions:[{id:'revision',createdAt:'2026-10-27',savedLabel:id,content:emptyContent(),digest:'content-'+id,baseRevision:null,changes:[]}]}
}
function initial():ModelSnapshot {
  return {library:{schemaVersion:1,templates:[template('one'),template('two')]},libraryDigest:'library',policyDigest:'file-0',workspaceDigest:'workspace-0',workspace:null,policy:{schemaVersion:1,templateRefs:[{templateId:'one',revisionId:'revision'}],defaultChannelId:'b',channels:[channel('a'),channel('b'),channel('c')]}}
}
const projection=():DraftProjection=>({rows:[],knownActiveModels:3,exactCatalogCount:null,notices:[]})
let disk:ModelSnapshot,pinia:ReturnType<typeof createPinia>
function receipt(policy:ModelPolicy):{changed:boolean;snapshot:ModelSnapshot} {
  const saved=copy(policy)
  for (const current of saved.channels) delete current.connection.importedConfig
  disk={...copy(disk),policy:saved,policyDigest:'file-'+(Number(disk.policyDigest.split('-')[1])+1),workspace:null,workspaceDigest:'saved-workspace'}
  return {changed:true,snapshot:copy(disk)}
}
function pending(id:string) {
  const store=useModelsStore(),current=store.channelDraft!.channels.find(channel=>channel.id===id)!
  const state=createModelRowsState(),fields=rowFields(current,'model')
  state.buffers.model={...copy(fields),base:copy(fields),error:''}
  state.buffers.model.rules.fast={mode:'custom',value:true}
  store.modelRows[id]=state
  return store.modelRows[id]!
}
async function importRaw() {
  const store=useModelsStore(),incoming=copy(disk.policy)
  for (const id of ['a','b']) incoming.channels.find(channel=>channel.id===id)!.connection.importedConfig={id:'import-'+id,digest:'raw-'+id}
  await store.adoptTransfer({expectedPolicy:disk.policyDigest,mergedPolicy:incoming},async()=>({...copy(disk),workspace:{basePolicy:disk.policyDigest,policy:copy(incoming)},workspaceDigest:'import-workspace'}))
}
beforeEach(()=>{
  vi.restoreAllMocks();vi.resetAllMocks();pinia=createPinia();setActivePinia(pinia)
  disk=initial();useModelsStore().acceptTransfer(copy(disk))
  vi.mocked(modelApi.project).mockResolvedValue(projection())
  vi.mocked(modelApi.writeFile).mockImplementation(async(expected,policy)=>{expect(expected).toBe(disk.policyDigest);return receipt(policy)})
  vi.mocked(modelApi.load).mockImplementation(async()=>copy(disk))
  vi.mocked(modelApi.executionDirectory).mockResolvedValue({channels:[],rows:[]})
})

describe('global shared store save transactions',()=>{
  it('projects and writes the frozen full policy without scope, consuming only submitted Key intents',async()=>{
    const store=useModelsStore();await importRaw()
    const draft=store.channelDraft!,a=draft.channels[0]!,b=draft.channels[1]!
    a.label='saved A';b.label='saved B';draft.channels=[b,channel('new'),a]
    draft.templateRefs=[];draft.defaultChannelId='new'
    store.editCredential('a','new A','old A');store.clearCredential('b');store.clearCredential('removed-channel')
    const candidate=copy(draft),accept=vi.spyOn(store,'acceptTransfer')
    await expect(store.saveAllChannels(candidate,'file-0')).resolves.toEqual({changed:true})
    expect(modelApi.project).toHaveBeenCalledExactlyOnceWith(candidate)
    expect(modelApi.writeFile).toHaveBeenCalledExactlyOnceWith('file-0',candidate,[{channelId:'a',action:'replace',value:'new A'},{channelId:'b',action:'clear',value:null}])
    expect(accept).not.toHaveBeenCalled();expect(store.channelDraft).toBe(draft)
    expect(draft).toEqual(disk.policy);expect(draft.channels.every(channel=>!channel.connection.importedConfig)).toBe(true)
    expect(store.credentialEdits).toEqual({'removed-channel':{channelId:'removed-channel',action:'clear',value:null}})
    expect(store.snapshot).toEqual(disk);expect(store.channelBaseline).toBe('file-1');expect(store.busy).toBe(false)
    delete store.credentialEdits['removed-channel'];expect(store.channelDirty).toBe(false)
  })

  it('freezes before projection and preserves later discovery, raw, row, template, Key and order input through receipt and retry',async()=>{
    const store=useModelsStore();await importRaw()
    store.channelDraft!.channels[0]!.label='submitted A';store.editCredential('a','submitted Key','old Key');store.clearCredential('b')
    store.edit(store.snapshot!.library.templates[0]);store.draft!.label='buffer one';const firstSession=store.draftSession
    store.edit(store.snapshot!.library.templates[1]);store.draft!.label='buffer two';const template=store.draft,session=store.draftSession
    const frozen=copy(store.channelDraft!),sent=copy(frozen),preview=deferred<DraftProjection>(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    vi.mocked(modelApi.project).mockReturnValueOnce(preview.promise);vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveAllChannels(frozen,'file-0'),live=store.channelDraft!
    frozen.channels[0]!.label='caller mutated frozen argument'
    live.channels[0]!.label='typed during projection';store.clearCredential('a');store.editCredential('b','later B Key','old B Key')
    preview.resolve(projection());await flushPromises()
    expect(modelApi.project).toHaveBeenCalledWith(sent)
    expect(modelApi.writeFile).toHaveBeenCalledExactlyOnceWith('file-0',sent,[{channelId:'a',action:'replace',value:'submitted Key'},{channelId:'b',action:'clear',value:null}])
    const b=live.channels[1]!
    b.discovered.push({id:'late-discovery',label:'late-discovery',rules:emptyRules()});b.modelOrder.push('late-discovery')
    b.connection.importedConfig={id:'later-import',digest:'later raw'}
    const row=pending('b')
    live.channels.reverse();live.channels.shift();live.channels.push(channel('late-added'))
    live.templateRefs=[];live.defaultChannelId='late-added'
    const result=receipt(sent);result.snapshot.policy.channels[1]!.connection.nodes[0]!.name='normalized node'
    write.resolve(result);await saving
    expect(store.channelDraft).toBe(live);expect(live.channels.map(channel=>channel.id)).toEqual(['b','a','late-added'])
    expect(live.channels[1]!.label).toBe('typed during projection');expect(live.channels[1]!.connection.importedConfig).toBeUndefined()
    expect(live.channels[0]!.connection).toMatchObject({nodes:[{name:'normalized node'}],importedConfig:{id:'later-import',digest:'later raw'}})
    expect(live.channels[0]!.discovered.map(model=>model.id)).toEqual(['model','late-discovery'])
    expect(store.modelRows.b).toBe(row);expect(store.draft).toBe(template);expect(store.draftSession).toBe(session)
    expect(store.credentialEdits).toEqual({a:{channelId:'a',action:'clear',value:null},b:{channelId:'b',action:'replace',value:'later B Key'}})
    expect(store.snapshot!.policy).toEqual(result.snapshot.policy);expect(store.channelBaseline).toBe('file-1')
    expect(store.channelIsDirty('a')).toBe(true);expect(store.channelIsDirty('b')).toBe(true);expect(store.channelDirty).toBe(true)
    delete store.modelRows.b
    const retry=copy(live);await store.saveAllChannels(retry,'file-1')
    expect(modelApi.writeFile).toHaveBeenNthCalledWith(2,'file-1',retry,[{channelId:'a',action:'clear',value:null},{channelId:'b',action:'replace',value:'later B Key'}])
    expect(store.channelDirty).toBe(false);expect(store.credentialEdits).toEqual({});expect(store.dirty).toBe(true)
    store.edit(store.snapshot!.library.templates[0]);expect(store.draft!.label).toBe('buffer one');expect(store.draftSession).toBe(firstSession)
  })

  it.each(['same replacement','same clear','mutated intent'] as const)('retains a later %s instead of consuming the submitted Key',async kind=>{
    const store=useModelsStore(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    if(kind==='same clear')store.clearCredential('a');else store.editCredential('a','submitted','original')
    vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const frozen=copy(store.channelDraft!),saving=store.saveAllChannels(frozen,'file-0');await flushPromises()
    if(kind==='same clear')store.clearCredential('a')
    else if(kind==='same replacement')store.editCredential('a','submitted','original')
    else store.credentialEdits.a!.value='mutated after submission'
    const intent=store.credentialEdits.a
    write.resolve(receipt(frozen));await saving
    expect(store.credentialEdits.a).toBe(intent);expect(store.channelIsDirty('a')).toBe(true);expect(store.busy).toBe(false)
  })

  it('guards global/scoped reentry during both projection and writing',async()=>{
    const store=useModelsStore(),preview=deferred<DraftProjection>(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    store.clearCredential('a');const frozen=copy(store.channelDraft!)
    vi.mocked(modelApi.project).mockReturnValueOnce(preview.promise);vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveAllChannels(frozen,'file-0')
    await expect(store.saveAllChannels(frozen,'file-0')).rejects.toMatchObject({message:expect.stringContaining('正在处理')})
    await expect(store.saveCurrentChannel('a')).rejects.toMatchObject({message:expect.stringContaining('正在处理')})
    preview.resolve(projection());await flushPromises()
    await expect(store.saveAllChannels(frozen,'file-0')).rejects.toMatchObject({message:expect.stringContaining('正在处理')})
    expect(modelApi.project).toHaveBeenCalledOnce();expect(modelApi.writeFile).toHaveBeenCalledOnce();expect(store.busy).toBe(true)
    write.resolve(receipt(frozen));await saving;expect(store.busy).toBe(false)
    store.staging=true
    await expect(store.saveAllChannels(frozen,'file-1')).rejects.toMatchObject({message:expect.stringContaining('正在处理')})
    expect(store.busy).toBe(false)
  })

  it.each(['before','after projection'] as const)('blocks pending rows in any channel %s, preserving all input',async when=>{
    const store=useModelsStore(),preview=deferred<DraftProjection>()
    store.clearCredential('a');const frozen=copy(store.channelDraft!)
    if(when==='before')pending('c');else vi.mocked(modelApi.project).mockReturnValueOnce(preview.promise)
    const saving=store.saveAllChannels(frozen,'file-0')
    if(when!=='before'){pending('b');preview.resolve(projection())}
    await expect(saving).rejects.toMatchObject({message:expect.stringContaining('全部渠道未确认')})
    expect(modelApi.writeFile).not.toHaveBeenCalled();expect(store.pendingRows).toHaveLength(1)
    expect(store.channelBaseline).toBe('file-0');expect(store.credentialEdits.a).toBeDefined();expect(store.busy).toBe(false)
  })

  it.each(['candidate recovery','live recovery','explicit recovery','stale expected','late baseline','late snapshot','late history'] as const)('rejects %s before ordinary writing',async kind=>{
    const store=useModelsStore(),preview=deferred<DraftProjection>()
    if(kind==='explicit recovery'){
      const recovery={...copy(disk.policy),recoveryId:'restore-id'}
      vi.mocked(modelApi.acceptRecovery).mockResolvedValue({...copy(disk),policyDigest:'legacy',workspace:{basePolicy:'legacy',policy:recovery}})
      await store.acceptHistoricalRecovery('restore-id','history-id')
    }
    const frozen=copy(store.channelDraft!)
    if(kind==='candidate recovery')frozen.recoveryId='restore-id'
    if(kind==='live recovery')store.channelDraft!.recoveryId='restore-id'
    if(kind.startsWith('late'))vi.mocked(modelApi.project).mockReturnValueOnce(preview.promise)
    const saving=store.saveAllChannels(frozen,kind==='stale expected'?'stale':'file-0')
    if(kind==='late baseline')store.channelBaseline='another-file'
    if(kind==='late snapshot')store.snapshot!.policyDigest='another-file'
    if(kind==='late history')store.channelDraft!.recoveryId='restore-id'
    preview.resolve(projection())
    await expect(saving).rejects.toMatchObject({message:expect.stringContaining(kind.includes('recovery')||kind==='late history'?'历史恢复':'保存基准')})
    expect(modelApi.writeFile).not.toHaveBeenCalled();expect(modelApi.preparePolicy).not.toHaveBeenCalled();expect(modelApi.applyPolicy).not.toHaveBeenCalled();expect(store.busy).toBe(false)
    if(kind==='explicit recovery')expect(store.historicalRecovery?.id).toBe('restore-id')
  })

  it.each(['preview error','over budget','write error'] as const)('retains baseline, raw material and credentials on %s, then permits retry',async failure=>{
    const store=useModelsStore();await importRaw();store.clearCredential('a');store.editCredential('b','new B','old B')
    const frozen=copy(store.channelDraft!),draft=store.channelDraft,snapshot=store.snapshot,keys=copy(store.credentialEdits)
    if(failure==='preview error')vi.mocked(modelApi.project).mockRejectedValueOnce({message:'配置校验失败'})
    else if(failure==='over budget')vi.mocked(modelApi.project).mockResolvedValueOnce({...projection(),knownActiveModels:101})
    else vi.mocked(modelApi.writeFile).mockRejectedValueOnce({message:'文件写入失败'})
    await expect(store.saveAllChannels(frozen,'file-0')).rejects.toMatchObject({message:expect.stringContaining(failure==='preview error'?'校验失败':failure==='over budget'?'100':'写入失败')})
    expect(store.channelDraft).toBe(draft);expect(draft).toEqual(frozen);expect(store.snapshot).toBe(snapshot)
    expect(store.channelBaseline).toBe('file-0');expect(store.credentialEdits).toEqual(keys);expect(store.busy).toBe(false)
    if(failure!=='write error')expect(modelApi.writeFile).not.toHaveBeenCalled()
    vi.mocked(modelApi.project).mockResolvedValueOnce({...projection(),knownActiveModels:100})
    await store.saveAllChannels(frozen,'file-0');expect(store.channelDirty).toBe(false);expect(store.busy).toBe(false)
  })

  it('accepts a metadata-only global receipt without changing the file digest or dropping later edits',async()=>{
    const store=useModelsStore(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    store.channelDraft!.channels[0]!.label='metadata A'
    const frozen=copy(store.channelDraft!);vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveAllChannels(frozen,'file-0');await flushPromises()
    store.channelDraft!.channels[1]!.label='late B';pending('c')
    write.resolve({changed:false,snapshot:{...copy(disk),policy:copy(frozen)}})
    await expect(saving).resolves.toEqual({changed:false})
    expect(store.channelBaseline).toBe('file-0');expect(store.channelIsDirty('a')).toBe(false)
    expect(store.channelDraft!.channels[1]!.label).toBe('late B');expect(store.channelIsDirty('b')).toBe(true)
    expect(store.pendingRows).toEqual([{channel:'c',model:'model'}]);expect(store.busy).toBe(false)
  })
})
afterEach(()=>pinia._s.forEach(store=>store.$dispose()))

describe('channel-scoped shared store saves',()=>{
  it('compares normalized individual channels and ignores shared refs, order and read receipts',()=>{
    const store=useModelsStore()
    store.channelDraft!.channels.reverse()
    store.channelDraft!.templateRefs=[];store.channelDraft!.defaultChannelId='a'
    const a=store.channelDraft!.channels.find(channel=>channel.id==='a')!
    a.connection={...a.connection,sourceReadId:'another-read',importedConfig:{id:'read-only',digest:'read-only'}}
    a.naming={model:{displayName:undefined}}
    expect(store.channelIsDirty('a')).toBe(false);expect(store.channelIsDirty('b')).toBe(false)
    expect(store.channelDirty).toBe(true)
    store.channelDraft!.channels.find(channel=>channel.id==='b')!.label='edited B'
    expect(store.channelIsDirty('a')).toBe(false);expect(store.channelIsDirty('b')).toBe(true)
    store.editCredential('a','new A','old A');expect(store.channelIsDirty('a')).toBe(true)
    store.editCredential('a','old A','old A');expect(store.channelIsDirty('a')).toBe(false)
  })

  it.each([['replace','clear'],['clear','replace']] as const)('consumes only A %s while retaining B %s, then saves B from the new file baseline',async(aAction,bAction)=>{
    const store=useModelsStore(),edits=store.credentialEdits
    for (const [id,action] of [['a',aAction],['b',bAction]] as const) {
      if(action==='clear')store.clearCredential(id);else store.editCredential(id,'new-'+id,'old-'+id)
    }
    const bIntent=store.credentialEdits.b
    const accept=vi.spyOn(store,'acceptTransfer')
    await expect(store.saveCurrentChannel('a')).resolves.toEqual({changed:true})
    expect(modelApi.writeFile).toHaveBeenNthCalledWith(1,'file-0',initial().policy,[{channelId:'a',action:aAction,value:aAction==='clear'?null:'new-a'}],{channelId:'a'})
    expect(accept).not.toHaveBeenCalled();expect(store.credentialEdits).toBe(edits)
    expect(store.credentialEdits.a).toBeUndefined();expect(store.credentialEdits.b).toBe(bIntent)
    expect(store.channelIsDirty('a')).toBe(false);expect(store.channelIsDirty('b')).toBe(true)
    expect(store.channelDirty).toBe(true);expect(store.channelBaseline).toBe('file-1')
    await expect(store.saveCurrentChannel('a')).resolves.toEqual({changed:false})
    expect(modelApi.writeFile).toHaveBeenCalledTimes(1);expect(modelApi.project).toHaveBeenCalledTimes(1)
    await store.saveCurrentChannel('b')
    expect(vi.mocked(modelApi.writeFile).mock.calls[1]).toEqual(['file-1',disk.policy,[{channelId:'b',action:bAction,value:bAction==='clear'?null:'new-b'}],{channelId:'b'}])
    expect(store.channelDirty).toBe(false);expect(store.busy).toBe(false)
  })

  it('builds a detached full saved-base candidate, preserving saved order, shared refs and unrelated additions/deletions',async()=>{
    const store=useModelsStore(),draft=store.channelDraft!
    const a=draft.channels[0]!,b=draft.channels[1]!
    a.label='save A';b.label='leave B'
    draft.channels=[channel('new-other'),b,channel('new-current'),a]
    draft.templateRefs=[];draft.defaultChannelId='new-other'
    draft.pendingRemovals=[{readId:'read',channelId:'removed-import',providerId:'removed',credentialRef:null}]
    const before=copy(draft),candidate=store.channelSaveCandidate('a')
    expect(candidate).toEqual({...initial().policy,channels:[copy(a),channel('b'),channel('c')]})
    candidate.channels[0]!.label='detached';expect(a.label).toBe('save A')
    await store.saveCurrentChannel('a')
    expect(store.channelDraft).toBe(draft);expect(store.channelDraft).toEqual(before)
    expect(store.channelIsDirty('a')).toBe(false);expect(store.channelDirty).toBe(true)
    const newCandidate=store.channelSaveCandidate('new-current')
    expect(newCandidate.channels.map(channel=>channel.id)).toEqual(['a','b','new-current','c'])
    expect(newCandidate.channels[0]!.label).toBe('save A')
    await store.saveCurrentChannel('new-current')
    expect(store.channelDraft!.channels.map(channel=>channel.id)).toEqual(['new-other','b','new-current','a'])
    expect(store.channelIsDirty('new-current')).toBe(false);expect(store.channelDirty).toBe(true)
    await store.saveCurrentChannel('new-current');expect(modelApi.writeFile).toHaveBeenCalledTimes(2)
  })

  it.each([
    {draft:['b','other-new','new-current','a','c'],saved:['a','b','new-current','c']},
    {draft:['c','new-current','b','a'],saved:['a','b','c','new-current']},
    {draft:['other-new','new-current','c','b','a'],saved:['new-current','a','b','c']},
    {draft:['a','new-current','c'],saved:['a','new-current','b','c']},
  ])('anchors a new channel in $draft after its nearest saved predecessor, leaving global order dirty',async({draft,saved})=>{
    const store=useModelsStore(),original=store.channelDraft!
    const channels=new Map(original.channels.map(channel=>[channel.id,channel]))
    original.channels=draft.map(id=>channels.get(id)??channel(id))
    const candidate=store.channelSaveCandidate('new-current')
    expect(candidate.channels.map(channel=>channel.id)).toEqual(saved)
    expect(candidate.channels.filter(channel=>channel.id!=='new-current')).toEqual(initial().policy.channels)
    expect(candidate.templateRefs).toEqual(initial().policy.templateRefs)
    await store.saveCurrentChannel('new-current')
    expect(store.snapshot!.policy.channels.map(channel=>channel.id)).toEqual(saved)
    expect(store.channelDraft).toBe(original);expect(original.channels.map(channel=>channel.id)).toEqual(draft)
    expect(store.channelIsDirty('new-current')).toBe(false);expect(store.channelDirty).toBe(true)
    await store.saveCurrentChannel('new-current');expect(modelApi.writeFile).toHaveBeenCalledOnce()
  })

  it('preserves active and retained template buffers, sessions, optimistic baselines and model-row state',async()=>{
    const store=useModelsStore(),[one,two]=store.snapshot!.library.templates
    store.edit(one);store.draft!.label='manual one';const firstSession=store.draftSession
    store.view.agentViews[firstSession]={initialized:true} as typeof store.view.agentViews[string]
    store.edit(two);store.draft!.label='manual two'
    const active=store.draft,session=store.draftSession,rows=pending('b'),allRows=store.modelRows
    store.channelDraft!.channels[0]!.label='save A'
    await store.saveCurrentChannel('a')
    expect(store.draft).toBe(active);expect(store.draftSession).toBe(session);expect(store.dirty).toBe(true)
    expect(store.modelRows).toBe(allRows);expect(store.modelRows.b).toBe(rows)
    expect(store.pendingRows).toEqual([{channel:'b',model:'model'}])
    store.edit(one);expect(store.draft!.label).toBe('manual one');expect(store.draftSession).toBe(firstSession)
    expect(store.protectsRevision('two','revision')).toBe(true)
    expect(store.view.agentViews[firstSession]).toMatchObject({initialized:true})
    expect(modelApi.project).toHaveBeenCalledWith({...initial().policy,channels:[{...channel('a'),label:'save A'},channel('b'),channel('c')]})
  })

  it('blocks only the current unconfirmed row and rechecks rows after projection',async()=>{
    const store=useModelsStore();store.channelDraft!.channels[0]!.label='save A'
    pending('a');pending('b')
    await expect(store.saveCurrentChannel('a')).rejects.toMatchObject({message:expect.stringContaining('当前渠道未确认')})
    expect(modelApi.project).not.toHaveBeenCalled();expect(modelApi.writeFile).not.toHaveBeenCalled();expect(store.busy).toBe(false)
    delete store.modelRows.a
    const preview=deferred<DraftProjection>();vi.mocked(modelApi.project).mockReturnValueOnce(preview.promise)
    const saving=store.saveCurrentChannel('a');pending('a');preview.resolve(projection())
    await expect(saving).rejects.toMatchObject({message:expect.stringContaining('当前渠道未确认')})
    expect(modelApi.writeFile).not.toHaveBeenCalled();expect(store.busy).toBe(false)
    delete store.modelRows.a;await store.saveCurrentChannel('a')
    expect(store.modelRows.b).toBeDefined();expect(store.channelIsDirty('a')).toBe(false)
  })

  it('treats adopted raw-only receipts as per-channel intent and consumes only the saved receipt',async()=>{
    const store=useModelsStore();await importRaw()
    expect(store.channelIsDirty('a')).toBe(true);expect(store.channelIsDirty('b')).toBe(true)
    const b=store.channelDraft!.channels[1]!,bReceipt=b.connection.importedConfig
    await store.saveCurrentChannel('a')
    const [expected,payload,credentials,scope]=vi.mocked(modelApi.writeFile).mock.calls[0]!
    expect(expected).toBe('file-0');expect(scope).toEqual({channelId:'a'});expect(credentials).toEqual([])
    expect(payload.channels[0]!.connection.importedConfig).toEqual({id:'import-a',digest:'raw-a'})
    expect(payload.channels[1]!.connection.importedConfig).toBeUndefined()
    expect(store.channelDraft!.channels[0]!.connection.importedConfig).toBeUndefined()
    expect(store.channelDraft!.channels[1]).toBe(b);expect(b.connection.importedConfig).toBe(bReceipt)
    expect(store.channelIsDirty('a')).toBe(false);expect(store.channelDirty).toBe(true)
    await store.saveCurrentChannel('a');expect(modelApi.writeFile).toHaveBeenCalledOnce()
    await store.saveCurrentChannel('b');expect(store.channelDirty).toBe(false)
    await store.load();expect(store.channelDirty).toBe(false);expect(store.channelBaseline).toBe('file-2')
  })

  it('freezes policy and Key before projection and retains later field, order, row and Key input',async()=>{
    const store=useModelsStore(),preview=deferred<DraftProjection>(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    store.channelDraft!.channels[0]!.label='submitted A';store.editCredential('a','submitted-key','old-key')
    vi.mocked(modelApi.project).mockReturnValueOnce(preview.promise);vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveCurrentChannel('a');expect(store.busy).toBe(true)
    const live=store.channelDraft!,a=live.channels[0]!
    a.label='later A';a.naming!.model={alias:'later alias'};a.connection.nodes[0]!.name='later node'
    store.clearCredential('a');store.editCredential('b','later B key','old B')
    live.channels.reverse();live.channels.push(channel('later-added'));live.templateRefs=[]
    preview.resolve(projection());await flushPromises()
    const [,payload,keys,scope]=vi.mocked(modelApi.writeFile).mock.calls[0]!
    expect(payload.channels[0]!.label).toBe('submitted A');expect(payload.channels.map(channel=>channel.id)).toEqual(['a','b','c'])
    expect(keys).toEqual([{channelId:'a',action:'replace',value:'submitted-key'}]);expect(scope).toEqual({channelId:'a'})
    const row=pending('a'),result=receipt(payload)
    result.snapshot.policy.channels[0]!.connection.authMode='key'
    write.resolve(result);await saving
    expect(store.channelDraft).toBe(live);expect(live.channels.map(channel=>channel.id)).toEqual(['c','b','a','later-added'])
    expect(live.channels[2]).toMatchObject({label:'later A',naming:{model:{alias:'later alias'}},connection:{authMode:'key',nodes:[{name:'later node'}]}})
    expect(store.credentialEdits).toEqual({a:{channelId:'a',action:'clear',value:null},b:{channelId:'b',action:'replace',value:'later B key'}})
    expect(store.modelRows.a).toBe(row);expect(store.channelIsDirty('a')).toBe(true);expect(store.channelDirty).toBe(true)
    expect(store.snapshot!.policy.channels[0]!.label).toBe('submitted A');expect(store.channelBaseline).toBe('file-1');expect(store.busy).toBe(false)
  })

  it.each(['replacement','clear','same replacement'] as const)('keeps a later %s Key intent instead of consuming it with the old submission',async kind=>{
    const store=useModelsStore(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    store.editCredential('a','submitted','old')
    vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveCurrentChannel('a');await flushPromises()
    if(kind==='clear')store.clearCredential('a');else store.editCredential('a',kind==='replacement'?'later':'submitted','old')
    const later=store.credentialEdits.a
    write.resolve(receipt(vi.mocked(modelApi.writeFile).mock.calls[0]![1]));await saving
    expect(store.credentialEdits.a).toBe(later);expect(store.channelIsDirty('a')).toBe(true)
  })

  it('preserves a later raw receipt and consumes the old raw receipt even when another field changes',async()=>{
    const store=useModelsStore();await importRaw()
    const write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveCurrentChannel('a');await flushPromises()
    store.channelDraft!.channels[0]!.connection.importedConfig={id:'later-import',digest:'later-raw'}
    write.resolve(receipt(vi.mocked(modelApi.writeFile).mock.calls[0]![1]));await saving
    expect(store.channelDraft!.channels[0]!.connection.importedConfig).toEqual({id:'later-import',digest:'later-raw'})
    expect(store.channelIsDirty('a')).toBe(true)
    const second=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    vi.mocked(modelApi.writeFile).mockReturnValueOnce(second.promise)
    const retry=store.saveCurrentChannel('a');await flushPromises();store.channelDraft!.channels[0]!.label='late label'
    second.resolve(receipt(vi.mocked(modelApi.writeFile).mock.calls[1]![1]));await retry
    expect(store.channelDraft!.channels[0]!.connection.importedConfig).toBeUndefined()
    expect(store.channelDraft!.channels[0]!.label).toBe('late label');expect(store.channelIsDirty('a')).toBe(true)
  })

  it('does not resurrect a channel removed while its save is pending',async()=>{
    const store=useModelsStore(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    store.channelDraft!.channels[0]!.label='submitted';vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveCurrentChannel('a');await flushPromises()
    store.channelDraft!.channels.shift()
    write.resolve(receipt(vi.mocked(modelApi.writeFile).mock.calls[0]![1]));await saving
    expect(store.channelDraft!.channels.map(channel=>channel.id)).toEqual(['b','c'])
    expect(store.channelIsDirty('a')).toBe(true);expect(store.channelDirty).toBe(true)
    await expect(store.saveCurrentChannel('a')).rejects.toMatchObject({message:expect.stringContaining('全部渠道保存')})
    expect(modelApi.writeFile).toHaveBeenCalledOnce();expect(store.busy).toBe(false)
  })

  it('owns the busy guard across preview and write, rejecting duplicate or competing scope submissions',async()=>{
    const store=useModelsStore(),preview=deferred<DraftProjection>(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    store.clearCredential('a');store.clearCredential('b')
    vi.mocked(modelApi.project).mockReturnValueOnce(preview.promise);vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveCurrentChannel('a')
    await expect(store.saveCurrentChannel('a')).rejects.toMatchObject({message:expect.stringContaining('正在处理')})
    preview.resolve(projection());await flushPromises()
    await expect(store.saveCurrentChannel('b')).rejects.toMatchObject({message:expect.stringContaining('正在处理')})
    expect(modelApi.project).toHaveBeenCalledOnce();expect(modelApi.writeFile).toHaveBeenCalledOnce()
    write.resolve(receipt(vi.mocked(modelApi.writeFile).mock.calls[0]![1]));await saving
    await store.saveCurrentChannel('a');expect(modelApi.writeFile).toHaveBeenCalledOnce();expect(store.busy).toBe(false)
    store.staging=true
    await expect(store.saveCurrentChannel('b')).rejects.toMatchObject({message:expect.stringContaining('正在处理')})
    expect(store.busy).toBe(false)
  })

  it.each(['preview','write'] as const)('preserves all drafts, baselines, Key and raw intents on %s failure and permits retry',async phase=>{
    const store=useModelsStore();await importRaw()
    store.channelDraft!.channels[0]!.label='local A';store.clearCredential('a');store.editCredential('b','local B','old B')
    pending('b');store.edit(store.snapshot!.library.templates[0]);store.draft!.label='manual template'
    const draft=store.channelDraft,before=copy(draft),snapshot=store.snapshot,keys=copy(store.credentialEdits),rows=store.modelRows
    const cause={code:'file_changed',message:'配置文件已改变'}
    if(phase==='preview')vi.mocked(modelApi.project).mockRejectedValueOnce(cause)
    else vi.mocked(modelApi.writeFile).mockRejectedValueOnce(cause)
    await expect(store.saveCurrentChannel('a')).rejects.toBe(cause)
    expect(store.snapshot).toBe(snapshot);expect(store.channelDraft).toBe(draft);expect(store.channelDraft).toEqual(before)
    expect(store.credentialEdits).toEqual(keys);expect(store.modelRows).toBe(rows);expect(store.draft!.label).toBe('manual template')
    expect(store.channelBaseline).toBe('file-0');expect(store.channelIsDirty('a')).toBe(true);expect(store.busy).toBe(false)
    await store.saveCurrentChannel('a');expect(store.channelIsDirty('a')).toBe(false);expect(store.channelIsDirty('b')).toBe(true)
  })

  it('validates the saved-base candidate budget and rejects preview errors before writing',async()=>{
    const store=useModelsStore();store.clearCredential('a')
    vi.mocked(modelApi.project).mockResolvedValueOnce({...projection(),knownActiveModels:101})
    await expect(store.saveCurrentChannel('a')).rejects.toMatchObject({message:expect.stringContaining('100')})
    expect(modelApi.writeFile).not.toHaveBeenCalled();expect(store.credentialEdits.a).toBeDefined();expect(store.busy).toBe(false)
    vi.mocked(modelApi.project).mockResolvedValueOnce({...projection(),knownActiveModels:100})
    await store.saveCurrentChannel('a');expect(store.channelDirty).toBe(false)
  })

  it('keeps no-change backend receipts clean without rewriting the same scope',async()=>{
    const store=useModelsStore();store.clearCredential('a')
    vi.mocked(modelApi.writeFile).mockResolvedValueOnce({changed:false,snapshot:copy(disk)})
    await expect(store.saveCurrentChannel('a')).resolves.toEqual({changed:false})
    expect(store.credentialEdits).toEqual({});expect(store.channelDirty).toBe(false)
    await expect(store.saveCurrentChannel('a')).resolves.toEqual({changed:false})
    expect(modelApi.writeFile).toHaveBeenCalledOnce();expect(store.busy).toBe(false)
  })

  it('absorbs the latest locked layout from a metadata-only A receipt even when the file digest is unchanged',async()=>{
    const store=useModelsStore(),draft=store.channelDraft!
    draft.channels[0]!.label='saved metadata A'
    const latest=copy(disk)
    latest.policy.channels[1]!.label='newly saved B'
    latest.policy.channels[1]!.connection.nodes[0]!.name='latest B node'
    latest.policy.channels[1]!.templateRefs=[{templateId:'two',revisionId:'revision'}]
    latest.policy.templateRefs=[];latest.policy.channels.reverse()
    vi.mocked(modelApi.writeFile).mockImplementationOnce(async(expected,policy,credentials,scope)=>{
      expect(expected).toBe('file-0');expect(scope).toEqual({channelId:'a'});expect(credentials).toEqual([])
      // Like the backend's latest_locked_layout test: the stale B and global
      // metadata in this request never replace the layout held by its lock.
      expect(policy.channels[1]!.label).toBe('b');expect(policy.templateRefs).toEqual(initial().policy.templateRefs)
      latest.policy.channels[2]=copy(policy.channels[0]!)
      disk=copy(latest);return {changed:false,snapshot:copy(latest)}
    })
    await expect(store.saveCurrentChannel('a')).resolves.toEqual({changed:false})
    expect(store.channelDraft).toBe(draft);expect(draft).toEqual(latest.policy)
    expect(store.channelBaseline).toBe('file-0');expect(store.channelDirty).toBe(false)
    expect(store.channelIsDirty('b')).toBe(false)
    await store.saveCurrentChannel('a');expect(modelApi.writeFile).toHaveBeenCalledOnce()
    const b=draft.channels.find(channel=>channel.id==='b')!
    b.defaults.fast={mode:'custom',value:true}
    const wrapper=mount(ChannelsView,{global:{stubs:{UiDialog:{props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'},ChannelEditor:true,ChannelNodeSelect:true,UiSelectField:true}}})
    try{
      await flushPromises();await wrapper.get('[data-testid="policy-save"]').trigger('click');await flushPromises()
      await wrapper.findAll('button').find(button=>button.text()==='确认保存')!.trigger('click');await flushPromises()
      const [expected,policy,,scope]=vi.mocked(modelApi.writeFile).mock.calls[1]!
      expect(expected).toBe('file-0');expect(scope).toBeUndefined()
      expect(policy.channels.find(channel=>channel.id==='b')).toMatchObject({label:'newly saved B',connection:{nodes:[{name:'latest B node'}]},templateRefs:[{templateId:'two',revisionId:'revision'}],defaults:{fast:{mode:'custom',value:true}}})
      expect(store.channelDirty).toBe(false)
    }finally{wrapper.unmount()}
  })

  it.each(['replace','clear'] as const)('rebases unedited B fields while preserving local B fields and its %s Key intent',async action=>{
    const store=useModelsStore(),b=store.channelDraft!.channels[1]!
    store.channelDraft!.channels[0]!.label='submitted A';b.label='local B'
    b.connection.nodes[0]!.url='https://local-b.example.test'
    b.defaults.fast={mode:'custom',value:true}
    if(action==='clear')store.clearCredential('b');else store.editCredential('b','local B Key','old B Key')
    const key=store.credentialEdits.b
    vi.mocked(modelApi.writeFile).mockImplementationOnce(async(_expected,policy,_credentials,scope)=>{
      expect(scope).toEqual({channelId:'a'});expect(policy.channels[1]).toEqual(initial().policy.channels[1])
      const result=receipt(policy),latestB=result.snapshot.policy.channels[1]!
      latestB.label='external B';latestB.connection.nodes[0]!.name='external node name'
      latestB.templateRefs=[{templateId:'two',revisionId:'revision'}]
      return result
    })
    await store.saveCurrentChannel('a')
    expect(store.channelDraft!.channels[1]).toMatchObject({label:'local B',connection:{nodes:[{name:'external node name',url:'https://local-b.example.test'}]},defaults:{fast:{mode:'custom',value:true}},templateRefs:[{templateId:'two',revisionId:'revision'}]})
    expect(store.credentialEdits.b).toBe(key);expect(store.channelIsDirty('b')).toBe(true)
    expect(store.channelIsDirty('a')).toBe(false);expect(store.channelDirty).toBe(true)
  })

  it('retains B row, raw import and template buffers plus global ordering/add/delete intent across a latest-layout receipt',async()=>{
    const store=useModelsStore();await importRaw()
    const rows=pending('b'),raw=store.channelDraft!.channels[1]!.connection.importedConfig
    store.edit(store.snapshot!.library.templates[0]);store.draft!.label='retained one';const firstSession=store.draftSession
    store.edit(store.snapshot!.library.templates[1]);store.draft!.label='active two';const active=store.draft,session=store.draftSession
    const a=store.channelDraft!.channels[0]!,b=store.channelDraft!.channels[1]!
    a.label='submitted A';store.channelDraft!.channels=[b,channel('new-local'),a]
    store.channelDraft!.templateRefs=[];store.channelDraft!.defaultChannelId='new-local'
    store.channelDraft!.pendingRemovals=[{readId:'old-import',channelId:'removed-local',providerId:'removed',credentialRef:null}]
    vi.mocked(modelApi.writeFile).mockImplementationOnce(async(_expected,policy)=>{
      const result=receipt(policy);result.snapshot.policy.channels[1]!.label='latest B'
      result.snapshot.policy.templateRefs=[{templateId:'two',revisionId:'revision'}]
      result.snapshot.policy.channels.reverse();return result
    })
    await store.saveCurrentChannel('a')
    expect(store.channelDraft!.channels.map(channel=>channel.id)).toEqual(['b','new-local','a'])
    expect(store.channelDraft!.channels[0]!.label).toBe('latest B')
    expect(store.channelDraft!.channels[0]!.connection.importedConfig).toEqual(raw)
    expect(store.modelRows.b).toBe(rows);expect(store.pendingRows).toEqual([{channel:'b',model:'model'}])
    expect(store.draft).toBe(active);expect(store.draftSession).toBe(session);expect(store.dirty).toBe(true)
    store.edit(store.snapshot!.library.templates[0]);expect(store.draft!.label).toBe('retained one');expect(store.draftSession).toBe(firstSession)
    expect(store.channelDraft!.templateRefs).toEqual([]);expect(store.channelDraft!.defaultChannelId).toBe('new-local')
    expect(store.channelDraft!.pendingRemovals).toHaveLength(1)
    expect(store.channelIsDirty('a')).toBe(false);expect(store.channelIsDirty('b')).toBe(true);expect(store.channelDirty).toBe(true)
  })

  it('preserves later B field/deletion/order input while accepting independent latest B fields',async()=>{
    const store=useModelsStore(),preview=deferred<DraftProjection>(),write=deferred<Awaited<ReturnType<typeof modelApi.writeFile>>>()
    store.channelDraft!.channels[0]!.label='submitted A'
    vi.mocked(modelApi.project).mockReturnValueOnce(preview.promise);vi.mocked(modelApi.writeFile).mockReturnValueOnce(write.promise)
    const saving=store.saveCurrentChannel('a')
    const b=store.channelDraft!.channels[1]!
    b.label='typed during projection';b.connection.nodes[0]!.url='https://late.example.test'
    store.clearCredential('b');preview.resolve(projection());await flushPromises()
    b.naming!.model={alias:'typed during write'};store.channelDraft!.channels.reverse();store.channelDraft!.channels.shift()
    const result=receipt(vi.mocked(modelApi.writeFile).mock.calls[0]![1])
    result.snapshot.policy.channels[1]!.label='latest B'
    result.snapshot.policy.channels[1]!.connection.nodes[0]!.name='latest B node'
    result.snapshot.policy.channels[1]!.templateRefs=[{templateId:'two',revisionId:'revision'}]
    write.resolve(result);await saving
    expect(store.channelDraft!.channels.map(channel=>channel.id)).toEqual(['b','a'])
    expect(store.channelDraft!.channels[0]).toMatchObject({label:'typed during projection',connection:{nodes:[{name:'latest B node',url:'https://late.example.test'}]},naming:{model:{alias:'typed during write'}},templateRefs:[{templateId:'two',revisionId:'revision'}]})
    expect(store.credentialEdits.b?.action).toBe('clear');expect(store.channelIsDirty('b')).toBe(true);expect(store.channelDirty).toBe(true)
  })

  it('merges independent first edits to optional B naming maps without combining exact template references',async()=>{
    const store=useModelsStore(),base=initial()
    delete base.policy.channels[1]!.naming
    base.policy.channels[1]!.modelRefs.model={templateId:'one',revisionId:'original'}
    store.acceptTransfer(base)
    store.channelDraft!.channels[0]!.label='submitted A'
    store.channelDraft!.channels[1]!.naming={model:{alias:'local alias'}}
    store.channelDraft!.channels[1]!.modelRefs.model={templateId:'two',revisionId:'original'}
    vi.mocked(modelApi.writeFile).mockImplementationOnce(async(_expected,policy)=>{
      const result=receipt(policy)
      result.snapshot.policy.channels[1]!.naming={model:{displayName:'latest display name'}}
      result.snapshot.policy.channels[1]!.modelRefs.model={templateId:'one',revisionId:'latest'}
      return result
    })
    await store.saveCurrentChannel('a')
    expect(store.channelDraft!.channels[1]!.naming).toEqual({model:{alias:'local alias',displayName:'latest display name'}})
    expect(store.channelDraft!.channels[1]!.modelRefs.model).toEqual({templateId:'two',revisionId:'original'})
    expect(store.channelIsDirty('b')).toBe(true)
  })

  it.each(['explicit recovery','provenance only'] as const)('rejects %s through a clear historical-flow message without consuming anything',async kind=>{
    const store=useModelsStore(),recovery={...copy(disk.policy),recoveryId:'restore-id'}
    if(kind==='explicit recovery'){
      vi.mocked(modelApi.acceptRecovery).mockResolvedValue({...copy(disk),policyDigest:'legacy-policy',workspaceDigest:'legacy-workspace',workspace:{basePolicy:'legacy-policy',policy:recovery}})
      await store.acceptHistoricalRecovery('restore-id','history-id')
    }else store.channelDraft=recovery
    store.clearCredential('b')
    const historical=store.historicalRecovery,draft=store.channelDraft,snapshot=store.snapshot
    expect(()=>store.channelSaveCandidate('a')).toThrow(expect.objectContaining({message:expect.stringContaining('原历史恢复保存流程')}))
    await expect(store.saveCurrentChannel('a')).rejects.toMatchObject({message:expect.stringContaining('原历史恢复保存流程')})
    expect(store.historicalRecovery).toBe(historical);expect(store.channelDraft).toBe(draft);expect(store.snapshot).toBe(snapshot)
    expect(store.credentialEdits.b).toBeDefined();expect(store.busy).toBe(false)
    expect(modelApi.project).not.toHaveBeenCalled();expect(modelApi.writeFile).not.toHaveBeenCalled()
    expect(modelApi.preparePolicy).not.toHaveBeenCalled();expect(modelApi.applyPolicy).not.toHaveBeenCalled();expect(modelApi.latestPolicy).not.toHaveBeenCalled()
  })

  it('uses the scoped receipt as the next global-save baseline while retaining all global draft changes',async()=>{
    const store=useModelsStore();await importRaw()
    store.channelDraft!.channels[0]!.label='saved A';store.editCredential('a','A replacement','A original');store.clearCredential('b')
    await store.saveCurrentChannel('a')
    const a=store.channelDraft!.channels[0]!,b=store.channelDraft!.channels[1]!
    b.label='global B';store.channelDraft!.channels=[b,channel('new'),a]
    store.channelDraft!.templateRefs=[];store.channelDraft!.defaultChannelId='new'
    const draft=copy(store.channelDraft)
    const wrapper=mount(ChannelsView,{global:{stubs:{UiDialog:{props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'},ChannelEditor:true,ChannelNodeSelect:true,UiSelectField:true}}})
    try{
      await flushPromises();await wrapper.get('[data-testid="policy-save"]').trigger('click');await flushPromises()
      await wrapper.findAll('button').find(button=>button.text()==='确认保存')!.trigger('click');await flushPromises()
      expect(modelApi.writeFile).toHaveBeenCalledTimes(2)
      expect(vi.mocked(modelApi.writeFile).mock.calls[1]).toEqual(['file-1',draft,[{channelId:'b',action:'clear',value:null}] satisfies CredentialEdit[]])
      expect(store.channelBaseline).toBe('file-2');expect(store.channelDraft).toEqual(disk.policy);expect(store.channelDirty).toBe(false)
      expect(store.snapshot!.policy.channels.map(channel=>channel.id)).toEqual(['b','new','a'])
    }finally{wrapper.unmount()}
  })
})
