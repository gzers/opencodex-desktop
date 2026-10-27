// Actual production Rust file writer via an isolated loopback test fixture.
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
import {spawn} from 'node:child_process'
import {fileURLToPath} from 'node:url'
import {pickEngine} from './engine.mjs'
const ui=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..'),tauri=path.resolve(ui,'../tauri')
const out=process.env.AUDIT_OUT_DIR||'/tmp/ocx-v14-file-browser';fs.mkdirSync(out,{recursive:true})
const logs=[];const checks=[]
function check(name,ok){checks.push({name,ok:!!ok});assert.ok(ok,name)}
const rust=spawn('cargo',['test','--test','model_file_browser','file_browser_fixture','--','--ignored','--nocapture'],{cwd:tauri,env:{...process.env,OCX_FILE_BROWSER_FIXTURE:'1'},stdio:['ignore','pipe','pipe']})
rust.stdout.on('data',x=>logs.push(String(x)));rust.stderr.on('data',x=>logs.push(String(x)))
const vite=spawn(path.join(ui,'node_modules/.bin/vite'),['--host','127.0.0.1','--port','5216','--strictPort'],{cwd:ui,stdio:'ignore'})
async function ready(url){for(let i=0;i<3000;i++){try{if((await fetch(url)).ok)return}catch{}if(rust.exitCode!==null)throw Error('Rust fixture exited early');await new Promise(r=>setTimeout(r,100))}throw Error('fixture startup timed out')}
async function rpc(command,args={}){const r=await(await fetch('http://127.0.0.1:5223/rpc',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({command,args})})).json();if(r.error)throw r.error;return r.value}
let browser
try{
 await Promise.all([ready('http://127.0.0.1:5223/ready'),ready('http://127.0.0.1:5216/')]);const engine=await pickEngine();browser=await engine.launch()
 const page=await browser.newPage({viewport:{width:1180,height:760}});const errors=[];page.on('pageerror',e=>errors.push(e.message))
 await page.goto('http://127.0.0.1:5216/?__audit=1&route=models&env=ready&runtime=running&effects=low&theme=dark');await page.waitForSelector('.channel-table')
 await page.evaluate(async()=>{
  const {modelApi}=await import('/src/features/models/api.ts');const {useModelsStore}=await import('/src/features/models/store.ts')
  window.__fileRpc=async(command,args={})=>{const r=await(await fetch('http://127.0.0.1:5223/rpc',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({command,args})})).json();if(r.error)throw r.error;return r.value}
  modelApi.load=()=>window.__fileRpc('snapshot');modelApi.credential=providerId=>window.__fileRpc('credential',{providerId});modelApi.project=policy=>window.__fileRpc('project',{policy});modelApi.writeFile=(expectedFile,policy,credentials)=>window.__fileRpc('save',{expectedFile,policy,credentials})
  useModelsStore().acceptTransfer(await modelApi.load())
 })
 check('actual file model appears in the channel table',(await page.locator('.channel-table').innerText()).includes('demo'))
 await page.locator('.channel-table button').filter({hasText:'编辑'}).first().click();await page.waitForFunction(()=>document.querySelector('input[aria-label="API Key"]')?.value==='fixture-original-key')
 const key=page.locator('input[aria-label="API Key"]');check('existing Key loaded masked',await key.getAttribute('type')==='password')
 await page.getByRole('button',{name:'显示 API Key',exact:true}).click();check('eye reveals the current saved Key',await key.getAttribute('type')==='text')
 await key.fill('fixture-browser-replacement');await page.getByRole('button',{name:'保存渠道草稿',exact:true}).click()
 const preview=page.getByRole('dialog',{name:'保存预览',exact:true});await preview.waitFor();check('preview includes exact custom ID',(await preview.textContent()).includes('site/custom-id'))
 await preview.getByRole('button',{name:'确认保存',exact:true}).click();await page.waitForSelector('.notif-layer .process-progress')
 const disk=await rpc('disk');check('confirmed UI save reached actual config.json',disk.key==='fixture-browser-replacement');check('unrelated raw field and model remain',disk.unknown.keep===42&&disk.models[0]==='site/custom-id');check('one prewrite backup exists',disk.backups===1)
 check('one global result, no bottom placeholder',await page.locator('.notif-layer .process-progress').count()===1&&await page.locator('main .process-progress,.save-result,.task-reopen').count()===0)
 await page.screenshot({path:path.join(out,'saved-dark.png')});await page.getByRole('button',{name:'稍后',exact:true}).click()
 await page.evaluate(async()=>{const s=(await import('/src/features/models/store.ts')).useModelsStore();s.view.channelEditor='';await s.load()})
 await page.locator('.channel-table button').filter({hasText:'编辑'}).first().click();await page.waitForFunction(()=>document.querySelector('input[aria-label="API Key"]')?.value==='fixture-browser-replacement')
 check('reopened editor reads the written Key masked',await key.getAttribute('type')==='password');check('no restart/check state resurrected',!((await page.locator('main').innerText()).includes('待重启')))
 await key.fill('');await page.getByRole('button',{name:'保存渠道草稿',exact:true}).click();await preview.waitFor();await preview.getByRole('button',{name:'确认保存',exact:true}).click();await page.waitForTimeout(250)
 check('empty input keeps saved Key',(await rpc('disk')).key==='fixture-browser-replacement')
 await page.getByRole('button',{name:'清空 Key',exact:true}).click();await page.getByRole('button',{name:'保存渠道草稿',exact:true}).click();await preview.waitFor();await preview.getByRole('button',{name:'确认保存',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.notif-layer .process-progress'))
 check('explicit clear persists deletion',!(await rpc('disk')).key)
 await page.getByRole('button',{name:'稍后',exact:true}).click()
 const transfer=await rpc('prepareTransfer')
 check('export uses current file despite damaged legacy policy/draft',transfer.sourceModels.includes('site/exact-import'))
 await page.evaluate(async()=>{
  const {modelTransferApi}=await import('/src/features/sync/modelTransfer.ts')
  modelTransferApi.channelCatalog=()=>window.__fileRpc('catalog');modelTransferApi.accountCatalog=async()=>[];modelTransferApi.extensionCatalog=async()=>({skills:[],mcps:[]})
  modelTransferApi.receive=()=>window.__fileRpc('receive');modelTransferApi.select=(id,scope)=>window.__fileRpc('select',{id,scope});modelTransferApi.accept=id=>window.__fileRpc('accept',{id})
  modelTransferApi.latestIncoming=async()=>null
  const previous=window.__TAURI_INTERNALS__?.invoke
  window.__TAURI_INTERNALS__={...window.__TAURI_INTERNALS__,invoke:async(command,args)=>command==='plugin:dialog|open'?'/fixture-transfer.ocxdconf':previous?previous(command,args):Promise.reject('fixture command unavailable')}
  const models=(await import('/src/features/models/store.ts')).useModelsStore();models.view.channelEditor=''
  models.channelDraft.channels[0].label='Local unsaved label';models.editCredential(models.channelDraft.channels[0].id,'fixture-local-pending-key','')
  ;(await import('/src/stores/routes.ts')).useRouteStore().go('sync')
 })
 await page.getByRole('button',{name:'导入配置',exact:true}).click()
 const comparison=page.getByRole('dialog',{name:'比较配置文件',exact:true});await comparison.waitFor()
 check('receiving and comparing does not write config.json',(await rpc('disk')).providerCount===1)
 await comparison.getByRole('button',{name:'比较所选范围',exact:true}).click()
 await comparison.getByRole('button',{name:'采用到本机',exact:true}).click();await comparison.waitFor({state:'hidden'})
 check('adopting imported channels still leaves file unchanged',(await rpc('disk')).providerCount===1)
 check('adoption retains independent channel and Key input',await page.evaluate(async()=>{const s=(await import('/src/features/models/store.ts')).useModelsStore();return s.channelDraft.channels[0].label==='Local unsaved label'&&Object.values(s.credentialEdits).some(e=>e.value==='fixture-local-pending-key')&&s.channelBaseline===s.snapshot.policyDigest}))
 await page.evaluate(async()=>{(await import('/src/stores/routes.ts')).useRouteStore().go('models')})
 await page.getByTestId('policy-save').click();await preview.waitFor()
 await preview.locator('button.preview-group-title').filter({hasText:'import-fixture'}).click()
 await page.waitForTimeout(250)
 check('the existing save preview contains the imported exact model',(await preview.textContent()).includes('site/exact-import'))
 await preview.getByRole('button',{name:'确认保存',exact:true}).click()
 await page.waitForFunction(async()=>{const s=await window.__fileRpc('disk');return s.providerCount===2})
 const importedDisk=await rpc('disk')
 check('confirmed Save writes imported raw fields and preserved local Key',importedDisk.imported.note==='current-file-source'&&importedDisk.imported.retryOn429===false&&importedDisk.key==='fixture-local-pending-key')
 await page.getByRole('button',{name:'稍后',exact:true}).click()
 await page.evaluate(async()=>{await (await import('/src/features/models/store.ts')).useModelsStore().load()})
 check('reopen reads imported channel and original opaque field',(await page.locator('.channel-table').innerText()).includes('import-fixture')&&(await rpc('disk')).unknown.keep===42)
 await page.screenshot({path:path.join(out,'imported-dark.png')})
 check('no browser exceptions',errors.length===0);await page.close()
 console.log(JSON.stringify({engine:engine.name,passed:checks.filter(x=>x.ok).length,failed:checks.filter(x=>!x.ok).length,out}))
}finally{
 if(browser)await browser.close();try{await rpc('finish')}catch{}vite.kill('SIGTERM');if(rust.exitCode===null)rust.kill('SIGTERM');fs.writeFileSync(path.join(out,'report.json'),JSON.stringify({adapter:'production Rust config_file via temporary loopback fixture',checks},null,2));fs.writeFileSync(path.join(out,'backend.log'),logs.join(''))
}
