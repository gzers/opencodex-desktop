// Actual installed WebView2 route handlers; no DEV store or intercepted IPC.
import assert from 'node:assert/strict'
import {writeFile} from 'node:fs/promises'
import {join,win32} from 'node:path'
function readablePrefix(value){
 const native=win32.normalize(value)
 const ordinary=/^\\\\\?\\UNC\\/i.test(native)?'\\\\'+native.slice(8):/^\\\\\?\\[a-z]:\\/i.test(native)?native.slice(4):native
 return ordinary.length<260?ordinary:native
}
export async function verifyWindowsInstallUi(call,output,install=false){
 async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
 async function wait(expression){for(let n=0;n<100;n++){const r=await evaluate(expression);if(r)return r;await new Promise(resolve=>setTimeout(resolve,100))}throw Error('native install UI wait: '+expression)}
 async function click(text,selector){const ok=await evaluate("(()=>{const root=document.querySelector("+JSON.stringify(selector)+");const b=[...root?.querySelectorAll('button')??[]].find(e=>(e.textContent.trim()==="+JSON.stringify(text)+"||e.getAttribute('aria-label')==="+JSON.stringify(text)+")&&!e.disabled&&e.getClientRects().length);if(!b)return false;b.click();return true})()");assert(ok,'native install action missing: '+text)}
 await click('概览','.side');await wait("!!document.querySelector('.ov')");
 const prerequisites=await evaluate("window.__TAURI_INTERNALS__.invoke('discover_environment',{request:null})");
 const initialSource=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");
 let overviewInstaller=false;
 if(prerequisites.gate==='missing_ocx'&&initialSource.kind==='unresolved'){
  await wait("!!document.querySelector('[data-testid=environment-gate] [data-testid=env-install-actions]')");
  await click('安装 OpenCodex','[data-testid=environment-gate]');await wait("!!document.querySelector('#wiz-prefix')");
  assert.equal(await evaluate("location.hash.startsWith('#overview')"),true,'overview install must not navigate to settings');
  assert.equal(await evaluate("[...document.querySelectorAll('.wiz-steps li')].map(e=>e.textContent.trim()).every((v,i)=>v.includes(['安装位置','安装源','确认','安装进度'][i]))"),true);
  await click('取消','[data-testid=runtime-install-modal]');await wait("!document.querySelector('[data-testid=runtime-install-modal]')");
  assert.equal(await evaluate("location.hash.startsWith('#overview')"),true);overviewInstaller=true;
 }
 await click('设置','.side');await wait("!!document.querySelector('.settings-tabs')");
 await click('安装配置','.route-section.active');await wait("!!document.querySelector('[data-testid=runtime-install]')");
 const source=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");
 await click('安装 OpenCodex','.route-section.active');await wait("!!document.querySelector('#wiz-prefix')");
 const initial=await evaluate("(()=>{const d=document.querySelector('[data-testid=runtime-install-modal]');return {value:d.querySelector('#wiz-prefix').value,nextDisabled:[...d.querySelectorAll('button')].find(e=>e.textContent.trim()==='开始安装').disabled,error:d.querySelector('.wiz-error')?.textContent||'',hint:d.querySelector('.wiz-hint')?.textContent||''}})()");
 assert.equal(initial.value,readablePrefix(source.managedPrefix));assert.equal(initial.nextDisabled,true,'explicit confirmation is required before installation');assert.equal(initial.error,'');assert.equal(win32.isAbsolute(initial.value),true);
 await evaluate("(()=>{const i=document.querySelector('#wiz-prefix');i.value='relative';i.dispatchEvent(new Event('input',{bubbles:true}));return true})()");
 await wait("[...document.querySelectorAll('[data-testid=runtime-install-modal] button')].find(e=>e.textContent.trim()==='开始安装').disabled");
 await click('恢复默认','[data-testid=runtime-install-modal]');
 await wait("document.querySelector('#wiz-prefix').value==="+JSON.stringify(readablePrefix(source.defaultPrefix)));
 await evaluate("document.querySelector('[data-testid=wiz-ack]').click()");
 await wait("![...document.querySelectorAll('[data-testid=runtime-install-modal] button')].find(e=>e.textContent.trim()==='开始安装').disabled");

 assert.equal(await evaluate("document.querySelector('[data-testid=runtime-install-modal]').textContent.includes('联网安装')"),true);
 await click('取消','[data-testid=runtime-install-modal]');await wait("!document.querySelector('[data-testid=runtime-install-modal]')");
 const after=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");for(const field of ['kind','path','managedPrefix','defaultPrefix','history'])assert.deepEqual(after[field],source[field],'wizard cancel changed '+field);
 if(install){
  await click('安装 OpenCodex','.route-section.active');await wait("!!document.querySelector('#wiz-prefix')");

  await evaluate("(()=>{document.querySelector('[data-testid=wiz-ack]').click();return true})()");
  await wait("!!document.querySelector('[data-testid=runtime-install-modal] input[aria-label=\"版本\"]')");
  await evaluate("(()=>{const i=document.querySelector('[data-testid=runtime-install-modal] input[aria-label=\"版本\"]');i.value='2.91.0';i.dispatchEvent(new Event('input',{bubbles:true}));return true})()");
  await click('开始安装','[data-testid=runtime-install-modal]');
  let outcome,explicitScriptRetry=false
  const deadline=Date.now()+300000
  while(Date.now()<deadline){
   outcome=await evaluate("(()=>{const d=document.querySelector('[data-testid=runtime-install-modal]');return {status:d?.querySelector('[data-testid=wiz-status]')?.textContent||'',error:d?.querySelector('[data-testid=wiz-error]')?.textContent||'',done:!!d?.querySelector('[data-testid=wiz-outcome]'),retryEnabled:!!d?.querySelector('[data-testid=wiz-allow-scripts]:not(:disabled)')}})()");
   if(outcome.done)break
   if(outcome.error){assert(outcome.retryEnabled&&!explicitScriptRetry,'native install failed: '+JSON.stringify(outcome));explicitScriptRetry=true;await click('允许安装脚本并重试','[data-testid=runtime-install-modal]')}
   await new Promise(resolve=>setTimeout(resolve,500))
  }
  assert(outcome?.done,'real native install did not finish: '+JSON.stringify(outcome))
  const committed=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");assert.equal(committed.kind,'managed');assert.equal(committed.version,'2.91.0');assert.equal(readablePrefix(committed.managedPrefix).toLowerCase(),readablePrefix(source.defaultPrefix).toLowerCase())
  await writeFile(join(output,'runtime-managed-ui-live.json'),JSON.stringify({result:'pass',version:committed.version,source:committed.kind,realNpm:true,actualProductionVueHandlers:true,defaultPrefixUnchanged:true,explicitScriptRetry,sourceRecordReadback:true,generationRequests:0},null,2)+'\n')
  await click('完成','[data-testid=runtime-install-modal]');await wait("!document.querySelector('[data-testid=runtime-install-modal]')")
 }
 await writeFile(join(output,'runtime-install-ui.json'),JSON.stringify({result:'pass',hostDefaultValue:true,noManualCorrection:true,restoreDefault:true,invalidRelativeBlocked:true,configurationReady:true,cancelSourceUnchanged:true,overviewInstaller,installsExecuted:install?1:0,method:'real installed Vue handlers and unmodified Tauri IPC; live install when requested, additional backend upgrade/offline evidence in managed-live.json'},null,2)+'\n');
}
