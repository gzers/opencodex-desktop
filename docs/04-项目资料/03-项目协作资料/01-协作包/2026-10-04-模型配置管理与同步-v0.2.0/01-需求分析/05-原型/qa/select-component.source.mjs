import {repoRoot as root, prototypeRoot} from '../../../../2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/原型/qa/prototype-location.mjs';
/* Node VM + 最小 DOM 行为 stub；无浏览器 / CSS 引擎 / 网络，不作视觉或真实焦点验收。 */
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const dir=path.join(prototypeRoot,'原型');
const hooks={},windowHooks={},pending=[];
let observer;
class FakeEvent{
  constructor(type,settings={}){this.type=type;Object.assign(this,settings);this.prevented=false;this.stopped=false;}
  preventDefault(){this.prevented=true;}
  stopImmediatePropagation(){this.stopped=true;}
}
class Node{
  constructor(tag='div'){
    this.tagName=tag.toUpperCase();this.nodeType=1;this.children=[];this.parentElement=null;this.attrs=new Map();this.dataset={};this.style={cssText:''};this.listeners={};this.disabled=false;this.hidden=false;this._text='';this._index=0;this.offsetWidth=160;this.rect={left:100,top:100,right:260,bottom:134,width:160,height:34};
    this.classList={contains:name=>this.className.split(/\s+/).includes(name),add:(...names)=>{this.className=[...new Set([...this.className.split(/\s+/).filter(Boolean),...names])].join(' ');},remove:(...names)=>{this.className=this.className.split(/\s+/).filter(name=>!names.includes(name)).join(' ');},toggle:(name,force)=>{const chosen=force??!this.classList.contains(name);if(chosen)this.classList.add(name);else this.classList.remove(name);return chosen;}};
  }
  get className(){return this.attrs.get('class')||'';}set className(value){this.attrs.set('class',value);}
  get id(){return this.attrs.get('id')||'';}set id(value){this.attrs.set('id',value);}
  get attributes(){return [...this.attrs].map(([name,value])=>({name,value}));}
  get title(){return this.attrs.get('title')||'';}set title(value){this.attrs.set('title',value);}
  setAttribute(name,value){this.attrs.set(name,String(value));if(name.startsWith('data-'))this.dataset[name.slice(5).replace(/-([a-z])/g,(_,char)=>char.toUpperCase())]=String(value);}
  getAttribute(name){return this.attrs.get(name)??null;}
  removeAttribute(name){this.attrs.delete(name);if(name.startsWith('data-'))delete this.dataset[name.slice(5).replace(/-([a-z])/g,(_,char)=>char.toUpperCase())];}
  get textContent(){return this._text+this.children.map(node=>node.textContent).join('');}set textContent(value){this._text=value;this.replaceChildren();}
  get options(){return this.querySelectorAll('option');}
  get selectedOptions(){return this.options.filter(option=>option.selected);}
  get selected(){return this.parentElement?.tagName==='SELECT'&&!this.parentElement.multiple?this.parentElement.options[this.parentElement._index]===this:!!this._selected;}set selected(value){this._selected=!!value;if(value&&this.parentElement?.tagName==='SELECT'&&!this.parentElement.multiple)this.parentElement._index=this.parentElement.options.indexOf(this);}
  get selectedIndex(){return this.multiple?this.options.findIndex(option=>option.selected):this.options.length?this._index:-1;}set selectedIndex(index){this._index=index;if(this.multiple)this.options.forEach((option,i)=>option.selected=i===index);}
  get value(){return this.tagName==='SELECT'?this.options[this.selectedIndex]?.value||'':this.getAttribute('value')||this.textContent;}set value(value){if(this.tagName==='SELECT')this._index=this.options.findIndex(option=>option.value===value);else this.setAttribute('value',value);}
  get isConnected(){return this===body||!!this.parentElement?.isConnected;}
  get previousElementSibling(){const siblings=this.parentElement?.children||[];return siblings[siblings.indexOf(this)-1]||null;}
  get labels(){return [];}
  get scrollHeight(){return this.children.filter(node=>!node.hidden).length*34+10;}
  matches(selector){
    if(selector===':popover-open')return !!this.popoverOpen;
    if(selector.includes('[')){const match=/^(.*?)\[([^=\]]+)(?:="([^"]*)")?\]$/.exec(selector);return !!match&&(!match[1]||this.matches(match[1]))&&(this.getAttribute(match[2])!==null||match[2].startsWith('data-')&&this.dataset[match[2].slice(5).replace(/-([a-z])/g,(_,char)=>char.toUpperCase())]!==undefined);}
    if(selector.startsWith('.'))return selector.slice(1).split('.').every(name=>this.classList.contains(name));
    return this.tagName===selector.toUpperCase();
  }
  closest(selector){return this.matches(selector)?this:this.parentElement?.closest(selector)||null;}
  querySelectorAll(selector){const found=[];for(const child of this.children){if(child.matches(selector))found.push(child);found.push(...child.querySelectorAll(selector));}return found;}
  querySelector(selector){return this.querySelectorAll(selector)[0]||null;}
  appendChild(child){child.remove();child.parentElement=this;this.children.push(child);return child;}
  remove(){if(this.parentElement){const siblings=this.parentElement.children;siblings.splice(siblings.indexOf(this),1);this.parentElement=null;}}
  before(child){const parent=this.parentElement;child.remove();child.parentElement=parent;parent.children.splice(parent.children.indexOf(this),0,child);}
  replaceChildren(...children){for(const child of this.children)child.parentElement=null;this.children=[];for(const child of children)this.appendChild(child);}
  contains(node){return node===this||this.children.some(child=>child.contains(node));}
  getClientRects(){return this.isConnected&&!this.hidden?[this.rect]:[];}
  getBoundingClientRect(){return this.rect;}
  focus(){document.activeElement=this;}
  showPopover(){this.popoverOpen=true;}
  hidePopover(){this.popoverOpen=false;}
  addEventListener(type,fn){(this.listeners[type]??=[]).push(fn);}
  dispatchEvent(event){event.target=this;this['on'+event.type]?.(event);let node=this;while(node){for(const fn of node.listeners[event.type]||[])fn(event);node=event.bubbles?node.parentElement:null;}if(event.bubbles)dispatch(event);return !event.prevented;}
}
const body=new Node('body'),html=new Node('html');
const document={body,documentElement:html,activeElement:null,createElement:tag=>new Node(tag),querySelectorAll:selector=>body.querySelectorAll(selector),getElementById:id=>body.querySelectorAll('*').find(node=>node.id===id),addEventListener:(type,fn,capture)=>{(hooks[type]??=[]).push({fn,capture});}};
function dispatch(event){for(const hook of [...hooks[event.type]||[]].sort((a,b)=>Number(!!b.capture)-Number(!!a.capture))){if(event.stopped)break;hook.fn(event);}return event;}
function click(target){return dispatch(new FakeEvent('click',{target}));}
function key(key,target=document.activeElement){return dispatch(new FakeEvent('keydown',{target,key}));}
function makeSelect(id,items){const source=new Node('select');source.id=id;source.setAttribute('aria-label',id);source.className='mp-select';for(const [value,label,disabled=false] of items){const option=new Node('option');option.value=value;option.textContent=label;option.disabled=disabled;source.appendChild(option);}body.appendChild(source);return source;}
function mutate(target,addedNodes=[]){observer.callback([{target,type:'childList',addedNodes,removedNodes:[]}]);let limit=10;while(pending.length&&limit-->0)pending.shift()();assert.ok(limit>0,'observer refresh converges');}
function trigger(source){return source.parentElement.querySelector('.select-trigger');}
function menu(source){return document.querySelectorAll('.select-menu').find(node=>node.id===trigger(source).getAttribute('aria-controls'));}
function list(source){return menu(source).querySelectorAll('.select-option');}
const source=makeSelect('model',[['a','Alpha'],['b','Blocked',true],['c','Codex']]);
const legacy=new Node();legacy.className='select';legacy.dataset.select='';
const legacyTrigger=new Node('button');legacyTrigger.className='select-trigger';legacyTrigger.setAttribute('aria-label','每页数量');
const legacyValue=new Node('span');legacyValue.className='select-value';legacyValue.textContent='10 条';legacyTrigger.appendChild(legacyValue);legacy.appendChild(legacyTrigger);
const legacyMenu=new Node();legacyMenu.className='select-menu';legacy.appendChild(legacyMenu);
for(const name of ['10 条','20 条']){const option=new Node('button');option.className='select-option'+(name==='10 条'?' selected':'');option.textContent=name;legacyMenu.appendChild(option);}body.appendChild(legacy);
const window={innerWidth:900,innerHeight:600,addEventListener:(type,fn)=>windowHooks[type]=fn};
const context={document,window,console,Event:FakeEvent,CustomEvent:FakeEvent,MutationObserver:class{constructor(callback){this.callback=callback;observer=this;}observe(){}},queueMicrotask:fn=>pending.push(fn),getComputedStyle:()=>({getPropertyValue:name=>({'--space-2':'8px','--space-3':'12px','--select-menu-max-width':'320px','--select-menu-max-height':'260px'}[name]||'')}),Date};
vm.createContext(context);vm.runInContext(fs.readFileSync(path.join(dir,'select-component.js'),'utf8'),context);
let passed=0;function check(name,fn){fn();passed++;console.log('PASS '+name);}
check('整个主入口接入公共样式及控制器，旧嵌套层处理已移除',()=>{const index=fs.readFileSync(path.join(dir,'index.html'),'utf8');assert.ok(index.indexOf('select-component.css')<index.indexOf('../glass-components.css'));assert.ok(index.indexOf('select-component.js')<index.indexOf('model-config.js'));assert.doesNotMatch(index,/const closeAllSelects|z-index:24;top:calc/);assert.match(index,/pageSizeRoot\?\.addEventListener\('selectchange'/);});
check('native 保留 ID / value，隐藏原控件且可见触发器包含字段与选值',()=>{assert.equal(source.id,'model');assert.equal(source.value,'a');assert.equal(source.hidden,true);assert.equal(source.tabIndex,-1);assert.equal(trigger(source).getAttribute('aria-label'),'model：Alpha');assert.equal(trigger(source).querySelector('.select-value').textContent,'Alpha');});
check('菜单脱离卡片进入 body 和 manual popover，关闭后还原',()=>{click(trigger(source));assert.equal(menu(source).parentElement,body);assert.equal(menu(source).getAttribute('popover'),'manual');assert.equal(menu(source).popoverOpen,true);assert.equal(trigger(source).getAttribute('aria-expanded'),'true');window.prototypeSelect.close();assert.equal(menu(source).parentElement,source.parentElement);assert.equal(menu(source).popoverOpen,false);});
check('选择更新 native 并各触发一次 input / change 及原 onchange',()=>{let input=0,change=0,callback=0;source.addEventListener('input',()=>input++);source.addEventListener('change',()=>change++);source.onchange=()=>callback++;click(trigger(source));click(list(source)[2]);assert.equal(source.value,'c');assert.deepEqual([input,change,callback],[1,1,1]);assert.equal(document.activeElement,trigger(source));click(trigger(source));click(list(source)[2]);assert.deepEqual([input,change,callback],[1,1,1]);});
check('Arrow / Home / End 跳过禁用项，Enter 选择当前项',()=>{click(trigger(source));key('Home');assert.equal(document.activeElement.textContent,'Alpha');key('ArrowDown');assert.equal(document.activeElement.textContent,'Codex');key('End');assert.equal(document.activeElement.textContent,'Codex');key('ArrowUp');assert.equal(document.activeElement.textContent,'Alpha');key('Enter');assert.equal(source.value,'a');});
check('Escape 先关闭下拉并消费事件，第二次才能交给弹窗',()=>{click(trigger(source));let modalEscape=0;document.addEventListener('keydown',event=>{if(event.key==='Escape')modalEscape++;});const first=key('Escape');assert.equal(first.prevented,true);assert.equal(modalEscape,0);assert.equal(document.activeElement,trigger(source));key('Escape');assert.equal(modalEscape,1);});
check('Tab 关闭下拉并把起点交还触发器，未阻止原生 Tab',()=>{click(trigger(source));const event=key('Tab');assert.equal(event.prevented,false);assert.equal(document.activeElement,trigger(source));assert.equal(trigger(source).getAttribute('aria-expanded'),'false');});
check('旧分页组件通过属主 selectchange 传值，portal 不丢原业务',()=>{let value;legacy.addEventListener('selectchange',event=>value=parseInt(event.detail.value,10));click(legacyTrigger);click(legacyMenu.children[1]);assert.equal(value,20);assert.equal(legacyValue.textContent,'20 条');assert.equal(legacyMenu.parentElement,legacy);});
check('只允许一个菜单打开，点击外部关闭',()=>{click(trigger(source));click(legacyTrigger);assert.equal(menu(source).parentElement,source.parentElement);assert.equal(legacyMenu.parentElement,body);click(body);assert.equal(legacyMenu.parentElement,legacy);});
check('动态表单与动态 options 自动刷新，禁用状态跟随',()=>{const dynamic=makeSelect('dynamic',[['one','One']]);mutate(body,[dynamic]);assert.equal(trigger(dynamic).disabled,false);const option=new Node('option');option.value='two';option.textContent='Two';dynamic.appendChild(option);mutate(dynamic,[option]);assert.equal(list(dynamic).length,2);dynamic.value='two';click(trigger(dynamic));assert.equal(document.activeElement.textContent,'Two');dynamic.disabled=true;observer.callback([{target:dynamic,type:'attributes'}]);pending.shift()();assert.equal(trigger(dynamic).disabled,true);assert.equal(menu(dynamic).parentElement,dynamic.parentElement);});
check('下游模型填充保留联动回调，change 后同步所有字段',()=>{const downstream=makeSelect('dependent',[['old','Old']]);window.prototypeSelect.enhance();source.onchange=()=>{const option=new Node('option');option.value='new';option.textContent='New';downstream.replaceChildren(option);};click(trigger(source));click(list(source)[2]);assert.equal(list(downstream)[0].textContent,'New');assert.equal(trigger(downstream).querySelector('.select-value').textContent,'New');});
check('typeahead 按可用选项查找；禁用选项点击不改值',()=>{click(trigger(source));key('a');assert.equal(document.activeElement.textContent,'Alpha');const previous=source.value;click(list(source)[1]);assert.equal(source.value,previous);window.prototypeSelect.close();});
check('不支持 Popover 时仍使用 body portal 并归还',()=>{menu(source).showPopover=undefined;click(trigger(source));assert.equal(menu(source).parentElement,body);assert.equal(menu(source).getAttribute('popover'),null);window.prototypeSelect.close();assert.equal(menu(source).parentElement,source.parentElement);});
check('滚动菜单保持展开，外部滚动关闭；删除触发器不残留菜单',()=>{click(trigger(source));dispatch(new FakeEvent('scroll',{target:menu(source)}));assert.equal(menu(source).parentElement,body);dispatch(new FakeEvent('scroll',{target:body}));assert.equal(menu(source).parentElement,source.parentElement);const removed=makeSelect('removed',[['x','X']]);window.prototypeSelect.enhance();const floating=menu(removed);click(trigger(removed));removed.parentElement.remove();window.prototypeSelect.enhance();assert.notEqual(floating.parentElement,body);});
check('边缘定位可向上展开、水平约束并按缩放换算尺寸',()=>{const place=window.prototypeSelect.placement;const rect={left:850,top:550,bottom:584,width:120};const p=place(rect,{width:900,height:600},{width:320,height:220,maxHeight:260});assert.equal(p.up,true);assert.ok(p.left+p.width<=888);assert.ok(p.top>=12);assert.ok(p.top+p.height<rect.top);const large=place({left:20,top:50,bottom:118,width:320},{width:900,height:600},{scale:2,width:320,height:100,maxHeight:260});assert.equal(large.width,640);assert.equal(large.height,200);const small=place(rect,{width:900,height:600},{scale:.5,width:320,height:100});assert.equal(small.width,160);});
let multi,multiEvents;
check('multiple 推理菜单显示多选值及 aria-multiselectable',()=>{multi=makeSelect('efforts',[['low','低'],['medium','中'],['high','高'],['max','最高',true]]);multi.multiple=true;multi.options[0].selected=true;multi.options[2].selected=true;window.prototypeSelect.enhance();assert.equal(menu(multi).getAttribute('aria-multiselectable'),'true');assert.equal(trigger(multi).querySelector('.select-value').textContent,'低 / 高');assert.deepEqual(list(multi).map(o=>o.getAttribute('aria-selected')),['true','false','true','false']);});
check('multiple 切换一项保留其它选择、菜单和焦点，每次只发送一次事件',()=>{multiEvents={input:0,change:0};for(const type of ['input','change'])multi.addEventListener(type,()=>multiEvents[type]++);click(trigger(multi));click(list(multi)[1]);assert.deepEqual(multi.selectedOptions.map(o=>o.value),['low','medium','high']);assert.equal(menu(multi).parentElement,body);assert.equal(trigger(multi).getAttribute('aria-expanded'),'true');assert.equal(document.activeElement,list(multi)[1]);assert.deepEqual(multiEvents,{input:1,change:1});click(list(multi)[0]);assert.deepEqual(multi.selectedOptions.map(o=>o.value),['medium','high']);assert.deepEqual(multiEvents,{input:2,change:2});});
check('multiple 键盘空格与 Enter 可连续多选，禁用项不变，Escape 关闭',()=>{key('Home');key(' ');assert.deepEqual(multi.selectedOptions.map(o=>o.value),['low','medium','high']);key('End');key('Enter');assert.deepEqual(multi.selectedOptions.map(o=>o.value),['low','medium']);const counts={...multiEvents};click(list(multi)[3]);assert.deepEqual(multiEvents,counts);assert.equal(menu(multi).parentElement,body);key('Escape');assert.equal(trigger(multi).getAttribute('aria-expanded'),'false');assert.equal(document.activeElement,trigger(multi));});
check('multiple 全取消显示未选择，外部 selected 改动和禁用同步',()=>{multi.options.forEach(o=>o.selected=false);window.prototypeSelect.enhance();assert.equal(trigger(multi).querySelector('.select-value').textContent,'未选择');multi.options[2].selected=true;mutate(multi);assert.equal(trigger(multi).querySelector('.select-value').textContent,'高');click(trigger(multi));multi.disabled=true;mutate(multi);assert.equal(trigger(multi).disabled,true);assert.equal(menu(multi).parentElement,multi.parentElement);});
console.log('SELECT SOURCE CHECK '+passed+' PASS; 浏览器布局与真实键盘焦点仍待验。');
