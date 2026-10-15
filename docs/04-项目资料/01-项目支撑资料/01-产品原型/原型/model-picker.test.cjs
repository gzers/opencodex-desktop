/* 最小 DOM 接线验证；不代表玻璃材质、滑杆布局或实屏焦点。 */
const {test}=require('node:test'),assert=require('node:assert/strict'),vm=require('node:vm'),fs=require('node:fs'),path=require('node:path');
function harness(){
 const nodes=new Map(),handlers={},observers=[],timers=new Map(),timerDelays=[];let timerId=0;
 function node(id){if(!nodes.has(id))nodes.set(id,new Node(id));return nodes.get(id);}
 class Node{
  constructor(id){this.id=id;this.dataset={};this.style={setProperty(){}};this.hidden=false;this.isConnected=true;this.scrollTop=0;this.children=[];this.attrs={};const classes=new Set();this.classList={add(c){classes.add(c);},remove(c){classes.delete(c);},contains(c){return classes.has(c);},toggle(c,on){on?classes.add(c):classes.delete(c);}};this.events={};this.html='';this.htmlWrites=0;this.queryResults={};}
  set innerHTML(value){this.htmlWrites++;this.html=value;for(const x of value.matchAll(/id="([^"]+)"/g))node(x[1]);}
  get innerHTML(){return this.html;}
  setAttribute(key,value){this.attrs[key]=String(value);if(key==='aria-expanded')this.expanded=value;}
  getAttribute(key){return this.attrs[key];}
  addEventListener(type,fn){(this.events[type]??=[]).push(fn);}
  querySelector(q){if(Object.hasOwn(this.queryResults,q))return this.queryResults[q];return q.startsWith('#')?node(q.slice(1)):node(this.id+' '+q);}
  querySelectorAll(q){return this.queryResults[q]||[];}
  appendChild(child){this.children.push(child);child.parent=this;nodes.set(child.id,child);}
  prepend(child){this.children.unshift(child);child.parent=this;nodes.set(child.id,child);}
  replaceChildren(){this.children=[];this.html='';}
  remove(){nodes.delete(this.id);}
  closest(q){return node(q);}
  focus(){doc.activeElement=this;}
  contains(x){return this===x;}
  getBoundingClientRect(){return {left:0,top:0,right:1180,bottom:760,width:1180,height:760};}
  getClientRects(){return [this.getBoundingClientRect()];}
 }
 const doc={activeElement:null,body:node('body'),documentElement:{dataset:{route:'models'}},createElement:()=>new Node('created'),querySelector:q=>q.startsWith('#')?node(q.slice(1)):node(q),querySelectorAll:()=>[],addEventListener(type,fn){(handlers[type]??=[]).push(fn);}};
 node('modalMask').style.display='none';const win={addEventListener(){},prototypePanel:{refresh(){}}};
 const ctx={window:win,document:doc,innerWidth:1580,innerHeight:900,console,JSON,Map,Set,URL,setTimeout:(fn,ms)=>{const id=++timerId;timers.set(id,fn);timerDelays.push(ms);return id;},clearTimeout(id){timers.delete(id);},requestAnimationFrame:fn=>fn(),MutationObserver:class{constructor(fn){observers.push(fn);}observe(){}},setRoute(route){doc.documentElement.dataset.route=route;},openModal(title,body,label,fn,opts={}){node('modalConfirm').hidden=!!opts.hideConfirm;node('modalBody').innerHTML=body;node('modalMask').style.display='flex';},closeModal(){node('modalMask').style.display='none';const fn=win.__modalOnClose;win.__modalOnClose=null;fn?.();}};
 vm.runInNewContext(fs.readFileSync(path.join(__dirname,'model-picker.js'),'utf8'),ctx);
 function click(action,key,extra={}){const target=node('target-'+action);target.tagName='BUTTON';target.dataset={mpk:action,key,effortKey:key,effortSource:'row',...extra};target.matches=()=>false;target.closest=q=>q==='[data-mpk]'?target:null;for(const fn of handlers.click||[])fn({target});}
 function outside(){const target=node('outside');target.closest=()=>null;for(const fn of handlers.focusin||[])fn({target});}
 function effort(action,value){const target=node('effort-'+action);target.closest=q=>q==='[data-effort-step]'&&action==='step'?{dataset:{effortStep:String(value)}}:q==='[data-effort-fast]'&&action==='fast'?target:q==='[data-effort-reset]'&&action==='reset'?target:null;for(const fn of node('mpkEffortPopover').events.click||[])fn({target});}
 function escape(){const event={key:'Escape',target:doc.activeElement,preventDefault(){},stopImmediatePropagation(){}};for(const fn of handlers.keydown||[])fn(event);}

 function hover(id){const target=node('hover-'+id);target.dataset={effortKey:id,effortSource:'row'};target.closest=q=>q==='[data-effort-key]'?target:null;for(const fn of handlers.pointerover||[])fn({target,relatedTarget:null});return target;}
 return {win,doc,nodes,node,ctx,click,outside,effort,escape,hover,timerDelays,flushTimers(){const queued=[...timers.values()];timers.clear();queued.forEach(fn=>fn());},api:win.modelPickerPrototype,close:ctx.closeModal,route(route){doc.documentElement.dataset.route=route;observers.forEach(fn=>fn());}};
}
const ref=(channelId='p',modelId='model')=>({kind:'channel',channelId,modelId});
const row=(extra={})=>({ref:ref(),channelName:'渠道 P',name:'模型',context:128000,effort:'medium',efforts:['low','medium','high'],reason:'',...extra});
const key=r=>JSON.stringify(['channel',r.channelId,r.modelId]);
test('业务调用使用同一弹窗不跳打样页；返回默认不写显式档位，不污染专题选择',()=>{
 const h=harness(),before=JSON.stringify(h.api.state().committed);let selected,cancelled=0;
 h.api.choose({mode:'single',reasoning:true,models:[ref()],getCatalog:()=>[row()],onSelect:r=>selected=r,onCancel:()=>cancelled++});
 assert.equal(h.doc.documentElement.dataset.route,'models');assert.equal(h.api.state().businessIntegration,true);assert.equal(h.api.state().pending.length,1);assert.equal(h.api.state().visible[0].effort,'medium');
 h.click('pick',key(ref()));assert.equal(selected,undefined);assert.equal(h.api.state().isOpen,true);h.outside();assert.equal(selected,undefined);assert.equal(h.api.state().isOpen,true);assert.equal(h.node('modalConfirm').hidden,false);h.click('accept');assert.equal(selected.models[0].modelId,'model');assert.equal(Object.hasOwn(selected.models[0],'reasoningEffort'),false);assert.equal(cancelled,0);assert.equal(h.api.state().isOpen,false);assert.equal(JSON.stringify(h.api.state().committed),before);
});
test('当前显式推理档位保留返回，显式 none 与沿用默认分别表示',()=>{
 const h=harness();let selected;
 h.api.choose({reasoning:true,models:[{...ref(),reasoningEffort:'high'}],getCatalog:()=>[row()],onSelect:r=>selected=r});h.click('pick',key(ref()));h.outside();h.click('accept');assert.equal(selected.models[0].reasoningEffort,'high');
 h.api.choose({reasoning:true,models:[{...ref(),reasoningEffort:'none'}],getCatalog:()=>[row({efforts:['none','medium']})],onSelect:r=>selected=r});h.click('pick',key(ref()));h.outside();h.click('accept');assert.equal(selected.models[0].reasoningEffort,'none');
});
test('取消只回调取消，离开业务页标记 routeChanged；专题推理开关与场景恢复',()=>{
 const h=harness(),before=h.api.state();let result,count=0;
 h.api.choose({reasoning:false,models:[ref()],getCatalog:()=>[row()],onSelect:()=>count++,onCancel:r=>result=r});h.close();assert.equal(result.routeChanged,false);assert.equal(count,0);assert.equal(h.api.state().reasoningEnabled,before.reasoningEnabled);
 h.api.choose({getCatalog:()=>[row()],onSelect:()=>count++,onCancel:r=>result=r});h.route('sync');assert.equal(result.routeChanged,true);assert.equal(count,0);
});
test('业务传入无额度、未绑定和原生限制，不可选记录仍显示原因',()=>{
 const h=harness();let count=0;const rows=[row({reason:'无额度'}),row({ref:ref('q','other'),channelName:'渠道 Q',reason:'未绑定'})];
 h.api.choose({getCatalog:()=>rows,onSelect:()=>count++});assert.deepEqual(h.api.state().visible.map(m=>m.reason),['无额度','未绑定']);h.click('pick',key(ref()));assert.equal(count,0);assert.equal(h.api.state().isOpen,true);
});
test('选中时重新核对动态目录，期间模型停用或档位失效阻止回调',()=>{
 const h=harness();let entry=row(),count=0;
 h.api.choose({reasoning:true,models:[{...ref(),reasoningEffort:'high'}],getCatalog:()=>[entry],onSelect:()=>count++});entry=row({efforts:['low']});h.click('pick',key(ref()));h.outside();h.click('accept');assert.equal(count,0);assert.equal(h.api.state().isOpen,true);
 entry=row({reason:'渠道已停用'});h.click('pick',key(ref()));h.outside();h.click('accept');assert.equal(count,0);
});
test('业务选择后可以立即打开调用方设置弹窗，关闭钩子不会再触发取消',()=>{
 const h=harness();let cancels=0;
 h.api.choose({getCatalog:()=>[row()],onSelect:()=>h.ctx.openModal('共用设置','<div id="settings"></div>','保存'),onCancel:()=>cancels++});h.click('pick',key(ref()));h.click('accept');assert.equal(h.node('modalMask').style.display,'flex');assert.ok(h.node('modalBody').innerHTML.includes('settings'));assert.equal(cancels,0);
});

test('连续修改推理与Fast和失焦均不返回，确认只回调一次带回组合值',()=>{
 const h=harness();let selected,count=0;
 h.api.choose({mode:'single',reasoning:true,fast:true,models:[ref()],getCatalog:()=>[row({fastCapability:'supported'})],onSelect:r=>{selected=r;count++;}});
 h.click('pick',key(ref()));h.effort('step',2);h.effort('fast');
 assert.equal(count,0);assert.equal(h.api.state().isOpen,true);assert.equal(h.node('mpkEffortPopover').hidden,false);
 h.outside();assert.equal(count,0);assert.equal(h.api.state().isOpen,true);assert.equal(h.node('mpkEffortPopover').hidden,true);h.click('accept');assert.equal(count,1);assert.equal(selected.models[0].reasoningEffort,'high');assert.equal(selected.models[0].fast,true);h.outside();assert.equal(count,1);
});
test('连续编辑Esc撤回本次档位与Fast，取消主窗不返回草稿',()=>{
 const h=harness();let count=0,cancelled=0;
 h.api.choose({reasoning:true,fast:true,models:[{...ref(),reasoningEffort:'medium',fast:false}],getCatalog:()=>[row({fastCapability:'supported'})],onSelect:()=>count++,onCancel:()=>cancelled++});
 h.click('pick',key(ref()));h.effort('step',2);h.effort('fast');h.escape();assert.equal(h.node('mpkEffortPopover').hidden,true);assert.equal(h.api.state().effortChoices[key(ref())],'medium');assert.equal(h.api.state().fastChoices[key(ref())],false);h.close();assert.equal(count,0);assert.equal(cancelled,1);
});
test('Fast能力未知不能开启，恢复推理默认保持浮层且不固化显示值',()=>{
 const h=harness();let selected;
 h.api.choose({reasoning:true,fast:true,models:[{...ref(),reasoningEffort:'high'}],getCatalog:()=>[row({fastCapability:'unknown'})],onSelect:r=>selected=r});
 h.click('pick',key(ref()));h.effort('fast');assert.equal(h.api.state().fastChoices[key(ref())],undefined);h.effort('reset');assert.equal(h.node('mpkEffortPopover').hidden,false);h.outside();assert.equal(selected,undefined);h.click('accept');assert.equal(Object.hasOwn(selected.models[0],'reasoningEffort'),false);assert.equal(Object.hasOwn(selected.models[0],'fast'),false);
});
test('多选失焦只收浮层，仍待统一确认',()=>{
 const h=harness();let count=0;
 h.api.choose({mode:'multiple',reasoning:true,fast:true,models:[ref()],getCatalog:()=>[row({fastCapability:'supported'})],onSelect:()=>count++});h.click('effort',key(ref()));h.effort('step',0);h.effort('fast');h.outside();assert.equal(count,0);assert.equal(h.api.state().isOpen,true);assert.equal(h.node('mpkEffortPopover').hidden,true);h.click('accept');assert.equal(count,1);
});

test('不启用推理或Fast的普通单选也须确认，连续点模型只替换一个待选项',()=>{
 const h=harness();let selected,count=0;const other=ref('p','other');
 h.api.choose({models:[ref()],getCatalog:()=>[row(),row({ref:other})],onSelect:r=>{selected=r;count++;}});
 h.click('pick',key(other));h.outside();assert.equal(count,0);assert.deepEqual(h.api.state().pending,[key(other)]);assert.equal(h.node('modalConfirm').hidden,false);h.click('pick',key(ref()));assert.equal(count,0);assert.deepEqual(h.api.state().pending,[key(ref())]);h.click('accept');assert.equal(count,1);assert.equal(selected.models[0].modelId,'model');
});
test('无有效单选项时禁用确认，确认无回调；主窗取消丢弃全部待选修改',()=>{
 const h=harness();let selects=0,cancels=0;
 h.api.choose({models:[],getCatalog:()=>[row()],onSelect:()=>selects++,onCancel:()=>cancels++});assert.equal(h.node('modalConfirm').disabled,true);h.click('accept');assert.equal(selects,0);h.click('pick',key(ref()));assert.equal(h.node('modalConfirm').disabled,false);h.close();assert.equal(selects,0);assert.equal(cancels,1);
});

test('长表格连续选择、调档和Fast不重建表格或渠道；焦点锚点保持',()=>{
 const h=harness();const rows=Array.from({length:160},(_,i)=>row({ref:ref('p','model-'+i),fastCapability:'supported'}));let selected;
 h.api.choose({reasoning:true,fast:true,models:[],getCatalog:()=>rows,onSelect:r=>selected=r});
 const table=h.node('mpkModels'),nav=h.node('mpkChannelRows'),tableWrites=table.htmlWrites,navWrites=nav.htmlWrites;
 const id=key(rows[0].ref),tr=h.node('table-row');tr.dataset.key=id;tr.setAttribute('aria-selected','false');
 const name=h.node('row-name'),label=h.node('row-effort');tr.queryResults['.mpk-model-name']=name;tr.queryResults['.mpk-effort-label']=label;name.queryResults['.mpk-current-check']=null;table.queryResults['.mpk-row']=[tr];
 h.click('pick',id);const focused=h.doc.activeElement;for(let i=0;i<20;i++){h.effort('step',i%3);h.effort('fast');}
 assert.equal(table.htmlWrites,tableWrites);assert.equal(nav.htmlWrites,navWrites);assert.equal(tr.getAttribute('aria-selected'),'true');assert.equal(tr.classList.contains('is-picked'),true);assert.equal(label.textContent,'中');assert.equal(h.doc.activeElement,focused);assert.equal(h.node('mpkEffortPopover').hidden,false);h.click('accept');assert.equal(selected.models[0].modelId,'model-0');
});
test('首次悬停短延迟，浮层打开后跨表格行立即切换，取消后不延迟复活',()=>{
 const h=harness();const other=ref('q','other');h.api.choose({reasoning:true,models:[ref()],getCatalog:()=>[row(),row({ref:other,channelName:'Q'})],onSelect(){}});
 h.hover(key(ref()));assert.equal(h.node('mpkEffortPopover').hidden,true);assert.equal(h.timerDelays.at(-1),70);h.flushTimers();assert.equal(h.node('mpkEffortPopover').hidden,false);
 const count=h.timerDelays.length;h.hover(key(other));assert.equal(h.timerDelays.length,count);assert.match(h.node('mpkEffortPopover').innerHTML,/Q \/ other/);assert.deepEqual(h.api.state().pending,[key(ref())]);
 h.outside();h.hover(key(ref()));h.close();h.flushTimers();assert.equal(h.node('mpkEffortPopover').hidden,true);
});

test('按档位排序时编辑不移动锚点行，浮层收起后才复排且不重建表格',()=>{
 const h=harness(),other=ref('q','other');h.api.choose({reasoning:true,models:[ref()],getCatalog:()=>[row(),row({ref:other,effort:'low'})],onSelect(){}});h.click('sort',null,{sort:'effort'});
 const table=h.node('mpkModels'),tbody=h.node('tbody'),a=h.node('row-a'),b=h.node('row-b');a.dataset.key=key(ref());b.dataset.key=key(other);table.queryResults['.mpk-row']=[b,a];table.queryResults.tbody=tbody;const writes=table.htmlWrites;
 h.click('effort',key(ref()));h.effort('step',0);assert.equal(tbody.children.length,0);assert.equal(h.node('mpkEffortPopover').hidden,false);assert.equal(table.htmlWrites,writes);h.outside();assert.deepEqual(tbody.children.map(n=>n.dataset.key),[key(ref()),key(other)]);assert.equal(table.htmlWrites,writes);
});
