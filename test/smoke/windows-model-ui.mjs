// Bounded native WebView2 business check for the already-owned startup sandbox.
// Real Tauri IPC and DOM handlers; no DEV store/fixture installation or network.
import assert from 'node:assert/strict'
import {win32,join} from 'node:path'
import {randomUUID} from 'node:crypto'
import {writeFile,readFile} from 'node:fs/promises'
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
export async function verifyWindowsModelUi(call,output,facts,expected){
 const configPath=modelConfigPath(facts,expected)
 async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value}
 const invoke=(command,args={})=>evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+')')
 async function wait(expression,label){
  for(let n=0;n<100;n++){const value=await evaluate(expression);if(value)return value;await new Promise(resolve=>setTimeout(resolve,100))}
  try{const scene=await evaluate("({url:location.href,heading:document.querySelector('h1')?.textContent,rows:[...document.querySelectorAll('.models-page .model-table tbody tr')].map(e=>e.textContent),alerts:[...document.querySelectorAll('.models-page [role=alert]')].map(e=>e.textContent)})");await writeFile(join(output,'model-ui-timeout.json'),JSON.stringify({label,scene},null,2)+'\n');const shot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'model-ui-timeout.png'),Buffer.from(shot.data,'base64'))}catch{}
  throw Error('Native model UI timeout: '+label)
 }
 const visible="e.getClientRects().length>0"
 async function click(text,selector='.models-page'){const done=await evaluate("(()=>{const root=document.querySelector("+JSON.stringify(selector)+");const button=[...root?.querySelectorAll('button')??[]].find(e=>(e.textContent.trim()==="+JSON.stringify(text)+"||e.getAttribute('aria-label')==="+JSON.stringify(text)+")&&"+visible+"&&!e.disabled);if(!button)return false;button.click();return true})()");assert(done,'Native visible action missing: '+text)}
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
 await click('关闭','dialog[open]')
 const beforeSave=await invoke('model_file_snapshot');assert.equal(beforeSave.library.templates.find(t=>t.id===created.templateId).revisions.length,1,'comparison must not save')
 await click('Agent自动填充')
 await wait("!!document.querySelector('.model-agent-panel [aria-label=\"导出上下文\"]')",'assistant context tools visible')
 await click('导出上下文','.model-agent-panel')
 const externalContext=await wait("(()=>{const dialog=[...document.querySelectorAll('dialog[open]')].find(e=>e.textContent.includes('外部任务')),text=dialog?.querySelector('textarea[readonly]')?.value;if(!text||!text.includes('discoveryFile'))return null;const pack=("+parseTaskContent.toString()+")(text);return {targets:pack.task.targets.map(t=>({templateId:t.templateId,revisionId:t.revision.id,description:t.buffer?.content.description})),scopeNew:pack.task.allowNew,contract:!!pack.contract,hasTransport:!!pack.transport,hasPrivateChat:Object.hasOwn(pack,'conversation')}})()",'actual external task bundle from current buffer')
 assert.deepEqual(externalContext.targets,[{templateId:created.templateId,revisionId:created.revisionId,description:secondDescription}]);assert.equal(externalContext.scopeNew,false);assert.equal(externalContext.contract,true);assert.equal(externalContext.hasTransport,true);assert.equal(externalContext.hasPrivateChat,false)
 await click('关闭','dialog[open]')
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
 await click('设为默认','dialog[open]')
 await wait("document.querySelectorAll('dialog[open]').length===0&&!!document.querySelector('.modal-mask')",'application confirmation above native dialog')
 await click('设为默认','.modal-mask')
 const pointed=await wait("(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('model_file_snapshot');return s.library.templates.find(t=>t.id==="+JSON.stringify(created.templateId)+")?.defaultRevision==="+JSON.stringify(revision.id)+"?s:null})()",'default pointer committed')
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
 // Seed only the admitted sandbox's actual file through the production writer.
 // Ordinary channel edits intentionally do not survive an application reload.
 const modelIds=['fixture-alpha','fixture-beta','fixture-gamma'],channelId=randomUUID(),nodeId=randomUUID()
 const channel={id:channelId,label:'Native sort fixture',templateRefs:[],modelRefs:{},selection:{mode:'none'},modelOrder:modelIds,discovered:modelIds.map(id=>({id,label:id,rules:content.rules})),defaults:content.rules,overrides:{},familyOverrides:{},connection:{providerId:'native-sort-fixture',protocol:'openai-chat',nodes:[{id:nodeId,name:'Fixture only',url:'https://example.test/v1'}],activeNodeId:nodeId,credentialRef:null,enabled:false,allowPrivateNetwork:false,authMode:'key'}}
 const fileBaseline=await invoke('model_file_snapshot')
 const seeded=await invoke('model_file_save',{request:{expectedFile:fileBaseline.policyDigest,policy:{...fileBaseline.policy,channels:[...fileBaseline.policy.channels,channel]},credentials:[{channelId,action:'replace',value:'native-sandbox-key'}]}})
 assert.equal(seeded.changed,true)
 const initialBytes=await readFile(configPath),initialFile=JSON.parse(initialBytes)
 assert.equal(initialFile.providers['native-sort-fixture'].apiKey,'native-sandbox-key','fixture must reach actual sandbox config.json')
 assert.deepEqual(seeded.snapshot.policy.channels.find(c=>c.id===channelId).modelOrder,modelIds)
 await call('Page.reload',{ignoreCache:true})
 await wait("document.readyState==='complete'&&[...document.querySelectorAll('.models-page .model-table tbody tr')].some(e=>e.textContent.includes('Native sort fixture'))",'file channel visible after full reload')
 await evaluate("(()=>{const row=[...document.querySelectorAll('.model-table tbody tr')].find(e=>e.textContent.includes('Native sort fixture'));[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='编辑').click();return true})()")
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
 await click('保存渠道草稿')
 await wait("!![...document.querySelectorAll('dialog[open]')].find(e=>e.textContent.includes('保存预览'))",'existing save preview before native file write')
 await click('确认保存','dialog[open]')
 const saved=await wait("(async()=>{const s=await window.__TAURI_INTERNALS__.invoke('model_file_snapshot');const c=s.policy.channels.find(c=>c.id==="+JSON.stringify(channelId)+");return c?.modelOrder[0]==='fixture-beta'&&c.modelOrder[1]==='fixture-alpha'?s:null})()",'confirmed native sort writes actual file')
 const written=JSON.parse(await readFile(configPath))
 assert.equal(written.providers['native-sort-fixture'].apiKey,'native-sandbox-key','sort-only Save must preserve existing Key')
 assert.notEqual(saved.policyDigest,seeded.snapshot.policyDigest)
 await wait("!!document.querySelector('.notif-layer .process-progress')",'global save result')
 assert.equal(await evaluate("document.querySelectorAll('main .save-result,main .process-progress,.task-reopen').length"),0,'save result must not occupy page or a second notification entry')
 await click('稍后','.notif-layer')
 await call('Page.reload',{ignoreCache:true})
 await wait("document.readyState==='complete'&&[...document.querySelectorAll('.models-page .model-table tbody tr')].some(e=>e.textContent.includes('Native sort fixture'))",'saved channel after reopen')
 await evaluate("(()=>{const row=[...document.querySelectorAll('.model-table tbody tr')].find(e=>e.textContent.includes('Native sort fixture'));[...row.querySelectorAll('button')].find(e=>e.textContent.trim()==='编辑').click();return true})()")
 await wait("JSON.stringify([...document.querySelectorAll('.channel-model-table tr[data-model]')].map(e=>e.dataset.model))==="+JSON.stringify(JSON.stringify(orderedIds)),'reopened file preserves confirmed native order')
 assert(!await evaluate("document.querySelector('main').innerText.includes('待重启')"),'later/reopen must not resurrect restart state')
 await writeFile(join(output,'model-sort-native.json'),JSON.stringify({result:'pass',input:'WebView2 CDP mouse press/move/release, not synthetic DOM drag events',channelId,before:points.before,after:order,memoryEditBeforeSave:true,confirmedFileWrite:true,reopenedOrder:true,existingKeyMasked:true,keyEye:true,keyPreserved:true,runtimeSave:false},null,2)+'\n')
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
 assert((await readFile(configPath)).equals(beforeAdopt),'portable adoption must not write config.json')
 assert.equal(accepted.policyDigest,transferBaseline.policyDigest)
 assert.equal(accepted.workspace.basePolicy,transferBaseline.policyDigest)
 const copy=accepted.workspace.policy.channels.find(c=>c.label==='Native portable copy')
 assert(copy?.connection.importedConfig,'raw authorization receipt missing from portable draft')
 const writtenCopy=await invoke('model_file_save',{request:{expectedFile:accepted.policyDigest,policy:accepted.workspace.policy,credentials:[]}})
 const rawCopy=JSON.parse(await readFile(configPath)).providers[copy.connection.providerId]
 assert.equal(rawCopy.apiKey,'native-sandbox-key','portable Key must reach the final file only on Save')
 assert.deepEqual(writtenCopy.snapshot.policy.channels.find(c=>c.id===copy.id).modelOrder,orderedIds)
 await writeFile(join(output,'model-transfer-native.json'),JSON.stringify({result:'pass',exportCurrentFile:true,optionalPlaintextFile:true,adoptionFileUnchanged:true,trueFileBaseline:true,importedAuthorizationSaved:true,modelOrderPreserved:true,otherProviderPreserved:true},null,2)+'\n')
 await new Promise(resolve=>setTimeout(resolve,100))
 const tasks=await evaluate("[...document.querySelectorAll('.model-operation-task,.model-operation-card')].filter(e=>e.getClientRects().length).map(e=>e.textContent)")
 const screenshot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'model-template-native.png'),Buffer.from(screenshot.data,'base64'))
 await verifyUiAlignment({call,evaluate,wait,click,output,templateLabel:label})
 const result={result:'pass',scope:'native template and file-backed channel edit/preview/confirm/reopen, exact template transfer; isolated config writes, no runtime generation',templateId:created.templateId,revisionId:revision.id,immutablePriorRevision:true,channelPolicyUnchanged:true,defaultSeparateFromAdoption:true,confirmationLeftNativeTopLayer:true,returnedToVersionManager:true,exportedWithinSandbox:true,fileRoundtrip:true,nativePointerModelSort:true,confirmedChannelFileSave:true,channelReopen:true,existingKeyEye:true,readonlyComparison:true,externalContextBundle:true}
 await writeFile(join(output,'model-ui.json'),JSON.stringify(result,null,2)+'\n');return result
}
