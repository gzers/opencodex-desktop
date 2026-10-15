/* 基于完整前后快照生成模版版本摘要；不使用来源名称或采用状态猜测变更。 */
(function(root,factory){const api=factory();if(typeof module==='object'&&module.exports)module.exports=api;else root.TemplateSummary=api;})(typeof window==='object'?window:globalThis,function(){
 'use strict';
 const copy=v=>v===undefined?undefined:JSON.parse(JSON.stringify(v));
 const normalize=v=>Array.isArray(v)?v.map(normalize):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,normalize(v[k])])):v;
 const same=(a,b)=>JSON.stringify(normalize(a))===JSON.stringify(normalize(b));
 const set=values=>(values||[]).map(v=>JSON.stringify(normalize(v))).sort();
 const ruleNames={protocol:'模型协议',context:'上下文窗口',modalities:'输入模态',efforts:'可选推理等级',default:'默认推理等级',fast:'快速模式能力',force:'请求强制档',omit:'推理参数发送方式',authority:'声明模式'};
 const matchNames={channelMode:'渠道范围',channels:'匹配渠道',include:'包含规则',exclude:'排除规则',priority:'匹配优先级',caseSensitive:'大小写规则'};
 function snapshot(content){
  return {name:String(content?.name||'').trim(),description:content?.description||'',match:copy(content?.match||{include:[],exclude:[],channels:[],caseSensitive:false,channelMode:'all',priority:0}),rules:copy(content?.rules||{}),variants:copy(content?.variants||[])};
 }
 function summarize(before,after,basis={kind:'revision'}){
  const next=snapshot(after),previous=before?snapshot(before):null,changes=[];
  const add=(path,a,b,text,equal=same)=>{if(!equal(a,b))changes.push({path,before:copy(a),after:copy(b),text});};
  if(previous){
   add(['name'],previous.name,next.name,'修改模版名称');
   add(['description'],previous.description,next.description,'修改模版说明');
   for(const [key,label] of Object.entries(matchNames)){
    const read=content=>{const mode=content.match.channelMode||(content.match.channels?.length?'whitelist':'all');return key==='channelMode'?mode:key==='channels'&&mode==='all'?[]:content.match[key];};
    add(['match',key],read(previous),read(next),'调整'+label,['channels','include','exclude'].includes(key)?(a,b)=>same(set(a),set(b)):same);
   }
   const rules=(a,b,path=[])=>{
    for(const [key,label] of Object.entries(ruleNames)){
     const av=a?.[key],bv=b?.[key],prefix=path.length?'型号「'+(next.variants.find(v=>v.id===path[1])?.name||previous.variants.find(v=>v.id===path[1])?.name||path[1])+'」':'';
     const text=bv===undefined?'恢复'+prefix+label+'继承':av===undefined?'设置'+prefix+label:'调整'+prefix+label;
     add([...path,'rules',key],av,bv,text,['modalities','efforts'].includes(key)?(x,y)=>x===undefined||y===undefined?same(x,y):same(set(x),set(y)):same);
    }
   };
   rules(previous.rules,next.rules);
   const oldIds=previous.variants.map(v=>v.id),newIds=next.variants.map(v=>v.id),added=next.variants.filter(v=>!oldIds.includes(v.id)),removed=previous.variants.filter(v=>!newIds.includes(v.id));
   if(added.length)changes.push({path:['variants','added'],before:[],after:copy(added),text:'新增 '+added.length+' 个型号规则'});
   if(removed.length)changes.push({path:['variants','removed'],before:copy(removed),after:[],text:'删除 '+removed.length+' 个型号规则'});
   for(const v of next.variants){const old=previous.variants.find(x=>x.id===v.id);if(!old)continue;
    add(['variants',v.id,'name'],old.name,v.name,'修改型号规则名称');
    add(['variants',v.id,'suffix'],old.suffix,v.suffix,'调整型号「'+v.name+'」匹配后缀');
    rules(old.rules,v.rules,['variants',v.id]);
   }
   const common=oldIds.filter(id=>newIds.includes(id));add(['variants','order'],common,newIds.filter(id=>oldIds.includes(id)),'调整型号规则顺序');
  }
  const creation={baseline:'内置基线',create:'首次创建模版',copy:'复制为独立模版'}[basis.kind];
  const entries=[...(creation?[creation]:[]),...changes.map(c=>c.text)];
  if(!entries.length)entries.push(basis.kind==='unavailable'||(!previous&&basis.kind==='revision')?'变更依据缺失':'内容无变化');
  const text=entries.slice(0,3).join('；')+(entries.length>3?'；另 '+(entries.length-3)+' 项':'');
  return {schema:1,basis:copy(basis),changed:!!creation||changes.length>0,changes,entries,text,fullText:entries.join('；')};
 }
 return {snapshot,summarize};
});
