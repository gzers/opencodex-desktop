import fs from 'node:fs';import path from 'node:path';import assert from 'node:assert/strict';
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-022-model-selector/apps/desktop/ui/qa/engine.mjs';
const entry='/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html';
const out=process.env.AUDIT_OUT_DIR;fs.mkdirSync(out,{recursive:true});const engine=await pickEngine(),browser=await engine.launch(),errors=[],checks=[];
const check=(name,passed,detail)=>{checks.push({name,passed:!!passed,detail});assert(passed,name+' '+JSON.stringify(detail));};
try{
 for(const size of [{w:1180,h:760,theme:'light'},{w:900,h:600,theme:'dark'}])for(const effects of ['high','mid','low']){
  const label=size.w+'-'+size.theme+'-'+effects,page=await browser.newPage({viewport:{width:size.w+470,height:size.h+220}});
  page.on('pageerror',e=>errors.push({case:label,message:e.message,stack:e.stack}));await page.route('http://**/*',r=>r.abort());await page.route('https://**/*',r=>r.abort());
  await page.goto('file://'+encodeURI(entry)+'#sync?tab=file');await page.waitForFunction(()=>!!window.syncPrototype);
  await page.evaluate(({size,effects})=>{const win=document.querySelector('.window');win.style.width=size.w+'px';win.style.height=size.h+'px';win.style.flex='none';window.applyTheme?.(size.theme);document.documentElement.dataset.theme=size.theme;document.documentElement.dataset.effects=effects;window.syncPrototype.open({tab:'file',scope:['skill:review']});},{size,effects});
  check(label+' explicit export range',await page.locator('.sx-table-head').innerText().then(t=>t.includes('导出范围'))&&await page.locator('section[aria-label="导出范围"]').count()===1);
  const fit=await page.locator('#syncContent').evaluate(n=>{const win=document.querySelector('.window').getBoundingClientRect();return {windowRight:win.right,contentRight:n.getBoundingClientRect().right,tabsRight:document.querySelector('.sx-tabs').getBoundingClientRect().right,scopeRight:document.querySelector('.sx-scope').getBoundingClientRect().right,secretsRight:document.querySelector('.sx-secrets').getBoundingClientRect().right,bodyOverflow:document.querySelector('.main').scrollHeight-document.querySelector('.main').clientHeight};});check(label+' file page fits',Math.max(fit.contentRight,fit.tabsRight,fit.scopeRight,fit.secretsRight)<=fit.windowRight+1&&fit.bodyOverflow<=1,fit);
  if(effects==='high')await page.locator('.window').screenshot({path:path.join(out,label+'-file-range.png')});
  await page.evaluate(()=>{window.syncPrototype.setImportSample('plaintext');window.syncPrototype.open({tab:'file',action:'import'});});await page.locator('[data-sx=choose-import-file]').click();
  check(label+' plaintext no decryption',!(await page.locator('#sxUnlockField').isVisible()));
  check(label+' plaintext credential choices',await page.locator('[data-sx-import-keys]').isEnabled()&&await page.locator('[data-sx-import-secrets]').isEnabled());
  await page.locator('[data-sx-import-keys]').check();await page.locator('[data-sx-import-secrets]').check();
  const modal=await page.locator('#modal').evaluate(n=>{const r=n.getBoundingClientRect(),win=document.querySelector('.window').getBoundingClientRect(),btn=document.querySelector('#modalConfirm').getBoundingClientRect();return {top:r.top-win.top,bottom:win.bottom-r.bottom,buttonBottom:win.bottom-btn.bottom,overflow:n.scrollWidth-n.clientWidth};});check(label+' shared import fits',modal.top>=-1&&modal.bottom>=-1&&modal.buttonBottom>=0&&modal.overflow<=1,modal);
  if(effects==='high')await page.locator('#modal').screenshot({path:path.join(out,label+'-plaintext-import.png')});
  await page.locator('#modalConfirm').click();await page.waitForFunction(()=>!window.syncPrototype.state().running);
  check(label+' import directly completes',await page.evaluate(()=>window.syncPrototype.state().dialogKind===''&&window.modelPrototype.state().saved.providers.find(p=>p.id==='newapi').connection.credentialRef.startsWith('mock-key:')));
  await page.evaluate(()=>{window.syncPrototype.setImportSample('encrypted');window.syncPrototype.open({tab:'file',action:'import',scope:['channel:newapi']});});await page.locator('[data-sx=choose-import-file]').click();await page.locator('#sxUnlock').fill('demo-sync');await page.locator('[data-sx-import-keys]').check();await page.locator('[data-sx-import-object="channel:newapi"]').uncheck();
  check(label+' unselected credentials disabled without losing password',await page.locator('[data-sx-import-keys]').isDisabled()&&!(await page.locator('[data-sx-import-keys]').isChecked())&&await page.locator('#sxUnlock').inputValue()==='demo-sync');await page.locator('#modalCancel').click();
  await page.evaluate(()=>{window.syncPrototype.setImportSample('keyless');window.syncPrototype.open({tab:'file',action:'import'});});await page.locator('[data-sx=choose-import-file]').click();check(label+' keyless cannot select phantom credentials',await page.locator('[data-sx-import-keys]').isDisabled()&&!(await page.locator('[data-sx-import-keys]').isChecked()));await page.locator('#modalCancel').click();
  await page.evaluate(()=>{window.setRoute('overview');window.renderWebdav('partial');});check(label+' overview uses direct-update rule',await page.locator('#syncTag').innerText()==='部分完成'&&await page.locator('#syncUpdateRule').innerText()==='较新覆盖');if(effects==='high')await page.locator('.window').screenshot({path:path.join(out,label+'-overview-partial.png')});
  await page.evaluate(()=>runNotifScenario(NOTIF_SCEN.findIndex(s=>s[1]==='配置导入')));check(label+' notification demo uses shared import',await page.locator('[data-sx=choose-import-file]').count()===1&&await page.locator('#modalTitle').innerText()==='导入配置');await page.locator('#modalCancel').click();
  check(label+' no cover-confirm demo',await page.evaluate(()=>!NOTIF_SCEN.some(s=>s[1]==='覆盖同步'||s[1]==='WebDAV 冲突')));
  await page.close();
 }
 check('no page errors',errors.length===0,errors);
}finally{await browser.close();fs.writeFileSync(path.join(out,'browser-report.json'),JSON.stringify({engine:engine.name,checks,errors,failed:checks.filter(x=>!x.passed).length,scope:'prototype only: fixed mock credentials/objects, no real files/network/Windows claims'},null,2));}
console.log(JSON.stringify({engine:engine.name,passed:checks.length,errors,out}));
