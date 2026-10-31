import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-022-model-selector/apps/desktop/ui/qa/engine.mjs';
const entry='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html';
const out=process.env.AUDIT_OUT_DIR;fs.mkdirSync(out,{recursive:true});
const e=await pickEngine(),b=await e.launch(),errors=[],checks=[];
const check=(n,ok,d)=>{checks.push({name:n,passed:!!ok,detail:d});assert(ok,n+' '+JSON.stringify(d));};
try{
for(const size of [{w:1180,h:760,theme:'light'},{w:900,h:600,theme:'dark'}])for(const effects of ['high','mid','low']){
 const label=size.w+'-'+size.theme+'-'+effects;
 const p=await b.newPage({viewport:{width:size.w+470,height:size.h+220}});
 p.on('pageerror',x=>errors.push({case:label,message:x.message}));
 await p.route('http://**/*',r=>r.abort());await p.route('https://**/*',r=>r.abort());
 await p.goto('file://'+encodeURI(entry)+'#overview');await p.waitForFunction(()=>window.renderWebdav);
 await p.evaluate(({size,effects})=>{const w=document.querySelector('.window');w.style.width=size.w+'px';w.style.height=size.h+'px';w.style.flex='none';window.applyTheme?.(size.theme);document.documentElement.dataset.theme=size.theme;document.documentElement.dataset.effects=effects;},{size,effects});
 // 1) 侧栏：所有可见导航项同字号同字重；选中项不加粗
 const nav=await p.evaluate(()=>{const set=new Set();const act=document.querySelector('.nav button[data-route].active .nav-label');for(const l of document.querySelectorAll('.nav button[data-route]:not([hidden]) .nav-label')){const s=getComputedStyle(l);set.add(s.fontSize+'|'+s.fontWeight);}const a=getComputedStyle(act);return {combos:[...set],activeWeight:a.fontWeight,activeColor:a.color,activeText:act.textContent.trim(),inactiveColor:getComputedStyle(document.querySelector('.nav button[data-route]:not(.active) .nav-label')).color};});
 check(label+' nav items share one size/weight',nav.combos.length===1&&/\|400$/.test(nav.combos[0]),nav);
 check(label+' active nav not bolded',nav.activeWeight==='400'&&nav.activeColor!==nav.inactiveColor,nav);
 // 1b) 中文回退一致：同字号下行内所有中文标签字宽必须相同（韩文字体会给出不同字宽）
 const ink=await p.evaluate(()=>{const words=[];for(const l of document.querySelectorAll('.nav button[data-route]:not([hidden]) .nav-label')){const r=document.createRange();r.selectNodeContents(l);const b=r.getBoundingClientRect();words.push({t:l.textContent.trim(),w:+b.width.toFixed(1),h:+b.height.toFixed(1),size:getComputedStyle(l).fontSize});}return words;});
 const widths=[...new Set(ink.map(x=>x.w))];
 check(label+' chinese labels share one glyph width',widths.length===1,ink);

 // 2) 卡片标记不占位、不压标题与 ⋯
 const geo=await p.evaluate(()=>[...document.querySelectorAll('#mods .mod')].map(card=>{const g=n=>{const b=n.getBoundingClientRect();return {l:b.left,r:b.right,t:b.top,b:b.bottom};};const m=g(card.querySelector('.proto-annotation')),h=g(card.querySelector('h3')),go=g(card.querySelector('.mod-go')),c=g(card);const ov=(a,b)=>!(a.r<=b.l||b.r<=a.l||a.b<=b.t||b.b<=a.t);return {id:card.id,overTitle:ov(m,h),overGo:ov(m,go),markerAbove:Math.round(h.t-m.t),cardTop:Math.round(c.t),titleTop:Math.round(h.t)};}));
 check(label+' annotation overlays without pushing title',geo.every(x=>!x.overTitle&&x.titleTop-x.cardTop<20),geo);
 check(label+' annotation does not cover mod-go',geo.every(x=>!x.overGo),geo);
 // 3) 三卡状态区仍单一字号字重字色
 const typo=await p.evaluate(()=>{const set=new Set();const walk=(root)=>{for(const n of root.childNodes){if(n.nodeType===3){const t=n.textContent.trim();if(!t)continue;const el=n.parentElement;if(el.classList.contains('ov-state-sep'))continue;const s=getComputedStyle(el);set.add(s.fontSize+'|'+s.fontWeight+'|'+s.color);}else if(n.nodeType===1&&!n.classList.contains('ov-state-sep'))walk(n);}};for(const c of document.querySelectorAll('#mods .mod-status'))walk(c);return [...set];});
 check(label+' status area single type scale',typo.length===1,typo);
 if(effects==='high'){await p.locator('nav.nav').screenshot({path:path.join(out,label+'-nav.png')});await p.locator('.window').screenshot({path:path.join(out,label+'-overview.png')});}
 await p.close();
}
check('no page errors',errors.length===0,errors);
}finally{await b.close();fs.writeFileSync(path.join(out,'nav-annotation-report.json'),JSON.stringify({engine:e.name,checks,errors,failed:checks.filter(x=>!x.passed).length,scope:'sidebar active weight + annotation overlay geometry, prototype only'},null,2));}
console.log(JSON.stringify({engine:e.name,checks:checks.length,errors}));

