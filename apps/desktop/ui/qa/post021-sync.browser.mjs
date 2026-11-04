// DEV-only UI integration. No native IPC, daily configuration, or external requests.
import fs from 'node:fs/promises'
import path from 'node:path'
import {pickEngine} from './engine.mjs'
const out=process.env.AUDIT_OUT,base=process.env.AUDIT_BASE||'http://127.0.0.1:53971'
if(!out)throw Error('AUDIT_OUT required')
await fs.mkdir(out,{recursive:true})
const engine=await pickEngine(),browser=await engine.launch(),checks=[],errors=[]
function check(name,passed){checks.push({name,passed:!!passed});if(!passed)throw Error(name)}
try{
 for(const spec of [{width:1440,height:900,theme:'light',effects:'low'},{width:1180,height:760,theme:'dark',effects:'mid'}]){
  const page=await browser.newPage({viewport:{width:spec.width,height:spec.height}}),prefix=engine.name+'-'+spec.theme
  page.on('pageerror',e=>errors.push(e.message))
  await page.route('**/*',route=>new URL(route.request().url()).origin===base?route.continue():route.abort())
  await page.goto(base+'/?__audit=1&route=models&env=ready&runtime=running&prefs=local&theme='+spec.theme+'&effects='+spec.effects)
  await page.waitForSelector('.channel-table')
  await page.evaluate(async()=>{
   (await import('/src/dev/modelFixtures.ts')).installModelFixtures()
   const api=await import('/src/features/sync/api.ts'),sync=(await import('/src/features/sync/store.ts')).useSyncStore()
   sync.adoptConfig(await api.syncEndpointApi.load())
   ;(await import('/src/stores/routes.ts')).useRouteStore().go('overview')
  })
  const card=page.locator('.mods .mod').last(),version=page.locator('[data-testid=overview-remote-version]')
  await version.waitFor()
  for(const state of ['unknown','known','absent','failed']){
   await page.evaluate(async state=>{
    const api=await import('/src/features/sync/api.ts')
    api.remoteVersionApi.read=async()=>{if(state==='failed')throw Error('isolated read failure');return {remoteExists:state!=='absent',remoteRevision:state==='absent'?null:'fixture-version-'+state,remoteSavedAt:state==='known'?'2026-11-04T04:00:00Z':null}}
   },state)
   await card.getByRole('button',{name:'检测版本',exact:true}).click()
   const expected={unknown:'已读取 · 保存时间未知',known:'上次观察',absent:'未发现快照',failed:'读取失败'}[state]
   await page.waitForFunction(expected=>document.querySelector('[data-testid=overview-remote-version]')?.textContent.includes(expected),expected)
   check(prefix+' '+state+' distinct version state',(await version.innerText()).includes(expected))
   await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))))
   await card.screenshot({path:path.join(out,prefix+'-'+state+'.png')})
  }
  await page.evaluate(async()=>{const api=await import('/src/features/sync/api.ts');api.remoteVersionApi.read=async()=>({remoteExists:true,remoteRevision:'fixture-retry',remoteSavedAt:null})})
  await card.getByRole('button',{name:'检测版本',exact:true}).click()
  await page.waitForFunction(()=>document.querySelector('[data-testid=overview-remote-version]')?.textContent.includes('已读取 · 保存时间未知'))
  await page.evaluate(async()=>{
   const api=await import('/src/features/sync/modelTransfer.ts')
   api.modelTransferApi.checkSync=async()=>({source:'unchanged',reason:'same_content',hasRemote:true,remoteRevision:'fixture-same',remoteSavedAt:null})
   api.modelTransferApi.prepareUpload=async()=>{throw Error('unexpected upload')}
   api.modelTransferApi.download=async()=>{throw Error('unexpected download')}
  })
  await card.getByRole('button',{name:'同步',exact:true}).click()
  const dialog=page.locator('dialog.transfer-review[open]');await dialog.waitFor()
  check(prefix+' same content opens no-write preview',(await dialog.innerText()).includes('所选内容已一致'))
  await page.keyboard.press('Escape');await dialog.waitFor({state:'hidden'})
  check(prefix+' check is not a transfer',(await card.innerText()).includes('最近检查一致'))
  check(prefix+' null saved time not unread',(await version.innerText()).includes('已读取 · 保存时间未知'))
  await page.evaluate(async()=>{(await import('/src/stores/routes.ts')).useRouteStore().go('sync',{tab:'webdav'})})
  await page.locator('.sync-connection-summary small').waitFor()
  check(prefix+' connection slash',(await page.locator('.sync-connection-summary small').innerText())==='https://dav.example.test/config-sync')
  await page.close()
 }
}finally{await browser.close();await fs.writeFile(path.join(out,'browser-'+engine.name+'.json'),JSON.stringify({engine:engine.name,scope:'explicit DEV fixtures; production Vue stores/routes; no native IPC or real sync',checks,errors},null,2))}
if(errors.length||checks.some(c=>!c.passed))throw Error(JSON.stringify({errors,checks}))
console.log(JSON.stringify({engine:engine.name,passed:checks.length}))
