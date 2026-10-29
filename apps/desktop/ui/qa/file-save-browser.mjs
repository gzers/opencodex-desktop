// Production Rust file writer; isolated temp data, RPC 5223 / Vite 5216.
// Run only after the current-channel store/UI contract lands.
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import {spawn} from 'node:child_process'
import {createServer} from 'node:net'
import {fileURLToPath} from 'node:url'
import {pickEngine} from './engine.mjs'
const ui=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..'),tauri=path.resolve(ui,'../tauri')
const out=process.env.AUDIT_OUT_DIR||'/tmp/ocx-v216-file-browser';fs.mkdirSync(out,{recursive:true})
const logs=[],checks=[]
function check(name,ok){checks.push({name,ok:!!ok});assert.ok(ok,name)}
// Never connect to or shut down another listener occupying these ports.
await Promise.all([5216,5223].map(port=>new Promise((resolve,reject)=>{
 const probe=createServer();probe.once('error',error=>reject(Error('isolated fixture port '+port+' unavailable: '+error.message)))
 probe.listen({host:'127.0.0.1',port,exclusive:true},()=>probe.close(error=>error?reject(error):resolve()))
})))
const rust=spawn('cargo',['test','--test','model_file_browser','file_browser_fixture','--','--ignored','--nocapture'],{cwd:tauri,env:{...process.env,OCX_FILE_BROWSER_FIXTURE:'1'},stdio:['ignore','pipe','pipe']})
rust.stdout.on('data',x=>logs.push(String(x)));rust.stderr.on('data',x=>logs.push(String(x)))
const vite=spawn(path.join(ui,'node_modules/.bin/vite'),['--host','127.0.0.1','--port','5216','--strictPort'],{cwd:ui,stdio:'ignore'})
async function ready(url){for(let i=0;i<3000;i++){if(rust.exitCode!==null||vite.exitCode!==null)throw Error('isolated fixture exited early');try{if((await fetch(url)).ok)return}catch{}await new Promise(r=>setTimeout(r,100))}throw Error('fixture startup timed out')}
async function exited(child){if(child.exitCode!==null||child.signalCode!==null)return;await new Promise(resolve=>{const timer=setTimeout(done,2000);function done(){clearTimeout(timer);child.off('close',done);resolve()}child.once('close',done)})}
async function rpc(command,args={}){const r=await(await fetch('http://127.0.0.1:5223/rpc',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({command,args})})).json();if(r.error)throw r.error;return r.value}
let browser,fixtureReady=false
try{
 await Promise.all([ready('http://127.0.0.1:5223/ready'),ready('http://127.0.0.1:5216/')]);fixtureReady=true
 const engine=await pickEngine();browser=await engine.launch()
 const page=await browser.newPage({viewport:{width:1180,height:760}}),errors=[];page.on('pageerror',e=>errors.push(e.message))
 await page.goto('http://127.0.0.1:5216/?__audit=1&route=models&env=ready&runtime=running&effects=low&theme=dark');await page.waitForSelector('.channel-table')
 await page.evaluate(async()=>{
  const {modelApi}=await import('/src/features/models/api.ts'),{useModelsStore}=await import('/src/features/models/store.ts')
  window.__fileRpc=async(command,args={})=>{const r=await(await fetch('http://127.0.0.1:5223/rpc',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({command,args})})).json();if(r.error)throw r.error;return r.value}
  window.__fileSaves=[]
  modelApi.load=()=>window.__fileRpc('snapshot');modelApi.credential=providerId=>window.__fileRpc('credential',{providerId});modelApi.project=policy=>window.__fileRpc('project',{policy})
  modelApi.writeFile=(expectedFile,policy,credentials,scope)=>{
   window.__fileSaves.push({scope:scope??null,credentialChannels:credentials.map(e=>e.channelId)})
   return window.__fileRpc('save',{expectedFile,policy,credentials,...(scope?{scope}:{})})
  }
  useModelsStore().acceptTransfer(await modelApi.load())
 })
 const preview=page.getByRole('dialog',{name:'保存预览',exact:true}),detailSave=page.getByRole('button',{name:'保存渠道草稿',exact:true}),key=page.locator('input[aria-label="API Key"]')
 async function channelId(providerId){return page.evaluate(async providerId=>(await import('/src/features/models/store.ts')).useModelsStore().channelDraft.channels.find(c=>c.connection.providerId===providerId).id,providerId)}
 async function openChannel(providerId,expectedKey){
  const id=await channelId(providerId)
  await page.locator('.channel-table tr[data-channel="'+id+'"]').getByRole('button',{name:'编辑',exact:true}).click()
  await page.waitForFunction(value=>{const input=document.querySelector('input[aria-label="API Key"]');return input&&!input.disabled&&input.value===value},expectedKey)
  return id
 }
 async function back(){await page.getByRole('button',{name:'返回渠道列表',exact:true}).click();await page.waitForSelector('.channel-table')}
 async function memory(id){return page.evaluate(async id=>{const s=(await import('/src/features/models/store.ts')).useModelsStore();return {channel:s.channelDraft.channels.find(c=>c.id===id),rows:s.modelRows[id],credential:s.credentialEdits[id]}},id)}
 async function directSave(id){
  const count=await page.evaluate(()=>window.__fileSaves.length)
  await detailSave.click()
  await page.waitForFunction(count=>window.__fileSaves.length===count+1&&!!document.querySelector('.channel-table'),count)
  check('detail Save returns to list without full Preview',await preview.count()===0)
  const request=await page.evaluate(()=>window.__fileSaves.at(-1))
  assert.deepEqual(request.scope,{channelId:id},'detail RPC must carry camelCase scope.channelId')
  check('detail RPC carries only current-channel credentials',request.credentialChannels.every(channel=>channel===id))
  await page.waitForSelector('.notif-layer .process-progress')
 }
 async function later(){await page.getByRole('button',{name:'稍后',exact:true}).click()}
 const initial=await rpc('disk')
 check('file contains demo and two existing scope channels',initial.providerCount===3)
 check('clean list Save is disabled',await page.getByTestId('policy-save').isDisabled())
 const a=await openChannel('fixture-a','fixture-a-original-key')
 check('clean detail Save is disabled',await detailSave.isDisabled())
 await page.getByLabel('渠道名称',{exact:true}).fill('Fixture A unsaved');await key.fill('fixture-a-pending-key')
 await page.locator('tr[data-model="site/model-a"]').getByRole('button',{name:'展开编辑',exact:true}).click()
 await page.getByLabel('site/model-a行编辑',{exact:true}).getByLabel('上下文窗口（tokens）',{exact:true}).fill('96000')
 await page.waitForSelector('tr[data-model="site/model-a"][data-pending="true"]');await back()
 const retainedA=await memory(a)
 check('A name, Key and pending model row stay in memory',retainedA.channel.label==='Fixture A unsaved'&&retainedA.credential.value==='fixture-a-pending-key'&&retainedA.rows.buffers['site/model-a'].rules.contextWindow.value===96000)
 check('returning from A leaves file bytes unchanged',(await rpc('disk')).configText===initial.configText)
 const b=await openChannel('fixture-b','fixture-b-original-key')
 check('clean B Save is disabled while A is dirty',await detailSave.isDisabled())
 await page.getByLabel('渠道名称',{exact:true}).fill('Fixture B saved');await key.fill('fixture-b-written-key');await directSave(b)
 const savedB=await rpc('disk')
 check('B detail Save actually writes B Key',savedB.providers['fixture-b'].apiKey==='fixture-b-written-key')
 assert.notDeepEqual(savedB.providers['fixture-b'],initial.providers['fixture-b'])
 assert.deepEqual(savedB.providers['fixture-a'],initial.providers['fixture-a'],'B scope must leave A raw provider untouched')
 assert.deepEqual(savedB.providers.demo,initial.providers.demo,'B scope preserves unrelated Key and fields')
 assert.deepEqual(await memory(a),retainedA,'B Save retains A name, Key and pending row exactly')
 check('B scope creates one prewrite backup',savedB.backups===initial.backups+1)
 check('one global result, no bottom placeholder',await page.locator('.notif-layer .process-progress').count()===1&&await page.locator('main .process-progress,.save-result,.task-reopen').count()===0)
 await page.screenshot({path:path.join(out,'scoped-b-dark.png')});await later();await openChannel('fixture-b','fixture-b-written-key')
 check('reopened B is clean with written name and masked Key',await detailSave.isDisabled()&&await key.getAttribute('type')==='password'&&await page.getByLabel('渠道名称',{exact:true}).inputValue()==='Fixture B saved')
 await page.screenshot({path:path.join(out,'scoped-b-reopened-dark.png')})
 await back();await openChannel('fixture-a','fixture-a-pending-key')
 check('reopened A is dirty with pending row',await detailSave.isEnabled()&&await page.locator('tr[data-model="site/model-a"][data-pending="true"]').count()===1)
 assert.deepEqual(await memory(a),retainedA)
 await page.screenshot({path:path.join(out,'scoped-a-retained-dark.png')})
 // Confirm just the row; the channel still awaits list Save.
 await page.getByLabel('site/model-a行编辑',{exact:true}).getByRole('button',{name:'加入草稿',exact:true}).click()
 await page.waitForSelector('tr[data-model="site/model-a"][data-pending="false"]');await back()
 await page.getByTestId('policy-save').click();await preview.waitFor()
 check('list Save retains global Preview with both channels',(await preview.textContent()).includes('Fixture A unsaved')&&(await preview.textContent()).includes('Fixture B saved'))
 await page.screenshot({path:path.join(out,'remaining-preview-dark.png')})
 await preview.getByRole('button',{name:'取消',exact:true}).click();await preview.waitFor({state:'hidden'})
 check('canceling list Preview cannot undo B or write A',(await rpc('disk')).configText===savedB.configText)
 check('canceling Preview retains remaining A Key',await page.evaluate(async id=>(await import('/src/features/models/store.ts')).useModelsStore().credentialEdits[id]?.value==='fixture-a-pending-key',a))
 await page.getByTestId('policy-save').click();await preview.waitFor();await preview.getByRole('button',{name:'确认保存',exact:true}).click();await page.waitForSelector('.notif-layer .process-progress')
 const savedA=await rpc('disk')
 check('remaining confirmation writes A Key and confirmed row',savedA.providers['fixture-a'].apiKey==='fixture-a-pending-key'&&savedA.providers['fixture-a'].modelContextWindows['site/model-a']===96000)
 assert.deepEqual(savedA.providers['fixture-b'],savedB.providers['fixture-b'],'remaining Save preserves already-saved B')
 check('list RPC uses global scope',await page.evaluate(()=>window.__fileSaves.at(-1).scope===null));await later()
 check('no remaining changes disables list Save',await page.getByTestId('policy-save').isDisabled())
 await page.screenshot({path:path.join(out,'remaining-saved-dark.png')})
 // Existing Key-only, eye, empty-keeps and explicit-clear regressions.
 const demo=await openChannel('demo','fixture-original-key')
 check('existing Key loaded masked',await key.getAttribute('type')==='password')
 await page.getByRole('button',{name:'显示 API Key',exact:true}).click();check('eye reveals saved Key',await key.getAttribute('type')==='text')
 await page.getByRole('button',{name:'隐藏 API Key',exact:true}).click();check('eye alone leaves Save disabled',await detailSave.isDisabled())
 await key.fill('fixture-browser-replacement');check('Key-only edit enables detail Save',await detailSave.isEnabled())
 const beforeKey=await rpc('disk');await directSave(demo)
 const disk=await rpc('disk')
 check('Key-only direct Save reached actual config.json',disk.key==='fixture-browser-replacement')
 check('unrelated raw field and exact custom ID remain',disk.unknown.keep===42&&disk.models[0]==='site/custom-id')
 check('Key-only Save creates one prewrite backup',disk.backups===beforeKey.backups+1)
 assert.deepEqual(disk.providers['fixture-a'],savedA.providers['fixture-a']);assert.deepEqual(disk.providers['fixture-b'],savedB.providers['fixture-b'])
 await page.screenshot({path:path.join(out,'saved-dark.png')});await later()
 await page.evaluate(async()=>{await (await import('/src/features/models/store.ts')).useModelsStore().load()});await openChannel('demo','fixture-browser-replacement')
 check('reopened editor reads written Key masked and clean',await key.getAttribute('type')==='password'&&await detailSave.isDisabled())
 check('no restart/check state resurrected',!((await page.locator('main').innerText()).includes('待重启')))
 await key.fill('temporary-key-input');await key.fill('');check('empty input alone keeps Save disabled',await detailSave.isDisabled())
 // A separate name edit exercises the real writer with an empty Key field.
 await page.getByLabel('渠道名称',{exact:true}).fill('Demo empty keep fixture');await directSave(demo)
 check('empty input keeps saved Key',(await rpc('disk')).key==='fixture-browser-replacement')
 await later();await openChannel('demo','fixture-browser-replacement')
 await page.getByRole('button',{name:'清空 Key',exact:true}).click();await directSave(demo)
 check('explicit clear persists deletion',!(await rpc('disk')).key);await later()
 const transfer=await rpc('prepareTransfer'),beforeTransfer=await rpc('disk')
 check('export uses current file despite damaged legacy policy/draft',transfer.sourceModels.includes('site/exact-import'))
 await page.evaluate(async demo=>{
  const {modelTransferApi}=await import('/src/features/sync/modelTransfer.ts')
  modelTransferApi.channelCatalog=()=>window.__fileRpc('catalog');modelTransferApi.accountCatalog=async()=>[];modelTransferApi.extensionCatalog=async()=>({skills:[],mcps:[]})
  modelTransferApi.receive=()=>window.__fileRpc('receive');modelTransferApi.select=(id,scope)=>window.__fileRpc('select',{id,scope});modelTransferApi.accept=id=>window.__fileRpc('accept',{id})
  modelTransferApi.latestIncoming=async()=>null
  const previous=window.__TAURI_INTERNALS__?.invoke
  window.__TAURI_INTERNALS__={...window.__TAURI_INTERNALS__,invoke:async(command,args)=>command==='plugin:dialog|open'?'/fixture-transfer.ocxdconf':previous?previous(command,args):Promise.reject('fixture command unavailable')}
  const models=(await import('/src/features/models/store.ts')).useModelsStore();models.view.channelEditor=''
  models.channelDraft.channels.find(c=>c.id===demo).label='Local unsaved label';models.editCredential(demo,'fixture-local-pending-key','')
  ;(await import('/src/stores/routes.ts')).useRouteStore().go('overview')
 },demo)
 await page.getByRole('button',{name:'导入配置',exact:true}).click()
 const picker=page.getByRole('dialog',{name:'导入配置',exact:true});await picker.waitFor()
 check('overview import stays on overview and requires a selected file',await page.locator('.ov').isVisible()&&await picker.getByRole('button',{name:'读取并比较',exact:true}).isDisabled())
 await picker.getByRole('button',{name:'选择文件',exact:true}).click()
 await picker.getByRole('button',{name:'读取并比较',exact:true}).click()
 const comparison=page.getByRole('dialog',{name:'比较配置文件',exact:true});await comparison.waitFor()
 check('receiving and comparing leaves complete file unchanged',(await rpc('disk')).configText===beforeTransfer.configText)
 await comparison.getByRole('button',{name:'比较所选范围',exact:true}).click()
 await comparison.getByRole('button',{name:'采用到本机',exact:true}).click();await comparison.waitFor({state:'hidden'})
 check('adopting imported channels leaves file unchanged',(await rpc('disk')).configText===beforeTransfer.configText)
 check('adoption retains independent channel and Key input',await page.evaluate(async demo=>{const s=(await import('/src/features/models/store.ts')).useModelsStore();return s.channelDraft.channels.find(c=>c.id===demo).label==='Local unsaved label'&&s.credentialEdits[demo]?.value==='fixture-local-pending-key'&&s.channelBaseline===s.snapshot.policyDigest},demo))
 await page.evaluate(async()=>{(await import('/src/stores/routes.ts')).useRouteStore().go('models')})
 await page.getByTestId('policy-save').click();await preview.waitFor()
 await preview.locator('button.preview-group-title').filter({hasText:'import-fixture'}).click()
 await page.waitForFunction(()=>!![...document.querySelectorAll('.selector-preview .preview-item')].find(e=>e.textContent.includes('site/exact-import')))
 check('global Preview contains imported exact model',(await preview.textContent()).includes('site/exact-import'))
 await preview.getByRole('button',{name:'确认保存',exact:true}).click()
 await page.waitForFunction(async count=>(await window.__fileRpc('disk')).providerCount===count+1,beforeTransfer.providerCount)
 const importedDisk=await rpc('disk')
 check('confirmed Save writes imported raw fields and local Key',importedDisk.imported.note==='current-file-source'&&importedDisk.imported.retryOn429===false&&importedDisk.key==='fixture-local-pending-key')
 assert.deepEqual(importedDisk.providers['fixture-a'],beforeTransfer.providers['fixture-a']);assert.deepEqual(importedDisk.providers['fixture-b'],beforeTransfer.providers['fixture-b'])
 await later();await page.evaluate(async()=>{await (await import('/src/features/models/store.ts')).useModelsStore().load()})
 check('reopen reads imported channel and original opaque field',(await page.locator('.channel-table').innerText()).includes('import-fixture')&&(await rpc('disk')).unknown.keep===42)
 await page.screenshot({path:path.join(out,'imported-dark.png')})
 check('no browser exceptions',errors.length===0);await page.close()
 console.log(JSON.stringify({engine:engine.name,passed:checks.filter(x=>x.ok).length,failed:checks.filter(x=>!x.ok).length,out}))
}finally{
 if(browser)await browser.close()
 if(fixtureReady){try{await rpc('finish')}catch{}await exited(rust)}
 vite.kill('SIGTERM');if(rust.exitCode===null&&rust.signalCode===null)rust.kill('SIGTERM')
 await Promise.all([exited(vite),exited(rust)])
 fs.writeFileSync(path.join(out,'report.json'),JSON.stringify({adapter:'production Rust config_file via temporary loopback fixture',checks},null,2));fs.writeFileSync(path.join(out,'backend.log'),logs.join(''))
}
