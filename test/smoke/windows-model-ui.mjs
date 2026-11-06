// Bounded native WebView2 business check for the already-owned startup sandbox.
// Real Tauri IPC and DOM handlers; no DEV store/fixture installation or network.
import assert from 'node:assert/strict'
import {win32,join} from 'node:path'
import {randomUUID} from 'node:crypto'
import {writeFile,readFile,readdir} from 'node:fs/promises'
import {verifyUiAlignment} from './windows-ui-alignment.mjs'
function normalize(value){assert(typeof value==='string'&&win32.isAbsolute(value),'absolute active root required');const canonical=value.startsWith('\\\\?\\UNC\\')?'\\\\'+value.slice(8):value.replace(/^\\\\\?\\/,'');return win32.resolve(canonical).toLowerCase()}
export function assertModelExportPath(path,expected){
 const file=normalize(path),directory=normalize(win32.join(expected,'exports'))
 assert(file.startsWith(directory+win32.sep),'export escaped admitted root')
}
export function parseTaskContent(text){
 const marker=text.indexOf('\n\n{')
 if(marker<0)throw Error('task content JSON boundary missing')
 return JSON.parse(text.slice(marker+2))
}
export function assertModelSandbox(facts,expected){
 assert(facts?.about?.platform==='Windows','Windows native facts required')
 assert(typeof expected==='string'&&win32.isAbsolute(expected),'explicit absolute startup sandbox required')
 assert.equal(normalize(facts.dataRoot.currentDataRoot),normalize(expected),'immutable startup root differs from admitted sandbox')
 assert.equal(normalize(facts.dataRoot.activeDataRoot),normalize(expected),'saved root differs from startup sandbox')
 assert.equal(facts.dataRoot.runtimeActive,true,'startup root is not active')
}
export function modelConfigPath(facts,expected){
 assertModelSandbox(facts,expected)
 const home=win32.join(expected,'opencodex-home')
 assert.equal(facts.dataRoot.opencodexHomeMode,'inside','native model writes require the isolated internal home')
 assert.equal(normalize(facts.dataRoot.currentOpencodexHome),normalize(home),'runtime model home escaped admitted sandbox')
 assert.equal(normalize(facts.dataRoot.opencodexHome),normalize(home),'saved model home escaped admitted sandbox')
 return win32.join(home,'config.json')
}
export function nativeModelKeyboardChecksRequired(version){
 assert(['0.2.16','0.2.17','0.2.18','0.2.19','0.2.20','0.2.21','0.2.22'].includes(version),'Unsupported native model UI acceptance version')
 return ['0.2.20','0.2.21','0.2.22'].includes(version)
}
export function nativeModelUiContract(version){
 nativeModelKeyboardChecksRequired(version)
 const publicDialogs=['0.2.21','0.2.22'].includes(version)
 return {publicDialogs,confirmation:publicDialogs?'dialog.app-modal-dialog[open]':'.modal-mask',close:publicDialogs?'dialog[open] > .ui-dialog-header':'dialog[open]'}
}
export async function confirmNativeTemplateDefault({wait,click},version){
 const {publicDialogs,confirmation}=nativeModelUiContract(version)
 await click('设为默认','dialog[open]')
 await wait(publicDialogs
  ? "(()=>{const layers=[...document.querySelectorAll('dialog[open]')],confirmations=[...document.querySelectorAll('dialog.app-modal-dialog[open]')],parents=[...document.querySelectorAll('dialog.model-version-manager[open]')];return layers.length===2&&confirmations.length===1&&parents.length===1&&confirmations[0]!==parents[0]&&confirmations[0] instanceof HTMLDialogElement&&parents[0] instanceof HTMLDialogElement&&confirmations[0].querySelector(':scope > .ui-dialog-header h2')?.textContent.trim()==='设为默认版本'&&/^版本管理 · .+/.test(parents[0].querySelector(':scope > .ui-dialog-header h2')?.textContent.trim()??'')&&!document.querySelector('.modal-mask')})()"
  : "document.querySelectorAll('dialog[open]').length===0&&!!document.querySelector('.modal-mask')",'application confirmation above version manager')
 await click('设为默认',confirmation)
}
// Serialized into the actual WebView. Observe real frames and trusted input;
// never adjust coordinates by a guessed DPI/chrome offset or replace a click.
function installNativeModelMenuObservation(){
 if(globalThis.__ocxModelMenuInput)throw Error('native model menu observation already active')
 const selector='dialog[open] button[aria-label="Fast支持"]',button=document.querySelector(selector)
 if(!button)throw Error('native Fast select absent')
 const describe=e=>e?{tag:e.tagName,label:e.getAttribute?.('aria-label')??null}:null
 const trace={frames:[],changes:[],events:[],stages:[],frame:0}
 function sample(point){
  const r=button.getBoundingClientRect(),x=point?.x??r.x+r.width/2,y=point?.y??r.y+r.height/2,hit=document.elementFromPoint(x,y),scroll=[]
  for(let e=button.parentElement;e;e=e.parentElement)scroll.push({top:e.scrollTop,left:e.scrollLeft})
  const v=window.visualViewport
  return {at:performance.now(),x,y,top:r.top,bottom:r.bottom,rect:{left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height},
   viewport:{width:innerWidth,height:innerHeight,dpr:devicePixelRatio,screenX:window.screenX,screenY:window.screenY,outerWidth:outerWidth,outerHeight:outerHeight,visual:v?{width:v.width,height:v.height,offsetLeft:v.offsetLeft,offsetTop:v.offsetTop,pageLeft:v.pageLeft,pageTop:v.pageTop,scale:v.scale}:null},scroll,
   sameTarget:document.querySelector(selector)===button,hit:!button.disabled&&r.width>0&&r.height>0&&x>0&&y>0&&x<innerWidth&&y<innerHeight&&button.contains(hit),hitTarget:describe(hit),focused:document.hasFocus(),hidden:document.hidden,activeElement:describe(document.activeElement)}
 }
 const same=(a,b)=>JSON.stringify([a.rect,a.viewport,a.scroll])===JSON.stringify([b.rect,b.viewport,b.scroll])
 let frameId
 function frame(){
  const value={...sample(),frame:++trace.frame},previous=trace.frames.at(-1)
  if(!previous||!same(previous,value)||previous.hit!==value.hit){trace.changes.push(value);if(trace.changes.length>60)trace.changes.shift()}
  trace.frames.push(value);if(trace.frames.length>120)trace.frames.shift()
  frameId=requestAnimationFrame(frame)
 }
 trace.ready=()=>{const frames=trace.frames.slice(-3);return frames.length===3&&frames.every(f=>f.sameTarget&&f.hit&&same(f,frames[0]))?{...frames[2],stableFrames:frames.map(f=>f.frame)}:null}
 trace.check=point=>{const current=sample(point),value={sameGeometry:same(current,point),sameHit:current.sameTarget&&current.hit,current};trace.stages.push(value);return value}
 const events=['pointerdown','pointerup','mousedown','mouseup','click']
 const record=event=>trace.events.push({type:event.type,trusted:event.isTrusted,target:describe(event.target),targetWithinButton:button.contains(event.target),...sample({x:event.clientX,y:event.clientY})})
 for(const type of events)document.addEventListener(type,record,true)
 trace.dispose=()=>{cancelAnimationFrame(frameId);for(const type of events)document.removeEventListener(type,record,true)}
 globalThis.__ocxModelMenuInput=trace
 button.scrollIntoView({block:'center'})
 frameId=requestAnimationFrame(frame)
 return true
}
export async function clickNativeModelMenu({evaluate,wait,call},output){
 let point,error,installed=false
 try{
  // The detail can mount before asynchronous model projection completes. Its
  // provisional explanation disappears once effective values arrive, moving
  // the trigger even after several initially stable frames. Await the real
  // resolved-values table before scrolling or acquiring any pointer geometry.
  await wait("document.querySelector('dialog[open] .model-rule-fields caption')?.textContent.trim()==='当前草稿有效值与来源'",'native model detail projection resolved before geometry sampling')
  installed=await evaluate('('+installNativeModelMenuObservation.toString()+')()')
  point=await wait('globalThis.__ocxModelMenuInput.ready()','native menu trigger is stable across frames and hit-testable')
  await call('Input.dispatchMouseEvent',{type:'mouseMoved',x:point.x,y:point.y})
  const guard=await evaluate('globalThis.__ocxModelMenuInput.check('+JSON.stringify(point)+')')
  assert(guard.sameGeometry&&guard.sameHit,'Native model menu target moved or became occluded before pointer input')
  // No filesystem/CDP diagnostics between the immediate hit check and press.
  await call('Input.dispatchMouseEvent',{type:'mousePressed',x:point.x,y:point.y,button:'left',buttons:1,clickCount:1})
  await call('Input.dispatchMouseEvent',{type:'mouseReleased',x:point.x,y:point.y,button:'left',buttons:0,clickCount:1})
  await wait("!!document.querySelector('dialog[open] [data-ui-escape-owner]')",'native inner select open')
 }catch(cause){error=cause;throw cause}
 finally{
  if(installed){
   try{
    const trace=await evaluate('(()=>{const t=globalThis.__ocxModelMenuInput;try{return {frames:t.frames,changes:t.changes,events:t.events,stages:t.stages}}finally{t.dispose();delete globalThis.__ocxModelMenuInput}})()')
    await writeFile(join(output,'model-menu-trigger-native.json'),JSON.stringify(point??null,null,2)+'\n')
    await writeFile(join(output,'model-menu-input-native.json'),JSON.stringify({result:error?'fail':'pass',error:error?.message,point,...trace},null,2)+'\n')
   }catch(diagnosticError){if(!error)throw diagnosticError;error.nativeMenuDiagnosticError=diagnosticError.message}
  }
 }
}
export async function modelFileEvidence(configPath,root){
 let backups
 try{backups=(await readdir(join(root,'backups','model-config'))).sort()}catch(error){if(error.code!=='ENOENT')throw error;backups=[]}
 return {bytes:await readFile(configPath),backups}
}
export async function assertModelFileWrite(before,after,root,{rename=false}={}){
 assert(!after.bytes.equals(before.bytes),'Save must change actual config.json bytes')
 const added=after.backups.filter(name=>!before.backups.includes(name))
 const journals=added.filter(name=>/^rename-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/.test(name))
 const prewrites=added.filter(name=>!journals.includes(name))
 assert.equal(prewrites.length,1,'Save must create exactly one prewrite backup')
 assert.equal(journals.length,rename?1:0,'Only provider rename Save may create one recovery journal')
 assert.deepEqual(after.backups.filter(name=>before.backups.includes(name)),before.backups,'Save must preserve earlier backups')
 assert((await readFile(join(root,'backups','model-config',prewrites[0],'config.json'))).equals(before.bytes),'backup must contain exact prewrite config.json bytes')
 for(const name of journals){
  const directory=join(root,'backups','model-config',name),files=await readdir(directory)
  assert(files.length>0&&files.every(file=>/^[0-9]+[.](before|after)$/.test(file)),'rename recovery journal must contain only indexed pre/post images')
  let configImage=false
  for(const file of files.filter(file=>file.endsWith('.before'))){
   const pre=await readFile(join(directory,file)),post=await readFile(join(directory,file.replace(/[.]before$/,'.after')))
   if(pre.equals(before.bytes)&&post.equals(after.bytes))configImage=true
  }
  assert(configImage,'rename recovery journal must retain exact config pre/post images')
 }
}
export async function verifyWindowsModelUi(call,output,facts,expected){
 const configPath=modelConfigPath(facts,expected)
 const checkNativeKeyboard=nativeModelKeyboardChecksRequired(facts.about.version)
 const contract=nativeModelUiContract(facts.about.version)
 async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value}
 const invoke=(command,args={})=>evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+')')
 async function wait(expression,label){
  for(let n=0;n<100;n++){const value=await evaluate(expression);if(value)return value;await new Promise(resolve=>setTimeout(resolve,100))}
  try{const scene=await evaluate("({url:location.href,heading:document.querySelector('h1')?.textContent,rows:[...document.querySelectorAll('.models-page .model-table tbody tr')].map(e=>e.textContent),alerts:[...document.querySelectorAll('.models-page [role=alert]')].map(e=>e.textContent)})");await writeFile(join(output,'model-ui-timeout.json'),JSON.stringify({label,scene},null,2)+'\n');const shot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'model-ui-timeout.png'),Buffer.from(shot.data,'base64'))}catch{}
  throw Error('Native model UI timeout: '+label)
 }
 const visible="e.getClientRects().length>0"
 async function click(text,selector='.models-page'){
  // Returning from a public confirmation can reopen its owning native dialog
  // one frame before the header becomes visible. Keep the same real selector,
  // visibility and enabled guards under the existing bounded native wait.
  await wait("(()=>{const root=document.querySelector("+JSON.stringify(selector)+");const button=[...root?.querySelectorAll('button')??[]].find(e=>(e.textContent.trim()==="+JSON.stringify(text)+"||e.getAttribute('aria-label')==="+JSON.stringify(text)+")&&"+visible+"&&!e.disabled);if(!button)return false;button.click();return true})()",'visible action '+text)
 }
 // Tauri defines invoke as a non-writable property. Inspect it without replacing IPC.
 const invokeDescriptor=await evaluate("(()=>{const d=Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__,'invoke');return {type:typeof d?.value,writable:d?.writable,configurable:d?.configurable}})()")
 await writeFile(join(output,'model-ipc-descriptor.json'),JSON.stringify(invokeDescriptor,null,2)+'\n')
 const label='Windows native fixture '+randomUUID().slice(0,8),firstDescription='source version',secondDescription='edited through native Vue handler'
 const before=await invoke('model_file_snapshot')
 const content={description:firstDescription,matching:{include:[{kind:'prefix',value:'windows-fixture-'}],exclude:[],caseSensitive:false,channelMode:'all',channels:[],priority:0},rules:Object.fromEntries(['protocol','contextWindow','inputModalities','reasoningEfforts','defaultEffort','fast','pinnedEffort','effortMap','noReasoning','reasoningAuthoritative','fastWire'].map(k=>[k,{mode:'inherit'}])),variants:[],lockedFields:[],sources:[]}
 const created=await invoke('model_template_save',{request:{expectedLibrary:before.libraryDigest,templateId:null,baseRevision:null,label,content}})
 assert(created.changed&&created.templateId&&created.revisionId)
 await click('模型','.side');await wait("!!document.querySelector('.models-page')",'models route')
 await click('模型模版');await wait("[...document.querySelectorAll('.model-table tbody tr')].some(e=>e.textContent.includes("+JSON.stringify(label)+"))",'saved template row')
 assert(await evaluate("(()=>{const row=[...document.querySelectorAll('.model-table tbody tr')].find(e=>e.textContent.includes("+JSON.stringify(label)+"));const button=[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='编辑');button.click();return true})()"))
 await wait("!!document.querySelector('#model-basic textarea')",'continuous editor')
 await evaluate("(()=>{const input=document.querySelector('#model-basic textarea');input.value="+JSON.stringify(secondDescription)+";input.dispatchEvent(new Event('input',{bubbles:true}));return true})()")
 await click('与版本比较')
 const comparison=await wait("(()=>{const dialog=[...document.querySelectorAll('dialog[open]')].find(e=>e.textContent.includes('与版本比较'));return dialog&&dialog.textContent.includes("+JSON.stringify(firstDescription)+")&&dialog.textContent.includes("+JSON.stringify(secondDescription)+")?{rows:dialog.querySelectorAll('.model-comparison-row').length}:null})()",'native readonly comparison shows source and current buffer')
 assert.equal(comparison.rows,1)
 await click('关闭',contract.close)
 const beforeSave=await invoke('model_file_snapshot');assert.equal(beforeSave.library.templates.find(t=>t.id===created.templateId).revisions.length,1,'comparison must not save')
 await click('Agent自动填充')
 await wait("!!document.querySelector('.model-agent-panel [aria-label=\"导出上下文\"]')",'assistant context tools visible')
 await click('导出上下文','.model-agent-panel')
 const externalContext=await wait("(()=>{const dialog=[...document.querySelectorAll('dialog[open]')].find(e=>e.textContent.includes('外部任务')),text=dialog?.querySelector('textarea[readonly]')?.value;if(!text||!text.includes('discoveryFile'))return null;const pack=("+parseTaskContent.toString()+")(text);return {targets:pack.task.targets.map(t=>({templateId:t.templateId,revisionId:t.revision.id,description:t.buffer?.content.description})),scopeNew:pack.task.allowNew,contract:!!pack.contract,hasTransport:!!pack.transport,hasPrivateChat:Object.hasOwn(pack,'conversation')}})()",'actual external task bundle from current buffer')
 assert.deepEqual(externalContext.targets,[{templateId:created.templateId,revisionId:created.revisionId,description:secondDescription}]);assert.equal(externalContext.scopeNew,false);assert.equal(externalContext.contract,true);assert.equal(externalContext.hasTransport,true);assert.equal(externalContext.hasPrivateChat,false)
 await click('关闭',contract.close)
 await evaluate("(()=>{document.querySelector('.model-agent-panel button[aria-label=\"收起\"]').click();return true})()")
 await writeFile(join(output,'model-editor-context-native.json'),JSON.stringify({result:'pass',comparisonRows:comparison.rows,comparisonDidNotSave:true,externalCurrentTargetCount:externalContext.targets.length,exactBufferDescription:true,includesSharedContractAndTransport:true,privateChatExcluded:true,taskActivated:false,generationRequests:0},null,2)+'\n')
 await click('保存')
 const changed=await wait("(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('model_file_snapshot');const t=s.library.templates.find(t=>t.id==="+JSON.stringify(created.templateId)+");return t?.revisions.length===2?{snapshot:s,template:t}:null})()",'native save persisted new revision')
 assert.equal(changed.template.revisions[0].content.description,firstDescription)
 const revision=changed.template.revisions.find(r=>r.content.description===secondDescription);assert(revision,'edited saved content missing')
 assert.equal(changed.template.defaultRevision,null,'saving must not silently choose a default')
 assert.deepEqual(changed.snapshot.policy,before.policy,'template save must not apply channel policy')
 await click('返回');await wait("!document.querySelector('.detail-workspace')",'template list return')
 await evaluate("(()=>{const row=[...document.querySelectorAll('.model-table tbody tr')].find(e=>e.textContent.includes("+JSON.stringify(label)+"));[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='版本管理').click();return true})()")
 await wait("!![...document.querySelectorAll('dialog[open]')].find(e=>e.textContent.includes('版本管理'))",'version manager')
 // The newest saved revision is shown first; this is the target we just saved.
 await confirmNativeTemplateDefault({wait,click},facts.about.version)
 const pointed=await wait("(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('model_file_snapshot');return s.library.templates.find(t=>t.id==="+JSON.stringify(created.templateId)+")?.defaultRevision==="+JSON.stringify(revision.id)+"?s:null})()",'default pointer committed')
 assert.deepEqual(pointed.policy,before.policy,'default pointer must not upgrade adoption')
 await wait("document.querySelectorAll('dialog[open]').length===1&&!!document.querySelector('dialog[open] h2')?.textContent.includes('版本管理')",'return to version manager')
 await click('关闭',contract.close)
 const scope={includePolicy:false,channelIds:[],includeGlobal:false,templateRefs:[{templateId:created.templateId,revisionId:revision.id}]}
 const plan=await invoke('model_transfer_prepare',{scope,passphrase:null,includeKeys:false,managerScope:{preferences:false,extensions:false,webdav:false,webdavPasswords:false}})
 assert.equal(plan.package.payload.templates.length,1)
 const file=await invoke('model_transfer_export',{planId:plan.id})
 await writeFile(join(output,'model-export-boundary.json'),JSON.stringify({admittedRoot:expected,exportedPath:file.path},null,2)+'\n')
 assertModelExportPath(file.path,expected)
 const incoming=await invoke('model_transfer_receive',{path:file.path,passphrase:null})
 assert.equal(incoming.package.payload.templates[0].id,created.templateId)
 const adopted=await invoke('model_transfer_accept',{incomingId:incoming.id,adoptKeys:false,adoptWebdavPasswords:false,adoptExtensionCredentials:false,passphrase:null})
 assert.deepEqual(adopted.policy,before.policy);assert.equal(adopted.library.templates.find(t=>t.id===created.templateId).defaultRevision,revision.id)
 // Seed only the admitted sandbox's actual file through the production writer.
 // Ordinary channel edits intentionally do not survive an application reload.
 const modelIds=['fixture-alpha','fixture-beta','fixture-gamma'],channelId=randomUUID(),nodeId=randomUUID(),otherChannelId=randomUUID(),otherNodeId=randomUUID()
 const channel={id:channelId,label:'Native sort fixture',templateRefs:[],modelRefs:{},selection:{mode:'none'},modelOrder:modelIds,discovered:modelIds.map(id=>({id,label:id,rules:content.rules})),defaults:content.rules,overrides:{},familyOverrides:{},connection:{providerId:'native-sort-fixture',protocol:'openai-chat',nodes:[{id:nodeId,name:'Fixture only',url:'https://example.test/v1'}],activeNodeId:nodeId,credentialRef:null,enabled:false,allowPrivateNetwork:false,authMode:'key'}}
 const otherChannel={...structuredClone(channel),id:otherChannelId,label:'Native scope A fixture',modelOrder:['fixture-scope-a'],discovered:[{id:'fixture-scope-a',label:'fixture-scope-a',rules:content.rules}],connection:{...channel.connection,providerId:'native-scope-a-fixture',nodes:[{id:otherNodeId,name:'Fixture A only',url:'https://a.example.test/v1'}],activeNodeId:otherNodeId}}
 const fileBaseline=await invoke('model_file_snapshot')
 const seeded=await invoke('model_file_save',{request:{expectedFile:fileBaseline.policyDigest,policy:{...fileBaseline.policy,channels:[...fileBaseline.policy.channels,otherChannel,channel]},credentials:[{channelId,action:'replace',value:'native-sandbox-key'},{channelId:otherChannelId,action:'replace',value:'native-scope-a-original-key'}]}})
 assert.equal(seeded.changed,true)
 const initialEvidence=await modelFileEvidence(configPath,expected),initialBytes=initialEvidence.bytes,initialFile=JSON.parse(initialBytes)
 assert.equal(initialFile.providers['native-sort-fixture'].apiKey,'native-sandbox-key','fixture must reach actual sandbox config.json')
 assert.equal(initialFile.providers['native-scope-a-fixture'].apiKey,'native-scope-a-original-key')
 assert.deepEqual(seeded.snapshot.policy.channels.find(c=>c.id===channelId).modelOrder,modelIds)
 await call('Page.reload',{ignoreCache:true})
 await wait("document.readyState==='complete'&&[...document.querySelectorAll('.models-page .model-table tbody tr')].some(e=>e.textContent.includes('Native sort fixture'))",'file channel visible after full reload')
 // Retain A through real DOM handlers before editing/saving B's model order.
 async function openChannel(id,expectedKey){
  await evaluate("(()=>{const row=[...document.querySelectorAll('.channel-table tr[data-channel]')].find(e=>e.dataset.channel==="+JSON.stringify(id)+");if(!row)throw Error('file channel row missing');[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='编辑').click();return true})()")
  await wait("(()=>{const input=document.querySelector('input[aria-label=\"API Key\"]');return input&&!input.disabled&&input.value==="+JSON.stringify(expectedKey)+"})()",'current channel Key ready')
 }
 async function inputValue(selector,value){await evaluate("(()=>{const input=document.querySelector("+JSON.stringify(selector)+");if(!input)throw Error('native input missing');input.value="+JSON.stringify(value)+";input.dispatchEvent(new Event('input',{bubbles:true}));return true})()")}
 const detailSaveDisabled="document.querySelector('button[aria-label=\"保存渠道草稿\"]').disabled"
 await wait("document.querySelector('[data-testid=policy-save]')?.disabled",'clean list Save disabled')
 // Keep the corrected native keyboard/menu contract mandatory for both 0.2.20
 // and 0.2.21. Older admitted OTA images retain their original baseline checks.
 if(checkNativeKeyboard){
  const escape=async()=>{for(const type of ['keyDown','keyUp'])await call('Input.dispatchKeyEvent',{type,key:'Escape',code:'Escape',windowsVirtualKeyCode:27,nativeVirtualKeyCode:27})}
  const originalPreferences=await invoke('get_preferences')
  await invoke('save_preferences',{preferences:{...originalPreferences,visualEffects:'low'}})
  await call('Page.reload',{ignoreCache:true})
  await wait("document.documentElement.dataset.effects==='low'&&!!document.querySelector('.channel-table')",'native low-effects menu acceptance')
  await openChannel(channelId,'native-sandbox-key')
  await click('编辑明细','.channel-model-table')
  await wait("!!document.querySelector('dialog[open] input')",'native model detail open')
  await inputValue('dialog[open] label input','discarded-native-alias')
  await evaluate("document.querySelector('dialog[open] label input').focus();true")
  await escape();await wait("!document.querySelector('dialog[open]')",'Escape closes a populated model detail input')
  assert.equal(await evaluate("document.querySelector('input[aria-label=\"fixture-alpha路由别名\"]').value"),'','cancel must discard detail alias')
  await click('编辑明细','.channel-model-table')
  await wait("!!document.querySelector('dialog[open]')",'reopen native model detail')
  await clickNativeModelMenu({evaluate,wait,call},output)
  const menuPosition=await wait("(()=>{const d=document.querySelector('dialog[open]'),t=d?.querySelector('button[aria-label=\"Fast支持\"]'),m=d?.querySelector('[data-ui-escape-owner]');if(!t||!m)return null;const a=t.getBoundingClientRect(),b=m.getBoundingClientRect(),zoom=Number(getComputedStyle(d).zoom)||1;return b.width&&b.height?{trigger:{left:a.left,right:a.right,top:a.top,bottom:a.bottom},menu:{left:b.left,right:b.right,top:b.top,bottom:b.bottom,width:b.width},zoom,width:innerWidth,height:innerHeight}:null})()",'native menu geometry')
  const idealLeft=Math.max(12*menuPosition.zoom,Math.min(menuPosition.trigger.right-menuPosition.menu.width,menuPosition.width-menuPosition.menu.width-12*menuPosition.zoom))
  await writeFile(join(output,'model-menu-position-native.json'),JSON.stringify(menuPosition,null,2)+'\n')
  assert(Math.abs(menuPosition.menu.left-idealLeft)<=3,'low-effects menu detached horizontally from its trigger')
  assert(Math.min(Math.abs(menuPosition.menu.top-menuPosition.trigger.bottom),Math.abs(menuPosition.trigger.top-menuPosition.menu.bottom))<=8*menuPosition.zoom+3,'low-effects menu detached vertically from its trigger')
  // AX activation can leave focus elsewhere in the same owner. Use the real
  // header button as that focus target; do not synthesize a DOM Escape event.
  await evaluate("document.querySelector('dialog[open] header button').focus();true")
  await escape();await wait("!!document.querySelector('dialog[open]')&&!document.querySelector('dialog[open] [data-ui-escape-owner]')",'first Escape closes only the inner select')
  await escape();await wait("!document.querySelector('dialog[open]')",'second Escape closes the owning dialog')
  const canceledEvidence=await modelFileEvidence(configPath,expected)
  assert(canceledEvidence.bytes.equals(initialBytes),'keyboard cancellation changed config.json')
  assert.deepEqual(canceledEvidence.backups,initialEvidence.backups,'keyboard cancellation added a backup')
  await writeFile(join(output,'model-dialog-keyboard-native.json'),JSON.stringify({result:'pass',version:facts.about.version,input:'real WebView2 CDP keyDown/keyUp',populatedInputEscape:true,aliasDiscarded:true,innerSelectPriorityFromOwnerFocus:true,secondEscapeClosesOwner:true,configAndBackupsUnchanged:true,noModelRequest:true,lowEffectsMenuPosition:menuPosition},null,2)+'\n')
  await click('返回渠道列表');await wait("!!document.querySelector('.channel-table')&&!document.querySelector('.channel-detail-name')",'keyboard checks return to clean list')
  await invoke('save_preferences',{preferences:originalPreferences})
  await call('Page.reload',{ignoreCache:true})
  await wait("!!document.querySelector('.channel-table')",'restore isolated display preferences before remaining checks')
 }
 await openChannel(otherChannelId,'native-scope-a-original-key')
 assert.equal(await evaluate(detailSaveDisabled),true,'clean detail Save disabled')
 const hasExplicitProvider=await evaluate("!!document.querySelector('input[aria-label=\"运行渠道标识\"]')")
 const pendingName=hasExplicitProvider?'Native scope A pending':'Native-scope-A-pending'
 const pendingProvider=hasExplicitProvider?'native-scope-a-fixture':'Native-scope-A-pending'
 await inputValue('#channel-config label input',pendingName)
 // Historical 0.2.21 exposes an explicit key; name-only candidates allocate it.
 // Both must still prove the same selective-save and pending-Key boundaries.
 if(hasExplicitProvider)await inputValue('input[aria-label="运行渠道标识"]','native-scope-a-fixture')
 await inputValue('input[aria-label="API Key"]','native-scope-a-pending-key')
 await click('展开编辑','.channel-model-table')
 await inputValue('.model-row-editor input[type=number]','96000')
 await wait("!!document.querySelector('tr[data-model=fixture-scope-a][data-pending=true]')",'A unconfirmed row')
 await click('返回渠道列表');await wait("!!document.querySelector('.channel-table')",'A return retains memory')
 assert((await readFile(configPath)).equals(initialBytes),'A edits must not write actual file')
 await openChannel(channelId,'native-sandbox-key')
 assert.equal(await evaluate(detailSaveDisabled),true,'clean B stays disabled while A is dirty')
 await wait("document.querySelectorAll('.channel-model-table tr[data-model]').length===3",'three actual model rows')
 // 行排序已从 HTML5 拖放改为指针拖拽（WKWebView 下 drop 不触发），起手元素是 .model-grip。
 // 这里仍走真实的 WebView2 鼠标事件，而不是合成 DOM 拖拽事件：验的是用户实际动作。
 const points=await evaluate("(()=>{const rows=[...document.querySelectorAll('.channel-model-table tr[data-model]')];const from=rows[0].querySelector('.model-grip');if(!from)throw new Error('model grip missing');from.scrollIntoView({block:'center'});const a=from.getBoundingClientRect(),b=rows[1].querySelector('strong').getBoundingClientRect();return {from:{x:a.x+a.width/2,y:a.y+a.height/2},to:{x:b.x+b.width/2,y:b.y+b.height/2},width:innerWidth,height:innerHeight,before:rows.map(e=>e.dataset.model)}})()")
 await writeFile(join(output,'model-sort-pointer.json'),JSON.stringify(points,null,2)+'\n')
 for(const point of [points.from,points.to])assert(point.x>0&&point.y>0&&point.x<points.width&&point.y<points.height,'native sort target outside viewport')
 await call('Input.dispatchMouseEvent',{type:'mouseMoved',...points.from})
 await call('Input.dispatchMouseEvent',{type:'mousePressed',...points.from,button:'left',buttons:1,clickCount:1})
 for(let step=1;step<=12;step++){await call('Input.dispatchMouseEvent',{type:'mouseMoved',x:points.from.x+(points.to.x-points.from.x)*step/12,y:points.from.y+(points.to.y-points.from.y)*step/12,button:'left',buttons:1});await new Promise(resolve=>setTimeout(resolve,25))}
 await call('Input.dispatchMouseEvent',{type:'mouseReleased',...points.to,button:'left',buttons:0,clickCount:1})
 const orderedIds=['fixture-beta','fixture-alpha','fixture-gamma']
 const order=await wait("(()=>{const order=[...document.querySelectorAll('.channel-model-table tr[data-model]')].map(e=>e.dataset.model);return order[0]==='fixture-beta'&&order[1]==='fixture-alpha'?order:null})()",'native pointer sort updates in-memory rows')
 assert.deepEqual(order,orderedIds)
 assert((await readFile(configPath)).equals(initialBytes),'unconfirmed pointer sort must not write config.json')
 const keyState=await wait("(()=>{const input=document.querySelector('input[aria-label=\"API Key\"]');return input?.value==='native-sandbox-key'?{type:input.type,value:input.value}:null})()",'native editor loads the current file Key')
 assert.equal(keyState.type,'password');assert.equal(keyState.value,'native-sandbox-key','native editor must load existing Key')
 await click('显示 API Key');assert.equal(await evaluate("document.querySelector('input[aria-label=\"API Key\"]').type"),'text')
 await click('隐藏 API Key')
 // Native scope is verified by file/backup/snapshot results. Request fields
 // are covered separately by store/API units and Rust/browser loopback tests.
 await click('保存渠道草稿')
 await wait("!!document.querySelector('.channel-table')&&!document.querySelector('.channel-detail-name')",'B detail Save returns directly to list')
 assert.equal(await evaluate("[...document.querySelectorAll('dialog[open]')].some(e=>e.textContent.includes('保存预览'))"),false,'detail Save must not open full Preview')
 const saved=await wait("(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('model_file_snapshot');const c=s.policy.channels.find(c=>c.id==="+JSON.stringify(channelId)+");return c?.modelOrder[0]==='fixture-beta'&&c.modelOrder[1]==='fixture-alpha'?s:null})()",'direct native sort Save writes actual file')
 const writtenEvidence=await modelFileEvidence(configPath,expected),written=JSON.parse(writtenEvidence.bytes)
 await assertModelFileWrite(initialEvidence,writtenEvidence,expected)
 assert.equal(written.providers['native-sort-fixture'].apiKey,'native-sandbox-key','sort-only Save must preserve existing Key')
 assert.deepEqual(written.modelPickerOrder.filter(slug=>slug.startsWith('native-sort-fixture/')),orderedIds.map(id=>'native-sort-fixture/'+id),'B actual file picker order')
 assert.deepEqual(written.providers['native-sort-fixture'].models,initialFile.providers['native-sort-fixture'].models,'sort preserves raw model inventory')
 for(const [providerId,provider] of Object.entries(initialFile.providers))if(providerId!=='native-sort-fixture')assert.deepEqual(written.providers[providerId],provider,'B scope changed another provider or Key: '+providerId)
 assert.equal(Object.keys(written.providers).length,Object.keys(initialFile.providers).length)
 assert.notEqual(saved.policyDigest,seeded.snapshot.policyDigest)
 await wait("!!document.querySelector('.notif-layer .process-progress')",'global save result')
 assert.equal(await evaluate("document.querySelectorAll('main .save-result,main .process-progress,.task-reopen').length"),0,'save result must not occupy page or a second notification entry')
 await click('稍后','.notif-layer')
 const savedBBytes=writtenEvidence.bytes
 await openChannel(channelId,'native-sandbox-key')
 assert.equal(await evaluate(detailSaveDisabled),true,'reopened B must be clean despite pending A')
 assert.equal(await evaluate("document.querySelector('input[aria-label=\"API Key\"]').type"),'password','reopened B Key is masked')
 await click('返回渠道列表');await wait("!!document.querySelector('.channel-table')",'B return')
 await openChannel(otherChannelId,'native-scope-a-pending-key')
 assert.equal(await evaluate(detailSaveDisabled),false,'A remains dirty after B Save')
 assert.equal(await evaluate("document.querySelector('#channel-config label input').value"),pendingName,'A name retained')
 assert.equal(await evaluate("document.querySelector('.model-row-editor input[type=number]').value"),'96000','A unconfirmed row input retained')
 assert.equal(await evaluate("!!document.querySelector('tr[data-model=fixture-scope-a][data-pending=true]')"),true,'A pending row retained')
 await click('加入草稿','.model-row-editor')
 await wait("!!document.querySelector('tr[data-model=fixture-scope-a][data-pending=false]')",'confirm remaining A row in memory')
 await click('返回渠道列表');await wait("!!document.querySelector('.channel-table')",'remaining changes in list')
 async function openSavePreview(){
  await evaluate("(()=>{const button=document.querySelector('[data-testid=policy-save]');if(!button||button.disabled)throw Error('remaining list Save missing');button.click();return true})()")
  await wait("!![...document.querySelectorAll('dialog[open]')].find(e=>e.textContent.includes('保存预览'))",'global list Preview')
 }
 await openSavePreview()
 assert.equal(await evaluate("(()=>{const d=document.querySelector('dialog[open]');return d.textContent.includes("+JSON.stringify(pendingName)+")&&d.textContent.includes('Native sort fixture')})()"),true,'list Preview still shows all channels')
 await click('取消','dialog[open]');await wait("!document.querySelector('dialog[open]')",'cancel list Preview')
 assert((await readFile(configPath)).equals(savedBBytes),'cancel Preview must neither revert B nor write A')
 const canceled=await modelFileEvidence(configPath,expected),canceledSnapshot=await invoke('model_file_snapshot')
 assert.deepEqual(canceled.backups,writtenEvidence.backups,'cancel Preview must not create a backup')
 assert.equal(canceledSnapshot.policyDigest,saved.policyDigest,'cancel Preview must preserve file digest')
 assert.deepEqual(canceledSnapshot.policy,saved.policy,'cancel Preview must not commit A or revert B layout')
 await openSavePreview();await click('确认保存','dialog[open]')
 await wait("!!document.querySelector('.notif-layer .process-progress')",'remaining A file Save result')
 const remainingEvidence=await modelFileEvidence(configPath,expected),remaining=JSON.parse(remainingEvidence.bytes)
 await assertModelFileWrite(writtenEvidence,remainingEvidence,expected,{rename:pendingProvider!=='native-scope-a-fixture'})
 assert.equal(remaining.providers[pendingProvider].apiKey,'native-scope-a-pending-key','remaining Save commits A Key')
 assert.equal(remaining.providers[pendingProvider].modelContextWindows['fixture-scope-a'],96000,'remaining Save commits confirmed A row')
 for(const [providerId,provider] of Object.entries(written.providers))if(providerId!=='native-scope-a-fixture')assert.deepEqual(remaining.providers[providerId],provider,'remaining Save changed saved B or another Key: '+providerId)
 assert.equal(Object.keys(remaining.providers).length,Object.keys(written.providers).length,'remaining Save preserves provider set')
 const remainingSnapshot=await invoke('model_file_snapshot'),remainingA=remainingSnapshot.policy.channels.find(c=>c.id===otherChannelId)
 assert.notEqual(remainingSnapshot.policyDigest,saved.policyDigest,'remaining Save changes real file digest')
 assert.equal(remainingA.label,pendingName,'remaining Save commits A name')
 assert.equal(remainingA.overrides['fixture-scope-a'].contextWindow.value,96000,'snapshot reads back confirmed A row')
 assert.deepEqual(remainingSnapshot.policy.channels.find(c=>c.id===channelId),saved.policy.channels.find(c=>c.id===channelId),'remaining Save preserves B snapshot and order')
 await wait("document.querySelector('[data-testid=policy-save]')?.disabled",'remaining changes now clean')
 await click('稍后','.notif-layer')
 await call('Page.reload',{ignoreCache:true})
 await wait("document.readyState==='complete'&&[...document.querySelectorAll('.models-page .model-table tbody tr')].some(e=>e.textContent.includes('Native sort fixture'))",'saved channel after reopen')
 await evaluate("(()=>{const row=[...document.querySelectorAll('.model-table tbody tr')].find(e=>e.textContent.includes('Native sort fixture'));[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='编辑').click();return true})()")
 await wait("JSON.stringify([...document.querySelectorAll('.channel-model-table tr[data-model]')].map(e=>e.dataset.model))==="+JSON.stringify(JSON.stringify(orderedIds)),'reopened file preserves directly saved native order')
 assert.equal(await evaluate(detailSaveDisabled),true,'full reload leaves B clean')
 assert(!await evaluate("document.querySelector('main').innerText.includes('待重启')"),'later/reopen must not resurrect restart state')
 await writeFile(join(output,'model-sort-native.json'),JSON.stringify({result:'pass',input:'WebView2 CDP mouse press/move/release, not synthetic DOM drag events',channelId,before:points.before,after:order,memoryEditBeforeSave:true,nativeFileScopeVerified:true,detailReturnedToList:true,noDetailPreview:true,otherChannelMemoryRetained:true,otherProviderAndKeysPreserved:true,canceledListPreviewPreservedFile:true,remainingListConfirmation:true,reopenedOrder:true,existingKeyMasked:true,keyEye:true,keyPreserved:true,runtimeSave:false},null,2)+'\n')
 // Portable configuration exercises the installed file-aware backend and native
 // credential staging. Only synthetic material in this admitted sandbox exists.
 const transferBaseline=await invoke('model_file_snapshot')
 const channelScope={includePolicy:true,channelIds:[channelId],includeGlobal:false,templateRefs:[]}
 const portable=await invoke('model_transfer_prepare',{scope:channelScope,passphrase:null,includeKeys:true,managerScope:{preferences:false,extensions:false,webdav:false,webdavPasswords:false}})
 const portableFile=await invoke('model_transfer_export',{planId:portable.id})
 assertModelExportPath(portableFile.path,expected)
 const received=await invoke('model_transfer_receive',{path:portableFile.path,passphrase:null})
 const sourceId=received.package.payload.policy.channels[0].id
 const selected=await invoke('model_transfer_select',{incomingId:received.id,scope:{templateRefs:[],channelIds:[sourceId],includeGlobal:false,channelMappings:{[sourceId]:{mode:'new',label:'Native portable copy'}}},passphrase:null})
 const beforeAdopt=await readFile(configPath)
 const accepted=await invoke('model_transfer_accept',{incomingId:selected.id,adoptKeys:true,adoptWebdavPasswords:false,adoptExtensionCredentials:false,passphrase:null})
 assert.equal(accepted.workspace??null,null,'ordinary portable acceptance must commit the saved file, not leave a second-save draft')
 assert.notEqual(accepted.policyDigest,transferBaseline.policyDigest,'portable acceptance must change the saved configuration baseline')
 const copy=accepted.policy.channels.find(c=>c.label==='Native portable copy')
 assert(copy?.connection.providerId,'accepted channel must retain its installed provider identity')
 const copyFile=JSON.parse(await readFile(configPath)),rawCopy=copyFile.providers[copy.connection.providerId]
 const beforeAdoptFile=JSON.parse(beforeAdopt)
 for(const [providerId,provider] of Object.entries(beforeAdoptFile.providers))assert.deepEqual(copyFile.providers[providerId],provider,'portable acceptance changed another provider or Key: '+providerId)
 assert.equal(Object.keys(copyFile.providers).length,Object.keys(beforeAdoptFile.providers).length+1,'portable acceptance must add only the selected channel')
 assert.equal(rawCopy.apiKey,'native-sandbox-key','selected portable Key must be saved on confirmation')
 const acceptedSnapshot=await invoke('model_file_snapshot')
 assert.equal(acceptedSnapshot.policyDigest,accepted.policyDigest,'accepted response must match the saved file snapshot')
 assert.equal(acceptedSnapshot.policy.channels.find(c=>c.id===copy.id)?.label,'Native portable copy','accepted channel must survive snapshot readback')
 assert.deepEqual(acceptedSnapshot.policy.channels.find(c=>c.id===copy.id).modelOrder,orderedIds)
 assert.deepEqual(acceptedSnapshot.policy.channels.filter(c=>c.id!==copy.id),transferBaseline.policy.channels,'portable acceptance must preserve every pre-existing channel and its order')
 await writeFile(join(output,'model-transfer-native.json'),JSON.stringify({result:'pass',exportCurrentFile:true,optionalPlaintextFile:true,acceptedDirectlyToSavedConfig:true,secondSaveRequired:false,importedAuthorizationSaved:true,modelOrderPreserved:true,otherProvidersAndKeysPreserved:true,preexistingChannelsPreserved:true},null,2)+'\n')
 await new Promise(resolve=>setTimeout(resolve,100))
 const tasks=await evaluate("[...document.querySelectorAll('.model-operation-task,.model-operation-card')].filter(e=>e.getClientRects().length).map(e=>e.textContent)")
 const screenshot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'model-template-native.png'),Buffer.from(screenshot.data,'base64'))
 await verifyUiAlignment({call,evaluate,wait,click,output,templateLabel:label,publicDialogs:contract.publicDialogs})
 let probes021
 if(contract.publicDialogs){
  const {verifyWindows021Probes}=await import('./windows-021-probes.mjs')
  probes021=await verifyWindows021Probes({call,evaluate,wait,click,invoke,output},facts,expected)
 }
 const result={result:'pass',scope:'native template, direct current-channel file Save/reopen, remaining global Preview/cancel/confirm, exact transfer; isolated config writes, no runtime generation',templateId:created.templateId,revisionId:revision.id,immutablePriorRevision:true,channelPolicyUnchanged:true,defaultSeparateFromAdoption:true,confirmationLeftNativeTopLayer:!contract.publicDialogs,returnedToVersionManager:true,exportedWithinSandbox:true,fileRoundtrip:true,nativePointerModelSort:true,nativeFileScopeVerified:true,otherChannelMemoryRetained:true,otherProviderAndKeysPreserved:true,canceledListPreviewPreservedFile:true,remainingGlobalFileSave:true,channelReopen:true,existingKeyEye:true,readonlyComparison:true,externalContextBundle:true}
 result.confirmationHost=contract.confirmation
 result.versionManagerSuspendedForConfirmation=!contract.publicDialogs
 result.versionManagerRetainedUnderConfirmation=contract.publicDialogs
 result.nativeModelKeyboardChecks=checkNativeKeyboard
 if(probes021)result.probes021=probes021
 await writeFile(join(output,'model-ui.json'),JSON.stringify(result,null,2)+'\n');return result
}
