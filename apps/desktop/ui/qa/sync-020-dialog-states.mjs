// Explicit DEV-only fixtures: no native IPC, real server or daily configuration.
import fs from 'node:fs/promises'
import path from 'node:path'
import {pickEngine} from './engine.mjs'
const out=process.env.AUDIT_OUT
if(!out)throw Error('AUDIT_OUT required')
await fs.mkdir(out,{recursive:true})
const engine=await pickEngine(),browser=await engine.launch(),checks=[],errors=[],screens=[]
const check=(name,passed)=>{checks.push({name,passed:!!passed});if(!passed)throw Error(name)}
try{
 for(const viewport of [{width:1180,height:760},{width:900,height:600}]){
  const p=await browser.newPage({viewport})
  p.on('pageerror',e=>errors.push(e.message))
  await p.route('**/*',r=>new URL(r.request().url()).hostname==='127.0.0.1'?r.continue():r.abort())
  const capture=async(name)=>{const file=viewport.width+'-'+name+'.png';await p.screenshot({path:path.join(out,file)});screens.push(file)}
  await p.goto('http://127.0.0.1:52422/?__audit=1&route=models&env=ready&runtime=running&importProtected=1&effects=low&theme=light')
  await p.waitForSelector('.channel-table')
  await p.evaluate(async()=>{
   const {modelTransferApi}=await import('/src/features/sync/modelTransfer.ts')
   const accept=modelTransferApi.accept;window.__qaAcceptCalls=0
   modelTransferApi.accept=async(...args)=>{window.__qaAcceptCalls++;return accept(...args)}
   ;(await import('/src/stores/routes.ts')).useRouteStore().go('sync',{tab:'file'})
  })
  await p.waitForSelector('.sync-scope-credentials')
  await p.getByRole('button',{name:'导入配置',exact:true}).click()
  const dialog=p.locator('dialog[open].transfer-review')
  await dialog.waitFor()
  check('empty import cannot read '+viewport.width,await dialog.getByRole('button',{name:'读取配置',exact:true}).isDisabled())
  check('one shared import dialog '+viewport.width,await p.locator('dialog[open]').count()===1)
  await capture('import-empty')
  await dialog.getByRole('button',{name:'选择文件',exact:true}).click()
  await dialog.getByLabel('导入解密口令',{exact:true}).waitFor()
  check('protected file stays in shared dialog '+viewport.width,await p.locator('dialog[open]').count()===1)
  check('empty password cannot read '+viewport.width,await dialog.getByRole('button',{name:'读取配置',exact:true}).isDisabled())
  await dialog.getByLabel('导入解密口令',{exact:true}).fill('wrong-fixture')
  await dialog.getByRole('button',{name:'读取配置',exact:true}).click()
  await dialog.locator('[role=alert]').waitFor()
  check('wrong password preserves dialog '+viewport.width,await dialog.isVisible())
  check('wrong password never adopts '+viewport.width,await p.evaluate(()=>window.__qaAcceptCalls===0))
  await capture('import-wrong-password')
  await dialog.getByLabel('导入解密口令',{exact:true}).fill('fixture-password')
  await dialog.getByRole('button',{name:'读取配置',exact:true}).click()
  await dialog.getByRole('button',{name:'确认更新',exact:true}).waitFor()
  check('unlocked file shows same shared dialog '+viewport.width,await p.locator('dialog[open]').count()===1)
  check('backup enabled before import '+viewport.width,await dialog.getByLabel('更新前备份',{exact:true}).isChecked())
  check('preview never adopts '+viewport.width,await p.evaluate(()=>window.__qaAcceptCalls===0))
  await capture('import-preview')
  await p.keyboard.press('Escape')
  await p.waitForFunction(()=>!document.querySelector('dialog[open].transfer-review'))
  check('cancel never adopts '+viewport.width,await p.evaluate(()=>window.__qaAcceptCalls===0))
  check('cancel preserves source route '+viewport.width,await p.evaluate(()=>location.hash.startsWith('#sync')))
  await p.getByRole('button',{name:'导入配置',exact:true}).click()
  await dialog.waitFor()
  check('reopen clears prior password '+viewport.width,await dialog.getByLabel('导入解密口令',{exact:true}).count()===0)
  check('reopen clears prior filename '+viewport.width,await dialog.getByText('尚未选择文件',{exact:true}).isVisible())
  await dialog.getByRole('button',{name:'取消',exact:true}).click()
  await p.close()
 }
}finally{
 await browser.close()
 await fs.writeFile(path.join(out,'report.json'),JSON.stringify({engine:engine.name,scope:'DEV fixtures; browser import empty/protected/error/preview/cancel states; no real IPC/config/file import',prototype:'cfbd90c4b+1da4cabca',checks,screens,errors},null,2))
}
if(errors.length)throw Error(JSON.stringify(errors))
console.log(engine.name,checks.length,'import-dialog state checks passed;',screens.length,'screens')
