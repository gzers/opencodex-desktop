// Explicit DEV visual fixtures: no fallback on a real IPC failure, no disk or API.
import { emptyContent, emptyRules, modelApi, type ModelSnapshot, type ChannelPolicy } from '@/features/models/api'
import { useModelsStore } from '@/features/models/store'
import {modelTransferApi} from '@/features/sync/modelTransfer'
import {importedConnectionApi} from '@/features/sync/api'
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
  modelApi.stage=async(_policy,_workspace,policy)=>{snapshot={...snapshot,workspace:{basePolicy:snapshot.policyDigest,policy:structuredClone(policy)},workspaceDigest:'visual-workspace'};return structuredClone(snapshot)}
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
  modelApi.project=async policy=>({rows:policy.channels.flatMap(c=>c.discovered.map(m=>({channelId:c.id,providerId:c.connection.providerId,modelId:m.id,label:m.label,enabled:c.connection.enabled,selected:true,discovered:true,templateRef:visualReference(policy,c,m.id),rules:{...m.rules,protocol:{mode:'custom' as const,value:c.connection.protocol}},source:{protocol:'渠道默认'}}))),knownActiveModels:36,exactCatalogCount:null,notices:['显式DEV数据，不连接真实运行服务']})
  modelApi.executionDirectory=async()=>({channels:snapshot.policy.channels.map(c=>({id:c.id,label:c.label,providerId:c.connection.providerId})),rows:snapshot.policy.channels.flatMap(c=>c.discovered.map(m=>({channelId:c.id,providerId:c.connection.providerId,modelId:m.id,label:m.label,enabled:c.connection.enabled,selected:true,discovered:true,templateRef:null,rules:m.rules,source:{},connectionDigest:'visual-binding'})))})
  modelApi.saveAgentSettings=async settings=>structuredClone(settings)
  modelApi.agentSettings=async()=>null
  const conversations=new Map<string,import('@/features/models/api').AgentConversation>()
  modelApi.agentConversation=async id=>structuredClone(conversations.get(id??'new')??null)
  modelApi.saveAgentConversation=async request=>{const saved={templateId:request.templateId,digest:crypto.randomUUID(),content:structuredClone(request.content)};conversations.set(request.templateId??'new',saved);return saved}
  const newChannelResults=new Map<string,import('@/features/models/api').NewChannelOutcome>()
  modelApi.latestNewChannel=async()=>null
  modelApi.previewNewChannel=async()=>({models:structuredClone(channels[0]!.discovered),credentialRef:null,elapsedMs:12})
  modelApi.addNewChannel=async request=>{const channel=structuredClone(request.channel);snapshot={...snapshot,workspace:{basePolicy:snapshot.policyDigest,policy:{...(snapshot.workspace?.policy??snapshot.policy),channels:request.replace?(snapshot.workspace?.policy.channels??snapshot.policy.channels).map(c=>c.id===channel.id?channel:c):[...(snapshot.workspace?.policy.channels??snapshot.policy.channels),channel]}},workspaceDigest:'visual-new-channel'};const outcome={id:request.id,channel,phase:'committed' as const,error:null,snapshot:structuredClone(snapshot)};newChannelResults.set(request.id,outcome);return structuredClone(outcome)}
  modelApi.newChannelResult=async id=>{const result=newChannelResults.get(id);if(!result)throw {message:'显式DEV加入记录不存在'};return structuredClone(result)}
  modelApi.latestPolicy=async()=>null
  const clientPlan=crypto.randomUUID()
  let clientState:import('@/features/models/api').ModelClientState={planId:clientPlan,status:'restart_required',checkedAt:null,checkId:null,confirmedAt:null,runtimeMatches:false,automaticClientVerification:false}
  modelApi.clientStatus=async()=>structuredClone(clientState)
  modelApi.checkClient=async(_plan,confirmation)=>{clientState={...clientState,status:confirmation?'user_confirmed':'pending_verification',checkedAt:new Date().toISOString(),checkId:crypto.randomUUID(),confirmedAt:confirmation?new Date().toISOString():null,runtimeMatches:true};return structuredClone(clientState)}
  const historyId=crypto.randomUUID()
  modelApi.policyHistory=async()=>[{id:historyId,createdAt:'2026-10-19T00:00:00Z',phase:'committed',channelLabels:['NewApi'],recoverable:true}]
  modelApi.prepareRecovery=async request=>({id:crypto.randomUUID(),sourceId:request.sourceId,side:request.side,scope:request.scope,restoreDefaults:request.restoreDefaults,changes:[{field:'channel/fixture/selection',before:{mode:'all'},after:{mode:'whitelist',models:['gpt-demo-0','gpt-demo-1']}}],defaultChanges:[],channelLabels:['NewApi'],notices:['恢复后仍需在模型页预览并保存。']})
  modelApi.acceptRecovery=async()=>structuredClone(snapshot)
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
  modelTransferApi.prepareDraft=async(scope,candidate)=>({id:crypto.randomUUID(),credentialCount:0,package:{format:'OCXD_MODEL_CONFIG',formatVersion:4,schemaVersion:1,createdAt:new Date().toISOString(),digest:'explicit-draft-fixture',payload:{candidate,templates:[],policy:{templateRefs:null,channelOrder:null,channels:(snapshot.workspace?.policy.channels??[]).filter(c=>scope.channelIds.includes(c.id)).map(c=>({id:c.id,label:c.label,selection:c.selection,naming:c.naming,connection:{nodes:c.connection.nodes}}))}}}})
  const store=useModelsStore();store.snapshot=structuredClone(snapshot);store.restoreChannels()
}
