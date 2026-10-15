/* 内存DOM集成检查。没有浏览器布局引擎，不能作为实屏/原生QA证据。 */
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
function jsdom(){
 const roots=[path.resolve(__dirname,'../../../../../apps/desktop/ui/node_modules/jsdom'),process.env.COMPONENT_JSDOM_PATH].filter(Boolean);
 for(const p of roots)try{return require(p);}catch{}
 throw Error('需要既有jsdom依赖；设置COMPONENT_JSDOM_PATH。检查不安装/修改生产依赖。');
}
async function make(hash='#settings'){
 const {JSDOM,VirtualConsole}=jsdom(),errors=[],writes=[],network=[],vc=new VirtualConsole();
 vc.on('jsdomError',e=>errors.push(e.message));
 const html=fs.readFileSync(path.join(__dirname,'index.html'),'utf8'),dom=new JSDOM(html,{url:'https://prototype.invalid/index.html'+hash,runScripts:'outside-only',pretendToBeVisual:true,virtualConsole:vc}),w=dom.window;
 const observers=new Set(),MO=w.MutationObserver;w.MutationObserver=class extends MO{constructor(fn){super(fn);observers.add(this);}};
 const timers=new Set(),intervals=new Set(),originalTimeout=w.setTimeout.bind(w),originalClear=w.clearTimeout.bind(w);
 w.setTimeout=(fn,ms=0,...args)=>{const id=originalTimeout(()=>{timers.delete(id);fn(...args);},ms);timers.add(id);return id;};w.clearTimeout=id=>{timers.delete(id);originalClear(id);};
 // 持续后台观察器在此不驱动；交互自身短计时与rAF照常执行。
 w.setInterval=(fn,ms)=>{intervals.add(fn);return intervals.size;};w.clearInterval=()=>{};
 w.requestAnimationFrame=fn=>w.setTimeout(()=>fn(w.performance.now()),0);w.cancelAnimationFrame=w.clearTimeout;
 w.matchMedia=q=>({matches:false,media:q,addEventListener(){},removeEventListener(){},addListener(){},removeListener(){}});
 w.ResizeObserver=class{observe(){}unobserve(){}disconnect(){}};
 w.IntersectionObserver=class{observe(){}unobserve(){}disconnect(){}};
 w.HTMLCanvasElement.prototype.getContext=()=>null;
 w.fetch=(...args)=>{network.push(args);throw Error('原型检查禁止网络');};
 w.Element.prototype.scrollIntoView=function(){};w.HTMLElement.prototype.scrollTo=function(arg){this.scrollTop=arg?.top||0;};
 w.HTMLElement.prototype.getBoundingClientRect=function(){
  const main=w.document.querySelector('.main'),ids=[...w.document.querySelectorAll('[data-ct-section-id]')].map(el=>el.id),idx=ids.indexOf(this.id),top=idx>=0?160+idx*800-(main?.scrollTop||0):this.matches('.main')?100:80;
  const width=this.matches('.window')?Number.parseInt(w.document.documentElement.style.getPropertyValue('--win-w'))||1180:this.matches('.main')?880:300,height=idx>=0?800:this.matches('.window')?760:this.matches('.main')?650:40;
  return {x:0,y:top,left:0,right:width,top,bottom:top+height,width,height,toJSON(){return this;}};
 };
 w.HTMLElement.prototype.getClientRects=function(){return this.hidden||this.closest('[hidden]')?[]:[this.getBoundingClientRect()];};
 const storage=w.Storage.prototype.setItem;w.Storage.prototype.setItem=function(k,v){writes.push([k,String(v)]);return storage.call(this,k,v);};
 w.addEventListener('error',e=>{errors.push(e.error?.stack||e.message);e.preventDefault();});
 const scripts=[...w.document.scripts];for(const script of scripts){
  if(script.type==='module'||script.type&&script.type!=='text/javascript')continue;
  let code=script.textContent,label='index.inline';
  if(script.getAttribute('src')){const file=path.resolve(__dirname,script.getAttribute('src').split('?')[0]);code=fs.readFileSync(file,'utf8');label=path.basename(file);}
  Object.defineProperty(w.document,'currentScript',{configurable:true,value:script});
  try{vm.runInContext(code,dom.getInternalVMContext(),{filename:label});}catch(e){errors.push(label+': '+e.stack);}
 }
 const settle=()=>new Promise(resolve=>originalTimeout(resolve,30));await settle();
 function click(selector){const el=w.document.querySelector(selector);if(!el)throw Error('缺少控件 '+selector);el.dispatchEvent(new w.MouseEvent('pointerdown',{bubbles:true}));el.focus();el.click();return el;}
 function change(selector,value){const el=w.document.querySelector(selector);if(!el)throw Error('缺少控件 '+selector);if(el.type==='checkbox')el.checked=Boolean(value);else el.value=value;el.dispatchEvent(new w.Event('change',{bubbles:true}));return el;}
 function input(selector,value){const el=w.document.querySelector(selector);el.value=value;el.dispatchEvent(new w.Event('input',{bubbles:true}));return el;}
 return {w,document:w.document,errors,writes,network,click,change,input,settle,wait:ms=>new Promise(r=>originalTimeout(r,ms)),dispose(){observers.forEach(o=>o.disconnect());timers.forEach(originalClear);dom.window.close();}};
}
module.exports={make};
