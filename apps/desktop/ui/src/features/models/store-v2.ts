import {computed,ref,shallowRef} from 'vue'
import {defineStore} from 'pinia'
import {createModelRowsState,pendingModelRows,type ModelRowsState} from './modelRows'
import {
  emptyContentV2,serviceUnavailableV2,modelV2Api,reviewedAgentResponseV2,type ModelV2Services,type LibraryViewV2,type TemplateContentV2,
  type TemplateV2,type ConditionPreviewView,type MatchIssueV2,type ChannelConnectionEditIntentV2,
  type ChannelDiscoveryInputV2,
  type FileSnapshotV2,type ReadExistingRequestV2,type ChannelIntentV2,type ModelEditV2,type ModelSelectionV2,type DraftProjectionV2,
  type TemplateActionIntentV2,
  type LibraryChangeV2,
  type AgentSettingsV2,type GenerateIntentV2,type GeneratedReviewV2,type RunTicketV2,type EditingBufferInputV2,type AgentMessageV2,type ResearchSourceV2,
} from './api-v2'

const clone=<T>(value:T):T=>JSON.parse(JSON.stringify(value)) as T
const message=(cause:unknown)=>cause instanceof Error?cause.message:typeof cause==='object'&&cause!==null&&'code' in cause?String(cause.code):'操作未完成，当前输入已保留。'
export interface RevisionBufferV2 {templateId:string;baseRevision:string|null;label:string;content:TemplateContentV2;makeDefault:boolean}
export interface AgentSessionV2 {input:string;history:AgentMessageV2[];sources:ResearchSourceV2[];researchQuery?:string;researchUrls?:string;ticket:RunTicketV2|null;result:GeneratedReviewV2|null;preparing:boolean;generating:boolean;stopping:boolean;accepting:boolean;error:string;status:string;inputGeneration:number|null;expectedLibrary:string;acceptRequest:{runId:string;expectedLibrary:string;reviewedResponse:string}|null;view?:{scrollTop:number;selectionStart:number;selectionEnd:number;maximized:boolean}}

export const useModelsV2Store=defineStore('models-v2',()=>{
  // This adapter is supplied by L0 after command registration. No invented invoke.
  const services=shallowRef<ModelV2Services>({
    loadLibrary:modelV2Api.library,
    validateContent:async content=>{await modelV2Api.validateContent(content);return []},
    previewConditions:async input=>{
      const response=await modelV2Api.previewConditions(input.content,[input.rawModelId],input.channelId)
      if(response.dtoVersion!==2||response.previewOnly!==true||response.rows.length!==1)throw new Error('预览回执不完整，当前输入保持。')
      const row=response.rows[0]!
      return {rawModelId:row.rawModelId,matchingView:null,matched:row.familyMatched,reason:row.familyMatched?(row.matchedConditionId?'命中族及条件覆盖':'命中族，使用通用规则'):'未命中族范围',conditions:row.matchedConditionId?[{conditionId:row.matchedConditionId,matched:true,reason:'解析器确认命中此覆盖项',atoms:[]}]:[],effective:null,configuration:{rules:row.rules,protocol:row.protocol,fieldOrigins:row.fieldOrigins},issues:[]}
    },
    saveRevision:modelV2Api.saveRevision,
  })
  const active=ref(false),loading=ref(false),saving=ref(false),previewBusy=ref(false)
  const snapshot=shallowRef<LibraryViewV2|null>(null),draft=ref<RevisionBufferV2|null>(null)
  const error=ref(''),previewError=ref(''),issues=shallowRef<readonly MatchIssueV2[]>([])
  const preview=shallowRef<ConditionPreviewView|null>(null)
  const previewModelId=ref(''),previewChannelId=ref(''),editGeneration=ref(0)
  const stagedConnectionIntents=shallowRef<readonly ChannelConnectionEditIntentV2[]>([])
  const executionPreference=ref<AgentSettingsV2|null>(null),executionPreferenceRevision=ref(0)
  const agentSessions=ref<Record<string,AgentSessionV2>>({})
  const externalCurrentId=ref(''),externalNotificationTarget=ref<{taskId:string}|null>(null)
  const policyNotificationTarget=ref<{operationId:string}|null>(null)
  const externalTasks=ref<Record<string,{request:Pick<GenerateIntentV2,'targets'|'scene'|'allowNew'|'expectedLibrary'>;expiresAt:string;activated:boolean;revoked:boolean}>>({})
  const agentSequences=new Map<string,number>(),agentStops=new Set<string>()
  let preferenceRead:Promise<void>|null=null
  function agentSession(key:string):AgentSessionV2 {if(!agentSessions.value[key])agentSessions.value[key]={input:'',history:[],sources:[],ticket:null,result:null,preparing:false,generating:false,stopping:false,accepting:false,error:'',status:'',inputGeneration:null,expectedLibrary:'',acceptRequest:null};return agentSessions.value[key]!}
  async function loadAgentSettings(){if(preferenceRead)return preferenceRead;const revision=executionPreferenceRevision.value;preferenceRead=(async()=>{const value=await modelV2Api.agentSettings();if(revision===executionPreferenceRevision.value)executionPreference.value=value})().finally(()=>{preferenceRead=null});return preferenceRead}
  async function bindAgent(settings:AgentSettingsV2){const value=await modelV2Api.bindAgent(clone(settings));executionPreference.value=value;executionPreferenceRevision.value++;return value}
  const file=shallowRef<FileSnapshotV2|null>(null),fileBusy=ref(false),fileError=ref(''),fileNeedsReconcile=ref(false)
  const channelIntents=ref<ChannelIntentV2[]>([]),modelEdits=ref<ModelEditV2[]>([])
  const modelRows=ref<Record<string,ModelRowsState>>({})
  const pendingRows=computed(()=>Object.entries(modelRows.value).flatMap(([channel,state])=>pendingModelRows(state).map(model=>({channel,model}))))
  function rowsFor(id:string){return modelRows.value[id]??=createModelRowsState()}
  const fileProjections=shallowRef<Record<string,{expected:DraftProjectionV2['expected'];effective:DraftProjectionV2['effective']}>>({})
  const effectiveRows=computed(()=>{const rows=new Map<string,DraftProjectionV2['effective']>();for(const row of file.value?.effective??[])rows.set(row.channelId,[...(rows.get(row.channelId)??[]),row]);const base=JSON.stringify(file.value?.draft?.base);for(const [id,value]of Object.entries(fileProjections.value))if(JSON.stringify(value.expected)===base)rows.set(id,value.effective);return [...rows.values()].flat()})
  const readResults=shallowRef<readonly import('./api-v2').ReadMatchDtoV2[]>([])
  const channelEditorId=ref(''),channelSearch=ref('')
  // Private, session-only Key input. Never included in $state, source DTOs or logs.
  const keyBuffers=new Map<string,{action:'replace'|'clear';value?:string;sourceGuard?:string;targetGuard?:string}>(),keyBufferVersion=ref(0)
  const credentialInputRevision=computed(()=>keyBufferVersion.value)
  const keySourceGuard=()=>JSON.stringify({source:file.value?.sourceDigest,library:file.value?.libraryDigest,root:file.value?.rootRevision})
  const pendingCredentialKinds=computed(()=>{void keyBufferVersion.value;return Object.fromEntries([...keyBuffers].map(([id,v])=>[id,v.action]))})
  function keyBuffer(channelId:string){const value=keyBuffers.get(channelId);return value?{...value}:null}
  const keyTargetGuard=(channelId:string)=>{const connection=(file.value?.draft?.policy??file.value?.savedPolicy)?.channels.find(c=>c.id===channelId)?.connection;return connection?JSON.stringify({channelId,nodes:connection.nodes.map(n=>({id:n.id,url:n.url})),authMode:connection.authMode}):''}
  function setKeyBuffer(channelId:string,value:{action:'replace'|'clear';value?:string}|null){if(value)keyBuffers.set(channelId,{...value,sourceGuard:keySourceGuard(),targetGuard:keyTargetGuard(channelId)});else keyBuffers.delete(channelId);keyBufferVersion.value++}
  const view=ref({tab:'channels' as 'channels'|'templates',templateSearch:'',templateEditor:false,templateNavCollapsed:false,templateSection:'model-basic',selectedVersions:{} as Record<string,string>,selectedTemplates:[] as string[]})
  // Key/node commands and active-node intent registered/compiled in L0 f9a79297192e1c8ee1e66569ae51299128e8ff8e.
  const channelCapabilities=ref({scopedSave:true,defaults:true,naming:true,templateRefs:true,copy:true,delete:true,credential:true,measure:true,activeNode:true})
  function configureChannelCapabilities(value:Partial<typeof channelCapabilities.value>){channelCapabilities.value={...channelCapabilities.value,...value}}
  let fileSequence=0,fileGeneration=0,openingDraft:Promise<void>|null=null
  const channels=computed(()=>{
    const result=clone(file.value?.draft?.policy.channels??file.value?.savedPolicy.channels??[])
    for(const intent of channelIntents.value){const channel=result.find(c=>c.id===intent.channelId);if(!channel)continue;switch(intent.kind){case'rename':channel.label=intent.name;break;case'set_enabled':channel.connection.enabled=intent.enabled;break;case'set_selection':channel.selection=clone(intent.selection);break;case'set_order':channel.modelOrder=[...intent.rawModelIds];break;case'set_defaults':channel.defaults=clone(intent.rules);break;case'set_family_defaults':channel.familyOverrides[intent.templateId]=clone(intent.rules);break;case'set_naming':channel.naming={...channel.naming,[intent.rawModelId]:{...(intent.alias===null?{}:{alias:intent.alias}),...(intent.displayName===null?{}:{displayName:intent.displayName})}};break;case'set_template_refs':channel.templateRefs=clone(intent.references);break;case'set_active_node':channel.connection.activeNodeId=intent.nodeId;break;case'delete':break}}
    return result
  })
  const fileDirty=computed(()=>Object.keys(pendingCredentialKinds.value).length>0||Object.keys(file.value?.draft?.credentialEdits??{}).length>0||channelIntents.value.length>0||modelEdits.value.length>0||!!file.value?.draft&&JSON.stringify(file.value.draft.policy)!==JSON.stringify(file.value.savedPolicy))
  function channelDirty(id:string){return pendingRows.value.some(row=>row.channel===id)||!!pendingCredentialKinds.value[id]||!!file.value?.draft?.credentialEdits?.[id]||channelIntents.value.some(i=>i.channelId===id)||modelEdits.value.some(i=>i.channelId===id)||JSON.stringify(file.value?.draft?file.value.draft.policy.channels.find(c=>c.id===id):file.value?.savedPolicy.channels.find(c=>c.id===id))!==JSON.stringify(file.value?.savedPolicy.channels.find(c=>c.id===id))}
  const canSaveFile=computed(()=>!!file.value?.draft&&!fileBusy.value&&!fileNeedsReconcile.value)
  let clean='',readSequence=0,previewSequence=0,baseLibraryDigest=''
  const retainedBuffers=new Map<string,{draft:RevisionBufferV2;clean:string;library:string}>()
  const bufferKey=(id:string,revision:string|null)=>id+'|'+(revision??'new')
  function retainBuffer(){if(draft.value)retainedBuffers.set(bufferKey(draft.value.templateId,draft.value.baseRevision),{draft:clone(draft.value),clean,library:baseLibraryDigest})}
  function protectsRevision(templateId:string,revisionId:string){return draft.value?.templateId===templateId&&draft.value.baseRevision===revisionId||[...retainedBuffers.values()].some(value=>value.draft.templateId===templateId&&value.draft.baseRevision===revisionId)}
  function discardNew(){if(saving.value||!draft.value||draft.value.baseRevision)return;retainedBuffers.delete(bufferKey(draft.value.templateId,null));draft.value=null;invalidate()}
  const dirty=computed(()=>!!draft.value&&(draft.value.baseRevision===null||JSON.stringify(draft.value)!==clean))
  const canLoad=computed(()=>!!services.value.loadLibrary)
  const canPreview=computed(()=>!!services.value.previewConditions)
  const canSave=computed(()=>!!services.value.saveRevision&&!!snapshot.value)
  const canDiscover=computed(()=>!!services.value.previewChannel)
  const canStageConnection=computed(()=>!!services.value.stageConnection)

  function acceptFile(value:FileSnapshotV2){if(value.dtoVersion!==2||value.savedPolicy.schemaVersion!==2||value.library.schemaVersion!==2||value.draft&&value.draft.schemaVersion!==2)throw new Error('渠道格式暂不支持，当前输入保持。');file.value=value;snapshot.value={dtoVersion:2,library:value.library,libraryDigest:value.libraryDigest}}
  function assertHistoryReady(){if(fileBusy.value||fileNeedsReconcile.value||fileDirty.value||pendingRows.value.length)throw new Error('请先保存或还原当前渠道、型号或 Key 修改，再生成历史恢复草稿；当前输入已保留。')}
  async function acceptHistory(recoveryId:string){assertHistoryReady();const context=fileContext();fileBusy.value=true;fileError.value='';try{const value=await modelV2Api.acceptHistory(recoveryId);if(context!==fileContext()){fileNeedsReconcile.value=true;throw new Error('接受恢复期间有新的明确输入，当前输入保持，请重新核对服务端草稿。')}if(value.draft?.policy.recoveryId!==recoveryId)throw new Error('恢复草稿不属于原恢复预览，当前输入保持。');acceptFile(value)}finally{fileBusy.value=false}}
  async function discardChannels(){
    if(fileBusy.value||!file.value)return
    const context=JSON.stringify({file:fileContext(),rows:modelRows.value}),expected=file.value.draft?.base
    fileBusy.value=true;fileError.value=''
    try{const value=expected?await modelV2Api.discardFile(clone(expected)):await modelV2Api.fileSnapshot();if(value.draft)throw new Error('未核对到草稿已放弃的回执，当前输入保持。');const changed=context!==JSON.stringify({file:fileContext(),rows:modelRows.value});acceptFile(value);if(changed){fileNeedsReconcile.value=true;throw new Error('后端草稿已放弃，但等待期间有新的输入；当前输入保留，请重新核对。')}channelIntents.value=[];modelEdits.value=[];modelRows.value={};keyBuffers.clear();keyBufferVersion.value++;fileGeneration++;fileNeedsReconcile.value=false;await ensureFileDraft()}
    catch(cause){fileError.value=message(cause)}finally{fileBusy.value=false}
  }
  function expectedFile():ReadExistingRequestV2 {if(!file.value)throw new Error('请先读取当前渠道工作区。');return {expectedSource:file.value.sourceDigest,expectedLibrary:file.value.libraryDigest,expectedDraft:file.value.draft?.base??null}}
  async function ensureFileDraft(){
    if(file.value?.draft)return
    if(openingDraft)return openingDraft
    const request=expectedFile()
    openingDraft=(async()=>{const value=await modelV2Api.beginDraft(request);acceptFile(value)})()
    try{await openingDraft}finally{openingDraft=null}
  }
  async function loadChannels(discardLocal=false){
    if(fileBusy.value)return
    if((channelIntents.value.length||modelEdits.value.length||pendingRows.value.length||keyBuffers.size)&&!discardLocal){fileError.value='当前有未保存的明确编辑，请先保存或还原后重新读取。';return}
    const seq=++fileSequence;fileBusy.value=true;fileError.value=''
    try{const value=await modelV2Api.fileSnapshot();if(seq!==fileSequence)return;acceptFile(value);if(discardLocal){modelRows.value={};channelIntents.value=[];modelEdits.value=[];keyBuffers.clear();keyBufferVersion.value++;fileGeneration++}await ensureFileDraft();fileNeedsReconcile.value=false}
    catch(cause){if(seq===fileSequence)fileError.value=message(cause)}finally{if(seq===fileSequence)fileBusy.value=false}
  }
  async function readExisting(){
    if(fileBusy.value)return
    if(channelIntents.value.length||modelEdits.value.length||pendingRows.value.length||keyBuffers.size){fileError.value='请先保存或还原当前明确编辑；本次读取未覆盖草稿。';return}
    const request=expectedFile(),generation=fileGeneration,seq=++fileSequence;fileBusy.value=true;fileError.value=''
    try{const response=await modelV2Api.readExisting(request);if(seq!==fileSequence||generation!==fileGeneration){fileNeedsReconcile.value=true;return}acceptFile(response.snapshot);readResults.value=response.results;fileNeedsReconcile.value=false}
    catch(cause){if(seq===fileSequence)fileError.value=message(cause)}finally{if(seq===fileSequence)fileBusy.value=false}
  }
  function queueChannel(intent:ChannelIntentV2){if((intent.kind==='set_defaults'||intent.kind==='set_family_defaults')&&intent.rules.protocol.mode!=='inherit'){fileError.value='渠道层不能设置请求协议，请在模版或单模型中明确配置。';return}const same=(old:ChannelIntentV2)=>old.channelId===intent.channelId&&old.kind===intent.kind&&(old.kind==='set_family_defaults'&&intent.kind==='set_family_defaults'?old.templateId===intent.templateId:old.kind==='set_naming'&&intent.kind==='set_naming'?old.rawModelId===intent.rawModelId:true);const unchanged=intent.kind==='rename'&&intent.name===(file.value?.draft?.policy??file.value?.savedPolicy)?.channels.find(c=>c.id===intent.channelId)?.label;channelIntents.value=channelIntents.value.filter(old=>!same(old)).concat(unchanged?[]:[clone(intent)]);fileGeneration++}
  function queueModel(edit:ModelEditV2){const same=(old:ModelEditV2)=>old.channelId===edit.channelId&&old.rawModelId===edit.rawModelId&&('field' in old&&'field' in edit?old.field===edit.field:!('field' in old)&&!('field' in edit));modelEdits.value=modelEdits.value.filter(old=>!same(old)).concat([clone(edit)]);fileGeneration++}
  function explicitRules(channelId:string,rawModelId:string){const model=channels.value.find(c=>c.id===channelId)?.models.find(m=>m.rawModelId===rawModelId);const rules=clone(model?.explicitOverrides??emptyContentV2().commonRules) as unknown as Record<string,unknown>;for(const edit of modelEdits.value.filter(e=>e.channelId===channelId&&e.rawModelId===rawModelId)){if(edit.kind==='set_override')rules[edit.field]={mode:'custom',value:clone(edit.value)};else if(edit.kind==='clear_override')rules[edit.field]={mode:'inherit'}}return rules as unknown as import('./api-v2').RulesV2}
  function fileContext(){return JSON.stringify({file:file.value,channelIntents:channelIntents.value,modelEdits:modelEdits.value,credentialInputRevision:credentialInputRevision.value})}
  async function projectFile(edits:ModelEditV2[]=[],intents:ChannelIntentV2[]=[],scopeChannelId?:string){
    if(fileBusy.value||!file.value?.draft)throw new Error('渠道工作区尚未就绪，当前输入保持。')
    const context=fileContext(),expected=clone(file.value.draft.base)
    const editKey=(e:ModelEditV2)=>JSON.stringify([e.channelId,e.rawModelId,'field'in e?e.field:'binding'])
    const intentKey=(i:ChannelIntentV2)=>JSON.stringify([i.channelId,i.kind,i.kind==='set_family_defaults'?i.templateId:i.kind==='set_naming'?i.rawModelId:''])
    const mergedEdits=[...new Map([...modelEdits.value,...edits].filter(e=>!scopeChannelId||e.channelId===scopeChannelId).map(e=>[editKey(e),clone(e)])).values()]
    const mergedIntents=[...new Map([...channelIntents.value,...intents].filter(i=>!scopeChannelId||i.channelId===scopeChannelId).map(i=>[intentKey(i),clone(i)])).values()]
    const projection=await modelV2Api.fileProject({expected,edits:mergedEdits,channelIntents:mergedIntents,...(scopeChannelId?{scopeChannelId}:{})})
    if(context!==fileContext())throw new Error('配置基准已变化，当前输入保持，请重新核对。')
    if(JSON.stringify(projection.expected)!==JSON.stringify(expected)||projection.intended.schemaVersion!==2||!projection.digest)throw new Error('校验回执不属于当前草稿，当前输入保持。')
    return {projection,context,scopeChannelId}
  }
  function adoptProjection(result:{projection:DraftProjectionV2;context:string;scopeChannelId?:string},edits:ModelEditV2[],intents:ChannelIntentV2[]){if(result.context!==fileContext())throw new Error('配置基准已变化，当前输入保持。');for(const edit of edits)queueModel(edit);for(const intent of intents)queueChannel(intent);const ids=result.scopeChannelId?[result.scopeChannelId]:[...new Set([...channels.value.map(c=>c.id),...result.projection.intended.channels.map(c=>c.id)])];fileProjections.value={...fileProjections.value,...Object.fromEntries(ids.map(id=>[id,{expected:clone(result.projection.expected),effective:result.projection.effective.filter(row=>row.channelId===id)}]))}}
  function setSelection(channelId:string,selection:ModelSelectionV2){queueChannel({kind:'set_selection',channelId,selection})}
  async function saveChannels(scopeChannelId?:string){
    if(!canSaveFile.value||!file.value?.draft)return
    if(file.value.draft.policy.recoveryId&&(scopeChannelId||channelIntents.value.length||modelEdits.value.length||keyBuffers.size||pendingRows.value.length)){fileError.value='历史恢复草稿不能混入新的渠道、型号、Key 或单渠道保存；当前输入已保留，请核对原恢复范围。';return}
    if(pendingRows.value.some(row=>!scopeChannelId||row.channel===scopeChannelId)){fileError.value='请先确认或还原本次范围未确认的型号编辑；当前输入已保留。';return}
    const scoped=channelCapabilities.value.scopedSave?scopeChannelId:undefined
    for(const [id,edit] of [...keyBuffers]){if(scoped&&id!==scoped)continue;if(edit.sourceGuard!==keySourceGuard()||edit.targetGuard!==keyTargetGuard(id)){fileError.value='Key 编辑期间来源基准已变化，输入仍保留；请重新核对该渠道与凭据意图。';return}try{await stageCredential(id,edit.action,edit.value);if(keyBuffers.get(id)===edit)setKeyBuffer(id,null)}catch(cause){fileError.value=message(cause);return}}
    if(!file.value?.draft)return
    if(pendingRows.value.some(row=>!scopeChannelId||row.channel===scopeChannelId)){fileError.value='请先确认或还原本次范围未确认的型号编辑；当前输入已保留。';return}
    const localChannel=channels.value.find(c=>c.id===scopeChannelId)
    if(localChannel&&!channelIntents.value.some(i=>(i.kind==='rename'||i.kind==='delete')&&i.channelId===localChannel.id))channelIntents.value.push({kind:'rename',channelId:localChannel.id,name:localChannel.label})
    const scope=channelCapabilities.value.scopedSave?scopeChannelId:undefined
    const submittedEdits=modelEdits.value.filter(e=>!scope||e.channelId===scope),submittedIntents=channelIntents.value.filter(i=>!scope||i.channelId===scope)
    const request={expected:clone(file.value.draft.base),edits:clone(submittedEdits),channelIntents:clone(submittedIntents),...(file.value.draft.policy.recoveryId?{scopeChannelId:null}:scope?{scopeChannelId:scope}:{})}
    const submittedSourceGuard=keySourceGuard(),submittedRoot=file.value.rootRevision,submittedLibrary=file.value.libraryDigest
    fileBusy.value=true;fileError.value=''
    try{const value=await modelV2Api.fileSave(request);acceptFile(value);if(value.rootRevision===submittedRoot&&value.libraryDigest===submittedLibrary)for(const [id,edit]of keyBuffers){if(edit.sourceGuard===submittedSourceGuard&&edit.targetGuard===keyTargetGuard(id))edit.sourceGuard=keySourceGuard()}channelIntents.value=channelIntents.value.filter(i=>!submittedIntents.includes(i));modelEdits.value=modelEdits.value.filter(e=>!submittedEdits.includes(e));await ensureFileDraft()}
    catch(cause){fileError.value=message(cause);if(typeof cause==='object'&&cause!==null&&'issues'in cause&&Array.isArray(cause.issues))issues.value=cause.issues as MatchIssueV2[]}
    finally{fileBusy.value=false}
  }
  async function stageFileConnection(intent:ChannelConnectionEditIntentV2,credential:{action:'replace';value:string}|{action:'unchanged'}){
    if(fileBusy.value)throw new Error('渠道操作正在处理，当前输入保持。')
    if(channelIntents.value.length||modelEdits.value.length||pendingRows.value.length||keyBuffers.size)throw new Error('请先保存或还原其它明确编辑，再修改连接；当前输入保持。')
    const beforeIds=new Set(channels.value.map(c=>c.id)),node=intent.nodes.find(n=>n.id===intent.activeNodeId)
    if(!node)throw new Error('请选择有效的活动节点。')
    const expected=expectedFile();fileBusy.value=true;fileError.value=''
    try{const value=await modelV2Api.stageConnection({expected:expected.expectedDraft,expectedSource:expected.expectedSource,expectedLibrary:expected.expectedLibrary,channelId:intent.kind==='create'?null:intent.channelId,name:intent.label,url:node.url,nodes:clone(intent.nodes),activeNodeId:intent.activeNodeId,authMode:intent.authMode,apiKey:credential.action==='replace'?credential.value:null,rawModelIds:[...intent.modelOrder],selectedModelIds:[...intent.selectedModelIds]});acceptFile(value);const added=intent.kind==='create'?channels.value.filter(c=>!beforeIds.has(c.id)):channels.value.filter(c=>c.id===intent.channelId);if(added.length!==1)throw new Error('连接草稿回执未定位到明确渠道，输入保持。');fileGeneration++;return {channelId:added[0]!.id}}
    finally{fileBusy.value=false}
  }
  // Discovery has no readId or saved protocol. Sequence protection belongs to its UI caller.
  services.value.previewChannel=async input=>{const kinds={compatible_models:'openai_list',anthropic_models:'anthropic_list',google_models:'google_list',ollama_models:'ollama_list'} as const;const value=await modelV2Api.discover({discoveryKind:kinds[input.kind],url:input.url,apiKey:input.apiKey});if(value.dtoVersion!==2)throw new Error('清单响应格式不支持。');return {models:value.models.map(m=>({id:m.rawModelId,label:m.label})),elapsedMs:value.elapsedMs}}
  services.value.stageConnection=stageFileConnection
  async function copyChannel(channelId:string,name:string,reuseSourceKey=false){
    if(fileBusy.value||!file.value?.draft||!channelCapabilities.value.copy)throw serviceUnavailableV2()
    if(channelIntents.value.some(i=>i.channelId===channelId)||modelEdits.value.some(e=>e.channelId===channelId)||pendingRows.value.some(row=>row.channel===channelId))throw new Error('请先保存或还原源渠道的明确编辑，再复制当前服务端渠道。')
    const keyEdit=keyBuffers.get(channelId)
    if(reuseSourceKey&&keyEdit){if(keyEdit.sourceGuard!==keySourceGuard()||keyEdit.targetGuard!==keyTargetGuard(channelId))throw new Error('Key 编辑期间来源基准已变化，输入保持，请重新核对。');await stageCredential(channelId,keyEdit.action,keyEdit.value);if(keyBuffers.get(channelId)===keyEdit)setKeyBuffer(channelId,null)}
    if(!file.value?.draft)throw new Error('复制草稿基准已变化，输入保持。')
    const beforeIds=new Set(channels.value.map(c=>c.id));fileBusy.value=true;fileError.value=''
    try{const value=await modelV2Api.copyChannel({expected:clone(file.value.draft.base),channelId,name,reuseSourceKey});acceptFile(value);const added=channels.value.filter(c=>!beforeIds.has(c.id));if(added.length!==1)throw new Error('复制回执未定位到明确的新渠道，原件保持。');fileGeneration++;return added[0]!.id}
    finally{fileBusy.value=false}
  }
  async function viewCredential(channelId:string){if(!channelCapabilities.value.credential)throw serviceUnavailableV2();return modelV2Api.credentialView(channelId)}
  async function stageCredential(channelId:string,action:'replace'|'clear',value?:string){
    if(!channelCapabilities.value.credential||fileBusy.value||!file.value?.draft)throw serviceUnavailableV2()
    fileBusy.value=true;fileError.value=''
    try{const result=await modelV2Api.stageCredential({expected:clone(file.value.draft.base),channelId,action,...(action==='replace'?{value:value??''}:{})});acceptFile(result);fileGeneration++}
    finally{fileBusy.value=false}
  }

  function configureServices(value:ModelV2Services){services.value=value;readSequence++;previewSequence++;loading.value=false;previewBusy.value=false}
  function invalidate(){editGeneration.value++;previewSequence++;preview.value=null;issues.value=[];previewError.value='';previewBusy.value=false}
  function start(template?:TemplateV2,revisionId?:string){
    if(saving.value)return
    const revision=template?.revisions.find(r=>r.id===(revisionId??template.defaultRevision))
    if(template&&!revision){error.value='指定的保存版本不可用，请核对精确依赖。';return}
    if(revision&&revision.contentVersion!==2){error.value='该内容格式暂不支持，原件保持。';return}
    if(template&&draft.value?.templateId===template.id&&draft.value.baseRevision===revision?.id)return
    retainBuffer()
    const retained=template?retainedBuffers.get(bufferKey(template.id,revision?.id??null)):undefined
    if(retained){draft.value=clone(retained.draft);clean=retained.clean;baseLibraryDigest=retained.library;invalidate();error.value='';return}
    draft.value={templateId:template?.id??crypto.randomUUID(),baseRevision:revision?.id??null,label:template?.label??'',content:revision?clone(revision.content):emptyContentV2(),makeDefault:false}
    clean=JSON.stringify(draft.value);baseLibraryDigest=snapshot.value?.libraryDigest??'';invalidate();error.value=''
  }
  function setContent(value:TemplateContentV2){if(!draft.value||saving.value)return;draft.value={...draft.value,content:clone(value)};invalidate()}
  function setLabel(value:string){if(!draft.value||saving.value)return;draft.value.label=value;invalidate()}
  function setDefault(value:boolean){if(!draft.value||saving.value)return;draft.value.makeDefault=value;invalidate()}
  function reset(){if(saving.value||!draft.value)return;draft.value=JSON.parse(clean) as RevisionBufferV2;invalidate();error.value=''}
  function copyTemplate(){if(!draft.value||saving.value)return;if(draft.value.baseRevision===null){error.value='请先保存或取消已有新建模版，再复制。';return}retainBuffer();draft.value={...clone(draft.value),templateId:crypto.randomUUID(),baseRevision:null,label:draft.value.label+' 副本',makeDefault:false};clean=JSON.stringify(draft.value);invalidate()}
  function close(){if(saving.value)return;active.value=false;readSequence++;previewSequence++;loading.value=false;previewBusy.value=false}
  async function load(){
    if(loading.value)return
    const reader=services.value.loadLibrary;if(!reader){error.value=serviceUnavailableV2().message;return}
    const seq=++readSequence;loading.value=true;error.value=''
    try{const value=await reader();if(seq!==readSequence)return;if(value.dtoVersion!==2||value.library.schemaVersion!==2)throw new Error('模型格式暂不支持，原件保持。');snapshot.value=value;if(draft.value?.baseRevision===null&&!baseLibraryDigest)baseLibraryDigest=value.libraryDigest}
    catch(cause){if(seq===readSequence)error.value=message(cause)}finally{if(seq===readSequence)loading.value=false}
  }
  async function open(){active.value=true;await load()}
  function dismissPreview(){previewSequence++;preview.value=null;previewBusy.value=false;previewError.value='';issues.value=[]}
  function setPreviewTarget(rawModelId:string,channelId:string){previewModelId.value=rawModelId;previewChannelId.value=channelId;invalidate()}
  async function requestPreview(){
    const reader=services.value.previewConditions;if(!draft.value||!reader){previewError.value=serviceUnavailableV2().message;return}
    const input={content:clone(draft.value.content),rawModelId:previewModelId.value,channelId:previewChannelId.value}
    if(!input.rawModelId.trim()){previewError.value='请输入完整原始模型 ID。';return}
    const seq=++previewSequence,generation=editGeneration.value;previewBusy.value=true;preview.value=null;previewError.value='';issues.value=[]
    try{
      if(services.value.validateContent){const result=await services.value.validateContent(input.content);if(seq!==previewSequence||generation!==editGeneration.value)return;issues.value=result;if(result.length)return}
      const result=await reader(input)
      if(seq!==previewSequence||generation!==editGeneration.value)return
      if(result.rawModelId!==input.rawModelId)throw new Error('预览结果不属于当前模型，当前输入已保留。')
      preview.value=result;issues.value=result.issues
    }catch(cause){if(seq===previewSequence&&generation===editGeneration.value){previewError.value=message(cause);if(typeof cause==='object'&&cause!==null&&'issues' in cause&&Array.isArray(cause.issues))issues.value=cause.issues as MatchIssueV2[]}}finally{if(seq===previewSequence)previewBusy.value=false}
  }
  async function save(){
    const writer=services.value.saveRevision;if(!writer||!snapshot.value||!draft.value){error.value=serviceUnavailableV2().message;return}
    if(saving.value)return
    const current=clone(draft.value),generation=editGeneration.value
    if(!current.label.trim()){error.value='请填写模版名称。';return}
    saving.value=true;error.value=''
    try{
      const beforeIds=new Set(snapshot.value.library.templates.find(t=>t.id===current.templateId)?.revisions.map(r=>r.id)??[])
      const result=await writer({...current,expectedLibrary:baseLibraryDigest||snapshot.value.libraryDigest})
      if(result.dtoVersion!==2||result.library.schemaVersion!==2)throw new Error('保存回执格式不支持，当前输入保留。')
      const appended=result.library.templates.find(t=>t.id===current.templateId)?.revisions.filter(r=>!beforeIds.has(r.id))
      if(appended?.length!==1||appended[0]!.contentVersion!==2)throw new Error('保存结果尚未核对到对应新版本，当前草稿已保留。请重新读取。')
      snapshot.value=result;baseLibraryDigest=result.libraryDigest
      if(generation===editGeneration.value&&draft.value){draft.value.baseRevision=appended[0]!.id;clean=JSON.stringify(draft.value)}
    }catch(cause){error.value=message(cause);if(typeof cause==='object'&&cause!==null&&'issues' in cause&&Array.isArray(cause.issues))issues.value=cause.issues as MatchIssueV2[]}finally{saving.value=false}
  }
  async function discover(input:ChannelDiscoveryInputV2){const reader=services.value.previewChannel;if(!reader)throw serviceUnavailableV2();return reader(input)}
  function agentResult(value:GeneratedReviewV2,runId?:string){if(value.dtoVersion!==2||runId&&value.runId!==runId||value.review&&value.review.expectedLibrary!==value.expectedLibrary)throw new Error('Agent 回执身份或版本不符，当前输入保持。');return value}
  async function runAgent(key:string,request:GenerateIntentV2,buffer?:EditingBufferInputV2){
    const session=agentSession(key);if(session.preparing||session.generating||session.accepting||session.stopping)return
    if(session.acceptRequest&&session.result?.phase!=='accepted'){session.error='原接受保存尚未确认，请先核对原任务；未新建生成任务。';return}
    const intent=clone(request),sequence=(agentSequences.get(key)??0)+1;agentSequences.set(key,sequence);agentStops.delete(key)
    session.preparing=true;session.error='';session.status='正在准备受控任务';session.inputGeneration=buffer?.generation??null;session.expectedLibrary=intent.expectedLibrary
    try{
      if(buffer){const value=await modelV2Api.agentBuffer(clone(buffer));if(sequence!==agentSequences.get(key))return;if(JSON.stringify({target:value.target,label:value.label,content:value.content,generation:value.generation})!==JSON.stringify(buffer)||!/^sha256:[0-9a-f]{64}$/.test(value.digest))throw new Error('编辑缓冲摘要回执不属于当前输入，未继续准备任务。');intent.buffer=value}
      if(agentStops.has(key)){session.status='已停止本次准备；未发起生成。';return}
      const ticket=await modelV2Api.agentPrepare(intent)
      if(ticket.dtoVersion!==2||ticket.expectedLibrary!==intent.expectedLibrary||ticket.phase!=='prepared')throw new Error('任务准备回执不完整，未发起生成。')
      if(sequence!==agentSequences.get(key))return
      session.ticket=ticket;session.preparing=false
      if(agentStops.has(key)){session.status='已停止等待；准备记录保留，未发起生成。';return}
      session.generating=true;session.status='正在执行原任务；未写入模版库。'
      const result=agentResult(await modelV2Api.agentGenerate(ticket.runId),ticket.runId)
      if(sequence!==agentSequences.get(key))return
      if(result.expectedLibrary!==intent.expectedLibrary)throw new Error('建议不属于本次库基准，原输入保持。')
      if(intent.scene==='check_only'&&result.phase==='review'&&(!result.report||result.review))throw new Error('检查任务返回了不允许的建议保存内容，未采用。')
      if(result.review?.proposals.some(item=>item.proposal.target?!intent.targets.some(target=>JSON.stringify(target)===JSON.stringify(item.proposal.target)):!intent.allowNew||intent.targets.length>0))throw new Error('建议目标超出本次精确范围，未采用。')
      session.result=result;session.acceptRequest=null;session.error=result.errorCode??''
      session.status=result.phase==='review'?(result.report?'检查报告已返回，没有配置写入。':'建议待审阅，尚未保存。'):result.phase==='stopped'?'原任务已停止。':result.phase==='failed'?'原任务未完成，输入与历史结果保持。':'原任务结果已返回。'
    }catch(cause){if(sequence===agentSequences.get(key)){session.error=message(cause);session.status='任务未确认完成；当前输入与原任务身份保留。'}}
    finally{if(sequence===agentSequences.get(key)){session.preparing=false;session.generating=false}}
  }
  async function stopAgent(key:string){const session=agentSession(key);agentStops.add(key);if(session.preparing){session.status='正在停止本次等待；不会在准备回执后自动发起生成。';return}if(!session.generating||!session.ticket||session.stopping)return;session.stopping=true;try{await modelV2Api.agentStop(session.ticket.runId);agentSequences.set(key,(agentSequences.get(key)??0)+1);session.generating=false;session.status='已停止原任务；既有结果保留，可重新读取结果。'}catch(cause){session.error=message(cause)}finally{session.stopping=false}}
  async function restoreAgent(key:string,templateId:string|null){const session=agentSession(key);if(session.preparing||session.generating||session.accepting)return;const sequence=(agentSequences.get(key)??0)+1;agentSequences.set(key,sequence);session.preparing=true;session.error='';try{const value=await modelV2Api.agentLatest(templateId);if(sequence!==agentSequences.get(key))return;if(!value){session.status='没有可恢复的原任务结果。';return}agentResult(value);if(session.acceptRequest&&value.runId!==session.acceptRequest.runId)throw new Error('最新结果不是待核对的原接受任务，原请求与建议保持。');if(templateId&&value.review&&!value.review.proposals.some(p=>p.proposal.target?.templateId===templateId))throw new Error('上次结果不属于当前模版，未替换当前结果。');session.result=value;session.status=value.phase==='accepted'?'原任务已接受保存；当前编辑缓冲保持。':'原任务结果已恢复；原始未发送输入保持，需重新核对目标与库基准。'}catch(cause){if(sequence===agentSequences.get(key))session.error=message(cause)}finally{if(sequence===agentSequences.get(key))session.preparing=false}}
  async function acceptAgent(key:string,templateId:string|null){
    const session=agentSession(key),result=session.result;if(!result||session.preparing||session.generating||session.accepting||saving.value)return
    if(result.report){session.error='检查报告没有可保存建议，未发起接受。';return}
    session.accepting=true;session.error='';saving.value=true
    try{
      if(session.acceptRequest){const latest=key.startsWith('external:')?(await modelV2Api.externalRun(session.acceptRequest.runId)).result:await modelV2Api.agentLatest(templateId);if(latest?.runId===session.acceptRequest.runId&&latest.phase==='accepted'){session.result=agentResult(latest);session.status='原接受操作已核对成功，未重复保存。';await load();return}if(!latest||latest.runId!==session.acceptRequest.runId)throw new Error('原任务结果尚无法安全核对，未重建或替换接受请求。')}
      if(!session.acceptRequest){if(snapshot.value?.libraryDigest!==result.expectedLibrary)throw {code:'model_save_stale'};session.acceptRequest={runId:result.runId,expectedLibrary:result.expectedLibrary,reviewedResponse:reviewedAgentResponseV2(result)}}
      const accepted=agentResult(await modelV2Api.agentAccept(clone(session.acceptRequest)),session.acceptRequest.runId)
      if(accepted.phase!=='accepted')throw new Error('接受结果尚未确认，原请求与输入已保留。')
      session.result=accepted;session.status='建议已按原任务另存版本；当前编辑与默认采用保持。';await load()
    }catch(cause){session.error=message(cause);session.status='接受保存未确认完成；原审阅内容和精确请求已保留。'}finally{session.accepting=false;saving.value=false}
  }
  async function changeLibrary(request:LibraryChangeV2){
    if(saving.value)throw new Error('模版保存正在处理，请稍后重试。')
    if(snapshot.value?.libraryDigest!==request.expectedLibrary)throw {code:'model_save_stale',issues:[]}
    if(request.intent.kind==='delete_revisions'&&request.intent.revisionIds.some(id=>protectsRevision(request.intent.kind==='delete_revisions'?request.intent.templateId:'',id)))throw {code:'model_revision_in_use',issues:[]}
    saving.value=true
    try{
      const result=await modelV2Api.changeLibrary(clone(request))
      if(result.dtoVersion!==2||result.library.schemaVersion!==2)throw new Error('模版变更回执格式不支持，当前库与编辑保持。')
      const intent=request.intent
      if(intent.kind==='set_default'&&result.library.templates.find(t=>t.id===intent.reference.templateId)?.defaultRevision!==intent.reference.revisionId)throw new Error('默认版本变更尚未核对到明确回执，当前库保持。')
      if(intent.kind==='clear_default'&&result.library.templates.find(t=>t.id===intent.reference.templateId)?.defaultRevision!==null)throw new Error('清除默认版本尚未核对到明确回执，当前库保持。')
      if(intent.kind==='delete_revisions'&&result.library.templates.find(t=>t.id===intent.templateId)?.revisions.some(r=>intent.revisionIds.includes(r.id)))throw new Error('删除结果未完整确认，当前版本选择保持。')
      snapshot.value=result
      // Rebase only this successful metadata operation, and only while its exact
      // editor dependency survives. External reads still cannot rebase an edit.
      const base=draft.value?.baseRevision
      if(baseLibraryDigest===request.expectedLibrary&&(!base||result.library.templates.find(t=>t.id===draft.value?.templateId)?.revisions.some(r=>r.id===base)))baseLibraryDigest=result.libraryDigest
      return result
    }finally{saving.value=false}
  }
  async function templateAction(intent:TemplateActionIntentV2){const handler=services.value.templateAction;if(!handler)throw serviceUnavailableV2();const value=await handler(clone(intent));if(value.library){if(value.library.dtoVersion!==2||value.library.library.schemaVersion!==2)throw new Error('返回的模版格式不支持，当前编辑保持。');snapshot.value=value.library}return value}
  async function stageConnection(intent:ChannelConnectionEditIntentV2,credential:{action:'replace';value:string}|{action:'unchanged'}){
    const writer=services.value.stageConnection;if(!writer)throw serviceUnavailableV2()
    const immutableIntent=clone(intent)
    const result=await writer(immutableIntent,credential)
    // Credentials/source/effective/auto binding never enter this inspectable intent list.
    stagedConnectionIntents.value=[...stagedConnectionIntents.value.filter(item=>item.channelId!==result.channelId),{...immutableIntent,channelId:result.channelId}]
    return result
  }
  return {discardChannels,assertHistoryReady,acceptHistory,policyNotificationTarget,dismissPreview,externalTasks,externalCurrentId,externalNotificationTarget,protectsRevision,discardNew,modelRows,pendingRows,rowsFor,projectFile,adoptProjection,fileContext,effectiveRows,active,loading,saving,previewBusy,snapshot,draft,error,previewError,issues,preview,previewModelId,previewChannelId,editGeneration,dirty,canLoad,canPreview,canSave,canDiscover,canStageConnection,stagedConnectionIntents,configureServices,start,setContent,setLabel,setDefault,reset,close,load,open,setPreviewTarget,requestPreview,save,discover,stageConnection,file,fileBusy,fileError,fileNeedsReconcile,channels,channelIntents,modelEdits,readResults,channelEditorId,channelSearch,fileDirty,channelDirty,canSaveFile,loadChannels,readExisting,queueChannel,queueModel,explicitRules,setSelection,saveChannels,channelCapabilities,configureChannelCapabilities,copyChannel,viewCredential,stageCredential,pendingCredentialKinds,keyBuffer,setKeyBuffer,credentialInputRevision,view,copyTemplate,changeLibrary,templateAction,executionPreference,executionPreferenceRevision,loadAgentSettings,bindAgent,agentSessions,agentSession,runAgent,stopAgent,restoreAgent,acceptAgent}
})
