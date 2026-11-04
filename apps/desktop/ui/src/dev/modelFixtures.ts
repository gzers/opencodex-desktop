// Explicit DEV visual fixtures: no fallback on a real IPC failure, no disk or API.
import { emptyContent, emptyRules, modelApi, type ModelSnapshot, type ChannelPolicy } from '@/features/models/api'
import { useModelsStore } from '@/features/models/store'
import {modelTransferApi,assetDetailApi,transferFileDialog,cloudCatalogApi,transferChoiceApi,transferProtectionApi,importCredentialChoiceApi,transferResultApi,type TransferChoice,type FilePlan,type TransferResult} from '@/features/sync/modelTransfer'
import {importedConnectionApi,syncEndpointApi,remoteVersionApi} from '@/features/sync/api'
export function installModelFixtures() {
  const channels:ChannelPolicy[]=Array.from({length:24},(_,i)=>{
    const id=crypto.randomUUID();const node=crypto.randomUUID();
    return {id,label:['NewApi','A6api','AiMaMi'][i]??'示例渠道 '+(i+1),templateRefs:[],modelRefs:{},connection:{providerId:'sample-'+i,protocol:'openai-responses',nodes:[{id:node,name:'默认节点',url:'https://example.test/v1'}],activeNodeId:node,credentialRef:null,enabled:i<3,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:[],discovered:Array.from({length:12},(_,n)=>({id:'gpt-demo-'+n,label:'GPT 示例 '+n,rules:emptyRules()})),defaults:emptyRules(),overrides:{},familyOverrides:{}}
  })
  let snapshot:ModelSnapshot={library:{schemaVersion:1,templates:['GPT','Claude','Gemini','Qwen','DeepSeek','Grok','GLM','HY / 混元','Doubao','Kimi','MiniMax','ERNIE'].map(label=>{
    const content=emptyContent();content.matching.include=[{kind:'prefix',value:label.toLowerCase()+'-'}]
    const revision={id:crypto.randomUUID(),createdAt:'2026-10-18T00:00:00Z',savedLabel:label,content,digest:'visual-fixture',baseRevision:null,changes:[]}
    return{id:crypto.randomUUID(),label,source:'builtin',defaultRevision:revision.id,revisions:[revision]}
  })},policy:{schemaVersion:1,templateRefs:[],channels},libraryDigest:'visual-library',policyDigest:'visual-policy',workspace:null,workspaceDigest:'visual-workspace'}
  // Opt-in family scope screen; ordinary audit fixtures retain their previous state.
  if(new URLSearchParams(location.search).get('familyScope')==='1') {
    const family=snapshot.library.templates[0]!;snapshot.policy.channels[0]!.templateRefs=[{templateId:family.id,revisionId:family.revisions[0]!.id}]
  }
  modelApi.measureNodes=async request=>({runId:request.runId,stopped:false,results:request.nodes.map((node,index)=>({nodeId:node.id,elapsedMs:42+index*11,modelCount:12,error:null}))})
  modelApi.stopNodeMeasurement=async()=>{}
  if(new URLSearchParams(location.search).get('versionReview')==='1'){
    const template=snapshot.library.templates[0]!,original=template.revisions[0]!
    template.revisions.push({...structuredClone(original),id:crypto.randomUUID(),createdAt:'2026-10-20T00:00:00Z',content:{...structuredClone(original.content),description:'显式DEV第二保存版'},digest:'explicit-second-version-fixture'})
  }
  modelApi.load=async()=>structuredClone(snapshot)
  modelApi.credential=async()=>({configured:true,value:'example-demo-key',source:'配置文件',viewable:true})
  modelApi.writeFile=async(_expected,policy)=>{snapshot={...snapshot,policy:structuredClone(policy),workspace:null,policyDigest:crypto.randomUUID()};return{snapshot:structuredClone(snapshot),changed:true}}
  const visualPlan=(policy:import('@/features/models/api').ModelPolicy,scope?:import('@/features/models/api').SaveScope):import('@/features/models/api').SavePlan=>{const changed=policy.channels.filter(c=>JSON.stringify(c)!==JSON.stringify(snapshot.policy.channels.find(old=>old.id===c.id)));return{id:crypto.randomUUID(),phase:'prepared',catalogCount:36,currentCatalogCount:36,changedChannels:changed.filter(c=>!scope||scope.channelIds.includes(c.id)).map(c=>c.label),deferredChannels:scope?changed.filter(c=>!scope.channelIds.includes(c.id)).map(c=>c.label):[],error:null}}
  modelApi.preparePolicy=async policy=>visualPlan(policy)
  modelApi.preparePolicyScoped=async(policy,_library,_saved,_workspace,scope)=>visualPlan(policy,scope)
  modelApi.templateUsage=async(templateId,revisionId)=>{const template=snapshot.library.templates.find(t=>t.id===templateId),revision=template?.revisions.find(r=>r.id===revisionId);return{templateId,revisionId,libraryDigest:snapshot.libraryDigest,policyDigest:snapshot.policyDigest,workspaceDigest:snapshot.workspaceDigest,references:[],historicalDependency:false,models:snapshot.policy.channels.flatMap(c=>c.discovered.map(m=>({state:'已提交策略',channelId:c.id,channelLabel:c.label,modelId:m.id,matched:!!revision?.content.matching.include.some(p=>m.id.startsWith(p.value)),reason:revision?.content.matching.include.some(p=>m.id.startsWith(p.value))?'已匹配':'未命中包含规则',adopted:false,resolutionError:null})))}}
  modelApi.preview=async(content,_channel,model)=>({matched:content.matching.include.some(p=>model.startsWith(p.value)),reason:'显式DEV样例',variantId:null,rules:content.rules})
  function visualReference(policy:import('@/features/models/api').ModelPolicy,c:ChannelPolicy,id:string){
    if(Object.hasOwn(c.modelRefs,id))return c.modelRefs[id]??null
    const adopted=new Map([...policy.templateRefs,...c.templateRefs].map(r=>[r.templateId,r]))
    return [...adopted.values()].find(r=>snapshot.library.templates.find(t=>t.id===r.templateId)?.revisions.find(v=>v.id===r.revisionId)?.content.matching.include.some(p=>id.startsWith(p.value)))??null
  }
  modelApi.project=async policy=>({rows:policy.channels.flatMap(c=>c.discovered.map(m=>({channelId:c.id,providerId:c.connection.providerId,modelId:m.id,label:(c.connection.providerId==='openai'?m.label:c.label+'/'+m.id),enabled:c.connection.enabled,selected:true,discovered:true,templateRef:visualReference(policy,c,m.id),rules:{...m.rules,protocol:{mode:'custom' as const,value:c.connection.protocol}},source:{protocol:'渠道默认'}}))),knownActiveModels:36,exactCatalogCount:null,notices:['显式DEV数据，不连接真实运行服务']})
  modelApi.executionDirectory=async()=>({channels:snapshot.policy.channels.map(c=>({id:c.id,label:c.label,providerId:c.connection.providerId})),rows:snapshot.policy.channels.flatMap(c=>c.discovered.map(m=>({channelId:c.id,providerId:c.connection.providerId,modelId:m.id,label:(c.connection.providerId==='openai'?m.label:c.label+'/'+m.id),enabled:c.connection.enabled,selected:true,discovered:true,templateRef:null,rules:m.rules,source:{},connectionDigest:'visual-binding'})))})
  modelApi.saveAgentSettings=async settings=>structuredClone(settings)
  modelApi.agentSettings=async()=>null
  const conversations=new Map<string,import('@/features/models/api').AgentConversation>()
  if(new URLSearchParams(location.search).get('agentView')==='1'){
    const template=snapshot.library.templates[0]!
    conversations.set(template.id,{templateId:template.id,digest:'explicit-conversation-fixture',content:{baseRevision:template.revisions[0]!.id,input:'尚未发送的纠偏输入',history:Array.from({length:20},(_,index)=>({role:index%2?'assistant' as const:'user' as const,text:'本机会话样例 '+index+' · 较长的中文与 English 上下文，不连接模型服务。'})),sources:[]}})
    modelApi.latestAgentResult=async()=>null
  }

  modelApi.agentConversation=async id=>structuredClone(conversations.get(id??'new')??null)
  modelApi.saveAgentConversation=async request=>{const saved={templateId:request.templateId,digest:crypto.randomUUID(),content:structuredClone(request.content)};conversations.set(request.templateId??'new',saved);return saved}
  modelApi.previewNewChannel=async()=>({models:structuredClone(channels[0]!.discovered),credentialRef:null,elapsedMs:12})
  // 审计入口：?partialSave=1 时假装「上次写入未完整结束」，用于实屏核对流程第 ④ 态。
  // 只影响 DEV 夹具，不写入任何真实状态。
  const partialSave=new URLSearchParams(location.search).get('partialSave')==='1'
  const partialPlan=()=>({
    id:'visual-partial-plan',
    phase:'verified_partial' as const,
    catalogCount:36,
    currentCatalogCount:36,
    changedChannels:['NewApi'],
    error:'provider_write',
    protocolGroups:[],
    routeChanges:[],
    removedProviders:[],
    historySource:null,
    runtimeChanges:[],
  })
  modelApi.policyResult=async()=>partialSave?partialPlan():{...visualPlan(snapshot.policy),id:'visual-plan',phase:'committed' as const}
  modelApi.latestPolicy=async()=>partialSave?partialPlan():null
  const historyId=crypto.randomUUID()
  modelApi.policyHistory=async()=>[{id:historyId,createdAt:'2026-10-19T00:00:00Z',phase:'committed',channelLabels:['NewApi'],recoverable:true}]
  modelApi.prepareRecovery=async request=>({id:crypto.randomUUID(),sourceId:request.sourceId,side:request.side,scope:request.scope,restoreDefaults:request.restoreDefaults,changes:[{field:'channel/fixture/selection',before:{mode:'all'},after:{mode:'whitelist',models:['gpt-demo-0','gpt-demo-1']}}],defaultChanges:[],channelLabels:['NewApi'],notices:['恢复后仍需在模型页预览并保存。']})
  modelApi.acceptRecovery=async()=>structuredClone(snapshot)
  assetDetailApi.load=async(kind)=>({source:'explicit fixture',fields:kind==='skills'?['SKILL.md正文与资源']:['command','env'],files:kind==='skills'?['SKILL.md','scripts/check.py','references/guide.md']:['MCP完整定义'],bytes:120,clients:kind==='mcps'?['codex','claude']:[],conflicts:[],warnings:[]});
  cloudCatalogApi.load=async()=>null;
  const choices=new Map<boolean,TransferChoice>();transferChoiceApi.load=async(remote)=>structuredClone(choices.get(remote)??null);transferChoiceApi.save=async(remote,choice)=>{choices.set(remote,structuredClone(choice));return structuredClone(choice)}
  let protection={enabled:false,hasPassword:false};transferProtectionApi.load=async()=>({...protection});transferProtectionApi.save=async(enabled,password)=>{protection={enabled,hasPassword:!!password||protection.hasPassword};return {...protection}}
  let importPrefs={keys:true,extensions:true};importCredentialChoiceApi.load=async()=>({...importPrefs});importCredentialChoiceApi.save=async(value)=>{importPrefs={...value}}
  remoteVersionApi.read=async()=>({remoteExists:true,remoteRevision:'fixture-snapshot',remoteSavedAt:null});
  let endpointRevision=0;let endpoint={endpointId:'fixture-webdav',revision:'a'.repeat(64),url:'https://dav.example.test',remotePath:'/config-sync',username:'fixture-user',credentialRefId:'fixture-opaque',tlsPolicy:'verify_required',payloadProtection:'integrity_only',conflictPolicy:'ask' as const};syncEndpointApi.load=async()=>({endpoint:{...endpoint}});syncEndpointApi.save=async(value)=>{endpoint={...endpoint,revision:(++endpointRevision).toString(16).padStart(64,'0'),url:value.baseUrl,remotePath:value.remotePath,username:value.username};return {endpoint:{...endpoint}}};syncEndpointApi.test=async()=>({connectionState:'synced',operationState:'succeeded',message:'WebDAV 连接成功，当前表单尚未保存。',snapshotId:null,backupId:null,etag:null})
  const frozen=new Map<string,FilePlan>();const results=new Map<string,TransferResult>();let lastResult:string|null=null;
  // Stay inside the explicit in-memory fixture when previews are cancelled.
  // Completed exports retain the same plan for result/retry inspection.
  modelTransferApi.discardUnattempted=async(id)=>results.has(id)?false:frozen.delete(id);
  transferResultApi.read=async(id)=>structuredClone(results.get(id??lastResult??'')??null);transferResultApi.readMode=async()=>structuredClone(results.get(lastResult??'')??null);transferResultApi.openDirectory=async()=>{};transferResultApi.backups=async()=>[];
  modelTransferApi.loadRemoteScope=async()=>({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false})
  importedConnectionApi.list=async()=>({baseline:'explicit-visual-fixture',items:[]})
  const accountFixtures:import('@/features/sync/modelTransfer').AccountCatalogEntry[]=[
    ...['personal','work'].map(kind=>({account:{id:crypto.randomUUID(),provider:'chatgpt',subject:'fixture-'+kind,label:'ChatGPT · '+(kind==='personal'?'个人账户':'工作账户'),email:kind+'@example.test',origin:{version:1,accountPlatform:'chatgpt',owner:'opencodex',authKind:'oauth',sourceNamespace:'codex-account-pool',sourceAccountId:'fixture-'+kind,tenant:kind}},loginTransfer:'manualImportDisabled' as const,localSource:true,informationSupported:true})),
    {account:{id:crypto.randomUUID(),provider:'google-antigravity',subject:'fixture-gravity',label:'Google Antigravity · 工作账户',email:'gravity@example.test',origin:{version:1,accountPlatform:'antigravity',owner:'opencodex',authKind:'oauth',sourceNamespace:'oauth-account-pool',sourceAccountId:'gravity',tenant:null}},loginTransfer:'crossDeviceEvidenceMissing',localSource:true,informationSupported:true},
    {account:{id:crypto.randomUUID(),provider:'chatgpt',subject:'fixture-native',label:'原生来源 · 尚未适配',email:null,origin:{version:1,accountPlatform:'chatgpt',owner:'platform-native',authKind:'oauth',sourceNamespace:'native',sourceAccountId:'native',tenant:null}},loginTransfer:'unsupportedProvider',localSource:false,informationSupported:false},
  ]
  modelTransferApi.accountCatalog=async()=>structuredClone(accountFixtures)
  modelTransferApi.extensionCatalog=async()=>({skills:['fixture-review-skill'],mcps:['fixture-review-mcp']})
  const originalChannel:ChannelPolicy={...structuredClone(channels[0]!),id:crypto.randomUUID(),label:'原有渠道 · 完整配置迁移样例',connection:{...structuredClone(channels[0]!.connection),providerId:'official-original',authMode:'key'},selection:{mode:'whitelist',models:['gpt-demo-0']}}
  modelTransferApi.channelCatalog=async()=>structuredClone([...snapshot.policy.channels,originalChannel])
  modelTransferApi.prepare=async(scope,_password,includeKeys)=>({id:crypto.randomUUID(),protected:!!_password,credentialCount:includeKeys?scope.channelIds.length:0,package:{format:'OCXD_MODEL_CONFIG',formatVersion:3,schemaVersion:1,createdAt:new Date().toISOString(),digest:'explicit-dev-sample',payload:{templates:snapshot.library.templates.filter(t=>scope.templateRefs.some(r=>r.templateId===t.id)),policy:scope.includePolicy?{templateRefs:null,channelOrder:null,channels:[...snapshot.policy.channels,originalChannel].filter(c=>scope.channelIds.includes(c.id)).map(c=>({id:c.id,label:c.label,selection:c.selection,connection:{nodes:c.connection.nodes,provider:{adapter:c.connection.protocol,authMode:'key',baseUrl:c.connection.nodes[0]!.url,modelAliases:{'gpt-demo-0':'friendly-gpt'},apiKeyPoolStrategy:'fill-first',modelDisplayNames:{'gpt-demo-0':'GPT示例'},defaultAliases:false,disabled:!c.connection.enabled}}}))}:null}}})

  const ordinaryPrepare=modelTransferApi.prepare
  modelTransferApi.prepare=async(scope,password,includeKeys,manager)=>{
    const result=await ordinaryPrepare(scope,password,includeKeys,manager)
    const fragments=Object.entries(scope.channelFamilies??{})
    if(!fragments.length)return result
    result.package.formatVersion=5
    for(const [id,familyScope] of fragments){
      const channel=result.package.payload.policy?.channels.find(c=>c.id===id)
      const source=snapshot.policy.channels.find(c=>c.id===id);if(!channel||!source)continue
      channel.familyScope=familyScope;channel.connection=null;channel.selection={mode:'none'}
      channel.modelSubset=source.discovered.filter(model=>familyScope.templateIds.some(id=>snapshot.library.templates.find(t=>t.id===id)?.revisions[0]?.content.matching.include.some(p=>model.id.startsWith(p.value)))).map(m=>m.id)
      for(const familyId of familyScope.templateIds){const template=snapshot.library.templates.find(t=>t.id===familyId);if(template&&!result.package.payload.templates.some(t=>t.id===familyId))result.package.payload.templates.push(structuredClone(template))}
    }
    result.credentialCount=includeKeys?scope.channelIds.filter(id=>!scope.channelFamilies?.[id]).length:0
    return result
  }
  const prepareFrozen=modelTransferApi.prepare;modelTransferApi.prepare=async(...args)=>{const plan=await prepareFrozen(...args);const manager=args[3],scope=args[0],includeKeys=args[2];plan.credentialCount=(includeKeys?scope.channelIds.length:0)+(manager?.extensionCredentials?(manager.skillNames?.length??0)+(manager.mcpNames?.length??0):0)+(manager?.webdav&&manager.webdavPasswords?1:0);plan.manager={preferences:manager?.preferences?{theme:'light'}:null,extensions:manager?.extensions?{skills:[],servers:[],sync_method:'symlink'}:null,webdav:[],accounts:accountFixtures.filter(a=>manager?.accountIds?.includes(a.account.id)).map(a=>a.account)};plan.assets={skills:(manager?.skillNames??[]).map(name=>({name,digest:'fixture-skill',files:3,bytes:120,warnings:[],protected:false})),mcps:(manager?.mcpNames??[]).map(name=>({name,digest:'fixture-mcp',secretFields:[],warnings:[]})),conflicts:[]};frozen.set(plan.id,structuredClone(plan));return plan};modelTransferApi.protect=async(id,password)=>{const plan=structuredClone(frozen.get(id)!);plan.id=crypto.randomUUID();plan.protected=!!password;frozen.set(plan.id,structuredClone(plan));return plan};modelTransferApi.export=async(id)=>{const plan=frozen.get(id)!;lastResult=id;results.set(id,{id,kind:'export',phase:'complete',startedAt:new Date().toISOString(),finishedAt:new Date().toISOString(),items:[],errorCode:null,exportPath:'/fixture/export/config.ocxdconf',sourceDigest:plan.package.digest,retry:null});return {path:'/fixture/export/config.ocxdconf',digest:plan.package.digest,templates:plan.package.payload.templates.length,channels:plan.package.payload.policy?.channels.length??0}};
  modelTransferApi.saveRemoteScope=async scope=>scope;modelTransferApi.checkSync=async()=>({source:'local',hasRemote:false,reason:'no_remote'});modelTransferApi.prepareUpload=async(scope,options)=>{const plan=await modelTransferApi.prepare(scope,options?.passphrase,options?.includeKeys,{preferences:options?.preferences??true,extensions:options?.extensions??true,webdav:false,webdavPasswords:false,...options});return {...plan,phase:'prepared',sync:{source:'local',changes:[{kind:'templates',id:'fixture-template',label:'模型模版',mark:'update'}],unchanged:2,createdAt:'2026-10-31T10:00:00Z'}}};modelTransferApi.publish=async(id)=>{lastResult=id;results.set(id,{id,kind:'upload',phase:'complete',startedAt:new Date().toISOString(),finishedAt:new Date().toISOString(),items:[{id:'templates:fixture',kind:'templates',label:'模型模版',status:'completed'}],errorCode:null,exportPath:null,sourceDigest:'fixture',retry:null});return {...frozen.get(id)!,phase:'published'}};
  transferFileDialog.open=async(options)=>options?.directory?'/fixture/custom-export':'/fixture/device-config.ocxdconf';
  modelTransferApi.receive=async(_path,password)=>{if(new URLSearchParams(location.search).get('importProtected')==='1'&&password!=='fixture-password')throw {code:'passphrase_required',message:'请填写配置包解密口令'};const packageValue=(await modelTransferApi.prepare({includePolicy:true,channelIds:[channels[0]!.id],includeGlobal:false,templateRefs:[]},undefined,false,{preferences:false,extensions:false,webdav:false,webdavPasswords:false})).package;return {id:crypto.randomUUID(),package:packageValue,expectedLibrary:snapshot.libraryDigest,expectedPolicy:snapshot.policyDigest,expectedWorkspace:snapshot.workspaceDigest,mergedPolicy:snapshot.policy,notices:[],availableKeys:[],availableExtensionIds:[],comparison:{source:'file',createdAt:'2026-10-31T10:00:00Z',changes:[{kind:'channels',id:channels[0]!.id,label:channels[0]!.label,mark:'update',before:3,after:5}],unchanged:0}}};
  modelTransferApi.select=async(id)=>({...await modelTransferApi.receive('/fixture/device-config.ocxdconf','fixture-password'),id});modelTransferApi.accept=async()=>structuredClone(snapshot);
  modelTransferApi.prepareDraft=async(scope,candidate)=>({id:crypto.randomUUID(),credentialCount:0,package:{format:'OCXD_MODEL_CONFIG',formatVersion:4,schemaVersion:1,createdAt:new Date().toISOString(),digest:'explicit-draft-fixture',payload:{candidate,templates:[],policy:{templateRefs:null,channelOrder:null,channels:(snapshot.workspace?.policy.channels??[]).filter(c=>scope.channelIds.includes(c.id)).map(c=>({id:c.id,label:c.label,selection:c.selection,naming:c.naming,connection:{nodes:c.connection.nodes}}))}}}})
  const store=useModelsStore();store.snapshot=structuredClone(snapshot);store.restoreChannels()
}
