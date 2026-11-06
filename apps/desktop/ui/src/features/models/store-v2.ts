import {computed,ref,shallowRef} from 'vue'
import {defineStore} from 'pinia'
import {
  emptyContentV2,serviceUnavailableV2,modelV2Api,type ModelV2Services,type LibraryViewV2,type TemplateContentV2,
  type TemplateV2,type ConditionPreviewView,type MatchIssueV2,type ChannelConnectionEditIntentV2,
  type ChannelDiscoveryInputV2,
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
  let clean='',readSequence=0,previewSequence=0,baseLibraryDigest=''
  const dirty=computed(()=>!!draft.value&&JSON.stringify(draft.value)!==clean)
  const canLoad=computed(()=>!!services.value.loadLibrary)
  const canPreview=computed(()=>!!services.value.previewConditions)
  const canSave=computed(()=>!!services.value.saveRevision&&!!snapshot.value)
  const canDiscover=computed(()=>!!services.value.previewChannel)
  const canStageConnection=computed(()=>!!services.value.stageConnection)

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
  async function stageConnection(intent:ChannelConnectionEditIntentV2,credential:{action:'replace';value:string}|{action:'unchanged'}){
    const writer=services.value.stageConnection;if(!writer)throw serviceUnavailableV2()
    const immutableIntent=clone(intent)
    const result=await writer(immutableIntent,credential)
    // Credentials/source/effective/auto binding never enter this inspectable intent list.
    stagedConnectionIntents.value=[...stagedConnectionIntents.value.filter(item=>item.channelId!==intent.channelId),immutableIntent]
    return result
  }
  return {active,loading,saving,previewBusy,snapshot,draft,error,previewError,issues,preview,previewModelId,previewChannelId,editGeneration,dirty,canLoad,canPreview,canSave,canDiscover,canStageConnection,stagedConnectionIntents,configureServices,start,setContent,setLabel,setDefault,reset,close,load,open,setPreviewTarget,requestPreview,save,discover,stageConnection}
})
