// Actual installed WebView2 route handlers; no DEV store or intercepted IPC.
import assert from 'node:assert/strict'
import {writeFile} from 'node:fs/promises'
import {join,win32} from 'node:path'
export async function verifyWindowsInstallUi(call,output){
 async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
 async function wait(expression){for(let n=0;n<100;n++){const r=await evaluate(expression);if(r)return r;await new Promise(resolve=>setTimeout(resolve,100))}throw Error('native install UI wait: '+expression)}
 async function click(text,selector){const ok=await evaluate("(()=>{const root=document.querySelector("+JSON.stringify(selector)+");const b=[...root?.querySelectorAll('button')??[]].find(e=>(e.textContent.trim()==="+JSON.stringify(text)+"||e.getAttribute('aria-label')==="+JSON.stringify(text)+")&&!e.disabled&&e.getClientRects().length);if(!b)return false;b.click();return true})()");assert(ok,'native install action missing: '+text)}
 await click('设置','.side');await wait("!!document.querySelector('.settings-tabs')");
 await click('安装配置','.route-section.active');await wait("!!document.querySelector('[data-testid=runtime-install]')");
 const source=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");
 await click('安装 OpenCodex','.route-section.active');await wait("!!document.querySelector('#wiz-prefix')");
 const initial=await evaluate("(()=>{const d=document.querySelector('[data-testid=runtime-install-modal]');return {value:d.querySelector('#wiz-prefix').value,nextDisabled:[...d.querySelectorAll('button')].find(e=>e.textContent.trim()==='下一步').disabled,error:d.querySelector('.wiz-error')?.textContent||'',hint:d.querySelector('.wiz-hint')?.textContent||''}})()");
 assert.equal(initial.value,source.managedPrefix);assert.equal(initial.nextDisabled,false,'host default path must pass without manual input');assert.equal(initial.error,'');assert.equal(win32.isAbsolute(initial.value),true);
 await evaluate("(()=>{const i=document.querySelector('#wiz-prefix');i.value='relative';i.dispatchEvent(new Event('input',{bubbles:true}));return true})()");
 await wait("[...document.querySelectorAll('[data-testid=runtime-install-modal] button')].find(e=>e.textContent.trim()==='下一步').disabled");
 await click('恢复默认','[data-testid=runtime-install-modal]');
 await wait("document.querySelector('#wiz-prefix').value==="+JSON.stringify(source.defaultPrefix));
 assert.equal(await evaluate("[...document.querySelectorAll('[data-testid=runtime-install-modal] button')].find(e=>e.textContent.trim()==='下一步').disabled"),false);
 await click('下一步','[data-testid=runtime-install-modal]');await wait("!document.querySelector('#wiz-prefix')");
 assert.equal(await evaluate("document.querySelector('[data-testid=runtime-install-modal]').textContent.includes('联网安装')"),true);
 await click('取消','[data-testid=runtime-install-modal]');await wait("!document.querySelector('[data-testid=runtime-install-modal]')");
 const after=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')");for(const field of ['kind','path','managedPrefix','defaultPrefix','history'])assert.deepEqual(after[field],source[field],'wizard cancel changed '+field);
 await writeFile(join(output,'runtime-install-ui.json'),JSON.stringify({result:'pass',hostDefaultValue:true,noManualCorrection:true,restoreDefault:true,invalidRelativeBlocked:true,nextStep:true,cancelSourceUnchanged:true,installsExecuted:0,method:'real installed Vue handlers and unmodified Tauri IPC; actual installs verified in managed-live.json'},null,2)+'\n');
}
