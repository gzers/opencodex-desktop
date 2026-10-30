import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-022-model-selector/apps/desktop/ui/qa/engine.mjs';
const root='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main',entry=root+'/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html',out=process.env.AUDIT_OUT_DIR;
fs.mkdirSync(out,{recursive:true});
const e=await pickEngine(),b=await e.launch(),errors=[],checks=[];
const check=(name,ok,detail)=>{checks.push({name,passed:!!ok,detail});assert(ok,name+' '+JSON.stringify(detail));};
try{
for(const size of [{w:1180,h:760,theme:'light'},{w:900,h:600,theme:'dark'}])for(const effects of ['high','mid','low']){
 const label=size.w+'-'+size.theme+'-'+effects;
 const p=await b.newPage({viewport:{width:size.w+470,height:size.h+220}});
 p.on('pageerror',x=>errors.push({case:label,message:x.message,stack:x.stack}));
 await p.route('http://**/*',r=>r.abort());await p.route('https://**/*',r=>r.abort());
 await p.goto('file://'+encodeURI(entry)+'#overview');await p.waitForFunction(()=>window.syncPrototype&&window.renderWebdav);
 await p.evaluate(({size,effects})=>{const w=document.querySelector('.window');w.style.width=size.w+'px';w.style.height=size.h+'px';w.style.flex='none';window.applyTheme?.(size.theme);document.documentElement.dataset.theme=size.theme;document.documentElement.dataset.effects=effects;const main=document.querySelector('.main');main.scrollTop=0;}, {size,effects});
 // 三卡不写固定规则文案
 const migrationText=await p.locator('#card-overview-migration').innerText();
 check(label+' file card shows recent facts',/最近导入/.test(migrationText)&&/最近导出/.test(migrationText)&&!/可选对象|独立选择|选择同步范围/.test(migrationText),migrationText);
 check(label+' import/export buttons',await p.locator('#card-overview-migration .mod-actions button').evaluateAll(l=>l.map(x=>x.textContent.trim())).then(x=>x.join('|'))==='导入|导出');
 // 版本行：未检查 -> 检查后 当前→最新
 const before=await p.locator('#card-overview-upgrade .mod-status').innerHTML();
 check(label+' version rows unchecked dot',before.includes('mnt-dot is-unchecked')&&before.includes('尚未检查'));
 await p.locator('[data-maint-action="check-all"]').click();
 await p.waitForTimeout(1200);
 await p.locator('#modalCancel').click();
 await p.waitForFunction(()=>document.querySelector('#card-overview-upgrade .mod-status').innerHTML.includes('mnt-dot is-update'),null,{timeout:15000});
 const rows=await p.locator('#card-overview-upgrade .mod-status .mnt-version-row').evaluateAll(list=>list.map(row=>({text:row.innerText,pair:row.querySelector('.mnt-version-latest')?.textContent,dot:row.querySelector('.mnt-dot')?.className})));
 check(label+' version row shows latest on right',rows.length===2&&rows.every(r=>/^\d+\.\d+\.\d+$/.test((r.pair||'').trim()))&&rows.every(r=>/update/.test(r.dot)),rows);
 const rowRects=await p.locator('#card-overview-upgrade .mnt-version-row').evaluateAll(list=>list.map(row=>{const n=row.firstChild;const v=row.querySelector('.mnt-version-latest').getBoundingClientRect();const name=row.getBoundingClientRect();return {versionRight:Math.round(v.right),rowRight:Math.round(name.right),nameLeft:Math.round(name.left)};}));
 check(label+' version sits at right edge',rowRects.every(r=>r.rowRight-r.versionRight<40),rowRects);
 const dotColor=await p.locator('#card-overview-upgrade .mnt-dot.is-update').first().evaluate(n=>getComputedStyle(n).backgroundColor);
 const green=await p.evaluate(()=>getComputedStyle(document.documentElement).getPropertyValue('--green').trim());
 check(label+' update dot uses green token',dotColor!=='rgba(0, 0, 0, 0)',{dotColor,green});
 // WebDAV 方向色
 const cases=[['pending-upload','待上传','is-up'],['pending-download','待下载','is-down'],['bidirectional','双向更新','is-both'],['synced','已一致','is-same'],['partial','未完成','is-partial']];
 for(const [state,text,cls] of cases){
  await p.evaluate(s=>window.renderWebdav(s),state);
  const got=await p.evaluate(()=>{const side=document.querySelector('#syncRemoteTime').closest('.ov-sync-side');return {text:document.querySelector('#syncDirectionText').textContent,cls:document.querySelector('#syncDirection').className,remote:document.querySelector('#syncRemoteTime').textContent,actions:[...document.querySelectorAll('#syncActions button')].map(b=>b.textContent.trim()),rows:document.querySelectorAll('#card-overview-sync .mod-status>span').length,sameLine:!!side&&side.contains(document.querySelector('#syncDirection'))&&getComputedStyle(side).display.includes('flex')};});
  check(label+' webdav '+state,got.text===text&&got.cls.includes(cls)&&got.actions[0]==='同步'&&got.rows===2&&got.sameLine,got);
  if(['pending-upload','pending-download','synced'].includes(state)){
   const color=await p.locator('#syncDirection i').evaluate(n=>getComputedStyle(n).backgroundColor);
   check(label+' webdav '+state+' colored dot',state==='synced'||color!=='rgb(110, 110, 110)',{color});
  }
 }
 await p.evaluate(()=>window.renderWebdav('bidirectional'));
 const cardText=await p.locator('#card-overview-sync').innerText();
 check(label+' webdav card concise',!/较新覆盖|更新规则/.test(cardText)&&/连接状态/.test(cardText)&&/线上版本/.test(cardText),cardText);
 // 卡片不溢出、动作按钮可点
 const bounds=await p.locator('#mods').evaluate(n=>{const win=document.querySelector('.window').getBoundingClientRect();const cards=[...n.children].map(c=>c.getBoundingClientRect());return {right:Math.max(...cards.map(c=>c.right)),windowRight:win.right,cardsWidth:cards.map(c=>Math.round(c.width))};});
 check(label+' cards fit window',bounds.right<=bounds.windowRight+1,bounds);
 check(label+' cards equal width',Math.max(...bounds.cardsWidth)-Math.min(...bounds.cardsWidth)<=2,bounds.cardsWidth);
 if(effects==='high')await p.locator('.window').screenshot({path:path.join(out,label+'-overview.png')});
 // 同步页工具栏
 await p.evaluate(()=>window.syncPrototype.open({tab:'webdav'}));
 const toolbar=await p.evaluate(()=>({preset:!!document.querySelector('[data-sx-preset]'),clear:!!document.querySelector('[data-sx=clear]'),search:!!document.querySelector('#sxSearch'),all:!!document.querySelector('[data-sx-all]')}));
 check(label+' toolbar simplified',!toolbar.preset&&!toolbar.clear&&toolbar.search&&toolbar.all,toolbar);
 const sxBounds=await p.locator('#syncContent').evaluate(n=>{const win=document.querySelector('.window').getBoundingClientRect();return {right:n.getBoundingClientRect().right,windowRight:win.right,overflow:document.querySelector('.main').scrollHeight-document.querySelector('.main').clientHeight};});
 check(label+' sync page fits',sxBounds.right<=sxBounds.windowRight+1&&sxBounds.overflow<=1,sxBounds);
 if(effects==='high')await p.locator('.window').screenshot({path:path.join(out,label+'-sync.png')});
 await p.close();
}
check('no page errors',errors.length===0,errors);
}finally{await b.close();fs.writeFileSync(path.join(out,'browser-report.json'),JSON.stringify({engine:e.name,checks,errors,failed:checks.filter(x=>!x.passed).length,scope:'prototype overview cards, version rows and scope toolbar only'},null,2));}
console.log(JSON.stringify({engine:e.name,checks:checks.length,errors}));

