import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-022-model-selector/apps/desktop/ui/qa/engine.mjs';
const root='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main';
const entry=root+'/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html';
const out=process.env.AUDIT_OUT_DIR;fs.mkdirSync(out,{recursive:true});
const e=await pickEngine(),b=await e.launch(),errors=[],checks=[];
const check=(name,ok,detail)=>{checks.push({name,passed:!!ok,detail});assert(ok,name+' '+JSON.stringify(detail));};
const rgb=str=>{const s=String(str||'').trim().replace('#','');const hex=s.length===3?s.split('').map(c=>c+c).join(''):s.slice(0,6);return [0,2,4].map(i=>parseInt(hex.slice(i,i+2),16)).join(',');};
try{
for(const size of [{w:1180,h:760,theme:'light'},{w:900,h:600,theme:'dark'}])for(const effects of ['high','mid','low']){
 const label=size.w+'-'+size.theme+'-'+effects;
 const p=await b.newPage({viewport:{width:size.w+470,height:size.h+220}});
 p.on('pageerror',x=>errors.push({case:label,message:x.message}));
 await p.route('http://**/*',r=>r.abort());await p.route('https://**/*',r=>r.abort());
 await p.goto('file://'+encodeURI(entry)+'#overview');await p.waitForFunction(()=>window.renderWebdav);
 await p.evaluate(({size,effects})=>{const w=document.querySelector('.window');w.style.width=size.w+'px';w.style.height=size.h+'px';w.style.flex='none';window.applyTheme?.(size.theme);document.documentElement.dataset.theme=size.theme;document.documentElement.dataset.effects=effects;},{size,effects});
 // 1) 未加载：只显示 —，不画点不写状态词
 const idle=await p.evaluate(()=>({remote:document.querySelector('#syncRemoteTime').textContent,dotHidden:document.querySelector('#syncDirection').hidden,textHidden:document.querySelector('#syncDirectionText').hidden,version:document.querySelector('#card-overview-upgrade .mnt-version-latest').textContent,dots:document.querySelectorAll('#card-overview-upgrade .mnt-dot').length,words:document.querySelector('#card-overview-upgrade .mod-status').innerText}));
 check(label+' no data shows dash only',idle.remote==='—'&&idle.dotHidden&&idle.textHidden&&idle.version==='—'&&idle.dots===0&&!/尚未检查|未连接/.test(idle.words),idle);
 // 2) 三卡字号/字重/字色一致（含紧凑模式），只靠位置分主次
 const typo=await p.evaluate(()=>{const set=new Set();const walk=(root)=>{for(const n of root.childNodes){if(n.nodeType===3){const t=n.textContent.trim();if(!t)continue;const el=n.parentElement;if(el.classList.contains('ov-state-sep'))continue;if(el.classList.contains('ov-state-dir')||el.closest('.ov-state-dir'))continue;const s=getComputedStyle(el);set.add(s.fontSize+'|'+s.fontWeight+'|'+s.color);}else if(n.nodeType===1&&!n.classList.contains('ov-state-sep'))walk(n);}};for(const c of document.querySelectorAll('#mods .mod-status'))walk(c);return [...set];});
 check(label+' three cards share one type scale',typo.length===1&&/400/.test(typo[0]),typo);
 // 3) 检查后：版本在右缘、状态点用统一色令牌
 await p.locator('[data-maint-action="check-all"]').click();await p.waitForTimeout(1400);await p.locator('#modalCancel').click();await p.waitForTimeout(150);
 const ver=await p.evaluate(()=>{const dot=document.querySelector('#card-overview-upgrade .mnt-dot');const latest=document.querySelector('#card-overview-upgrade .mnt-version-latest');const row=latest.closest('.mnt-version-row');const side=latest.closest('.mnt-version-side');const dr=dot.getBoundingClientRect(),lr=latest.getBoundingClientRect(),rr=row.getBoundingClientRect(),sr=side.getBoundingClientRect();const kids=[...side.children].map(n=>n.className||n.tagName);return {dotCls:dot.className,dotW:dr.width,dotH:dr.height,dotCy:Math.round((dr.top+dr.height/2)*10)/10,latestCy:Math.round((lr.top+lr.height/2)*10)/10,sideGap:Math.round(rr.right-sr.right),dotAfterVersion:dr.left>=lr.right,kids,dotColor:getComputedStyle(dot).backgroundColor,size:getComputedStyle(row).fontSize};});
 const green=await p.evaluate(()=>getComputedStyle(document.documentElement).getPropertyValue('--green').trim());
 check(label+' version dot unified ok color',/is-ok/.test(ver.dotCls)&&ver.dotW===7&&ver.dotH===7,ver);
 check(label+' dot and version share baseline',ver.dotCy===ver.latestCy,ver);
 check(label+' status block hugs right edge',ver.sideGap<=1&&ver.dotAfterVersion,ver);
 check(label+' dot uses green token',ver.dotColor.replace(/\s/g,'')===('rgb('+rgb(green)+')'),{dot:ver.dotColor,green});
 // 4) 方向语义色：绿待上传 / 蓝待下载、双向 / 灰已一致 / 琥珀未完成
 for(const [state,cls,text] of [['pending-upload','is-ok','待上传'],['pending-download','is-info','待下载'],['bidirectional','is-info','双向更新'],['synced','is-idle','已一致'],['partial','is-warn','未完成']]){
  await p.evaluate(s=>window.renderWebdav(s),state);
  const got=await p.evaluate(()=>({cls:document.querySelector('#syncDirection').className,text:document.querySelector('#syncDirectionText').textContent,color:getComputedStyle(document.querySelector('#syncDirectionText')).color,dot:getComputedStyle(document.querySelector('#syncDirection')).backgroundColor,rows:document.querySelectorAll('#card-overview-sync .mod-status>span').length}));
  check(label+' webdav '+state,got.cls.includes(cls)&&got.text===text&&got.rows===2,got);
  if(effects==='high'&&size.w===1180){
   const token={ok:'--green',info:'--blue',idle:'--muted',warn:'--amber'}[cls.slice(3)];
   const want=await p.evaluate(t=>getComputedStyle(document.documentElement).getPropertyValue(t).trim(),token);
   check(label+' webdav '+state+' uses token',got.dot.replace(/\s/g,'')===('rgb('+rgb(want)+')'),{got:got.dot,want});
  }
 }
 if(effects==='high')await p.locator('.window').screenshot({path:path.join(out,label+'-overview.png')});
 await p.close();
}
check('no page errors',errors.length===0,errors);
}finally{await b.close();fs.writeFileSync(path.join(out,'status-color-report.json'),JSON.stringify({engine:e.name,checks,errors,failed:checks.filter(x=>!x.passed).length,scope:'overview status typography + unified semantic colors, prototype only'},null,2));}
console.log(JSON.stringify({engine:e.name,checks:checks.length,errors}));
