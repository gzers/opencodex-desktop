import {emptyRules,type TemplateDraft,type Revision} from './api'
export interface TemplateDifference {field:string;before:unknown;after:unknown}
function stable(value:unknown):string {
 if(Array.isArray(value))return '['+value.map(stable).join(',')+']'
 if(value!==null&&typeof value==='object')return '{'+Object.entries(value).sort(([a],[b])=>a.localeCompare(b)).map(([key,item])=>JSON.stringify(key)+':'+stable(item)).join(',')+'}'
 return JSON.stringify(value)??'undefined'
}
export function templateDifferences(revision:Revision,draft:TemplateDraft):TemplateDifference[]{
 const rows:TemplateDifference[]=[];const add=(field:string,before:unknown,after:unknown)=>{if(stable(before)!==stable(after))rows.push({field,before,after})}
 add('名称',revision.savedLabel,draft.label);add('说明',revision.content.description,draft.content.description)
 for(const key of ['include','exclude','caseSensitive','channelMode','channels','priority'] as const)add('匹配规则 · '+key,revision.content.matching[key],draft.content.matching[key])
 for(const key of Object.keys(emptyRules()) as (keyof ReturnType<typeof emptyRules>)[])add('通用配置 · '+key,revision.content.rules[key]??{mode:'inherit'},draft.content.rules[key]??{mode:'inherit'})
 const ids=new Set([...revision.content.variants,...draft.content.variants].map(v=>v.id))
 for(const id of ids){const before=revision.content.variants.find(v=>v.id===id),after=draft.content.variants.find(v=>v.id===id);add('具体型号 · '+(after?.name||before?.name||id),before,after)}
 add('人工锁',revision.content.lockedFields,draft.content.lockedFields);add('来源',revision.content.sources,draft.content.sources)
 return rows
}
