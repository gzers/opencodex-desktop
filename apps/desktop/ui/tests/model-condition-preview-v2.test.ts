import {beforeEach,expect,it,vi} from 'vitest'
import {mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ConditionPreview from '@/features/models/ConditionPreview.vue'
import {emptyRulesV2,emptyContentV2,conditionPreviewViewV2,type ConditionPreviewView,type EffectiveModelV2,type PreviewConditionsResponseV2} from '@/features/models/api-v2'
import {useModelsV2Store} from '@/features/models/store-v2'
const ipc=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
const origin={kind:'source' as const,evidence:{kind:'imported_file' as const,sourceDigest:'sha256:'+'a'.repeat(64),providerId:'p',pointer:'/provider/p/adapter'}}
function result(rawModelId='gpt-6-1-sol'):ConditionPreviewView {const effective:EffectiveModelV2={channelId:'c',rawModelId,selected:true,enabled:true,observed:true,binding:null,rules:{...emptyRulesV2(),fast:{mode:'custom',value:false},fastWire:{mode:'custom',value:null}},protocol:{kind:'resolved',value:'openai-chat',origin},fieldOrigins:{protocol:origin,fast:{kind:'explicit_model'}},matchedConditionId:null,notices:[]};return {rawModelId,matchingView:'backend-matching-view',matched:true,family:{matched:true,reasonCode:'family_fixture',reason:'后端族理由'},reasonCode:'summary_fixture',reason:'原始协议来源保留',conditions:[],effective,issues:[]}}
beforeEach(()=>{setActivePinia(createPinia());ipc.mockReset()})
it('renders exact raw ID, actual source and explicit false/null without inventing matching-view or binding facts',()=>{const view=result(),before=JSON.stringify(view),w=mount(ConditionPreview,{props:{result:view}});expect(w.text()).toContain('gpt-6-1-sol');expect(w.text()).toContain('实际导入事实');expect(w.text()).toContain('backend-matching-view');expect(w.text()).toContain('后端族理由');expect(w.text()).not.toContain('未返回独立匹配视图');expect(w.text()).toContain('明确禁用');expect(w.text()).toContain('单模型人工覆盖');expect(JSON.stringify(view)).toBe(before);expect(w.findAll('details')).toHaveLength(1);expect(w.get('summary').text()).toBe('条件标识与匹配诊断');w.unmount()})
it('missing and unsupported protocols are visible, never silently responses',async()=>{const view=result();view.effective={...view.effective!,protocol:{kind:'missing'}};const w=mount(ConditionPreview,{props:{result:view}});expect(w.text()).toContain('待配置');expect(w.text()).not.toContain('openai-responses');await w.setProps({result:{...view,effective:{...view.effective!,protocol:{kind:'unsupported',value:'custom-adapter',origin}}}});expect(w.text()).toContain('custom-adapter');w.unmount()})
it('loading/error cannot display the old green result and retry is explicit',async()=>{const w=mount(ConditionPreview,{props:{result:result(),busy:true}});expect(w.text()).toContain('正在核对');expect(w.text()).not.toContain('原始协议来源保留');await w.setProps({busy:false,error:'model_read_stale'});expect(w.get('[role=alert]').text()).toContain('model_read_stale');await w.get('button').trigger('click');expect(w.emitted('retry')).toHaveLength(1);w.unmount()})
it('input changes reject a late preview response and retain the newer draft',async()=>{let finish!:(value:ConditionPreviewView)=>void;const store=useModelsV2Store();store.configureServices({previewConditions:()=>new Promise(resolve=>{finish=resolve})});store.start();store.setPreviewTarget('first','channel');const pending=store.requestPreview();store.setPreviewTarget('later','channel');finish(result('first'));await pending;expect(store.preview).toBeNull();expect(store.previewModelId).toBe('later');expect(store.previewBusy).toBe(false)})
it('real preview is validate -> one Rust command; in-memory reference is not submitted as a saved binding',async()=>{const store=useModelsV2Store();store.start();store.setPreviewTarget('raw-id','00000000-0000-4000-8000-000000000001');ipc.mockImplementation(async command=>command==='model_v2_validate_content'?undefined:{dtoVersion:2,previewOnly:true,contentDigest:'digest',rows:[{rawModelId:'raw-id',matchingView:'backend-view',familyMatched:true,family:{matched:true,reasonCode:'family_fixture',reason:'后端原文族理由'},reasonCode:'summary_fixture',reason:'后端原文汇总',conditions:[],matchedConditionId:null,rules:emptyRulesV2(),protocol:{kind:'missing'},fieldOrigins:{}}]});await store.requestPreview();expect(ipc.mock.calls.map(c=>c[0])).toEqual(['model_v2_validate_content','model_v2_preview_conditions']);expect(ipc.mock.calls[1]![1]).toHaveProperty('request.rawModelIds',['raw-id']);expect(store.preview?.effective).toBeNull();expect(store.preview?.configuration?.protocol.kind).toBe('missing');expect(ipc).not.toHaveBeenCalledWith('model_v2_save_revision',expect.anything());})
it('preserves backend error paths and recovery actions when validation rejects',async()=>{const store=useModelsV2Store();store.start();store.setPreviewTarget('raw-id','c');ipc.mockRejectedValue({code:'model_selector_duplicate',issues:[{code:'model_selector_duplicate',channelId:null,rawModelId:null,fieldPath:'conditionOverrides[0].selector',conditionIds:['condition'],reference:null,message:'等价条件重复',recoveryActions:['edit_conditions']}]});await store.requestPreview();expect(store.issues[0]?.fieldPath).toBe('conditionOverrides[0].selector');expect(store.issues[0]?.recoveryActions).toEqual(['edit_conditions']);expect(ipc).toHaveBeenCalledTimes(1);expect(store.draft).not.toBeNull()})

function traceFixture(){
  const content=emptyContentV2()
  content.nameLayouts=[{id:'layout',literalPrefix:'raw/',versionCapture:'complete_numeric_components',numericJoiners:['.','-'],tailCapture:'literal_remainder',trailingDatePolicy:'retain_valid_YYYYMMDD_after_hyphen',ambiguousNumericTail:'no_equivalence'}]
  content.conditionOverrides=[{id:'first',name:'first',selector:{all:[{kind:'numeric_version',components:['06','01']},{kind:'literal_segment',value:'sol',position:'after_version',boundary:'hyphen_or_string_edge'}]},rules:emptyRulesV2(),lockedFields:[],sourceExactId:'raw/06-01-sol',scope:'known_identity_only'},{id:'second',name:'second',selector:{all:[{kind:'exact_id',value:'raw/06-01-sol'}]},rules:emptyRulesV2(),lockedFields:[],sourceExactId:'raw/06-01-sol',scope:'known_identity_only'}]
  content.conditionOverrides[0]!.selector.layoutId='layout';content.conditionOverrides[0]!.selector.literalTail='-sol'
  const trace=(matched:boolean|null,reason:string)=>({matched,reasonCode:'backend_'+reason,reason})
  const response:PreviewConditionsResponseV2={dtoVersion:2,previewOnly:true,contentDigest:'digest',rows:[{rawModelId:'raw/06-01-sol',matchingView:'06-01-sol',familyMatched:true,family:{...trace(true,'family'),matched:true},reasonCode:'backend_summary',reason:'后端汇总原文',conditions:[{conditionId:'first',applicable:true,applied:false,...trace(false,'tail-failure'),preconditions:[{kind:'layout',...trace(true,'layout')},{kind:'literal_tail',...trace(false,'literal-tail')}],atoms:[{index:0,kind:'numeric_version',...trace(null,'not-evaluated-numeric')},{index:1,kind:'literal_segment',...trace(null,'not-evaluated-name')}]},{conditionId:'second',applicable:true,applied:true,...trace(true,'exact-match'),preconditions:[],atoms:[{index:0,kind:'exact_id',...trace(true,'exact-id')}]}],matchedConditionId:'second',rules:emptyRulesV2(),protocol:{kind:'missing'},fieldOrigins:{}}]}
  return {input:{content,channelId:'c',rawModelId:'raw/06-01-sol'},response}
}
it('copies the full ordered backend trace without deriving names, matching view, reasons or applied flags',()=>{
  const {input,response}=traceFixture(),before=JSON.stringify(response),view=conditionPreviewViewV2(response,input)
  expect(view.rawModelId).toBe('raw/06-01-sol');expect(view.matchingView).toBe('06-01-sol');expect(view.reason).toBe('后端汇总原文');expect(view.conditions).toEqual(response.rows[0]!.conditions);expect(view.conditions.map(c=>c.applied)).toEqual([false,true]);expect(view.conditions[0]!.atoms.map(a=>a.matched)).toEqual([null,null]);expect(JSON.stringify(response)).toBe(before)
})
it('renders every condition, failed precondition and unevaluated atom separately from false and adopted results',()=>{
  const {input,response}=traceFixture(),w=mount(ConditionPreview,{props:{result:conditionPreviewViewV2(response,input)}})
  const first=w.get('[aria-label="条件核对1"]'),second=w.get('[aria-label="条件核对2"]')
  expect(first.text()).toContain('未命中覆盖');expect(first.text()).toContain('未采用');expect(first.text()).toContain('literal-tail');expect(first.get('[aria-label="AND 条件"]').text()).toContain('未评估／不适用');expect(first.get('[aria-label="AND 条件"]').text()).not.toContain('不满足');expect(second.text()).toContain('已采用');expect(second.text()).toContain('exact-id');expect(w.get('details').text()).toContain('backend_summary');w.unmount()
})
it('family rejection displays null as not applicable rather than a failed atom',()=>{
  const {input,response}=traceFixture(),row=response.rows[0]!
  row.familyMatched=false;row.family={matched:false,reasonCode:'backend_excluded',reason:'后端排除理由'}
  for(const c of row.conditions){c.applicable=false;c.matched=null;c.applied=false;for(const a of c.atoms)a.matched=null}
  const w=mount(ConditionPreview,{props:{result:conditionPreviewViewV2(response,input)}})
  expect(w.get('[aria-label="族范围核对"]').text()).toContain('后端排除理由');expect(w.get('[aria-label="条件核对1"]').text()).toContain('未评估／不适用');expect(w.get('[aria-label="条件核对2"]').text()).toContain('未采用');w.unmount()
})
it('keeps identifiers and every original diagnostic code in a closed details, while default content retains reasons and states',async()=>{
  const {input,response}=traceFixture(),view=conditionPreviewViewV2(response,input),before=JSON.stringify(view)
  view.conditions[0]!.conditionId='11111111-1111-4111-8111-111111111111'
  const w=mount(ConditionPreview,{props:{result:view}}),details=w.get('details[aria-label="条件标识与匹配诊断"]')
  expect(details.get('summary').text()).toBe('条件标识与匹配诊断');expect(details.attributes('open')).toBeUndefined()
  const visible=w.element.cloneNode(true) as HTMLElement;visible.querySelectorAll('details').forEach(d=>d.remove())
  expect(visible.textContent).not.toContain('11111111-1111-4111-8111-111111111111');expect(visible.textContent).not.toContain('backend_')
  for(const text of ['后端汇总原文','tail-failure','not-evaluated-name','未评估／不适用','不满足','已采用','未采用'])expect(visible.textContent).toContain(text)
  expect(JSON.parse(details.get('pre').text())).toEqual({reasonCode:view.reasonCode,family:view.family,conditions:view.conditions})
  ;(details.element as HTMLDetailsElement).open=true;await details.trigger('toggle');expect(details.attributes('open')).toBeDefined()
  ;(details.element as HTMLDetailsElement).open=false;await details.trigger('toggle');expect(details.attributes('open')).toBeUndefined()
  view.conditions[0]!.conditionId='first';expect(JSON.stringify(view)).toBe(before);w.unmount()
})
it('rejects legacy or partial trace receipts instead of manufacturing empty arrays or explanations',()=>{
  const {input,response}=traceFixture()
  for(const mutate of [
    (row:Record<string,unknown>)=>delete row.matchingView,
    (row:Record<string,unknown>)=>delete row.family,
    (row:Record<string,unknown>)=>delete row.conditions,
    (row:Record<string,unknown>)=>delete row.reasonCode,
    (row:Record<string,unknown>)=>{row.conditions=[]},
    (row:Record<string,unknown>)=>{(row.conditions as Record<string,unknown>[])[0]!.atoms=[]},
    (row:Record<string,unknown>)=>{delete (row.conditions as Record<string,unknown>[])[0]!.preconditions},
  ]){const changed=JSON.parse(JSON.stringify(response));mutate(changed.rows[0]);expect(()=>conditionPreviewViewV2(changed,input)).toThrow('缺少完整匹配过程')}
})
it('default IPC adapter retains input and exposes incomplete legacy receipt as error',async()=>{
  const store=useModelsV2Store();store.start();store.setPreviewTarget('raw-id','c');const before=JSON.stringify(store.draft)
  ipc.mockImplementation(async command=>command==='model_v2_validate_content'?undefined:{dtoVersion:2,previewOnly:true,contentDigest:'digest',rows:[{rawModelId:'raw-id',familyMatched:true,matchedConditionId:null,rules:emptyRulesV2(),protocol:{kind:'missing'},fieldOrigins:{}}]})
  await store.requestPreview();expect(store.preview).toBeNull();expect(store.previewError).toContain('缺少完整匹配过程');expect(JSON.stringify(store.draft)).toBe(before)
})
it('dismiss cancels a pending preview and its late response cannot restore the result',async()=>{
  let finish!:(value:ConditionPreviewView)=>void;const store=useModelsV2Store();store.configureServices({previewConditions:()=>new Promise(resolve=>{finish=resolve})});store.start();store.setPreviewTarget('raw-id','c');const pending=store.requestPreview();store.dismissPreview();finish(result('raw-id'));await pending;expect(store.preview).toBeNull();expect(store.previewBusy).toBe(false);expect(store.previewError).toBe('')
})
