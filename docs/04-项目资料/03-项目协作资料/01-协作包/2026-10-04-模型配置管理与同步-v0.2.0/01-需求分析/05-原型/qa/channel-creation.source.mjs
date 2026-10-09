import {repoRoot as root, prototypeRoot} from '../../../../2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/原型/qa/prototype-location.mjs';
/* 源码级渲染/状态核对。无浏览器、布局引擎、网络或运行文件写入；不替代视觉 QA。 */
import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const source=path.join(prototypeRoot,'原型/model-config.js');
const nodes=new Map(), listeners={}, timers=[];
let confirm=null,options={},bodyOwner=[];
const decodeAttribute=value=>value.replace(/&quot;/g,'"').replace(/&#39;/g,"'").replace(/&lt;/g,'<').replace(/&gt;/g,'>').replace(/&amp;/g,'&');
class Node {
 constructor(id=''){this.id=id;this.dataset={};this.style={};this.disabled=false;this.checked=false;this.value='';this.classList={add(){},remove(){}};this._html='';}
 set innerHTML(html){this._html=html;
  if(this.id==='modalBody'){
   for(const id of bodyOwner)nodes.delete(id);bodyOwner=[];
   for(const m of html.matchAll(/<([a-z]+)[^>]*\bid="([^"]+)"[^>]*>/g)){
    const node=new Node(m[2]);node.value=decodeAttribute(/\bvalue="([^"]*)"/.exec(m[0])?.[1]||'');node.checked=/\bchecked\b/.test(m[0]);node.disabled=/\bdisabled\b/.test(m[0]);
    if(m[1]==='select')node.innerHTML=html.slice(m.index+m[0].length,html.indexOf('</select>',m.index));
    nodes.set(node.id,node);bodyOwner.push(node.id);
   }
  }
  if(this.id.startsWith('mpAgent')||this.id==='mpGenerationTemplate'||this.id==='mpGenerationSources'||this.id==='mpExportSource'||this.id==='mpExportScope'||this.id==='mpRestoreScope'||this.id.startsWith('mpChannel')){
   const options=[...html.matchAll(/<option\s+([^>]*)>/g)];
   const chosen=options.find(m=>/\bselected\b/.test(m[1]))||options[0];this.value=decodeAttribute(/value="([^"]*)"/.exec(chosen?.[1]||'')?.[1]||'');
  }
 }
 get innerHTML(){return this._html;}
 focus(){}
 setSelectionRange(){}
 addEventListener(){}
 matches(){return false;}
 closest(q){return q==='.modal-mask'?nodes.get('modalMask'):null;}
 querySelector(){return new Node();}
 querySelectorAll(){return [];}
}
for(const id of ['mpContent','mpSettingsContent','modal','modalBody','modalMask','modalConfirm'])nodes.set(id,new Node(id));
const document={activeElement:null,querySelector:q=>q==='#modalBody .mp-dialog'?new Node():q.startsWith('#')?nodes.get(q.slice(1))||null:q==='[data-field-editor]'?new Node():null,querySelectorAll:()=>[],addEventListener:(type,fn)=>{listeners[type]=fn;}};
const context={document,window:{},JSON,Date,URL,Set,Object,String,Number,Array,console,setTimeout:fn=>timers.push(fn),toast(){},setRoute(){},showSettingsSection(){},openModal(title,body,label,fn,opts){nodes.get('modalBody').innerHTML=body;nodes.get('modalMask').style.display='flex';confirm=fn;options=opts||{};},closeModal(){nodes.get('modalMask').style.display='none';confirm=null;context.window.__modalOnClose?.();}};
vm.createContext(context);vm.runInContext(fs.readFileSync(source,'utf8'),context,{filename:source});
const state=()=>context.window.modelPrototype.state(),page=()=>nodes.get('mpContent').innerHTML,dialog=()=>nodes.get('modalBody').innerHTML;
function click(action,data={}){const target={dataset:{mp:action,...data},closest(){return this;}};listeners.click({target});}
function runConfirm(){const fn=confirm;assert.equal(typeof fn,'function');fn();}
function drain(){let limit=30;while(timers.length&&limit-->0)timers.shift()();assert.ok(limit>0,'finite timers');}
const results=[];function check(name,fn){fn();results.push(name);console.log('PASS '+name);}

const baseline=()=>JSON.stringify(state().saved);
function fill({id='newapi-backup',name='NewApi 备用',url='https://newapi.example.com/v1',auth='apikey',key='demo-secret-sentinel'}={}){
 nodes.get('mpChannelName').value=name;nodes.get('mpChannelId').value=id;nodes.get('mpChannelURL').value=url;nodes.get('mpChannelAuth').value=auth;nodes.get('mpChannelKey').value=key;
}
function begin(data={}){click('new-channel');fill(data);runConfirm();}
function mark(id,value=true){const target={dataset:{channelModel:id},checked:value,matches:q=>q==='[data-channel-model]'};listeners.change({target});}
function search(value){nodes.get('mpChannelSearch').value=value;nodes.get('mpChannelSearch').oninput();}
check('列表新增入口可达，连接表单复用公共 Select',()=>{assert.match(page(),/data-mp="new-channel"/);click('new-channel');assert.match(dialog(),/id="mpChannelProtocol"/);assert.match(dialog(),/class="mp-select" id="mpChannelAuth"/);assert.match(dialog(),/请勿输入真实密钥/);runConfirm();assert.match(nodes.get('mpChannelError').textContent,/渠道名称/);});
check('重复标识、带密钥 URL 和错误 Base URL 被阻止',()=>{fill({id:'newapi'});runConfirm();assert.match(nodes.get('mpChannelError').textContent,/已存在/);fill({url:'https://user:password@example.com/v1'});runConfirm();assert.match(nodes.get('mpChannelError').textContent,/Base URL/);fill({url:'https://newapi.example.com/v1/models'});runConfirm();assert.match(nodes.get('mpChannelError').textContent,/不填写/);fill({key:''});runConfirm();assert.match(nodes.get('mpChannelError').textContent,/API Key/);context.closeModal();});
check('新渠道获取前没有创建，取消整个表单不改草稿',()=>{const before=JSON.stringify(state().draft);begin();assert.equal(nodes.get('modalConfirm').disabled,true);assert.equal(JSON.stringify(state().draft),before);context.closeModal();assert.equal(JSON.stringify(state().draft),before);assert.equal(state().pending,false);});
check('获取成功列出完整 fixture，全部默认不勾选，不携能力结论',()=>{begin();click('channel-fetch');assert.match(dialog(),/正在模拟获取/);drain();assert.match(dialog(),/已获取 8 个模型/);assert.doesNotMatch(dialog(),/data-channel-model="[^"]+" checked/);assert.equal(nodes.get('modalConfirm').disabled,true);assert.equal(state().draft.providers.length,4);});
check('搜索勾选只作用匹配结果，返回不改连接时保留选择',()=>{search('gpt-');click('channel-select');search('');assert.match(dialog(),/已选 3 \/ 清单 8/);click('channel-back');runConfirm();assert.match(dialog(),/已选 3 \/ 清单 8/);});
check('重复获取保留选择、取消等待使晚回失效',()=>{click('channel-fetch');drain();assert.match(dialog(),/已选 3 \/ 清单 8/);click('channel-fetch');click('channel-cancel-fetch');const body=dialog();drain();assert.equal(dialog(),body);});
check('加入只改草稿，追加渠道顺序且不丢失基础模型原身份',()=>{const before=baseline();runConfirm();const p=state().draft.providers.at(-1);assert.equal(p.id,'newapi-backup');assert.equal(p.mode,'whitelist');assert.equal(p.models.filter(m=>m.selected).length,3);assert.equal(p.models[0].official.fast,'unknown');assert.equal(p.connection.keyConfigured,true);assert.equal(baseline(),before);assert.equal(state().pending,false);assert.match(page(),/data-channel-config="baseUrl"/);assert.match(page(),/data-mp="apply-channel-connection"/);assert.match(page(),/模型默认/);assert.doesNotMatch(JSON.stringify(state()),/demo-secret-sentinel/);});
check('原型导出不携本机连接摘要或凭据',()=>{click('export');nodes.get('mpExportSource').value='draft';runConfirm();assert.doesNotMatch(dialog(),/keyConfigured|baseUrl|demo-secret-sentinel|addedInPrototype/);assert.match(dialog(),/newapi-backup/);context.closeModal();});
check('撤销新增和重做后当前页与批量范围保持可用',()=>{click('undo');assert.equal(state().draft.providers.length,4);assert.match(page(),/mp-provider-table/);click('redo');assert.equal(state().draft.providers.at(-1).id,'newapi-backup');click('view-provider',{id:'newapi-backup'});});
check('已有渠道刷新保留选择与位置，新增项不自动入白名单',()=>{const before=state().draft.providers.at(-1).models.map(m=>m.id);click('refresh');drain();const p=state().draft.providers.at(-1);assert.deepEqual(p.models.slice(0,before.length).map(m=>m.id),before);assert.equal(p.models.find(m=>m.id==='upstream-added-demo').selected,false);assert.equal(p.models.filter(m=>m.selected).length,3);});
check('保存提交新增渠道，标记待重启；仅草稿加入不等于保存',()=>{click('save');assert.match(dialog(),/NewApi 备用 新增/);runConfirm();drain();assert.equal(state().saved.providers.at(-1).id,'newapi-backup');assert.equal(state().pending,true);assert.equal(state().operation.status,'success');});
check('撤回新增提交的恢复草稿不会插入 undefined 或删除其他渠道',()=>{const id=state().records.at(-1).id;click('restore',{id:String(id)});runConfirm();assert.equal(state().draft.providers.length,4);assert.ok(state().draft.providers.every(Boolean));assert.equal(state().saved.providers.length,5);assert.equal(state().pending,true);assert.match(page(),/mp-provider-table/);});
check('原生来源没有连接设置；现有连接更换地址需重新选择',()=>{context.window.modelPrototype.scene('normal');click('view-provider',{id:'native'});assert.doesNotMatch(page(),/data-mp="channel-connection"/);click('view-provider',{id:'newapi'});listeners.change({target:{value:'all',matches:q=>q==='[data-mode]'}});click('channel-connection');assert.match(dialog(),/id="mpChannelId"[^>]*readonly/);runConfirm();assert.match(dialog(),/已选 30 \/ 清单 30/);click('channel-back');nodes.get('mpChannelURL').value='https://changed.example.com/v1';nodes.get('mpChannelURL').oninput();runConfirm();assert.match(dialog(),/清单 0/);assert.equal(nodes.get('modalConfirm').disabled,true);context.closeModal();assert.equal(state().draft.providers[0].connection.baseUrl,'https://newapi.example.com/v1');});
for(const [scenario,pattern] of [['discovery-fail',/获取失败/],['discovery-empty',/空清单/],['discovery-unauthorized',/认证失败/],['discovery-unsupported',/未提供可用/]]){
 check(scenario+' 保留草稿，手动 ID 去重后可显式选择',()=>{context.window.modelPrototype.scene(scenario);const before=baseline();begin();click('channel-fetch');drain();assert.match(dialog(),pattern);assert.equal(baseline(),before);nodes.get('mpChannelManual').value='exact/custom-id\nexact/custom-id';click('channel-manual');assert.match(dialog(),/清单 1/);assert.match(dialog(),/手动添加 \/ 能力未知/);mark('exact/custom-id');runConfirm();assert.equal(state().draft.providers.at(-1).models.length,1);assert.equal(state().draft.providers.at(-1).models[0].id,'exact/custom-id');assert.equal(state().pending,false);assert.equal(baseline(),before);});
}
check('关闭向导后晚回不会创建渠道或重开弹窗',()=>{context.window.modelPrototype.scene('normal');begin();click('channel-fetch');context.closeModal();drain();assert.equal(state().draft.providers.length,4);assert.equal(nodes.get('modalMask').style.display,'none');});
check('更新连接后迟到结果不能污染新目标，名称 HTML 安全转义',()=>{begin({name:'<b>Backup</b>'});click('channel-fetch');click('channel-cancel-fetch');click('channel-back');nodes.get('mpChannelURL').value='https://second.example.com/v1';nodes.get('mpChannelURL').oninput();runConfirm();drain();assert.match(dialog(),/清单 0/);assert.doesNotMatch(dialog(),/<b>Backup<\/b>/);nodes.get('mpChannelManual').value='manual-"<tag>';click('channel-manual');mark('manual-"<tag>');runConfirm();assert.equal(state().draft.providers.at(-1).models[0].id,'manual-"<tag>');assert.doesNotMatch(page(),/data-id="[^"]*manual-"<tag>/);assert.match(page(),/manual-&quot;&lt;tag&gt;/);assert.doesNotMatch(page(),/<b>Backup<\/b>/);assert.match(page(),/&lt;b&gt;Backup&lt;\/b&gt;/);});
check('新增也受全渠道预算约束，不截断发现清单',()=>{context.window.modelPrototype.scene('over-budget');begin();click('channel-fetch');drain();click('channel-select');runConfirm();const p=state().draft.providers.at(-1);assert.equal(p.models.length,8);assert.ok(context.window.modelPrototype.projection().length>100);const before=baseline();click('save');assert.equal(baseline(),before);assert.equal(state().pending,false);});
console.log('SOURCE CHECK '+results.length+' PASS; 无真实 API / 浏览器 / 视觉验收。');
