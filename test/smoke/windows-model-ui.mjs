// Bounded native WebView2 business check for the already-owned startup sandbox.
// Real Tauri IPC and DOM handlers; no DEV store/fixture installation or network.
import assert from 'node:assert/strict'
import {win32,join} from 'node:path'
import {randomUUID} from 'node:crypto'
import {writeFile} from 'node:fs/promises'
function normalize(value){assert(typeof value==='string'&&win32.isAbsolute(value),'absolute active root required');const canonical=value.startsWith('\\\\?\\UNC\\')?'\\\\'+value.slice(8):value.replace(/^\\\\\?\\/,'');return win32.resolve(canonical).toLowerCase()}
export function assertModelExportPath(path,expected){
 const file=normalize(path),directory=normalize(win32.join(expected,'exports'))
 assert(file.startsWith(directory+win32.sep),'export escaped admitted root')
}
export function assertModelSandbox(facts,expected){
 assert(facts?.about?.platform==='Windows','Windows native facts required')
 assert(typeof expected==='string'&&win32.isAbsolute(expected),'explicit absolute startup sandbox required')
 assert.equal(normalize(facts.dataRoot.currentDataRoot),normalize(expected),'immutable startup root differs from admitted sandbox')
 assert.equal(normalize(facts.dataRoot.activeDataRoot),normalize(expected),'saved root differs from startup sandbox')
 assert.equal(facts.dataRoot.runtimeActive,true,'startup root is not active')
}
export async function verifyWindowsModelUi(call,output,facts,expected){
 assertModelSandbox(facts,expected)
 async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value}
 const invoke=(command,args={})=>evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+')')
 async function wait(expression,label){for(let n=0;n<100;n++){const value=await evaluate(expression);if(value)return value;await new Promise(resolve=>setTimeout(resolve,100))}throw Error('Native model UI timeout: '+label)}
 const visible="e.getClientRects().length>0"
 async function click(text,selector='.models-page'){const done=await evaluate("(()=>{const root=document.querySelector("+JSON.stringify(selector)+");const button=[...root?.querySelectorAll('button')??[]].find(e=>e.textContent.trim()==="+JSON.stringify(text)+"&&"+visible+"&&!e.disabled);if(!button)return false;button.click();return true})()");assert(done,'Native visible action missing: '+text)}
 const label='Windows native fixture '+randomUUID().slice(0,8),firstDescription='source version',secondDescription='edited through native Vue handler'
 const before=await invoke('model_config_snapshot')
 const content={description:firstDescription,matching:{include:[{kind:'prefix',value:'windows-fixture-'}],exclude:[],caseSensitive:false,channelMode:'all',channels:[],priority:0},rules:Object.fromEntries(['protocol','contextWindow','inputModalities','reasoningEfforts','defaultEffort','fast','pinnedEffort','effortMap','noReasoning','reasoningAuthoritative','fastWire'].map(k=>[k,{mode:'inherit'}])),variants:[],lockedFields:[],sources:[]}
 const created=await invoke('model_template_save',{request:{expectedLibrary:before.libraryDigest,templateId:null,baseRevision:null,label,content}})
 assert(created.changed&&created.templateId&&created.revisionId)
 await click('模型','.side');await wait("!!document.querySelector('.models-page')",'models route')
 await click('模型模版');await wait("[...document.querySelectorAll('.model-table tbody tr')].some(e=>e.textContent.includes("+JSON.stringify(label)+"))",'saved template row')
 assert(await evaluate("(()=>{const row=[...document.querySelectorAll('.model-table tbody tr')].find(e=>e.textContent.includes("+JSON.stringify(label)+"));const button=[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='编辑');button.click();return true})()"))
 await wait("!!document.querySelector('#model-basic textarea')",'continuous editor')
 await evaluate("(()=>{const input=document.querySelector('#model-basic textarea');input.value="+JSON.stringify(secondDescription)+";input.dispatchEvent(new Event('input',{bubbles:true}));return true})()")
 await click('保存')
 const changed=await wait("(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('model_config_snapshot');const t=s.library.templates.find(t=>t.id==="+JSON.stringify(created.templateId)+");return t?.revisions.length===2?{snapshot:s,template:t}:null})()",'native save persisted new revision')
 assert.equal(changed.template.revisions[0].content.description,firstDescription)
 const revision=changed.template.revisions.find(r=>r.content.description===secondDescription);assert(revision,'edited saved content missing')
 assert.equal(changed.template.defaultRevision,null,'saving must not silently choose a default')
 assert.deepEqual(changed.snapshot.policy,before.policy,'template save must not apply channel policy')
 await click('返回');await wait("!document.querySelector('.detail-workspace')",'template list return')
 await evaluate("(()=>{const row=[...document.querySelectorAll('.model-table tbody tr')].find(e=>e.textContent.includes("+JSON.stringify(label)+"));[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='版本管理').click();return true})()")
 await wait("!![...document.querySelectorAll('dialog[open]')].find(e=>e.textContent.includes('版本管理'))",'version manager')
 // The newest saved revision is shown first; this is the target we just saved.
 await click('设为默认','dialog[open]')
 await wait("document.querySelectorAll('dialog[open]').length===0&&!!document.querySelector('.modal-mask')",'application confirmation above native dialog')
 await click('设为默认','.modal-mask')
 const pointed=await wait("(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('model_config_snapshot');return s.library.templates.find(t=>t.id==="+JSON.stringify(created.templateId)+")?.defaultRevision==="+JSON.stringify(revision.id)+"?s:null})()",'default pointer committed')
 assert.deepEqual(pointed.policy,before.policy,'default pointer must not upgrade adoption')
 await wait("document.querySelectorAll('dialog[open]').length===1",'return to version manager')
 await click('关闭','dialog[open]')
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
 // Persist explicit fixture-only channels through the real draft IPC, never runtime Save.
 // This exercises native HTML5 row sorting; no DEV store injection or provider request.
 const modelIds=['fixture-alpha','fixture-beta','fixture-gamma'],channelId=randomUUID(),nodeId=randomUUID()
 const channel={id:channelId,label:'Native sort fixture',templateRefs:[],modelRefs:{},selection:{mode:'none'},modelOrder:modelIds,discovered:modelIds.map(id=>({id,label:id,rules:content.rules})),defaults:content.rules,overrides:{},familyOverrides:{},connection:{providerId:'native-sort-fixture',protocol:'openai-chat',nodes:[{id:nodeId,name:'Fixture only',url:'https://example.test/v1'}],activeNodeId:nodeId,credentialRef:null,enabled:false,allowPrivateNetwork:false,authMode:'none'}}
 const staged=await invoke('model_policy_stage',{expectedPolicy:adopted.policyDigest,expectedWorkspace:adopted.workspaceDigest,policy:{...adopted.policy,channels:[...adopted.policy.channels,channel]}})
 assert.deepEqual(staged.policy,adopted.policy,'staging must not commit runtime policy')
 await evaluate("(()=>{[...document.querySelectorAll('aside button')].find(e=>e.textContent.trim()==='设置').click();return true})()")
 await wait("!document.querySelector('.models-page')",'leave model route')
 await evaluate("(()=>{[...document.querySelectorAll('aside button')].find(e=>e.textContent.trim()==='模型').click();return true})()")
 await wait("!!document.querySelector('.models-page')",'restore staged model route')
 await click('渠道模型');await wait("[...document.querySelectorAll('.model-table tbody tr')].some(e=>e.textContent.includes('Native sort fixture'))",'staged channel visible')
 await evaluate("(()=>{const row=[...document.querySelectorAll('.model-table tbody tr')].find(e=>e.textContent.includes('Native sort fixture'));[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='编辑').click();return true})()")
 await wait("document.querySelectorAll('.channel-model-table tr[data-model]').length===3",'three actual model rows')
 const points=await evaluate("(()=>{const rows=[...document.querySelectorAll('.channel-model-table tr[data-model]')];const from=rows[0].querySelector('button[draggable=true]');from.scrollIntoView({block:'center'});const a=from.getBoundingClientRect(),b=rows[1].querySelector('strong').getBoundingClientRect();return {from:{x:a.x+a.width/2,y:a.y+a.height/2},to:{x:b.x+b.width/2,y:b.y+b.height/2},width:innerWidth,height:innerHeight,before:rows.map(e=>e.dataset.model)}})()")
 await writeFile(join(output,'model-sort-pointer.json'),JSON.stringify(points,null,2)+'\n')
 for(const point of [points.from,points.to])assert(point.x>0&&point.y>0&&point.x<points.width&&point.y<points.height,'native sort target outside viewport')
 await call('Input.dispatchMouseEvent',{type:'mouseMoved',...points.from})
 await call('Input.dispatchMouseEvent',{type:'mousePressed',...points.from,button:'left',buttons:1,clickCount:1})
 for(let step=1;step<=12;step++){await call('Input.dispatchMouseEvent',{type:'mouseMoved',x:points.from.x+(points.to.x-points.from.x)*step/12,y:points.from.y+(points.to.y-points.from.y)*step/12,button:'left',buttons:1});await new Promise(resolve=>setTimeout(resolve,25))}
 await call('Input.dispatchMouseEvent',{type:'mouseReleased',...points.to,button:'left',buttons:0,clickCount:1})
 const order=await wait("(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('model_config_snapshot');const c=s.workspace?.policy.channels.find(c=>c.id==="+JSON.stringify(channelId)+");return c?.modelOrder[0]==='fixture-beta'&&c.modelOrder[1]==='fixture-alpha'?{order:c.modelOrder,snapshot:s}:null})()",'native pointer sort persisted to draft')
 assert.deepEqual(order.order,['fixture-beta','fixture-alpha','fixture-gamma']);assert.deepEqual(order.snapshot.policy,before.policy,'sorting draft must not save runtime')
 await writeFile(join(output,'model-sort-native.json'),JSON.stringify({result:'pass',input:'WebView2 CDP mouse press/move/release, not synthetic DOM drag events',channelId,before:points.before,after:order.order,policyUnchanged:true,runtimeSave:false},null,2)+'\n')
 await new Promise(resolve=>setTimeout(resolve,100))
 const tasks=await evaluate("[...document.querySelectorAll('.model-operation-task,.model-operation-card')].filter(e=>e.getClientRects().length).map(e=>e.textContent)")
 const screenshot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'model-template-native.png'),Buffer.from(screenshot.data,'base64'))
 const result={result:'pass',scope:'native template edit/save/default-confirm plus exact template file export/receive/adopt; no runtime generation',templateId:created.templateId,revisionId:revision.id,immutablePriorRevision:true,channelPolicyUnchanged:true,defaultSeparateFromAdoption:true,confirmationLeftNativeTopLayer:true,returnedToVersionManager:true,exportedWithinSandbox:true,fileRoundtrip:true,nativePointerModelSort:true}
 await writeFile(join(output,'model-ui.json'),JSON.stringify(result,null,2)+'\n');return result
}
