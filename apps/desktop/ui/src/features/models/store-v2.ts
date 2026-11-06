import {computed,ref,shallowRef} from 'vue'
import {defineStore} from 'pinia'
import {
  emptyContentV2,serviceUnavailableV2,modelV2Api,type ModelV2Services,type LibraryViewV2,type TemplateContentV2,
  type TemplateV2,type ConditionPreviewView,type MatchIssueV2,type ChannelConnectionEditIntentV2,
  type ChannelDiscoveryInputV2,
  type FileSnapshotV2,type ReadExistingRequestV2,type ChannelIntentV2,type ModelEditV2,type ModelSelectionV2,
  type TemplateActionIntentV2,
} from './api-v2'

const clone=<T>(value:T):T=>JSON.parse(JSON.stringify(value)) as T
const message=(cause:unknown)=>cause instanceof Error?cause.message:typeof cause==='object'&&cause!==null&&'code' in cause?String(cause.code):'操作未完成，当前输入已保留。'
export interface RevisionBufferV2 {templateId:string;baseRevision:string|null;label:string;content:TemplateContentV2;makeDefault:boolean}

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
  const file=shallowRef<FileSnapshotV2|null>(null),fileBusy=ref(false),fileError=ref(''),fileNeedsReconcile=ref(false)
  const channelIntents=ref<ChannelIntentV2[]>([]),modelEdits=ref<ModelEditV2[]>([])
  const readResults=shallowRef<readonly import('./api-v2').ReadMatchDtoV2[]>([])
  const channelEditorId=ref(''),channelSearch=ref('')
  // Private, session-only Key input. Never included in $state, source DTOs or logs.
  const keyBuffers=new Map<string,{action:'replace'|'clear';value?:string;sourceGuard?:string}>(),keyBufferVersion=ref(0)
  const credentialInputRevision=computed(()=>keyBufferVersion.value)
  const keySourceGuard=()=>JSON.stringify({source:file.value?.sourceDigest,library:file.value?.libraryDigest,root:file.value?.rootRevision})
  const pendingCredentialKinds=computed(()=>{void keyBufferVersion.value;return Object.fromEntries([...keyBuffers].map(([id,v])=>[id,v.action]))})
  function keyBuffer(channelId:string){const value=keyBuffers.get(channelId);return value?{...value}:null}
  function setKeyBuffer(channelId:string,value:{action:'replace'|'clear';value?:string}|null){if(value)keyBuffers.set(channelId,{...value,sourceGuard:keySourceGuard()});else keyBuffers.delete(channelId);keyBufferVersion.value++}
  const view=ref({tab:'channels' as 'channels'|'templates',templateSearch:'',templateEditor:false,templateNavCollapsed:false,templateSection:'model-basic',selectedVersions:{} as Record<string,string>,selectedTemplates:[] as string[]})
  // Registered/compiled together with L0 baseline a3a20621d29318353bfe8afe624222d0b4b12584.
  const channelCapabilities=ref({scopedSave:true,defaults:true,naming:true,templateRefs:true,copy:true,delete:true,credential:false,measure:false,activeNode:false})
  function configureChannelCapabilities(value:Partial<typeof channelCapabilities.value>){channelCapabilities.value={...channelCapabilities.value,...value}}
  let fileSequence=0,fileGeneration=0,openingDraft:Promise<void>|null=null
  const channels=computed(()=>{
    const result=clone(file.value?.draft?.policy.channels??file.value?.savedPolicy.channels??[])
    for(const intent of channelIntents.value){const channel=result.find(c=>c.id===intent.channelId);if(!channel)continue;switch(intent.kind){case'rename':channel.label=intent.name;break;case'set_enabled':channel.connection.enabled=intent.enabled;break;case'set_selection':channel.selection=clone(intent.selection);break;case'set_order':channel.modelOrder=[...intent.rawModelIds];break;case'set_defaults':channel.defaults=clone(intent.rules);break;case'set_family_defaults':channel.familyOverrides[intent.templateId]=clone(intent.rules);break;case'set_naming':channel.naming[intent.rawModelId]={...(intent.alias===null?{}:{alias:intent.alias}),...(intent.displayName===null?{}:{displayName:intent.displayName})};break;case'set_template_refs':channel.templateRefs=clone(intent.references);break;case'set_active_node':channel.connection.activeNodeId=intent.nodeId;break;case'delete':break}}
    return result
  })
  const fileDirty=computed(()=>Object.keys(pendingCredentialKinds.value).length>0||Object.keys(file.value?.draft?.credentialEdits??{}).length>0||channelIntents.value.length>0||modelEdits.value.length>0||!!file.value?.draft&&JSON.stringify(file.value.draft.policy)!==JSON.stringify(file.value.savedPolicy))
  const canSaveFile=computed(()=>!!file.value?.draft&&!fileBusy.value&&!fileNeedsReconcile.value)
  let clean='',readSequence=0,previewSequence=0,baseLibraryDigest=''
  const dirty=computed(()=>!!draft.value&&(draft.value.baseRevision===null||JSON.stringify(draft.value)!==clean))
  const canLoad=computed(()=>!!services.value.loadLibrary)
  const canPreview=computed(()=>!!services.value.previewConditions)
  const canSave=computed(()=>!!services.value.saveRevision&&!!snapshot.value)
  const canDiscover=computed(()=>!!services.value.previewChannel)
  const canStageConnection=computed(()=>!!services.value.stageConnection)

  function acceptFile(value:FileSnapshotV2){if(value.dtoVersion!==2||value.savedPolicy.schemaVersion!==2||value.library.schemaVersion!==2||value.draft&&value.draft.schemaVersion!==2)throw new Error('渠道格式暂不支持，当前输入保持。');file.value=value;snapshot.value={dtoVersion:2,library:value.library,libraryDigest:value.libraryDigest}}
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
    if((channelIntents.value.length||modelEdits.value.length)&&!discardLocal){fileError.value='当前有未保存的明确编辑，请先保存或还原后重新读取。';return}
    const seq=++fileSequence;fileBusy.value=true;fileError.value=''
    try{const value=await modelV2Api.fileSnapshot();if(seq!==fileSequence)return;acceptFile(value);if(discardLocal){channelIntents.value=[];modelEdits.value=[];keyBuffers.clear();keyBufferVersion.value++;fileGeneration++}await ensureFileDraft();fileNeedsReconcile.value=false}
    catch(cause){if(seq===fileSequence)fileError.value=message(cause)}finally{if(seq===fileSequence)fileBusy.value=false}
  }
  async function readExisting(){
    if(fileBusy.value)return
    if(channelIntents.value.length||modelEdits.value.length){fileError.value='请先保存或还原当前明确编辑；本次读取未覆盖草稿。';return}
    const request=expectedFile(),generation=fileGeneration,seq=++fileSequence;fileBusy.value=true;fileError.value=''
    try{const response=await modelV2Api.readExisting(request);if(seq!==fileSequence||generation!==fileGeneration){fileNeedsReconcile.value=true;return}acceptFile(response.snapshot);readResults.value=response.results;fileNeedsReconcile.value=false}
    catch(cause){if(seq===fileSequence)fileError.value=message(cause)}finally{if(seq===fileSequence)fileBusy.value=false}
  }
  function queueChannel(intent:ChannelIntentV2){if(fileBusy.value)return;if((intent.kind==='set_defaults'||intent.kind==='set_family_defaults')&&intent.rules.protocol.mode!=='inherit'){fileError.value='渠道层不能设置请求协议，请在模版或单模型中明确配置。';return}const same=(old:ChannelIntentV2)=>old.channelId===intent.channelId&&old.kind===intent.kind&&(old.kind==='set_family_defaults'&&intent.kind==='set_family_defaults'?old.templateId===intent.templateId:old.kind==='set_naming'&&intent.kind==='set_naming'?old.rawModelId===intent.rawModelId:true);channelIntents.value=channelIntents.value.filter(old=>!same(old)).concat([clone(intent)]);fileGeneration++}
  function queueModel(edit:ModelEditV2){if(fileBusy.value)return;const same=(old:ModelEditV2)=>old.channelId===edit.channelId&&old.rawModelId===edit.rawModelId&&('field' in old&&'field' in edit?old.field===edit.field:!('field' in old)&&!('field' in edit));modelEdits.value=modelEdits.value.filter(old=>!same(old)).concat([clone(edit)]);fileGeneration++}
  function explicitRules(channelId:string,rawModelId:string){const model=channels.value.find(c=>c.id===channelId)?.models.find(m=>m.rawModelId===rawModelId);const rules=clone(model?.explicitOverrides??emptyContentV2().commonRules) as unknown as Record<string,unknown>;for(const edit of modelEdits.value.filter(e=>e.channelId===channelId&&e.rawModelId===rawModelId)){if(edit.kind==='set_override')rules[edit.field]={mode:'custom',value:clone(edit.value)};else if(edit.kind==='clear_override')rules[edit.field]={mode:'inherit'}}return rules as unknown as import('./api-v2').RulesV2}
  function setSelection(channelId:string,selection:ModelSelectionV2){queueChannel({kind:'set_selection',channelId,selection})}
  async function saveChannels(scopeChannelId?:string){
    if(!canSaveFile.value||!file.value?.draft)return
    const scoped=channelCapabilities.value.scopedSave?scopeChannelId:undefined
    for(const [id,edit] of [...keyBuffers]){if(scoped&&id!==scoped)continue;if(edit.sourceGuard!==keySourceGuard()){fileError.value='Key 编辑期间来源基准已变化，输入仍保留；请重新核对该渠道与凭据意图。';return}try{await stageCredential(id,edit.action,edit.value);setKeyBuffer(id,null)}catch(cause){fileError.value=message(cause);return}}
    if(!file.value?.draft)return
    const localChannel=channels.value.find(c=>c.id===scopeChannelId)
    if(localChannel&&!channelIntents.value.some(i=>(i.kind==='rename'||i.kind==='delete')&&i.channelId===localChannel.id))queueChannel({kind:'rename',channelId:localChannel.id,name:localChannel.label})
    const scope=channelCapabilities.value.scopedSave?scopeChannelId:undefined
    const request={expected:clone(file.value.draft.base),edits:clone(modelEdits.value.filter(e=>!scope||e.channelId===scope)),channelIntents:clone(channelIntents.value.filter(i=>!scope||i.channelId===scope)),...(scope?{scopeChannelId:scope}:{})},generation=fileGeneration
    fileBusy.value=true;fileError.value=''
    try{const value=await modelV2Api.fileSave(request);acceptFile(value);if(generation===fileGeneration){channelIntents.value=channelIntents.value.filter(i=>!!scope&&i.channelId!==scope);modelEdits.value=modelEdits.value.filter(e=>!!scope&&e.channelId!==scope)}else fileNeedsReconcile.value=true;await ensureFileDraft()}
    catch(cause){fileError.value=message(cause);if(typeof cause==='object'&&cause!==null&&'issues'in cause&&Array.isArray(cause.issues))issues.value=cause.issues as MatchIssueV2[]}
    finally{fileBusy.value=false}
  }
  async function stageFileConnection(intent:ChannelConnectionEditIntentV2,credential:{action:'replace';value:string}|{action:'unchanged'}){
    if(fileBusy.value)throw new Error('渠道操作正在处理，当前输入保持。')
    if(channelIntents.value.length||modelEdits.value.length)throw new Error('请先保存或还原其它明确编辑，再修改连接；当前输入保持。')
    const beforeIds=new Set(channels.value.map(c=>c.id)),node=intent.nodes.find(n=>n.id===intent.activeNodeId)
    if(!node)throw new Error('请选择有效的活动节点。')
    const expected=expectedFile();fileBusy.value=true;fileError.value=''
    try{const value=await modelV2Api.stageConnection({expected:expected.expectedDraft,expectedSource:expected.expectedSource,expectedLibrary:expected.expectedLibrary,channelId:intent.kind==='create'?null:intent.channelId,name:intent.label,url:node.url,nodes:clone(intent.nodes),activeNodeId:intent.activeNodeId,authMode:intent.authMode,apiKey:credential.action==='replace'?credential.value:null,rawModelIds:[...intent.modelOrder],selectedModelIds:[...intent.selectedModelIds]});acceptFile(value);const added=intent.kind==='create'?channels.value.filter(c=>!beforeIds.has(c.id)):channels.value.filter(c=>c.id===intent.channelId);if(added.length!==1)throw new Error('连接草稿回执未定位到明确渠道，输入保持。');fileGeneration++;return {channelId:added[0]!.id}}
    finally{fileBusy.value=false}
  }
  // Discovery has no readId or saved protocol. Sequence protection belongs to its UI caller.
  services.value.previewChannel=async input=>{const kinds={compatible_models:'openai_list',anthropic_models:'anthropic_list',google_models:'google_list',ollama_models:'ollama_list'} as const;const value=await modelV2Api.discover({discoveryKind:kinds[input.kind],url:input.url,apiKey:input.apiKey});if(value.dtoVersion!==2)throw new Error('清单响应格式不支持。');return {models:value.models.map(m=>({id:m.rawModelId,label:m.label})),elapsedMs:value.elapsedMs}}
  services.value.stageConnection=stageFileConnection
  async function copyChannel(channelId:string,name:string){
    if(fileBusy.value||!file.value?.draft||!channelCapabilities.value.copy)throw serviceUnavailableV2()
    if(channelIntents.value.length||modelEdits.value.length)throw new Error('请先保存或还原明确编辑，再复制当前服务端渠道。')
    const beforeIds=new Set(channels.value.map(c=>c.id));fileBusy.value=true;fileError.value=''
    try{const value=await modelV2Api.copyChannel({expected:clone(file.value.draft.base),channelId,name});acceptFile(value);const added=channels.value.filter(c=>!beforeIds.has(c.id));if(added.length!==1)throw new Error('复制回执未定位到明确的新渠道，原件保持。');fileGeneration++;return added[0]!.id}
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
    draft.value={templateId:template?.id??crypto.randomUUID(),baseRevision:revision?.id??null,label:template?.label??'',content:revision?clone(revision.content):emptyContentV2(),makeDefault:false}
    clean=JSON.stringify(draft.value);baseLibraryDigest=snapshot.value?.libraryDigest??'';invalidate();error.value=''
  }
  function setContent(value:TemplateContentV2){if(!draft.value||saving.value)return;draft.value={...draft.value,content:clone(value)};invalidate()}
  function setLabel(value:string){if(!draft.value||saving.value)return;draft.value.label=value;invalidate()}
  function setDefault(value:boolean){if(!draft.value||saving.value)return;draft.value.makeDefault=value;invalidate()}
  function reset(){if(saving.value||!draft.value)return;draft.value=JSON.parse(clean) as RevisionBufferV2;invalidate();error.value=''}
  function copyTemplate(){if(!draft.value||saving.value)return;if(draft.value.baseRevision===null){error.value='请先保存或取消已有新建模版，再复制。';return}draft.value={...clone(draft.value),templateId:crypto.randomUUID(),baseRevision:null,label:draft.value.label+' 副本',makeDefault:false};clean=JSON.stringify(draft.value);invalidate()}
  function close(){if(saving.value)return;active.value=false;readSequence++;previewSequence++;loading.value=false;previewBusy.value=false}
  async function load(){
    if(loading.value)return
    const reader=services.value.loadLibrary;if(!reader){error.value=serviceUnavailableV2().message;return}
    const seq=++readSequence;loading.value=true;error.value=''
    try{const value=await reader();if(seq!==readSequence)return;if(value.dtoVersion!==2||value.library.schemaVersion!==2)throw new Error('模型格式暂不支持，原件保持。');snapshot.value=value;if(draft.value?.baseRevision===null&&!baseLibraryDigest)baseLibraryDigest=value.libraryDigest}
    catch(cause){if(seq===readSequence)error.value=message(cause)}finally{if(seq===readSequence)loading.value=false}
  }
  async function open(){active.value=true;await load()}
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
  async function templateAction(intent:TemplateActionIntentV2){const handler=services.value.templateAction;if(!handler)throw serviceUnavailableV2();const value=await handler(clone(intent));if(value.library){if(value.library.dtoVersion!==2||value.library.library.schemaVersion!==2)throw new Error('返回的模版格式不支持，当前编辑保持。');snapshot.value=value.library}return value}
  async function stageConnection(intent:ChannelConnectionEditIntentV2,credential:{action:'replace';value:string}|{action:'unchanged'}){
    const writer=services.value.stageConnection;if(!writer)throw serviceUnavailableV2()
    const immutableIntent=clone(intent)
    const result=await writer(immutableIntent,credential)
    // Credentials/source/effective/auto binding never enter this inspectable intent list.
    stagedConnectionIntents.value=[...stagedConnectionIntents.value.filter(item=>item.channelId!==result.channelId),{...immutableIntent,channelId:result.channelId}]
    return result
  }
  return {active,loading,saving,previewBusy,snapshot,draft,error,previewError,issues,preview,previewModelId,previewChannelId,editGeneration,dirty,canLoad,canPreview,canSave,canDiscover,canStageConnection,stagedConnectionIntents,configureServices,start,setContent,setLabel,setDefault,reset,close,load,open,setPreviewTarget,requestPreview,save,discover,stageConnection,file,fileBusy,fileError,fileNeedsReconcile,channels,channelIntents,modelEdits,readResults,channelEditorId,channelSearch,fileDirty,canSaveFile,loadChannels,readExisting,queueChannel,queueModel,explicitRules,setSelection,saveChannels,channelCapabilities,configureChannelCapabilities,copyChannel,viewCredential,stageCredential,pendingCredentialKinds,keyBuffer,setKeyBuffer,credentialInputRevision,view,copyTemplate,templateAction}
})
