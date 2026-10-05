/* 源码级渲染/状态核对。无浏览器、布局引擎、网络或运行文件写入；不替代视觉 QA。 */
import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../../../../../../../../');
const source=path.join(root,'docs/04-项目资料/03-项目协作资料/03-共享原型/原型/model-config.js');
const sectionNames=['models','connection','endpoints','defaults'];
function make(){
const nodes=new Map(), listeners={}, timers=[];
let confirm=null,options={},bodyOwner=new Map(),modalTitle='',headerHeight=72,mainPadding=0;const routes=[];
// 固定几何仅模拟滚动与重排，不包含浏览器布局引擎。
function layout(){const collapsed=nodes.get('mpChannelWorkspace')?.classList.contains('nav-collapsed');return Object.fromEntries(sectionNames.map((id,i)=>[id,[280,1800,2600,3000][i]-(collapsed&&i>=1?160:0)]));}
class Node {
 constructor(id=''){this.id=id;this.dataset={};const properties=new Map();this.style={setProperty:(key,value)=>properties.set(key,value),getPropertyValue:key=>properties.get(key)||''};this.disabled=false;this.checked=false;this.indeterminate=false;this.value='';this.attributes=new Map();this.events={};const classes=new Set();this.classList={add:(...items)=>items.forEach(x=>classes.add(x)),remove:(...items)=>items.forEach(x=>classes.delete(x)),contains:x=>classes.has(x),toggle:(x,force)=>{const on=force??!classes.has(x);if(on)classes.add(x);else classes.delete(x);return on;}};this._html='';this._scrollTop=0;this.clientHeight=600;}
 setAttribute(key,value){this.attributes.set(key,String(value));if(key==='class')this.classList.add(...String(value).split(/\s+/).filter(Boolean));if(key.startsWith('data-'))this.dataset[key.slice(5).replace(/-([a-z])/g,(_,c)=>c.toUpperCase())]=String(value);}
 getAttribute(key){return this.attributes.get(key)??null;}
 get scrollHeight(){return this.id==='mpMain'?layout().defaults+300:600;}
 get scrollTop(){return this._scrollTop;}
 set scrollTop(value){this._scrollTop=this.id==='mpMain'?Math.max(0,Math.min(value,this.scrollHeight-this.clientHeight)):value;}
 scrollTo(options){this.lastScroll={...options};this.scrollTop=options.top;listeners.scroll?.({target:this});}
 getBoundingClientRect(){if(this.id==='mpChannelHeader'){const top=Math.max(100,280-headerHeight-nodes.get('mpMain').scrollTop);return {top,bottom:top+headerHeight,height:headerHeight};}const offset=layout()[this.dataset.channelSection],top=this.id==='mpMain'?100:100+(offset??0)-nodes.get('mpMain').scrollTop,next=sectionNames.indexOf(this.dataset.channelSection)+1,bottom=offset===undefined?top+600:100+(layout()[sectionNames[next]]??nodes.get('mpMain').scrollHeight)-nodes.get('mpMain').scrollTop;return {top,bottom,height:bottom-top};}
 set innerHTML(html){this._html=html;
  if(this.id==='mpContent'){nodes.delete('mpScopeAll');const tag=html.match(/<input[^>]*data-scope-all[^>]*>/)?.[0];if(tag){const node=new Node('mpScopeAll');node.checked=/\bchecked(?:\s|>)/.test(tag);node.disabled=/\bdisabled\b/.test(tag);nodes.set(node.id,node);}}
  if(['modalBody','mpContent','mpAssistantHost'].includes(this.id)){
   for(const id of bodyOwner.get(this.id)||[])nodes.delete(id);bodyOwner.set(this.id,[]);
   for(const m of html.matchAll(/<([a-z]+)[^>]*\bid="([^"]+)"[^>]*>/g)){
   const node=new Node(m[2]);node.value=/\bvalue="([^"]*)"/.exec(m[0])?.[1]||'';node.checked=/\bchecked\b/.test(m[0]);node.disabled=/\bdisabled\b/.test(m[0]);
    for(const a of m[0].matchAll(/([a-z][a-z0-9-]*)="([^"]*)"/g))node.setAttribute(a[1],a[2]);
    if(m[1]==='select')node.innerHTML=html.slice(m.index+m[0].length,html.indexOf('</select>',m.index));
    if(m[1]==='textarea')node.value=html.slice(m.index+m[0].length,html.indexOf('</textarea>',m.index)).replace(/&quot;/g,'\"').replace(/&#39;/g,"'").replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&amp;/g,'&');
    nodes.set(node.id,node);bodyOwner.get(this.id).push(node.id);
   }
   if(this.id==='mpContent'){
    this.navNodes=[...html.matchAll(/<button class="mp-nav-item"[^>]*>/g)].map(tag=>{const node=new Node();for(const a of tag[0].matchAll(/([a-z][a-z0-9-]*)="([^"]*)"/g))node.setAttribute(a[1],a[2]);return node;});
    this.fieldEditor=html.includes('data-field-editor')?new Node():null;
   }
  }
  if(this.id.startsWith('mpExternal')||this.id.startsWith('mpMatch')||this.id.startsWith('mpBatch')||this.id.startsWith('mpAgent')||this.id.startsWith('mpChannel')||this.id.startsWith('mpAdopt')||this.id.startsWith('mpExport')||this.id==='mpGenerationTemplate'||this.id==='mpGenerationSources'){
   const options=[...html.matchAll(/<option\s+([^>]*)>/g)];
   const chosen=options.find(m=>/\bselected\b/.test(m[1]))||options[0];this.value=/value="([^"]*)"/.exec(chosen?.[1]||'')?.[1]||'';
  }
 }
 get innerHTML(){return this._html;}
 set outerHTML(value){this._outerHTML=value;}
 get outerHTML(){return this._outerHTML||'';}
 addEventListener(type,fn){this.events[type]=fn;}
 matches(q){return this.id==='mpScopeAll'&&q==='[data-scope-all]';}
 select(){this.selected=true;}
 focus(options){document.activeElement=this;this.focusOptions=options;}
 setSelectionRange(start,end){this.selectionStart=start;this.selectionEnd=end;}
 closest(q){return q==='.main'?nodes.get('mpMain'):q==='.modal-mask'?nodes.get('modalMask'):null;}
 querySelector(q){return q==='[data-field-editor]'&&this.id==='mpContent'?this.fieldEditor:q.startsWith('#')?(this._html.includes('id="'+q.slice(1)+'"')?nodes.get(q.slice(1)):null):new Node();}
 querySelectorAll(q){if(q==='[data-channel-section]')return sectionNames.map(id=>nodes.get('mpChannelSection-'+id)).filter(Boolean);return q==='[data-mp="channel-section"]'||q==='.mp-nav-item'?this.navNodes||[]:[];}
}
for(const id of ['mpMain','mpAssistantHost','mpContent','mpSettingsContent','modal','modalBody','modalMask','modalConfirm'])nodes.set(id,new Node(id));
const document={activeElement:null,documentElement:{dataset:{route:'models',theme:'light'}},querySelector:q=>q==='#modalBody .mp-dialog'?new Node():q.startsWith('#')?nodes.get(q.slice(1))||null:q==='[data-field-editor]'?new Node():q==='[data-scope-all]'?nodes.get('mpScopeAll'):null,querySelectorAll:q=>q==='[data-scope-all]'&&nodes.has('mpScopeAll')?[nodes.get('mpScopeAll')]:[],addEventListener:(type,fn)=>{listeners[type]=fn;}};
const context={document,navigator:{},window:{location:{hash:'#models'},matchMedia:()=>({matches:false}),getComputedStyle:()=>({paddingTop:mainPadding+'px'}),addEventListener:(type,fn)=>listeners['window:'+type]=fn},JSON,Date,URL,Set,Object,String,Number,Array,console,setTimeout:fn=>timers.push(fn),toast(message){context.lastToast=message;},setRoute(route){routes.push(route);document.documentElement.dataset.route=route;},showSettingsSection(section){routes.push(section);},openModal(title,body,label,fn,opts){modalTitle=title;nodes.get('modalBody').innerHTML=body;nodes.get('modalMask').style.display='flex';confirm=fn;options=opts||{};},closeModal(){nodes.get('modalMask').style.display='none';confirm=null;context.window.__modalOnClose?.();}};
vm.createContext(context);vm.runInContext(fs.readFileSync(source,'utf8'),context,{filename:source});
const state=()=>context.window.modelPrototype.state(),page=()=>nodes.get('mpContent').innerHTML,dialog=()=>nodes.get('modalBody').innerHTML;
function click(action,data={}){const target={dataset:{mp:action,...data},closest(){return this;}};listeners.click({target});}
function runConfirm(){const fn=confirm;assert.equal(typeof fn,'function');fn();}
function drain(){let limit=30;while(timers.length&&limit-->0)timers.shift()();assert.ok(limit>0,'finite timers');}

 const proto=context.window.modelPrototype;
 const event=(type,selector,dataset={},value='',checked=false)=>listeners[type]({target:{dataset,value,checked,matches:q=>q===selector}});
 const scroll=(top,target=nodes.get('mpMain'))=>{target.scrollTop=top;listeners.scroll({target});};
 return {nodes,context,state,page,dialog,click,runConfirm,drain,proto,event,scroll,routes,resize:(height,viewport=600,padding=mainPadding,border=0)=>{headerHeight=height;mainPadding=padding;nodes.get('mpMain').clientHeight=viewport;nodes.get('mpMain').clientTop=border;listeners['window:resize']?.();}};
}
const results=[];
function check(name,fn){const h=make();h.click('view-provider',{id:'newapi'});fn(h);results.push(name);console.log('PASS '+name);}
const p=(h,id='newapi')=>h.state().draft.providers.find(x=>x.id===id);
const edit=(h,key,value,type='input')=>h.event(type,'[data-channel-config]',{channelConfig:key},value);
const stage=h=>h.proto.stagedChannelConnections();
const status=h=>h.nodes.get('mpPolicyStatus').outerHTML||h.page();
const addManual=(h,id='manual-connection-demo')=>{h.nodes.get('mpChannelManual').value=id;h.click('channel-manual');h.event('change','[data-channel-model]',{channelModel:id},'',true);};
check('标题导航与快捷操作合并，四分区与连接表单可直接访问',h=>{const page=h.page();assert.match(page,/<header id="mpChannelHeader"/);for(const id of sectionNames)assert.match(page,new RegExp('id="mpChannelSection-'+id+'"'));assert.match(page,/data-channel-config="baseUrl"/);assert.match(page,/data-channel-config="protocol"/);assert.match(page,/data-channel-config="auth"/);assert.doesNotMatch(page,/data-mp="channel-connection"/);for(const action of ['refresh','order-preview','more','settings','save','view-draft'])assert.equal([...page.matchAll(new RegExp('data-mp="'+action+'"','g'))].length,1);const nav=page.slice(page.indexOf('<nav'),page.indexOf('</nav>'));assert.match(nav,/id="mpPolicyStatus"/);assert.match(nav,/模型激活数 89\/100/);});
check('初始保存禁用且不重复 title，输入后还原按钮立即可用',h=>{assert.equal(h.nodes.get('mpChannelReset').disabled,true);const saveTag=h.page().match(/<button[^>]*data-mp="save"[^>]*>/)[0];assert.match(saveTag,/disabled/);assert.equal([...saveTag.matchAll(/\btitle=/g)].length,1);edit(h,'name','备用');assert.equal(h.nodes.get('mpChannelReset').disabled,false);assert.match(h.nodes.get('mpChannelConnectionPending').textContent,/尚未加入草稿/);assert.match(status(h),/有待确认编辑/);assert.equal(p(h).name,'NewApi');});
check('模型标识与能力摘要分行，CSS 允许换行且单元格居中',h=>{const row=h.page().split('class="mp-model-row"')[1];assert.equal([...row.matchAll(/class="mp-model-meta-line"/g)].length,2);const css=fs.readFileSync(source.replace(/\.js$/,'.css'),'utf8');assert.match(css,/\.mp-model-row>div\{[^}]*align-self:center/);assert.match(css,/\.mp-model-identity>strong\{[^}]*overflow-wrap:anywhere/);assert.match(css,/\.mp-model-meta-line\{[^}]*flex-wrap:wrap/);});
check('标题固定抵消顶部内距，导航与锚点按实际标题高度避让',h=>{h.resize(88,600,24,2);const style=h.nodes.get('mpContent').style;assert.equal(style.getPropertyValue('--mp-editor-top'),'-24px');assert.equal(style.getPropertyValue('--mp-editor-nav-top'),'76px');assert.equal(style.getPropertyValue('--mp-editor-offset'),'102px');assert.equal(style.getPropertyValue('--mp-editor-nav-height'),'488px');h.click('channel-section',{id:'connection'});assert.equal(h.nodes.get('mpMain').lastScroll.top,1698);assert.equal(h.state().channelSection,'connection');h.resize(116,600,24,2);h.click('channel-section',{id:'endpoints'});assert.equal(h.nodes.get('mpMain').lastScroll.top,2470);});
check('手动滚动跟踪分区，底部最后分区，忽略弹窗滚动',h=>{h.scroll(1750);assert.equal(h.state().channelSection,'connection');h.scroll(2580);assert.equal(h.state().channelSection,'endpoints');const before=h.state().channelSection;h.scroll(40,h.nodes.get('modalBody'));assert.equal(h.state().channelSection,before);h.scroll(4000);assert.equal(h.state().channelSection,'defaults');});
check('折叠导航保留当前可见分区位置，重绘保持渠道测量',h=>{h.resize(88,600,24);h.scroll(1750);const before=h.nodes.get('mpChannelSection-connection').getBoundingClientRect().top;h.click('toggle-channel-nav');assert.equal(h.nodes.get('mpChannelSection-connection').getBoundingClientRect().top,before);assert.equal(h.state().channelNavCollapsed,true);assert.equal(h.nodes.get('mpChannelNavToggle').getAttribute('aria-expanded'),'false');h.proto.render();assert.equal(h.nodes.get('mpContent').style.getPropertyValue('--mp-editor-nav-top'),'76px');assert.equal(h.nodes.get('mpChannelSection-connection').getBoundingClientRect().top,before);});
check('连接缓冲在筛选重绘、返回列表与切换渠道后按渠道保留',h=>{edit(h,'name','NewApi 待确认');edit(h,'baseUrl','https://changed.example.com/v1');h.scroll(1750);const top=h.nodes.get('mpMain').scrollTop;h.event('input','[data-model-search]',{},'mini');assert.equal(h.nodes.get('mpMain').scrollTop,top);assert.match(h.page(),/value="NewApi 待确认"/);h.event('change','[data-current]',{},'aimami');assert.equal(h.nodes.get('mpMain').scrollTop,0);edit(h,'name','另一个待确认');h.click('back-channels');h.click('view-provider',{id:'newapi'});assert.equal(h.state().channelConnections.newapi.name,'NewApi 待确认');assert.equal(h.state().channelConnections.aimami.name,'另一个待确认');assert.equal(stage(h).length,2);});
check('未确认连接阻止页级保存，还原只清本渠道缓冲',h=>{h.click('toggle-model',{id:'newapi::0'});edit(h,'name','未确认');const before=h.dialog(),saved=JSON.stringify(h.state().saved);h.click('save');assert.equal(h.dialog(),before);assert.match(h.context.lastToast,/连接配置/);h.click('reset-channel-connection');assert.equal(stage(h).length,0);h.click('save');assert.match(h.dialog(),/保存渠道模型配置|完整投影/);assert.equal(JSON.stringify(h.state().saved),saved);});
check('仅未确认编辑也可丢弃，清空各渠道缓冲且不标记待重启',h=>{const before=JSON.stringify(h.state().draft);edit(h,'name','一');h.event('change','[data-current]',{},'aimami');edit(h,'name','二');h.click('discard');h.runConfirm();assert.equal(stage(h).length,0);assert.equal(JSON.stringify(h.state().draft),before);assert.equal(h.state().pending,false);});
check('密钥只保存替换意图，认证切换与重绘不保留输入值',h=>{edit(h,'key','channel-secret-sentinel');assert.doesNotMatch(JSON.stringify(h.state()),/channel-secret-sentinel/);assert.equal(h.state().channelConnections.newapi.keyChanged,1);assert.match(h.nodes.get('mpChannelKeyStatus').textContent,/替换意图/);edit(h,'auth','none','change');assert.equal(h.state().channelConnections.newapi.connection.keyConfigured,false);edit(h,'auth','apikey','change');assert.equal(h.state().channelConnections.newapi.connection.keyConfigured,true);assert.equal(h.nodes.get('mpChannelInline-key').value,'');h.click('export');h.nodes.get('mpExportSource').value='draft';h.runConfirm();assert.doesNotMatch(h.dialog(),/channel-secret-sentinel|keyConfigured|baseUrl/);});
check('名称修改保留模型清单，取消确认保留行内输入',h=>{const before=JSON.stringify(h.state().draft);edit(h,'name','NewApi 新名称');h.click('apply-channel-connection');assert.match(h.dialog(),/已选 17 \/ 清单 30/);h.context.closeModal();assert.equal(h.state().channelConnections.newapi.name,'NewApi 新名称');assert.equal(JSON.stringify(h.state().draft),before);assert.equal(stage(h).length,1);});
check('名称确认只加入草稿，保留字段顺序且清除对应缓冲',h=>{const before=p(h),saved=JSON.stringify(h.state().saved);edit(h,'name','NewApi 新名称');h.click('apply-channel-connection');h.runConfirm();assert.equal(p(h).name,'NewApi 新名称');assert.deepEqual(p(h).models,before.models);assert.equal(stage(h).length,0);assert.equal(JSON.stringify(h.state().saved),saved);assert.equal(h.state().pending,false);});
for(const [key,value] of [['baseUrl','https://changed.example.com/v1'],['protocol','Chat'],['auth','none'],['key','demo-replacement']])check(key+' 修改使旧发现清单失效，取消保留待确认值',h=>{const before=JSON.stringify(h.state().draft);edit(h,key,value,key==='auth'||key==='protocol'?'change':'input');h.click('apply-channel-connection');assert.match(h.dialog(),/已选 0 \/ 清单 0/);assert.equal(h.nodes.get('modalConfirm').disabled,true);h.context.closeModal();assert.equal(JSON.stringify(h.state().draft),before);assert.equal(stage(h).length,1);});
check('新地址手动加入模型确认，更新草稿不写成功快照',h=>{const saved=JSON.stringify(h.state().saved);edit(h,'baseUrl','https://changed.example.com/v1/');h.click('apply-channel-connection');addManual(h);h.runConfirm();assert.equal(p(h).connection.baseUrl,'https://changed.example.com/v1');assert.equal(p(h).models[0].id,'manual-connection-demo');assert.equal(p(h).models.length,1);assert.equal(stage(h).length,0);assert.equal(JSON.stringify(h.state().saved),saved);});
check('空名称及非法地址拒绝连接确认，不污染草稿',h=>{const before=JSON.stringify(h.state().draft);edit(h,'name','');h.click('apply-channel-connection');assert.match(h.context.lastToast,/渠道名称/);assert.equal(h.nodes.get('modalMask').style.display,'none');h.click('reset-channel-connection');edit(h,'baseUrl','https://user:secret@example.com/v1');h.click('apply-channel-connection');assert.match(h.context.lastToast,/Base URL/);assert.equal(JSON.stringify(h.state().draft),before);});
check('确认期间后来连接输入或模型草稿变化拒绝旧确认',h=>{edit(h,'name','第一版');h.click('apply-channel-connection');edit(h,'name','第二版');h.runConfirm();assert.match(h.nodes.get('mpChannelError').textContent,/已变化/);assert.equal(p(h).name,'NewApi');h.context.closeModal();h.click('apply-channel-connection');h.proto.commitModelPatches('newapi',[{key:'newapi::0',enabled:false}]);const before=JSON.stringify(h.state().draft);h.runConfirm();assert.match(h.nodes.get('mpChannelError').textContent,/已变化/);assert.equal(JSON.stringify(h.state().draft),before);});
check('保存预览后出现未确认连接，冻结计划拒绝旧确认',h=>{h.click('toggle-model',{id:'newapi::0'});h.click('save');const saved=JSON.stringify(h.state().saved);edit(h,'name','晚到输入');h.runConfirm();assert.match(h.nodes.get('mpSaveError').textContent,/基准已变化/);assert.equal(JSON.stringify(h.state().saved),saved);assert.equal(h.state().operation,null);});
check('原生只读与保存中保护连接字段和确认入口',h=>{h.click('view-provider',{id:'native'});assert.doesNotMatch(h.page(),/data-channel-config|data-mp="apply-channel-connection"/);const before=JSON.stringify(h.state().draft);edit(h,'name','伪造修改');h.click('apply-channel-connection');assert.equal(JSON.stringify(h.state().draft),before);h.click('view-provider',{id:'newapi'});h.click('toggle-model',{id:'newapi::0'});h.click('save');h.runConfirm();const frozen=JSON.stringify(h.state().draft);edit(h,'name','保存中');h.click('apply-channel-connection');h.click('reset-channel-connection');assert.equal(JSON.stringify(h.state().draft),frozen);assert.equal(stage(h).length,0);h.drain();assert.equal(h.state().pending,true);assert.match(h.page(),/已保存 · 待重启/);});
check('跨渠道待确认连接可由左侧状态入口定位处理',h=>{edit(h,'name','需处理');h.event('change','[data-current]',{},'aimami');h.click('review-channel-connection');assert.match(h.page(),/<h2>NewApi<\/h2>/);assert.equal(h.state().channelSection,'connection');assert.equal(h.nodes.get('mpMain').lastScroll.top,1716);});
console.log('CHANNEL WORKFLOW '+results.length+' PASS; 仅源码状态与固定几何 stub，无浏览器视觉、真实 API 或运行写入。');
