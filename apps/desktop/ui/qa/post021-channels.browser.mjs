import fs from 'node:fs/promises'
import path from 'node:path'
import {pickEngine} from './engine.mjs'
const out=process.env.AUDIT_OUT,base=process.env.AUDIT_BASE||'http://127.0.0.1:53972'
if(!out)throw Error('AUDIT_OUT required')
await fs.mkdir(out,{recursive:true})
const engine=await pickEngine(),browser=await engine.launch(),checks=[],errors=[]
function check(name,passed){checks.push({name,passed:!!passed});if(!passed)throw Error(name)}
try{
 for(const theme of ['light','dark']){
  const page=await browser.newPage({viewport:{width:1180,height:900}})
  page.on('pageerror',e=>errors.push(e.message))
  await page.route('**/*',r=>new URL(r.request().url()).origin===base?r.continue():r.abort())
  await page.goto(base+'/?__audit=1&route=models&env=ready&runtime=running&prefs=local&effects=low&theme='+theme)
  await page.waitForSelector('.channel-table')
  await page.evaluate(async()=>{
   (await import('/src/dev/modelFixtures.ts')).installModelFixtures()
   const {modelApi}=await import('/src/features/models/api.ts')
   window.__discoveryCalls=[]
   modelApi.previewNewChannel=async request=>{window.__discoveryCalls.push(request);return {models:[{id:'vendor/browser-model',label:'vendor/browser-model',rules:{}}],credentialRef:null,elapsedMs:1}}
  })
  await page.getByRole('button',{name:'新增渠道',exact:true}).click()
  const dialog=page.locator('dialog[open]')
  check(theme+' no second ID',await dialog.getByLabel('运行渠道标识',{exact:true}).count()===0)
  check(theme+' no private permission',!(await dialog.innerText()).includes('允许访问可信私有网络'))
  await dialog.getByLabel('渠道名称',{exact:true}).fill('中文 渠道 / one')
  await dialog.getByLabel('服务基地址',{exact:true}).fill('http://127.0.0.1:11434/v1')
  await dialog.getByLabel('API Key',{exact:true}).fill('synthetic-browser-key')
  await dialog.screenshot({path:path.join(out,engine.name+'-'+theme+'-new-name-only.png')})
  await dialog.getByRole('button',{name:'下一步',exact:true}).click()
  await dialog.getByRole('button',{name:'获取模型列表',exact:true}).click()
  await dialog.getByLabel('选择新渠道型号 vendor/browser-model',{exact:true}).check()
  check(theme+' request has no private gate',await page.evaluate(()=>!Object.hasOwn(window.__discoveryCalls[0],'allowPrivateNetwork')))
  await dialog.getByRole('button',{name:'加入草稿',exact:true}).click();await dialog.waitFor({state:'hidden'})
  check(theme+' editor no ID',await page.getByLabel('运行渠道标识',{exact:true}).count()===0)
  await page.getByLabel('渠道名称',{exact:true}).fill('openai')
  check(theme+' reserved display name valid',await page.locator('#channel-rename-error').count()===0)
  const state=await page.evaluate(async()=>{const store=(await import('/src/features/models/store.ts')).useModelsStore();const c=store.channelDraft.channels.at(-1);return {label:c.label,id:c.connection.providerId,selection:c.selection}})
  check(theme+' internal key allocated',state.label==='openai'&&state.id.startsWith('channel-')&&state.selection.models[0]==='vendor/browser-model')
  await page.screenshot({path:path.join(out,engine.name+'-'+theme+'-rename.png')})
  await page.close()
 }
}finally{await browser.close();await fs.writeFile(path.join(out,'browser-'+engine.name+'.json'),JSON.stringify({engine:engine.name,scope:'DEV fixtures, no native IPC/real network; actual Vue new and rename flows',checks,errors},null,2))}
if(errors.length||checks.some(x=>!x.passed))throw Error(JSON.stringify({errors,checks}))
console.log(JSON.stringify({engine:engine.name,passed:checks.length}))
