import {test} from 'node:test'
import assert from 'node:assert/strict'
import {runInNewContext} from 'node:vm'
import {readFile} from 'node:fs/promises'
import {updateCenterHost,updateCenterSceneExpression,closeNativeUpdateCenter,nativeUiFaultSelector,assertNoNativeUiFault} from './windows-021-public-hosts.mjs'
import {runtimeInstallDomExpression,runtimeInstallHost} from './windows-021-runtime-host.mjs'

function updateFixture(version,change={}){
 const host=updateCenterHost(version),state={open:true,closeEnabled:true,closeCount:1,scale:1,...change},clicks=[],queries=[]
 class NativeDialog {}
 const close={get disabled(){return !state.closeEnabled},getBoundingClientRect:()=>({height:32*state.scale,x:720*state.scale,right:752*state.scale,bottom:64*state.scale})}
 const panel=host.publicDialogs?new NativeDialog():{}
 Object.defineProperty(panel,'open',{get:()=>state.open})
 panel.querySelectorAll=selector=>{assert.equal(selector,host.closeButton);return Array(state.closeCount).fill(close)}
 panel.getBoundingClientRect=()=>({x:100,y:100,right:100+780*state.scale,bottom:100+300*state.scale})
 const document={querySelector:selector=>{queries.push(selector);if(!state.open)return null;return selector===host.panel||selector===host.selector?panel:null}}
 const evaluate=expression=>runInNewContext(expression,{document,HTMLDialogElement:NativeDialog,getComputedStyle:()=>({zoom:String(state.scale)}),innerWidth:2000,innerHeight:1200})
 const driver={wait:async expression=>{const result=evaluate(expression);assert(result,'update host wait');return result},click:async(label,root)=>{clicks.push({label,root});assert.equal(root,host.closeRoot);assert.equal(label,host.closeLabel);state.open=false}}
 return {state,host,queries,clicks,evaluate,driver,panel}
}
for(const version of ['0.2.20','0.2.21'])test(version+' update center closes its own real host and waits for the open layer to end',async()=>{
 const fixture=updateFixture(version)
 const scene=fixture.evaluate(updateCenterSceneExpression(version));assert(scene.open&&scene.closeEnabled)
 assert.equal(scene.native,version==='0.2.21')
 const receipt=await closeNativeUpdateCenter(fixture.driver,version)
 assert.equal(receipt.selector,fixture.host.selector);assert.equal(fixture.clicks.length,1)
 assert.equal(fixture.state.open,false)
})
test('021 update geometry uses native header X at 100/150/200 scale without any footer dependency',()=>{
 const fixture=updateFixture('0.2.21'),samples=[]
 for(const scale of [1,1.5,2]){fixture.state.scale=scale;samples.push(fixture.evaluate(updateCenterSceneExpression('0.2.21')))}
 assert.deepEqual(samples.map(s=>s.buttonHeight),[32,48,64]);assert(samples.every(s=>s.native&&s.closeCount===1))
 assert(fixture.queries.every(selector=>selector==='dialog.update-center[open][data-testid=update-center]'))
 assert(!fixture.host.closeButton.includes('foot'))
})
test('021 close refuses absent, duplicate or disabled header X before clicking',async()=>{
 for(const change of [{closeCount:0},{closeCount:2},{closeEnabled:false},{open:false}]){
  const fixture=updateFixture('0.2.21',change)
  await assert.rejects(closeNativeUpdateCenter(fixture.driver,'0.2.21'));assert.deepEqual(fixture.clicks,[])
 }
})
test('021 update selectors cannot accidentally sample a retained hidden legacy panel',async()=>{
 const fixture=updateFixture('0.2.21',{open:false})
 assert.equal(fixture.evaluate(updateCenterSceneExpression('0.2.21')),null)
 await assert.rejects(closeNativeUpdateCenter(fixture.driver,'0.2.21'));assert.deepEqual(fixture.clicks,[])
})
for(const version of ['0.2.20','0.2.21'])test(version+' UI fault gate finds the real versioned host and never dismisses it',async()=>{
 const selectors=[],selector=nativeUiFaultSelector(version)
 for(const present of [false,true]){
  const driver={evaluate:async expression=>runInNewContext(expression,{document:{querySelector:actual=>{selectors.push(actual);assert.equal(actual,selector);return present?{querySelector:()=>({textContent:'界面出现异常'})}:null}}})}
  if(present)await assert.rejects(assertNoNativeUiFault(driver,version),/real UI fault/)
  else assert.equal((await assertNoNativeUiFault(driver,version)).present,false)
 }
 assert.equal(selectors.length,2)
})
test('021 runtime DOM helpers wait on open dialog descendants, ignoring retained closed wizard inputs',()=>{
 const version='0.2.21',queries=[],open=runtimeInstallHost(version).selector
 const expression=runtimeInstallDomExpression(version,root=>!!document.querySelector(root+' #wiz-prefix'))
 const document={querySelector:selector=>{queries.push(selector);return selector==='#wiz-prefix'?{value:'stale-hidden'}:null}}
 assert.equal(runInNewContext(expression,{document}),false);assert.deepEqual(queries,[open+' #wiz-prefix'])
})
test('query/OTA/offline acceptance callers reach versioned host helpers, leaving signed OTA business unchanged',async()=>{
 const query=await readFile(new URL('./windows-query-ui.mjs',import.meta.url),'utf8'),ota=await readFile(new URL('./windows-ota.mjs',import.meta.url),'utf8'),drop=await readFile(new URL('./windows-offline-drop.mjs',import.meta.url),'utf8')
 assert.equal(query.split('await closeNativeUpdateCenter({wait,click},about.version)').length-1,5)
 assert(ota.includes('evaluate(updateCenterSceneExpression(facts.about.version))'));assert(ota.includes('await closeNativeUpdateCenter({'))
 for(const source of [query,ota]){assert(!source.includes('关闭更新中心'));assert(!source.includes('.update-center-foot'));assert(!source.includes('[data-testid=update-center] .update-center'))}
 assert(ota.includes("await invoke('install_update'"));assert(ota.includes('status.signatureVerified,true'));assert(ota.includes('prior.configSha256'))
 assert(drop.includes('assertNoNativeUiFault({evaluate},about.version)'));assert(!drop.includes('.ui-fault-modal'))
 assert(!drop.includes("!document.querySelector('[data-testid=runtime-install-modal]')"))
})
