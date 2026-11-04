// Ordinary WebView acceptance helpers. Legacy hosts are admitted only for
// their exact app_about version; the 021 selectors always name an open dialog.
import assert from 'node:assert/strict'
import {nativeModelUiContract} from './windows-model-ui.mjs'

export function updateCenterHost(version){
 const {publicDialogs}=nativeModelUiContract(version)
 return publicDialogs
  ? {publicDialogs,selector:'dialog.update-center[open][data-testid=update-center]',panel:'dialog.update-center[open][data-testid=update-center]',closeRoot:'dialog.update-center[open][data-testid=update-center] > .ui-dialog-header',closeButton:'.ui-dialog-header .modal-close',closeLabel:'关闭'}
  : {publicDialogs,selector:'[data-testid=update-center]',panel:'[data-testid=update-center] .update-center',closeRoot:'[data-testid=update-center]',closeButton:'.update-center-foot button',closeLabel:'关闭更新中心'}
}
export function readUpdateCenterScene(host){
 const d=document.querySelector(host.panel),buttons=d?.querySelectorAll(host.closeButton)??[],b=buttons[0]
 if(!d||!b)return null
 const r=d.getBoundingClientRect(),a=b.getBoundingClientRect()
 return {native:d instanceof HTMLDialogElement,open:d instanceof HTMLDialogElement?d.open:true,closeCount:buttons.length,closeEnabled:!b.disabled,
  buttonHeight:a.height,buttonX:a.x,buttonRight:a.right,buttonBottom:a.bottom,x:r.x,y:r.y,right:r.right,bottom:r.bottom,
  viewportWidth:innerWidth,viewportHeight:innerHeight,zoom:getComputedStyle(d).zoom}
}
export function updateCenterSceneExpression(version){return '('+readUpdateCenterScene.toString()+')('+JSON.stringify(updateCenterHost(version))+')'}
export async function closeNativeUpdateCenter({wait,click},version){
 const host=updateCenterHost(version)
 const scene=await wait('('+updateCenterSceneExpression(version)+')')
 assert(scene.open&&scene.closeEnabled,'Update center close action is unavailable')
 if(host.publicDialogs)assert(scene.native&&scene.closeCount===1,'021 update observation must have one public header X')
 await click(host.closeLabel,host.closeRoot)
 await wait('!document.querySelector('+JSON.stringify(host.selector)+')')
 return {version,selector:host.selector,exit:host.publicDialogs?'public-header-X':host.closeLabel}
}
export function nativeUiFaultSelector(version){return nativeModelUiContract(version).publicDialogs?'dialog.ui-fault-dialog[open]':'.ui-fault-modal'}
export async function assertNoNativeUiFault({evaluate},version){
 const selector=nativeUiFaultSelector(version)
 const fault=await evaluate('(()=>{const d=document.querySelector('+JSON.stringify(selector)+');return {present:!!d,title:d?.querySelector("h2")?.textContent??null}})()')
 assert.equal(fault.present,false,'A real UI fault must not be hidden or dismissed to pass native acceptance: '+JSON.stringify(fault))
 return {selector,...fault}
}
