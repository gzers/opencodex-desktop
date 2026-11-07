// Actual installed WebView2 route handlers; no DEV store or intercepted IPC.
import assert from 'node:assert/strict'
import {verifyWindowsOfflineDrop} from './windows-offline-drop.mjs'
import {writeFile} from 'node:fs/promises'
import {join,win32} from 'node:path'
import {runtimeInstallHost,runtimeInstallDomExpression,runtimeInstallExpression,closeRuntimeInstallTerminal} from './windows-021-runtime-host.mjs'
import {readInitialRuntimeEnvironment} from './windows023-runtime-layout.mjs'
import {assertModelSandbox} from './windows-model-ui.mjs'
function readablePrefix(value){
 const native=win32.normalize(value)
 const ordinary=/^\\\\\?\\UNC\\/i.test(native)?'\\\\'+native.slice(8):/^\\\\\?\\[a-z]:\\/i.test(native)?native.slice(4):native
 return ordinary.length<260?ordinary:native
}
export async function verifyWindowsInstallUi(call,output,install=false,sandbox){
 async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
 async function wait(expression){for(let n=0;n<100;n++){const r=await evaluate(expression);if(r)return r;await new Promise(resolve=>setTimeout(resolve,100))}throw Error('native install UI wait: '+expression)}
 async function click(text,selector){const ok=await evaluate("(()=>{const root=document.querySelector("+JSON.stringify(selector)+");const b=[...root?.querySelectorAll('button')??[]].find(e=>(e.textContent.trim()==="+JSON.stringify(text)+"||e.getAttribute('aria-label')==="+JSON.stringify(text)+")&&!e.disabled&&e.getClientRects().length);if(!b)return false;b.click();return true})()");assert(ok,'native install action missing: '+text)}
 const about=await evaluate("window.__TAURI_INTERNALS__.invoke('app_about')")
 const host=runtimeInstallHost(about.version)
 const initialDataRoot=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")
 const facts={about,dataRoot:initialDataRoot};assertModelSandbox(facts,sandbox)
 const invoke=(command,args={})=>evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+')')
 const driver={call,evaluate,wait,click,invoke}
 const inInstall=(fn,...args)=>runtimeInstallDomExpression(about.version,fn,...args)
 const closed='!document.querySelector('+JSON.stringify(host.selector)+')'
 let installedSource,readyEndpoint,readyBefore,readyAfter
 await click('概览','.side');await wait("!!document.querySelector('.ov')");
 const prerequisites=await evaluate('('+readInitialRuntimeEnvironment.toString()+')(window.__TAURI_INTERNALS__.invoke,'+JSON.stringify(about.version)+')');
 if(about.version==='0.2.23'&&prerequisites.gate==='integrated_runtime_not_ready'){
  assert.equal(prerequisites.readyVerified,false)
  await wait("!!document.querySelector('[data-testid=environment-gate]')")
  await writeFile(join(output,'runtime-initial-unresolved.json'),JSON.stringify({result:'expected-not-ready',discovery:prerequisites,reason:'fresh sandbox deliberately skips automatic bundle provision; real install below is required'},null,2)+'\n')
 }
 const initialSource=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");
 let overviewInstaller=false;
 if(prerequisites.gate==='missing_ocx'&&initialSource.kind==='unresolved'){
  await wait("!!document.querySelector('[data-testid=environment-gate] [data-testid=env-install-actions]')");
  await click('安装 OpenCodex','[data-testid=environment-gate]');await wait(inInstall(root=>!!document.querySelector(root+' #wiz-prefix')));
  assert.equal(await evaluate("location.hash.startsWith('#overview')"),true,'overview install must not navigate to settings');
  assert.equal(await evaluate("[...document.querySelectorAll('.wiz-steps li')].map(e=>e.textContent.trim()).every((v,i)=>v.includes(['安装位置','安装源','确认','安装进度'][i]))"),true);
  await click('取消',host.selector);await wait(closed);
  assert.equal(await evaluate("location.hash.startsWith('#overview')"),true);overviewInstaller=true;
 }
 await click('设置','.side');await wait("!!document.querySelector('.settings-tabs')");
 await click('安装配置','.route-section.active');await wait("!!document.querySelector('[data-testid=runtime-install]')");
 const source=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");
 await click(about.version==='0.2.23'?'安装程序':'安装 OpenCodex','.route-section.active');await wait(inInstall(root=>!!document.querySelector(root+' #wiz-prefix')));
 const initial=await evaluate(inInstall(root=>{const d=document.querySelector(root);return {value:d.querySelector('#wiz-prefix').value,nextDisabled:[...d.querySelectorAll('button')].find(e=>e.textContent.trim()==='开始安装').disabled,error:d.querySelector('.wiz-error')?.textContent||'',hint:d.querySelector('.wiz-hint')?.textContent||''}}));
 assert.equal(initial.value,readablePrefix(source.managedPrefix));assert.equal(initial.nextDisabled,true,'explicit confirmation is required before installation');assert.equal(initial.error,'');assert.equal(win32.isAbsolute(initial.value),true);
 await evaluate(inInstall(root=>{const i=document.querySelector(root+' #wiz-prefix');i.value='relative';i.dispatchEvent(new Event('input',{bubbles:true}));return true}));
 await wait(inInstall(root=>[...document.querySelectorAll(root+' button')].find(e=>e.textContent.trim()==='开始安装').disabled));
 await click('恢复默认',host.selector);
 await wait(inInstall((root,prefix)=>document.querySelector(root+' #wiz-prefix').value===prefix,readablePrefix(source.defaultPrefix)));
 await evaluate(inInstall(root=>document.querySelector(root+' [data-testid=wiz-ack]').click()));
 await wait(inInstall(root=>![...document.querySelectorAll(root+' button')].find(e=>e.textContent.trim()==='开始安装').disabled));

 assert.equal(await evaluate(inInstall(root=>document.querySelector(root).textContent.includes('联网安装'))),true);
 await click('取消',host.selector);await wait(closed);
 const after=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");for(const field of ['kind','path','managedPrefix','defaultPrefix','history'])assert.deepEqual(after[field],source[field],'wizard cancel changed '+field);
 if(install){
  await click(about.version==='0.2.23'?'安装程序':'安装 OpenCodex','.route-section.active');await wait(inInstall(root=>!!document.querySelector(root+' #wiz-prefix')));

  await evaluate(inInstall(root=>{document.querySelector(root+' [data-testid=wiz-ack]').click();return true}));
  await wait(inInstall(root=>!!document.querySelector(root+' input[aria-label="版本"]')));
  await evaluate(inInstall(root=>{const i=document.querySelector(root+' input[aria-label="版本"]');i.value='2.91.0';i.dispatchEvent(new Event('input',{bubbles:true}));return true}));
  await click('开始安装',host.selector);
  let outcome,explicitScriptRetry=false
  const deadline=Date.now()+300000
  while(Date.now()<deadline){
   outcome=await evaluate(runtimeInstallExpression(about.version));
   if(outcome.done)break
   if(outcome.error){assert(outcome.retryEnabled&&!explicitScriptRetry,'native install failed: '+JSON.stringify(outcome));explicitScriptRetry=true;await click('允许安装脚本并重试',host.selector)}
   await new Promise(resolve=>setTimeout(resolve,500))
  }
  assert(outcome?.done,'real native install did not finish: '+JSON.stringify(outcome))
  const committed=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");assert.equal(committed.kind,'managed');assert.equal(committed.version,'2.91.0');assert.equal(readablePrefix(committed.managedPrefix).toLowerCase(),readablePrefix(source.defaultPrefix).toLowerCase())
  installedSource=committed
  await writeFile(join(output,'runtime-managed-ui-live.json'),JSON.stringify({result:'pass',version:committed.version,source:committed.kind,realNpm:true,actualProductionVueHandlers:true,defaultPrefixUnchanged:true,explicitScriptRetry,sourceRecordReadback:true,generationRequests:0},null,2)+'\n')
  const closure=await closeRuntimeInstallTerminal({evaluate,wait,click},about.version,'done')
  await writeFile(join(output,'runtime-managed-ui-host.json'),JSON.stringify({version:about.version,host:host.selector,taskPhase:outcome.phase,taskVersion:outcome.taskVersion,closure},null,2)+'\n')
  if(host.publicDialogs){
   const {prepare021ReadyWebdav,verify021ReadyWebdav}=await import('./windows-021-ready-webdav.mjs')
   readyEndpoint=await prepare021ReadyWebdav(driver,facts,sandbox,false)
   readyBefore=await verify021ReadyWebdav(driver,output,facts,sandbox,readyEndpoint,installedSource,'managed-before-offline-drop')
  }
 }
 const dataRoot=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')");
 assertModelSandbox({about,dataRoot},sandbox)
 await verifyWindowsOfflineDrop(call,output,sandbox);
 if(install&&host.publicDialogs){
  const {verify021ReadyWebdav}=await import('./windows-021-ready-webdav.mjs')
  readyAfter=await verify021ReadyWebdav(driver,output,facts,sandbox,readyEndpoint,installedSource,'managed-after-offline-drop')
 }
 await writeFile(join(output,'runtime-install-ready-webdav.json'),JSON.stringify({required:install&&host.publicDialogs,result:readyBefore&&readyAfter?'pass':'not-requested',reason:install?'version-specific install acceptance':'this path does not perform a real managed installation',before:readyBefore?{stage:readyBefore.stage,readiness:readyBefore.readiness,result:readyBefore.result}:null,after:readyAfter?{stage:readyAfter.stage,readiness:readyAfter.readiness,result:readyAfter.result}:null},null,2)+String.fromCharCode(10))
 await writeFile(join(output,'runtime-install-ui.json'),JSON.stringify({result:'pass',hostDefaultValue:true,noManualCorrection:true,restoreDefault:true,invalidRelativeBlocked:true,configurationReady:true,cancelSourceUnchanged:true,overviewInstaller,installsExecuted:install?1:0,method:'real installed Vue handlers and unmodified Tauri IPC; live install when requested, additional backend upgrade/offline evidence in managed-live.json'},null,2)+'\n');
}
