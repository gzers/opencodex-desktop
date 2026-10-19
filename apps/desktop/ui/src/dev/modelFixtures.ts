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
  modelApi.project=async policy=>({rows:policy.channels.flatMap(c=>c.discovered.map(m=>({channelId:c.id,providerId:c.connection.providerId,modelId:m.id,label:m.label,enabled:c.connection.enabled,selected:true,discovered:true,templateRef:null,rules:m.rules,source:{protocol:'渠道默认'}}))),knownActiveModels:36,exactCatalogCount:null,notices:['显式DEV数据，不连接真实运行服务']})
  modelApi.agentSettings=async()=>null
  modelApi.latestPolicy=async()=>null
  modelTransferApi.loadRemoteScope=async()=>({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false})
  importedConnectionApi.list=async()=>({baseline:'explicit-visual-fixture',items:[]})
  modelTransferApi.accountCatalog=async()=>[{account:{id:crypto.randomUUID(),provider:'chatgpt',subject:'fixture-subject',label:'账户信息样例',email:'fixture@example.test'},loginTransfer:'manualImportDisabled',localSource:true}]
  modelTransferApi.extensionCatalog=async()=>({skills:['fixture-review-skill'],mcps:['fixture-review-mcp']})
  const store=useModelsStore();store.snapshot=structuredClone(snapshot);store.restoreChannels()
}
