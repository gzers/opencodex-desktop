// Explicit 023 native boundary: actual WebView2 DOM and production V2 IPC.
// Shared detailed matcher/transfer/editor behavior is verified by the complete
// same-source suites; this platform probe checks paths, schema, transactions,
// picker projection, wizard cancellation, and source ownership on Windows.
import assert from 'node:assert/strict'
import {win32,join} from 'node:path'
import {readFile,writeFile,access} from 'node:fs/promises'
const normalized=p=>win32.resolve(p.replace(/^\\\\\?\\/, '')).toLowerCase()
export function assert023Paths(facts,sandbox){
 assert.equal(facts.about.platform,'Windows');assert.equal(facts.about.version,'0.2.23')
 const aggregate=win32.join(sandbox,'data'),desktop=win32.join(aggregate,'desktop'),home=win32.join(aggregate,'opencodex/home')
 for(const field of ['currentDataRoot','activeDataRoot'])assert.equal(normalized(facts.dataRoot[field]),normalized(desktop),field)
 for(const field of ['currentOpencodexHome','opencodexHome'])assert.equal(normalized(facts.dataRoot[field]),normalized(home),field)
 assert.equal(normalized(facts.dataRoot.aggregateRoot),normalized(aggregate));assert.equal(facts.dataRoot.runtimeActive,true)
 return {aggregate,desktop,home,config:win32.join(home,'config.json')}
}
export async function verify023Native(call,output,facts,sandbox){
 const paths=assert023Paths(facts,sandbox),checks=[]
 async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
 const invoke=(name,args={})=>evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(name)+','+JSON.stringify(args)+')')
 const sleep=()=>new Promise(r=>setTimeout(r,100))
 async function wait(expression){for(let i=0;i<100;i++){const v=await evaluate(expression);if(v)return v;await sleep()}throw Error('023 native DOM did not reach expected state: '+expression)}
 async function click(text,root='document'){assert(await evaluate('(()=>{const root='+root+';const b=[...root.querySelectorAll("button,a,summary")].find(x=>x.textContent.trim()==='+JSON.stringify(text)+'&&!x.disabled&&x.getClientRects().length);if(!b)return false;b.click();return true})()'),'Missing visible native action: '+text)}
 const before=await readFile(paths.config).catch(e=>{if(e.code==='ENOENT')return null;throw e})
 await click('模型');await wait('!!document.querySelector(".channel-list-toolbar")')
 await wait('[...document.querySelectorAll(".channel-list-toolbar button")].some(b=>b.textContent.trim()==="新增渠道"&&!b.disabled)')
 await click('新增渠道');await wait('[...document.querySelectorAll("dialog[open] h2")].some(h=>h.textContent.includes("新增渠道 · 连接配置"))')
 assert.equal(await evaluate('!!document.querySelector("dialog[open] .channel-wizard-steps")'),true)
 assert.equal(await evaluate('[...document.querySelectorAll("dialog[open] label")].some(e=>e.textContent.includes("默认协议"))'),false)
 await click('取消','document.querySelector("dialog[open]")');await wait('!document.querySelector("dialog[open]")')
 assert.deepEqual(await readFile(paths.config).catch(e=>{if(e.code==='ENOENT')return null;throw e}),before)
 checks.push('actual 023 two-step wizard opens/cancels without config write or channel protocol')
 let snapshot=await invoke('model_v2_file_snapshot');assert.equal(snapshot.dtoVersion,2);assert.equal(snapshot.library.schemaVersion,2)
 const library=JSON.stringify(snapshot.library),id='gpt-6.1-sol'
 snapshot=await invoke('model_v2_stage_connection',{request:{expected:snapshot.draft?.base??null,expectedSource:snapshot.sourceDigest,expectedLibrary:snapshot.libraryDigest,channelId:null,name:'Native023',url:'https://native023.invalid/v1',authMode:'none',apiKey:null,rawModelIds:[id],selectedModelIds:[id]}})
 const channel=snapshot.draft.policy.channels.find(c=>c.label==='Native023');assert(channel)
 assert.deepEqual(await readFile(paths.config).catch(e=>{if(e.code==='ENOENT')return null;throw e}),before)
 // Explicit per-model protocol is test input, not a hidden channel default.
 // Seed templates can intentionally inherit unknown protocol capabilities.
 const request={expected:snapshot.draft.base,edits:[{kind:'set_override',channelId:channel.id,rawModelId:id,field:'protocol',value:'openai-responses'}],channelIntents:[{kind:'set_enabled',channelId:channel.id,enabled:true}],scopeChannelId:null}
 const project=await invoke('model_v2_file_project',{request});assert.equal(project.intended.channels.find(c=>c.id===channel.id).label,'Native023')
 snapshot=await invoke('model_v2_file_save',{request});assert(!snapshot.draft)
 assert.equal(snapshot.savedPolicy.channels.find(c=>c.id===channel.id).label,'Native023')
 const firstRow=snapshot.savedPolicy.channels.find(c=>c.id===channel.id)
 const firstRaw=JSON.parse(await readFile(paths.config,'utf8'))
 assert(firstRaw.providers[firstRow.connection.providerId].models.includes(id),'explicitly enabled model must materialize in actual provider')
 assert.equal(firstRaw.providers[firstRow.connection.providerId].adapter,'openai-responses')
 assert(JSON.stringify(snapshot.library)===library,'channel writes must not rewrite immutable library')
 checks.push('stage and project do not save; explicit save commits real V2 config with original raw model ID')
 const initialSaved=await readFile(paths.config)
 snapshot=(await invoke('model_v2_read_existing',{request:{expectedSource:snapshot.sourceDigest,expectedLibrary:snapshot.libraryDigest,expectedDraft:null}})).snapshot
 const firstReadPolicy=JSON.stringify(snapshot.draft.policy)
 snapshot=(await invoke('model_v2_read_existing',{request:{expectedSource:snapshot.sourceDigest,expectedLibrary:snapshot.libraryDigest,expectedDraft:snapshot.draft.base}})).snapshot
 assert.equal(JSON.stringify(snapshot.draft.policy),firstReadPolicy)
 assert.deepEqual(await readFile(paths.config),initialSaved)
 snapshot=await invoke('model_v2_file_discard',{expected:snapshot.draft.base})
 assert.deepEqual(await readFile(paths.config),initialSaved)
 checks.push('reading existing saved channel matches templates into draft, repeated read is idempotent and cancellation preserves config')
 for(const name of ['Native023 renamed','本机/渠道 023']){
  snapshot=await invoke('model_v2_begin_draft',{request:{expectedSource:snapshot.sourceDigest,expectedLibrary:snapshot.libraryDigest,expectedDraft:snapshot.draft?.base??null}})
  snapshot=await invoke('model_v2_file_save',{request:{expected:snapshot.draft.base,edits:[],channelIntents:[{kind:'rename',channelId:channel.id,name}],scopeChannelId:channel.id}})
  const row=snapshot.savedPolicy.channels.find(c=>c.id===channel.id);assert.equal(row.label,name);assert.equal(row.models[0].rawModelId,id)
  const raw=JSON.parse(await readFile(paths.config,'utf8'));assert(raw.providers[row.connection.providerId]);assert(raw.providers[row.connection.providerId].models.includes(id))
  const savedView=await invoke('model_v2_file_snapshot');assert.equal(savedView.savedPolicy.channels.find(c=>c.id===channel.id).label,name)
  assert.equal(raw.providers[row.connection.providerId].name,name)
  checks.push('actual provider name, stable saved identity and fresh snapshot: '+name)
 }
 const saved=await readFile(paths.config),reopened=await invoke('model_v2_file_snapshot');assert.equal(reopened.sourceDigest,snapshot.sourceDigest)
 snapshot=await invoke('model_v2_begin_draft',{request:{expectedSource:snapshot.sourceDigest,expectedLibrary:snapshot.libraryDigest,expectedDraft:null}})
 await invoke('model_v2_file_discard',{expected:snapshot.draft.base});assert.deepEqual(await readFile(paths.config),saved)
 assert.equal((await invoke('model_v2_file_snapshot')).draft,null)
 checks.push('reopen/discard preserves exact saved config bytes and stable channel identity')
 await assert.rejects(access(win32.join(sandbox,'opencodex-home')))
 const source=await invoke('runtime_source');assert(source&&typeof source==='object')
 const runtimeDirectory=await evaluate('(async()=>{try{return {ok:true,value:await window.__TAURI_INTERNALS__.invoke("model_v2_execution_directory")}}catch(error){return {ok:false,error}}})()')
 // A standalone sandbox intentionally lacks the original runtime. Preserve
 // its actual refusal; the normal complete-bundle port/catalog test owns live
 // execution-directory acceptance, never this source-only native probe.
 if(runtimeDirectory.ok)assert(runtimeDirectory.value.channels.some(c=>c.id===channel.id))
 else assert.equal(runtimeDirectory.error?.code,'storage')
 await click('更多');await click('刷新当前快照')
 await wait('[...document.querySelectorAll(".channel-model-table tbody tr,.model-table tbody tr")].some(r=>r.textContent.includes("本机/渠道 023"))')
 checks.push('actual channel table refresh renders the saved Unicode renamed channel')
 checks.push('new D/H paths and actual runtime source readback; no legacy sandbox HOME created')
 await writeFile(join(output,'model-native-ui.json'),JSON.stringify({result:'pass',version:'0.2.23',checks,configPath:paths.config,runtimeDirectory,liveRuntimeDirectoryAccepted:runtimeDirectory.ok,sharedCoverage:'same-source complete Rust and browser suites; no v1 API or old-path fallback',networkGeneration:false},null,2))
 const screenshot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'model023-native.png'),Buffer.from(screenshot.data,'base64'))
}
