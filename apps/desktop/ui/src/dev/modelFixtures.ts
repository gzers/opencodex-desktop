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
  modelApi.load=async()=>structuredClone(snapshot)
  modelApi.stage=async(_policy,_workspace,policy)=>{snapshot={...snapshot,workspace:{basePolicy:snapshot.policyDigest,policy:structuredClone(policy)},workspaceDigest:'visual-workspace'};return structuredClone(snapshot)}
  modelApi.preview=async(content,_channel,model)=>({matched:content.matching.include.some(p=>model.startsWith(p.value)),reason:'显式DEV样例',variantId:null,rules:content.rules})
  modelApi.project=async policy=>({rows:policy.channels.flatMap(c=>c.discovered.map(m=>({channelId:c.id,providerId:c.connection.providerId,modelId:m.id,label:m.label,enabled:c.connection.enabled,selected:true,discovered:true,templateRef:null,rules:{...m.rules,protocol:{mode:'custom' as const,value:c.connection.protocol}},source:{protocol:'渠道默认'}}))),knownActiveModels:36,exactCatalogCount:null,notices:['显式DEV数据，不连接真实运行服务']})
  modelApi.executionDirectory=async()=>({channels:snapshot.policy.channels.map(c=>({id:c.id,label:c.label,providerId:c.connection.providerId})),rows:snapshot.policy.channels.flatMap(c=>c.discovered.map(m=>({channelId:c.id,providerId:c.connection.providerId,modelId:m.id,label:m.label,enabled:c.connection.enabled,selected:true,discovered:true,templateRef:null,rules:m.rules,source:{},connectionDigest:'visual-binding'})))})
  modelApi.saveAgentSettings=async settings=>structuredClone(settings)
  modelApi.agentSettings=async()=>null
  const conversations=new Map<string,import('@/features/models/api').AgentConversation>()
  modelApi.agentConversation=async id=>structuredClone(conversations.get(id??'new')??null)
  modelApi.saveAgentConversation=async request=>{const saved={templateId:request.templateId,digest:crypto.randomUUID(),content:structuredClone(request.content)};conversations.set(request.templateId??'new',saved);return saved}
  const newChannelResults=new Map<string,import('@/features/models/api').NewChannelOutcome>()
  modelApi.latestNewChannel=async()=>null
  modelApi.previewNewChannel=async()=>({models:structuredClone(channels[0]!.discovered),credentialRef:null,elapsedMs:12})
  modelApi.addNewChannel=async request=>{const channel=structuredClone(request.channel);snapshot={...snapshot,workspace:{basePolicy:snapshot.policyDigest,policy:{...(snapshot.workspace?.policy??snapshot.policy),channels:[...(snapshot.workspace?.policy.channels??snapshot.policy.channels),channel]}},workspaceDigest:'visual-new-channel'};const outcome={id:request.id,channel,phase:'committed' as const,error:null,snapshot:structuredClone(snapshot)};newChannelResults.set(request.id,outcome);return structuredClone(outcome)}
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
  modelTransferApi.accountCatalog=async()=>[{account:{id:crypto.randomUUID(),provider:'chatgpt',subject:'fixture-subject',label:'账户信息样例',email:'fixture@example.test'},loginTransfer:'manualImportDisabled',localSource:true}]
  modelTransferApi.extensionCatalog=async()=>({skills:['fixture-review-skill'],mcps:['fixture-review-mcp']})
  const originalChannel:ChannelPolicy={...structuredClone(channels[0]!),id:crypto.randomUUID(),label:'原有渠道 · 完整配置迁移样例',connection:{...structuredClone(channels[0]!.connection),providerId:'official-original',authMode:'key'},selection:{mode:'whitelist',models:['gpt-demo-0']}}
  modelTransferApi.channelCatalog=async()=>structuredClone([...snapshot.policy.channels,originalChannel])
  modelTransferApi.prepare=async(scope,_password,includeKeys)=>({id:crypto.randomUUID(),protected:!!_password,credentialCount:includeKeys?scope.channelIds.length:0,package:{format:'OCXD_MODEL_CONFIG',formatVersion:3,schemaVersion:1,createdAt:new Date().toISOString(),digest:'explicit-dev-sample',payload:{templates:snapshot.library.templates.filter(t=>scope.templateRefs.some(r=>r.templateId===t.id)),policy:scope.includePolicy?{templateRefs:null,channelOrder:null,channels:[...snapshot.policy.channels,originalChannel].filter(c=>scope.channelIds.includes(c.id)).map(c=>({id:c.id,label:c.label,selection:c.selection,connection:{nodes:c.connection.nodes,provider:{adapter:c.connection.protocol,authMode:'key',baseUrl:c.connection.nodes[0]!.url,modelAliases:{'gpt-demo-0':'friendly-gpt'},apiKeyPoolStrategy:'fill-first',modelDisplayNames:{'gpt-demo-0':'GPT示例'},defaultAliases:false,disabled:!c.connection.enabled}}}))}:null}}})

  const store=useModelsStore();store.snapshot=structuredClone(snapshot);store.restoreChannels()
}
