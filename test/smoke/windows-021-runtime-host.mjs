// Completion is a production task/DOM fact, independent of a legacy footer.
import assert from 'node:assert/strict'
import {nativeModelUiContract} from './windows-model-ui.mjs'

export function runtimeInstallHost(version){
 const {publicDialogs}=nativeModelUiContract(version)
 return {publicDialogs,selector:publicDialogs?'dialog.runtime-install-dialog[open][data-testid=runtime-install-modal]':'[data-testid=runtime-install-modal]'}
}
export function runtimeInstallDomExpression(version,fn,...args){
 return '('+fn.toString()+')('+[runtimeInstallHost(version).selector,...args].map(value=>JSON.stringify(value)).join(',')+')'
}
// Serialized into the real WebView. This only reads mounted production state.
export function readRuntimeInstallScene(publicDialogs){
 const selector=publicDialogs?'dialog.runtime-install-dialog[open][data-testid=runtime-install-modal]':'[data-testid=runtime-install-modal]'
 const d=document.querySelector(selector),runtime=document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?.state.value.runtime
 const status=d?.querySelector('[data-testid=wiz-status]')?.textContent??'',error=d?.querySelector('[data-testid=wiz-error]')?.textContent??'',outcome=d?.querySelector('[data-testid=wiz-outcome]')?.textContent?.trim()??''
 const x=d?.querySelectorAll('.ui-dialog-header .modal-close')??[],retry=d?.querySelector('[data-testid=wiz-allow-scripts]:not(:disabled)')
 const phase=runtime?.install?.phase??null,installing=runtime?.installing??null
 const progress=d?.querySelector('[role=progressbar]')?.getAttribute('aria-valuenow')??null
 const legacyFinish=!publicDialogs&&!!d?.querySelector('[data-testid=wiz-finish]:not(:disabled)')
 const done=!!outcome&&status.includes('安装完成')&&!error&&(publicDialogs?phase==='done'&&installing===false&&!!runtime?.installOutcome&&!runtime?.installError&&progress==='100':legacyFinish)
 return {present:!!d,native:!!d&&d instanceof HTMLDialogElement,status,error,outcome,done,phase,installing,progress,
  taskOutcome:!!runtime?.installOutcome,taskVersion:runtime?.installOutcome?.version??null,taskError:runtime?.installError??'',
  closeCount:x.length,closeEnabled:!!x[0]&&!x[0].disabled,footerCount:d?.querySelectorAll('footer').length??0,
  legacyFinish,retryEnabled:!!retry,scriptsEnabled:!!retry,console:d?.querySelector('[data-testid=wiz-console]')?.textContent??'',offlineSteps:!!d?.querySelector('[data-testid=wiz-substeps]')}
}
export function runtimeInstallExpression(version){
 return '('+readRuntimeInstallScene.toString()+')('+JSON.stringify(runtimeInstallHost(version).publicDialogs)+')'
}
export function runtimeInstallTerminalExpression(version,terminal){
 const {publicDialogs}=runtimeInstallHost(version)
 assert(['done','failed'].includes(terminal),'Unknown native install terminal')
 const condition=terminal==='done'?'scene.done':"scene.error&&!scene.done"+(publicDialogs?"&&scene.phase==='failed'&&scene.installing===false":'')
 return '(()=>{const scene='+runtimeInstallExpression(version)+';return '+condition+'?scene:false})()'
}
export function assertRuntimeInstallTerminal(scene,version,terminal){
 const {publicDialogs}=runtimeInstallHost(version)
 assert(scene.present,'Native install observation is missing')
 if(terminal==='done')assert(scene.done&&scene.outcome&&!scene.error,'Native install has not actually completed')
 else if(terminal==='failed'){
  assert(scene.error&&!scene.done,'Native install has no real terminal failure')
  if(publicDialogs)assert(scene.phase==='failed'&&scene.installing===false,'Native install failure task is not settled')
 }else assert.fail('Unknown native install terminal: '+terminal)
 if(publicDialogs){
  assert(scene.native&&scene.closeCount===1&&scene.closeEnabled,'021 terminal install needs its enabled public header X')
  if(terminal==='done'){
   assert.equal(scene.phase,'done');assert.equal(scene.installing,false);assert(scene.taskOutcome&&!scene.taskError);assert.equal(scene.progress,'100')
   assert.equal(scene.footerCount,0,'021 completed install must have no footer')
  }
 }
}
export async function closeRuntimeInstallTerminal({wait,click},version,terminal){
 const scene=await wait(runtimeInstallTerminalExpression(version,terminal))
 assertRuntimeInstallTerminal(scene,version,terminal)
 const host=runtimeInstallHost(version),label=host.publicDialogs?'关闭':terminal==='done'?'完成':'关闭'
 await click(label,host.selector+(host.publicDialogs?' > .ui-dialog-header':''))
 await wait('!document.querySelector('+JSON.stringify(host.selector)+')')
 return {version,terminal,selector:host.selector,exit:host.publicDialogs?'public-header-X':label,phase:scene.phase,outcomePresent:!!scene.outcome,progress:scene.progress}
}
