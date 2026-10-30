import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-022-model-selector/apps/desktop/ui/qa/engine.mjs';
const entry='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html';
const out=process.env.AUDIT_OUT_DIR;fs.mkdirSync(out,{recursive:true});const e=await pickEngine(),browser=await e.launch(),errors=[],checks=[];
const check=(name,ok,detail)=>{checks.push({name,passed:!!ok,detail});assert(ok,name+' '+JSON.stringify(detail));};
try{
for(const size of [{w:1180,h:760,theme:'light'},{w:900,h:600,theme:'dark'}])for(const effects of ['high','mid','low']){
 const label=size.w+'-'+size.theme+'-'+effects,page=await browser.newPage({viewport:{width:size.w+470,height:size.h+220}});
 page.on('pageerror',err=>errors.push({case:label,message:err.message,stack:err.stack}));await page.route('http://**/*',route=>route.abort());await page.route('https://**/*',route=>route.abort());
 await page.goto('file://'+encodeURI(entry)+'#sync?tab=webdav');await page.waitForFunction(()=>!!window.syncPrototype);
 await page.evaluate(({size,effects})=>{const root=document.documentElement,win=document.querySelector('.window');win.style.width=size.w+'px';win.style.height=size.h+'px';win.style.flex='none';root.dataset.theme=size.theme;window.applyTheme?.(size.theme);root.dataset.effects=effects;window.syncPrototype.open({tab:'webdav',scope:['channel:newapi','template:gpt','skill:review','mcp:docs']});},{size,effects});
 await page.locator('[data-sx=check]').waitFor();check(label+' single sync action',await page.locator('[data-sx=check]').innerText()==='同步'&&await page.locator('[data-sx=save-scope]').count()===0);
 check(label+' no review checkbox',await page.locator('[data-sx-reviewed]').count()===0);
 const geometry=await page.locator('#syncContent').evaluate(n=>{const r=n.getBoundingClientRect(),win=document.querySelector('.window').getBoundingClientRect();return {width:r.width,height:r.height,right:r.right,windowRight:win.right,overflow:n.scrollWidth-n.clientWidth,bodyOverflow:document.querySelector('.main').scrollHeight-document.querySelector('.main').clientHeight}});check(label+' content fits window',geometry.right<=geometry.windowRight+1&&geometry.bodyOverflow<=1,geometry);
 await page.locator('.window').screenshot({path:path.join(out,label+'-sync-before.png')});
 await page.locator('[data-sx=check]').click();await page.waitForFunction(()=>!window.syncPrototype.state().running&&!!window.syncPrototype.state().result);
 const state=await page.evaluate(()=>window.syncPrototype.state());check(label+' direct two-way',state.result.downloaded>=2&&state.result.uploaded===1&&state.result.failed.length===0&&state.dialogKind==='',state.result);check(label+' model saved already updated',await page.evaluate(()=>window.modelPrototype.state().saved.providers.find(p=>p.id==='newapi').models[0].naming.alias)==='primary-model');
 await page.locator('.window').screenshot({path:path.join(out,label+'-sync-done.png')});
 await page.locator('[data-sx=check]').click();await page.waitForFunction(()=>!window.syncPrototype.state().running);check(label+' repeated sync no reapply',await page.evaluate(()=>window.syncPrototype.state().result.downloaded+window.syncPrototype.state().result.uploaded)===0);
 await page.evaluate(()=>{window.syncPrototype.setScenario('partial');});await page.locator('[data-sx=check]').click();await page.waitForFunction(()=>!window.syncPrototype.state().running);check(label+' independent failure',await page.evaluate(()=>window.syncPrototype.state().result.failed.length)===1);await page.locator('.window').screenshot({path:path.join(out,label+'-sync-partial.png')});
 await page.locator('[data-sx=retry]').click();await page.waitForFunction(()=>!window.syncPrototype.state().running);check(label+' retry failed only',await page.evaluate(()=>window.syncPrototype.state().result.items.length===1&&window.syncPrototype.state().result.failed.length===0));
 await page.evaluate(()=>window.syncPrototype.open({tab:'file',scope:['channel:newapi','skill:review'],action:'import'}));check(label+' import single dialog',await page.locator('#modalTitle').innerText()==='导入配置');await page.locator('[data-sx=choose-import-file]').click();await page.locator('#sxUnlock').fill('demo-sync');
 const fit=await page.locator('#modal').evaluate(n=>{const r=n.getBoundingClientRect(),win=document.querySelector('.window').getBoundingClientRect(),btn=document.querySelector('#modalConfirm').getBoundingClientRect();return {top:r.top-win.top,bottom:win.bottom-r.bottom,buttonBottom:win.bottom-btn.bottom,overflow:n.scrollWidth-n.clientWidth}});check(label+' import fits and actions visible',fit.top>=-1&&fit.bottom>=-1&&fit.buttonBottom>=0&&fit.overflow<=1,fit);await page.locator('#modal').screenshot({path:path.join(out,label+'-import-scope.png')});await page.locator('#modalCancel').click();
 await page.evaluate(()=>{window.syncPrototype.open({tab:'webdav'});window.syncPrototype.render();});check(label+' range remains selected',await page.evaluate(()=>window.syncPrototype.state().selected.includes('channel:newapi')));
 if(effects==='high'){
  await page.evaluate(()=>window.windowPanelMock('load-failed'));check(label+' page fail is not start fail',await page.locator('#panelEmpty strong').innerText()==='官方面板加载失败'&&await page.locator('#panelEmpty span').innerText().then(t=>t.includes('正在运行')));await page.locator('.window').screenshot({path:path.join(out,label+'-panel-load-failed.png')});await page.locator('#panelRetryBtn').click();check(label+' retry loads panel',await page.locator('#panelReal').evaluate(n=>n.classList.contains('show')));
 }
 await page.close();
}
check('no page errors',errors.length===0,errors);
}finally{await browser.close();fs.writeFileSync(path.join(out,'browser-report.json'),JSON.stringify({engine:e.name,checks,errors,failed:checks.filter(x=>!x.passed).length,scope:'prototype only: fixed in-memory data, no filesystem or WebDAV operations'},null,2));}
console.log(JSON.stringify({engine:e.name,passed:checks.length,errors,out}));
