/* 模版更新原型的纯数据逻辑；不检索、不持久化、不发模型请求。 */
(function(root,factory){const api=factory();if(typeof module==='object'&&module.exports)module.exports=api;else root.TemplateUpdates=api;})(typeof window==='object'?window:globalThis,function(){
 'use strict';
 const copy=x=>x===undefined?undefined:JSON.parse(JSON.stringify(x));
 const same=(a,b)=>JSON.stringify(a)===JSON.stringify(b);
 let serial=0;
 function stamp(existing=[],time=Date.now(),uuid){
  const taken=new Set(existing.map(v=>v.revisionId)),tags=new Set(existing.map(v=>v.shortId));
  let id=uuid||globalThis.crypto?.randomUUID?.()||('mock-'+time.toString(36)+'-'+Math.random().toString(36).slice(2)+'-'+(++serial));
  while(taken.has(id))id+='-'+(++serial);
  const raw=id.replace(/[^a-z0-9]/gi,'').toUpperCase();let size=4,short=raw.slice(-size);
  while(tags.has(short)){size++;short=raw.slice(-size);if(size>raw.length){short=raw+'-'+(++serial);break;}}
  const savedAt=new Date(time).toISOString(),date=new Date(time+8*3600000).toISOString().slice(0,19).replace('T',' ');
  return {revisionId:id,savedAt,shortId:short,versionLabel:date+' · '+short};
 }
 function changes(current,suggested,protect=true){
  const rows=[];
  function add(path,before,after,label){if(same(before,after))return;rows.push({id:'change-'+rows.length,path,before:copy(before),after:copy(after),label,conflict:protect&&before!==undefined,selected:!(protect&&before!==undefined)});}
  for(const section of ['rules','match'])for(const key of Object.keys(suggested[section]||{}))add([section,key],current[section]?.[key],suggested[section][key],key);
  for(const v of suggested.variants||[]){const old=current.variants?.find(x=>x.id===v.id||x.suffix===v.suffix);if(!old)add(['variants',v.id],undefined,v,'新增型号 '+v.name);else for(const key of Object.keys(v.rules||{}))add(['variants',old.id,'rules',key],old.rules[key],v.rules[key],old.name+' · '+key);}
  return rows;
 }
 function compose(current,rows){
  const result=copy(current);
  for(const row of rows.filter(r=>r.selected)){
   const [section,id,part,key]=row.path;
   if(section==='variants'){
    result.variants??=[];
    if(part){const item=result.variants.find(v=>v.id===id);if(!item)throw new Error('型号已变化，请重新检查');item.rules[key]=copy(row.after);}
    else {if(result.variants.some(v=>v.id===id||v.suffix===row.after.suffix))throw new Error('型号已存在，请重新检查');result.variants.push(copy(row.after));}
   }else {result[section]??={};result[section][id]=copy(row.after);}
  }
  return result;
 }
 // 只撤回该次提交改变且未被后续编辑的字段；数组按稳定 ID 合并。
 function restore(current,before,after){
  const conflicts=[];const object=x=>x&&typeof x==='object'&&!Array.isArray(x);
  function merge(now,old,applied,path){
   if(same(old,applied))return copy(now);
   if(same(now,applied))return copy(old);
   if(object(now)&&object(old)&&object(applied)){
    const result=copy(now);
    for(const key of new Set([...Object.keys(old),...Object.keys(applied)])){
     const value=merge(now[key],old[key],applied[key],path.concat(key));if(value===undefined)delete result[key];else result[key]=value;
    }return result;
   }
   if([now,old,applied].every(Array.isArray)){
    const key=['id','key'].find(k=>[now,old,applied].every(list=>list.every(v=>object(v)&&typeof v[k]==='string')&&new Set(list.map(v=>v[k])).size===list.length));
    if(key){const result=copy(now);for(const id of new Set([...old,...applied].map(v=>v[key]))){const at=result.findIndex(v=>v[key]===id),value=merge(now.find(v=>v[key]===id),old.find(v=>v[key]===id),applied.find(v=>v[key]===id),path.concat(id));if(value===undefined){if(at>=0)result.splice(at,1);}else if(at>=0)result[at]=value;else result.push(value);}
     if(same(now.map(v=>v[key]),applied.map(v=>v[key])))result.sort((a,b)=>{const ai=old.findIndex(v=>v[key]===a[key]),bi=old.findIndex(v=>v[key]===b[key]);return (ai<0?old.length:ai)-(bi<0?old.length:bi);});
     else if(!same(old.map(v=>v[key]),applied.map(v=>v[key])))conflicts.push(path.join('.')+' · 后续顺序保留');
     return result;
    }
   }
   conflicts.push(path.join('.')||'配置');return copy(now);
  }
  return {value:merge(current,before,after,[]),conflicts};
 }
 return {stamp,changes,compose,restore};
});
