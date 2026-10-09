// 执行实际工具分区与锚点脚本，使用主原型侧栏标记和最小 DOM；不启动浏览器。
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
let root=here;while(!fs.existsSync(path.join(root,'.git')))root=path.dirname(root);
const base=path.join(root,'docs/04-项目资料/01-项目支撑资料/01-产品原型');
const files=['原型/prototype-panel.js','proto-controls.js','原型/index.html'];
const [panel,anchors,html]=files.map(file=>fs.readFileSync(path.join(base,file),'utf8'));
function make(){
 const observers=[],events={},historyWrites=[],scrolls=[];
 class Node{
  constructor(tag,attrs={}){this.tag=tag;this.attrs={...attrs};this.children=[];this.parentElement=null;this.dataset=new Proxy({},{set:(target,key,value)=>{target[key]=value;this.attrs['data-'+key.replace(/[A-Z]/g,c=>'-'+c.toLowerCase())]=value;return true;}});this.handlers={};this.textContent='';this.hidden='hidden' in attrs;this.value=attrs.value||'';this.scrollTop=0;this.scrollHeight=2400;this.clientHeight=700;for(const [k,v] of Object.entries(attrs))if(k.startsWith('data-'))this.dataset[k.slice(5).replace(/-([a-z])/g,(_,c)=>c.toUpperCase())]=v;this.classList={contains:c=>(this.attrs.class||'').split(/\s+/).includes(c),toggle:(c,on)=>{const s=new Set((this.attrs.class||'').split(/\s+/).filter(Boolean));on?s.add(c):s.delete(c);this.attrs.class=[...s].join(' ');}};}
  get id(){return this.attrs.id||'';}
  appendChild(c){if(c.parentElement)c.parentElement.children=c.parentElement.children.filter(n=>n!==c);c.parentElement=this;this.children.push(c);return c;}
  getAttribute(k){return this.attrs[k]??null;}
  setAttribute(k,v){this.attrs[k]=v;}
  matchesSimple(s){const bare=s.replace(/\[[^\]]*\]/g,"");const tag=bare.match(/^[a-z]+/)?.[0];if(tag&&tag!==this.tag)return false;const id=bare.match(/#([\w-]+)/)?.[1];if(id&&id!==this.id)return false;for(const [,c]of bare.matchAll(/\.([\w-]+)/g))if(!this.classList.contains(c))return false;for(const [,k,op,v]of s.matchAll(/\[([\w-]+)(?:(\^?=)"([^"]*)")?\]/g)){if(!(k in this.attrs))return false;if(op==='='&&this.attrs[k]!==v)return false;if(op==='^='&&!this.attrs[k].startsWith(v))return false;}return true;}
  matches(s){return s.split(',').some(part=>{const bits=part.trim().split(/\s+/);if(!this.matchesSimple(bits.pop()))return false;let n=this.parentElement;while(bits.length){const next=bits.pop();while(n&&!n.matchesSimple(next))n=n.parentElement;if(!n)return false;n=n.parentElement;}return true;});}
  querySelectorAll(s){return this.children.flatMap(c=>[...(c.matches(s)?[c]:[]),...c.querySelectorAll(s)]);}
  querySelector(s){return this.querySelectorAll(s)[0]||null;}
  closest(s){return this.matches(s)?this:this.parentElement?.closest(s)||null;}
  addEventListener(type,fn){(this.handlers[type]??=[]).push(fn);}
  getBoundingClientRect(){return {top:this.id==='ctl-mock'?1500:0};}
  scrollTo(value){this.scrollTop=value.top;scrolls.push({target:this,value});}
 }
 const document=new Node('document'),body=new Node('body'),main=new Node('main',{class:'main'}),rootEl=new Node('html');
 document.appendChild(rootEl);rootEl.appendChild(body);body.appendChild(main);document.body=body;document.documentElement=rootEl;rootEl.dataset.route='sync';
 const start=html.indexOf('<aside class="prototype-side'),sideHTML=html.slice(start,html.indexOf('</aside>',start)+8),stack=[body];
 for(const match of sideHTML.matchAll(/<\/?([a-z][\w-]*)\b[^>]*>/gi)){if(match[0].startsWith('</')){stack.pop();continue;}const attrs={};for(const [,k,v]of match[0].slice(match[1].length+1,-1).matchAll(/([\w-]+)(?:="([^"]*)")?/g))attrs[k]=v??'';const n=stack.at(-1).appendChild(new Node(match[1],attrs));if(!['input','br','hr','img'].includes(n.tag))stack.push(n);}
 const side=document.querySelector('.prototype-side'),lower=document.querySelector('#protoStdCards');
 const lowerBody=document.querySelector('#protoStdCards .proto-group-body')||lower;
 const windowsCard=lowerBody.appendChild(new Node('div',{id:'windowsDemoCard'}));
 const modelTab=main.appendChild(new Node('button',{'data-mp':'tab','data-tab':'channels','aria-selected':'true'}));
 const diagTab=main.appendChild(new Node('button',{'data-diag-tab':'doctor',class:'active'}));
 const extTab=main.appendChild(new Node('button',{'data-ext-tab':'skills',class:'active'}));
 const settings=main.appendChild(new Node('nav',{class:'settings-tabs'})),settingsTab=settings.appendChild(new Node('button',{'data-settings-section':'installation',class:'active'}));
 document.getElementById=id=>document.querySelector('#'+id);
 const state={tab:'file',importSample:'encrypted',scenario:'normal'};
 const window={syncPrototype:{state:()=>state,setImportSample:v=>state.importSample=v,setScenario:v=>state.scenario=v},addEventListener:(type,fn)=>(events[type]??=[]).push(fn),scrollY:0,innerHeight:800,scrollTo:v=>scrolls.push({target:window,value:v})};
 const ctx=vm.createContext({document,window,location:{hash:'#sync?tab=file'},URLSearchParams,MutationObserver:class{constructor(fn){this.fn=fn;}observe(target,opts){observers.push({target,opts,fn:this.fn});}},requestAnimationFrame:fn=>fn(),getComputedStyle:n=>({overflowY:n===side?'auto':'visible'}),history:{replaceState:(...args)=>historyWrites.push(args)},performance:{now:()=>2000}});
 vm.runInContext(panel,ctx,{filename:files[0]});vm.runInContext(anchors,ctx,{filename:files[1]});
 const get=s=>{const n=document.querySelector(s);assert(n,'missing '+s);return n;};
 const changed=(target,type='attributes',attributeName='class')=>observers.filter(o=>o.target===target||o.opts.subtree&&target.closest('.main')).forEach(o=>o.fn([{target,type,attributeName}]));
 const route=(page,tab)=>{rootEl.dataset.route=page;if(page==='sync')state.tab=tab||'file';if(page==='models')modelTab.dataset.tab=tab||'channels';if(page==='logs')diagTab.dataset.diagTab=tab||'doctor';if(page==='extensions')extTab.dataset.extTab=tab||'skills';if(page==='settings')settingsTab.dataset.settingsSection=tab||'general';changed(rootEl,'attributes','data-route');};
 const visible=n=>!n.hidden&&(!n.parentElement||visible(n.parentElement));
 return {get,side,lower,lowerBody,windowsCard,state,route,changed,main,body,visible,historyWrites,scrolls,change:(id,value)=>{const target=get('#'+id);target.value=value;side.handlers.change.forEach(fn=>fn({target}));},clickAnchor:id=>get('a[href="#'+id+'"]').handlers.click.forEach(fn=>fn({preventDefault(){}}))};
}
const results=[];
function check(name,fn){try{fn(make());results.push({name,status:'PASS'});}catch(e){results.push({name,status:'FAIL',error:e.message});}console.log(results.at(-1).status+' '+name);}
check('文件同步只展示当前文件与应用场景，公共设置保留',h=>{assert.equal(h.side.dataset.context,'sync:file');assert(h.visible(h.get('#protoCommon')));assert(h.visible(h.get('#syncFileDemoCard')));assert(h.visible(h.get('#syncDemoCard')));assert(!h.visible(h.get('#mpDemoCard')));assert(!h.visible(h.get('#trayStateTester')));h.change('sxImportSample','corrupt');h.change('sxMockScenario','backup-fail');assert.equal(h.state.importSample,'corrupt');assert.equal(h.state.scenario,'backup-fail');});
check('Tab 与主内容重绘更新 Mock，往返保留场景',h=>{h.state.tab='webdav';h.changed(h.main,'childList');assert.equal(h.side.dataset.context,'sync:webdav');assert(!h.visible(h.get('#syncFileDemoCard')));assert(h.visible(h.get('#syncDemoCard')));assert.match(h.get('#syncMockHint').textContent,/检查变化/);h.route('models','templates');h.state.importSample='legacy';h.state.scenario='partial';h.route('sync','file');assert.equal(h.get('#sxImportSample').value,'legacy');assert.equal(h.get('#sxMockScenario').value,'partial');});
check('渠道和模版场景按当前模型 Tab 展示',h=>{h.route('models','channels');assert(h.visible(h.get('#mpDemoCard')));assert(h.visible(h.get('[data-scene="discovery-fail"]')));assert(!h.visible(h.get('[data-scene="late-generation"]')));h.route('models','templates');assert(!h.visible(h.get('[data-scene="discovery-fail"]')));assert(h.visible(h.get('[data-scene="late-generation"]')));assert(!h.visible(h.get('#syncDemoCard')));});
check('诊断日志与检查、设置安装等上下文独立',h=>{h.route('logs','logs');assert(h.visible(h.get('#logDemoCard')));assert(!h.visible(h.get('#scenarios')));h.route('logs','doctor');assert(!h.visible(h.get('#logDemoCard')));assert(h.visible(h.get('#scenarios')));h.route('settings','installation');assert(h.visible(h.get('#envStates')));h.route('settings','general');assert(!h.visible(h.get('#envStates')));h.route('extensions','mcp');assert.equal(h.side.dataset.context,'extensions:mcp');assert(!h.visible(h.get('#syncDemoCard')));});
check('通知专题仍有公共设置，退出恢复当前页',h=>{h.body.classList.toggle('notif-mode',true);h.changed(h.body);assert.equal(h.side.dataset.context,'notify');assert(h.visible(h.get('#notifDemoCard')));assert(h.visible(h.get('#protoCommon')));assert(!h.visible(h.get('#syncDemoCard')));h.body.classList.toggle('notif-mode',false);h.changed(h.body);assert.equal(h.side.dataset.context,'sync:file');});
check('Windows 动态工具移入下半区，公共区同在一个滚动容器',h=>{assert.equal(h.windowsCard.parentElement,h.lowerBody);h.body.classList.toggle('windows-mode',true);h.changed(h.body);assert.equal(h.side.dataset.context,'windows');assert(h.visible(h.windowsCard));assert(h.visible(h.get('#protoCommon')));assert(!h.visible(h.get('#syncDemoCard')));h.body.classList.toggle('windows-mode',false);h.changed(h.body);assert(!h.visible(h.windowsCard));});
check('工具锚点只滚动右侧，不覆盖产品路由',h=>{h.clickAnchor('ctl-mock');assert.equal(h.historyWrites.length,0);assert.equal(h.scrolls.length,1);assert.equal(h.scrolls[0].target,h.side);assert(h.scrolls[0].value.top>0);});
check('两个顶层折叠段命名等长，默认全局收起 / 本页展开',h=>{const common=h.get('#protoCommon'),std=h.get('#protoStdCards');assert(common,'#protoCommon');assert(std,'#protoStdCards');assert(!('open' in common.attrs),'全局评审条件默认收起');assert('open' in std.attrs,'本页演示场景默认展开');const names=[common,std].map(group=>{const start=html.indexOf('id="'+group.id+'"');const section=html.slice(start,html.indexOf('</details>',start));return section.match(/class="proto-group-name"[^>]*>([^<]+)/)?.[1];});assert(names.every(Boolean),'两个主折叠段都有名称');assert.equal(names.length,2);assert.equal([...names[0]].length,[...names[1]].length);assert(html.indexOf('id="protoCommon"')<html.indexOf('id="protoStdCards"'),'全局评审条件在前');});
check('面板 / 拓展 / 通知历史 / 设置各 Tab 都有专属 Mock 卡片',h=>{const cards=h.side.querySelectorAll('#protoStdCards [data-proto-contexts]').map(c=>c.getAttribute('data-proto-contexts'));for(const want of ['panel','extensions:skills','extensions:mcp','logs:notifications','settings:general','settings:backup','settings:extensions','settings:cleanup','settings:cli','settings:upgrade','settings:about'])assert(cards.includes(want),'缺少 '+want);});
const report={date:'2026-10-08',sources:files.map((file,i)=>({file:path.relative(root,path.join(base,file)),sha256:crypto.createHash('sha256').update([panel,anchors,html][i]).digest('hex')})),boundary:'Node VM 执行实际工具分区及锚点脚本，读取主原型侧栏标记，最小 DOM 提供事件与几何替身；验证上下文、选择器联动和滚动目标，不验证 CSS 绘制、真实焦点或滚动高度。',results};
if(!process.argv.includes('--no-report'))fs.writeFileSync(path.join(here,'../配置同步/源码回归结果-原型工具板结构.json'),JSON.stringify(report,null,2)+'\n');
console.log('TOTAL '+results.length+' / FAIL '+results.filter(x=>x.status==='FAIL').length);
if(results.some(x=>x.status==='FAIL')){console.log(JSON.stringify(results.filter(x=>x.status==='FAIL')));process.exitCode=1;}
