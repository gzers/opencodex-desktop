<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import ModelChoiceOptions from './ModelChoiceOptions.vue'
import type { ModelProjection } from './api'
export type ModelChoice = ({channelId:string;modelId:string;nativeModelId?:never}|{nativeModelId:string;channelId?:never;modelId?:never}) & {reasoningEffort?:string;fast?:boolean}
const props = defineProps<{ open: boolean; rows: ModelProjection[]; channels: { id: string; label: string }[]; multiple?: boolean; ordered?: boolean; reasoning?: boolean; current?: ModelChoice[]; title?: string;recommendations?:ModelChoice[];loading?:boolean;error?:string }>()
const emit = defineEmits<{ close: []; confirm: [choices: ModelChoice[]];retry:[] }>()
const search=ref('');const channelSearch=ref('');const channel=ref('');const picks=ref<ModelChoice[]>([]);const focused=ref('');const table=ref<HTMLElement|null>(null)
type SortColumn='model'|'channel'|'context'|'effort'|'state'|'recommendation'
const sorting=ref<SortColumn|null>(null);const descending=ref(false)
const heads=computed<Array<readonly [SortColumn,string]>>(()=>[['model','模型'],['channel','渠道'],['context','上下文'],['effort','推理档位'],['state','状态'],...(props.recommendations?.length?[['recommendation','推荐'] as const]:[])])
let floatingBefore:ModelChoice[]=[]
watch(focused,value=>{if(value)floatingBefore=picks.value.map(p=>({...p}))})
function closeOptions(instance:string){if(focused.value===instance)focused.value=''}
function revertOptions(){picks.value=floatingBefore.map(p=>({...p}));focused.value=''}
const key=(row:{channelId?:string;modelId?:string;nativeModelId?:string|null})=>row.nativeModelId?JSON.stringify(['native',row.nativeModelId]):JSON.stringify([row.channelId,row.modelId])
const identity=(row:ModelChoice)=>row.nativeModelId?{nativeModelId:row.nativeModelId}:{channelId:row.channelId!,modelId:row.modelId!}
const rowChoice=(row:ModelProjection):ModelChoice=>row.nativeModelId?{nativeModelId:row.nativeModelId}:{channelId:row.channelId,modelId:row.modelId}
const pickId=(pick:ModelChoice)=>pick.nativeModelId??pick.modelId
const channelLabel=(id?:string)=>!id?'官方 / 原生':props.channels.find(c=>c.id===id)?.label??id
/**
 * 模型列展示的标识。官方展示名可能恰好等于裸 id（或与上游 namespaced 相同），
 * 那样同一渠道里就会出现“有的带前缀、有的不带”。标识统一带渠道：
 * 原生保留原生 id，渠道型号用“渠道标签/原始 id”。
 */
const modelIdentity=(row:ModelProjection)=>row.nativeModelId?row.label:row.channelId?channelLabel(row.channelId)+'/'+row.modelId:row.label
/**
 * 目录未就绪与型号确实不可用是两件事：
 * - loading/error 时整表只读，行用骨架占位，不把“还没读到”判成“型号坏了”；
 * - 目录到了以后才按渠道停用 / 未发现 / 不可用原因逐个判。
 */
const catalogReady=()=>!props.loading&&!props.error
const available=(row:ModelProjection)=>catalogReady()&&row.enabled&&row.discovered&&!row.unavailableReason
const chosen=(row:ModelProjection)=>picks.value.some(p=>key(p)===key(row))
watch(()=>props.open,value=>{if(value){picks.value=props.current?.map(p=>({...p}))??[];search.value='';channelSearch.value='';channel.value='';focused.value='';sorting.value=null;descending.value=false}},{immediate:true})
// 渠道列表只保留仍有可选型号的渠道；隐藏或停用渠道不再出现在选择器里。
const channelRows=computed(()=>{const usable=new Set(props.rows.filter(row=>row.pickerVisible!==false&&row.enabled&&row.discovered&&!row.unavailableReason).map(row=>row.channelId));return props.channels.filter(c=>usable.has(c.id)&&(c.label+' '+c.id).toLowerCase().includes(channelSearch.value.toLowerCase()))})
function effort(row:ModelProjection){if(row.rules.noReasoning?.mode==='custom'&&row.rules.noReasoning.value)return undefined;return (props.reasoning?picks.value.find(p=>key(p)===key(row))?.reasoningEffort:undefined)??(row.rules.defaultEffort.mode==='custom'?row.rules.defaultEffort.value:undefined)}
function sortValue(row:ModelProjection,column:SortColumn):string|number|undefined{
 if(column==='recommendation'){const index=props.recommendations?.findIndex(p=>key(p)===key(row));return index!==undefined&&index>=0?index:undefined}
 if(column==='model')return modelIdentity(row)
 if(column==='channel')return channelLabel(row.channelId)
 if(column==='context')return row.rules.contextWindow.mode==='custom'?row.rules.contextWindow.value:undefined
 if(column==='state')return available(row)?0:1
 const rank=['none','minimal','low','medium','high','xhigh','max','ultra'].indexOf(effort(row)??'');return rank<0?undefined:rank
}
// 可浏览列表只列真正可选的型号：隐藏、停用渠道、待首次确认都由可用性判定挡在外面。
// 已保存但如今失效的选择仍留在“已选模型”里，并在那里写明原因。
const rows=computed(()=>{
 const values=props.rows.filter(row=>available(row)&&(!channel.value||row.channelId===channel.value)&&(row.label+' '+modelIdentity(row)+' '+row.modelId+' '+(row.aliases??[]).join(' ')+' '+channelLabel(row.channelId)).toLowerCase().includes(search.value.toLowerCase()))
 const column=sorting.value;if(!column)return values
 return values.map((row,index)=>({row,index})).sort((a,b)=>{
  const left=sortValue(a.row,column);const right=sortValue(b.row,column)
  if(left===undefined||right===undefined)return left===right?a.index-b.index:left===undefined?1:-1
  const value=typeof left==='number'&&typeof right==='number'?left-right:String(left).localeCompare(String(right),'zh-CN',{numeric:true})
  return value?(descending.value?-value:value):a.index-b.index
 }).map(item=>item.row)
})
function sort(column:SortColumn){if(sorting.value!==column){sorting.value=column;descending.value=false}else if(!descending.value)descending.value=true;else sorting.value=null;focused.value=''}
function sortState(column:SortColumn):'none'|'ascending'|'descending'{return sorting.value!==column?'none':descending.value?'descending':'ascending'}
function toggle(row:ModelProjection){
 if(!available(row))return
 if(chosen(row)){picks.value=picks.value.filter(p=>key(p)!==key(row));return}
 if(!props.multiple)picks.value=[]
 picks.value.push(rowChoice(row))
}
function chooseRow(row:ModelProjection){if(!available(row))return;if(props.multiple){toggle(row);return}if(!chosen(row))toggle(row);focused.value='row:'+key(row)}
const selectable=computed(()=>rows.value.filter(available))
const someSelected=computed(()=>selectable.value.some(chosen)&&!allSelected.value)
const allSelected=computed(()=>selectable.value.length>0&&selectable.value.every(chosen))
function selectResults(){const ids=new Set(selectable.value.map(key));if(allSelected.value)picks.value=picks.value.filter(p=>!ids.has(key(p)));else for(const row of selectable.value)if(!chosen(row))picks.value.push(rowChoice(row))}
function problem(pick:ModelChoice):string{
 // 目录未就绪时给出等待态，而不是“缺失”；两者文案与样式必须能区分。
 if(props.loading)return '目录读取中';if(props.error)return '目录读取失败'
 const row=props.rows.find(row=>key(row)===key(pick));if(!row)return '型号已缺失，请重新选择'
 if(row.unavailableReason)return row.unavailableReason;
 if(!row.enabled)return '渠道停用';if(!row.discovered)return '型号未发现'
 if(props.reasoning&&pick.reasoningEffort&&((row.rules.noReasoning?.mode==='custom'&&row.rules.noReasoning.value)||row.rules.reasoningEfforts.mode!=='custom'||!row.rules.reasoningEfforts.value.includes(pick.reasoningEffort)))return '原推理档位已不支持'
 if(props.reasoning&&pick.fast===true&&(row.rules.fast.mode!=='custom'||!row.rules.fast.value))return 'Fast能力已变化'
 return ''
}
const invalid=computed(()=>picks.value.some(p=>problem(p)))
/** 已选芯片的状态：目录读取中属“等待核对”，只有目录到位后才可能是真问题。 */
const pendingCatalog=()=>props.loading&&!props.error
const chipNeedsReview=(pick:ModelChoice)=>!pendingCatalog()&&!!problem(pick)
function movePick(index:number,direction:number){const to=index+direction;if(to<0||to>=picks.value.length)return;const [pick]=picks.value.splice(index,1);picks.value.splice(to,0,pick!)}
function confirm(){if(props.loading||props.error||invalid.value||(!props.multiple&&!picks.value.length))return;emit('confirm',picks.value.map(item=>props.reasoning?({...item}):identity(item)))}
function focusRow(index:number){const values=table.value?.querySelectorAll<HTMLElement>('.model-picker-row');values?.[Math.max(0,Math.min(index,(values?.length??1)-1))]?.focus()}
function rowKey(event:KeyboardEvent,row:ModelProjection,index:number){if(event.target!==event.currentTarget)return;if(['ArrowDown','ArrowUp','Home','End'].includes(event.key)){event.preventDefault();focusRow(event.key==='Home'?0:event.key==='End'?rows.value.length-1:index+(event.key==='ArrowDown'?1:-1))}else if(event.key==='Enter'||event.key===' '){event.preventDefault();chooseRow(row)}}
function option(row:ModelProjection,field:'reasoningEffort'|'fast',value?:string|boolean){
 if(!chosen(row))toggle(row)
 const pick=picks.value.find(p=>key(p)===key(row));if(!pick)return
 if(value===undefined){delete pick[field];return}
 if(field==='fast'&&typeof value==='boolean')pick.fast=value
 if(field==='reasoningEffort'&&typeof value==='string')pick.reasoningEffort=value
}
</script>
<template>
 <UiDialog :open="open" :title="title??'选择渠道模型'" wide @close="emit('close')">
  <!-- 目录未就绪：状态条 + 骨架行；不把“还没读到”显示成“型号坏了”。 -->
  <div v-if="loading" class="model-picker-status" role="status"><span class="model-picker-spinner" aria-hidden="true"></span><span>正在读取模型目录</span><small>已有选择保持</small></div><div v-if="error" role="alert"><p>{{ error }}</p><UiButton :disabled="loading" @click="emit('retry')">重新读取目录</UiButton></div>
  <div class="model-picker">
   <nav aria-label="渠道"><h3>渠道</h3><input class="input" v-model="channelSearch" type="search" aria-label="搜索渠道" placeholder="搜索渠道" /><div class="model-picker-channels"><UiButton :variant="!channel?'primary':'default'" :surface="channel?'plain':'control'" @click="channel='';focused=''">全部渠道</UiButton><UiButton v-for="c in channelRows" :key="c.id" :variant="channel===c.id?'primary':'default'" :surface="channel===c.id?'control':'plain'" @click="channel=c.id;focused=''">{{ c.label }}</UiButton></div></nav>
   <div class="model-picker-browser"><h3>模型 <small>{{ rows.length }} 个结果</small></h3><input class="input" v-model="search" type="search" aria-label="搜索模型" placeholder="搜索型号或原始ID" @input="focused=''" @keydown.down.prevent="focusRow(0)" />
    <div class="model-picker-list" :aria-busy="loading" @scroll="focused=''"><table ref="table" class="model-picker-table" role="grid" :aria-label="(channel?channelLabel(channel):'全部渠道')+'模型'"><thead><tr><th v-if="multiple"><input type="checkbox" :checked="allSelected" :indeterminate="someSelected" :disabled="!selectable.length" :aria-label="'全选当前结果 '+selectable.length+' 项'" @change="selectResults" /></th><th v-for="head in heads" :key="head[0]" :aria-sort="sortState(head[0])"><button type="button" @click="sort(head[0])">{{ head[1] }} {{ sorting===head[0]?descending?'↓':'↑':'↕' }}</button></th><th v-if="reasoning">设置</th></tr></thead>
     <tbody v-if="loading" aria-hidden="true"><tr v-for="n in 6" :key="n" class="model-picker-row is-skeleton"><td v-if="multiple"><span class="model-picker-skeleton is-box"></span></td><td><span class="model-picker-skeleton" style="width:72%"></span><span class="model-picker-skeleton is-faint" style="width:46%"></span></td><td><span class="model-picker-skeleton" style="width:58%"></span></td><td><span class="model-picker-skeleton" style="width:44%"></span></td><td><span class="model-picker-skeleton" style="width:52%"></span></td><td><span class="model-picker-skeleton" style="width:38%"></span></td><td v-if="recommendations?.length"><span class="model-picker-skeleton" style="width:32%"></span></td><td v-if="reasoning"><span class="model-picker-skeleton" style="width:60%"></span></td></tr></tbody><tbody v-else>
     <tr v-for="(row,index) in rows" :key="key(row)" class="model-picker-row" :class="{'is-selected':chosen(row),'is-unavailable':!available(row)}" tabindex="0" :aria-selected="chosen(row)" :aria-disabled="!available(row)" @click="chooseRow(row)" @keydown="rowKey($event,row,index)">
      <td v-if="multiple"><input type="checkbox" :aria-label="'选择 '+modelIdentity(row)" :checked="chosen(row)" :disabled="!available(row)" @click.stop @change="toggle(row)" /></td><td><strong>{{ modelIdentity(row) }}</strong><span v-if="chosen(row)" class="model-picker-chosen">✓ 已选</span><small v-if="row.label&&row.label!==modelIdentity(row)">{{ row.label }}</small><small v-if="row.aliases?.length">别名：{{ row.aliases.join('、') }}</small></td><td>{{ channelLabel(row.channelId) }}</td><td>{{ row.rules.contextWindow.mode==='custom'?row.rules.contextWindow.value.toLocaleString():'未知' }}</td><td>{{ effort(row)??'未知' }}</td><td>{{ row.unavailableReason??(!row.enabled?'渠道停用':!row.discovered?'未发现':'可选') }}</td>
      <td v-if="recommendations?.length">{{ recommendations.findIndex(p=>key(p)===key(row))>=0?'推荐 '+(recommendations.findIndex(p=>key(p)===key(row))+1):'—' }}</td><td v-if="reasoning" @click.stop><ModelChoiceOptions :row="row" :choice="picks.find(p=>key(p)===key(row))" :open="focused==='row:'+key(row)" :another-open="!!focused" :disabled="!available(row)" label="推理 / Fast" @open="focused='row:'+key(row)" @close="closeOptions('row:'+key(row))" @select="!chosen(row)&&toggle(row)" @revert="revertOptions" @option="(field,value)=>option(row,field,value)" /></td>
     </tr>
    </tbody></table><p v-if="!rows.length&&!loading&&!error">没有可选模型</p></div>
   </div>
  </div>
  <section class="model-picker-picked" aria-label="已选模型"><div class="model-inline"><strong>已选模型 · {{ picks.length }}</strong><UiButton v-if="multiple&&picks.length" size="sm" variant="ghost" @click="picks=[]">清空</UiButton></div><div class="model-picker-summary"><div v-for="(pick,index) in picks" :key="key(pick)" class="model-picker-chip" :class="{'is-invalid':chipNeedsReview(pick)}"><ModelChoiceOptions v-if="reasoning&&props.rows.find(r=>key(r)===key(pick))" :row="props.rows.find(r=>key(r)===key(pick))!" :choice="pick" :open="focused==='chip:'+key(pick)" :another-open="!!focused" :disabled="!available(props.rows.find(r=>key(r)===key(pick))!)" :label="channelLabel(pick.channelId)+' / '+pickId(pick)+' · '+(pick.reasoningEffort??effort(props.rows.find(r=>key(r)===key(pick))!)??'未知')+(pick.fast===true?' · Fast':'')" @open="focused='chip:'+key(pick)" @close="closeOptions('chip:'+key(pick))" @revert="revertOptions" @option="(field,value)=>option(props.rows.find(r=>key(r)===key(pick))!,field,value)" /><span v-else>✓ {{ channelLabel(pick.channelId) }} / {{ pickId(pick) }}{{ pick.reasoningEffort?' · '+pick.reasoningEffort:'' }}{{ pick.fast===undefined?'':pick.fast?' · Fast开':' · Fast关' }}<span v-if="pendingCatalog()" class="model-picker-pending">正在核对</span><small v-if="chipNeedsReview(pick)">{{ problem(pick) }}</small></span><small v-if="reasoning&&chipNeedsReview(pick)">{{ problem(pick) }}</small><template v-if="multiple"><UiButton v-if="ordered" size="sm" variant="ghost" :disabled="index===0" :aria-label="pickId(pick)+'上移'" @click="movePick(index,-1)">↑</UiButton><UiButton v-if="ordered" size="sm" variant="ghost" :disabled="index===picks.length-1" :aria-label="pickId(pick)+'下移'" @click="movePick(index,1)">↓</UiButton><UiButton size="sm" variant="ghost" :aria-label="'移除 '+pickId(pick)" @click="picks.splice(index,1)">×</UiButton></template></div></div></section>
  <template #actions><span>已选 {{ picks.length }} 项</span><UiButton @click="emit('close')">取消</UiButton><UiButton variant="primary" :disabled="loading||!!error||invalid||(!multiple&&!picks.length)" @click="confirm">确定选择</UiButton></template>
 </UiDialog>
</template>
<style scoped>
.model-picker{display:grid;grid-template-columns:160px minmax(0,1fr);gap:20px}.model-picker nav,.model-picker-browser{min-width:0}.model-picker h3{display:flex;align-items:center;justify-content:space-between;margin:0 0 10px;font-size:var(--text-body)}.model-picker h3 small{color:var(--muted);font-weight:400}.model-picker input[type=search]{width:100%;margin-bottom:12px}.model-picker-channels :deep(.btn){white-space:normal;text-align:left;overflow-wrap:anywhere}
.model-picker-channels{display:grid;gap:6px;max-height:360px;overflow:auto;align-content:start}.model-picker-list{max-height:360px;overflow:auto}.model-picker-table{width:100%;border-collapse:collapse;font-size:var(--text-caption)}.model-picker-table th{position:sticky;top:0;background:var(--surface-panel-bg,var(--panel));z-index:1;font-weight:500;text-align:left;white-space:nowrap}.model-picker-table th button{font:inherit;color:inherit;background:none;border:0;cursor:pointer;padding:0}.model-picker-table td,.model-picker-table th{padding:10px 8px;border-bottom:1px solid var(--border)}.model-picker-row{cursor:pointer}.model-picker-row.is-selected{background:color-mix(in srgb,var(--accent) 10%,transparent);box-shadow:inset 3px 0 var(--accent)}.model-picker-row:hover{background:var(--hover)}.model-picker-row.is-selected:hover{background:color-mix(in srgb,var(--accent) 14%,transparent)}.model-picker-row.is-unavailable{color:var(--muted)}.model-picker-row small{display:block;color:var(--muted);overflow-wrap:anywhere}.model-picker-chosen{margin-left:6px;color:var(--accent);white-space:nowrap}.model-picker-options{display:grid;gap:12px;width:100%;background:var(--surface-panel-bg,var(--panel));padding:12px;border-radius:var(--radius)}.model-picker-picked{border-top:1px solid var(--border);margin-top:16px;padding-top:12px}.model-picker-summary{display:flex;flex-wrap:wrap;gap:6px;margin-top:10px;max-height:130px;overflow:auto}.model-picker-chip{display:flex;gap:6px;align-items:center;padding:6px 8px;background:var(--surface-panel-bg,var(--panel));border:1px solid var(--border);border-radius:7px;font-size:var(--text-caption);overflow-wrap:anywhere}.model-picker-chip small{display:block;color:var(--red)}.model-picker-chip.is-invalid{border-color:var(--red)}/* 目录读取中：状态条 + 骨架行，列宽与真实表一致，加载完成不跳版。 */
.model-picker-status{display:flex;align-items:center;gap:8px;padding:8px 0 10px;font-size:var(--text-caption);color:var(--text)}
.model-picker-status small{color:var(--muted)}
.model-picker-spinner{width:12px;height:12px;flex:none;border:2px solid var(--border);border-top-color:var(--text);border-radius:50%;animation:model-picker-spin .8s linear infinite}
@keyframes model-picker-spin{to{transform:rotate(360deg)}}
.model-picker-skeleton{display:block;height:9px;border-radius:5px;background:color-mix(in srgb,var(--text) 11%,transparent);position:relative;overflow:hidden;max-width:100%}
.model-picker-skeleton.is-faint{height:7px;margin-top:6px;background:color-mix(in srgb,var(--text) 7%,transparent)}
.model-picker-skeleton.is-box{width:14px;height:14px;border-radius:4px}
.model-picker-skeleton::after{content:"";position:absolute;inset:0;transform:translateX(-100%);background:linear-gradient(90deg,transparent,color-mix(in srgb,var(--text) 12%,transparent),transparent);animation:model-picker-shimmer 1.4s ease-in-out infinite}
@keyframes model-picker-shimmer{to{transform:translateX(100%)}}
.model-picker-row.is-skeleton{pointer-events:none;cursor:default}
.model-picker-row.is-skeleton:hover{background:transparent}
/* 加载期已选芯片的弱提示：中性色，与“型号已缺失”的警示色区分。 */
.model-picker-pending{margin-left:6px;padding:0 4px;border-radius:4px;background:color-mix(in srgb,var(--text) 8%,transparent);color:var(--muted);font-size:var(--text-caption);white-space:nowrap}
/* 中低画质与减少动态：只留静态底，不播扫光与旋转。 */
html:not([data-effects="high"]) .model-picker-skeleton::after{animation:none;display:none}
html:not([data-effects="high"]) .model-picker-spinner{animation:none;border-top-color:var(--muted)}
@media (prefers-reduced-motion:reduce){.model-picker-skeleton::after{animation:none;display:none}.model-picker-spinner{animation:none}}
@media(max-width:700px){.model-picker{grid-template-columns:120px minmax(0,1fr);gap:12px}}
</style>
