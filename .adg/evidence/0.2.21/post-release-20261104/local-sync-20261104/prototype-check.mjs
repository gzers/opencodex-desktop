import fs from 'node:fs/promises'
import path from 'node:path'
import {fileURLToPath,pathToFileURL} from 'node:url'
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs'
const out=path.dirname(fileURLToPath(import.meta.url)),engine=await pickEngine(),browser=await engine.launch(),checks=[],errors=[]
const prototype=process.env.PROTOTYPE_FILE||'/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html'
try{
 const page=await browser.newPage({viewport:{width:1559,height:1100}})
 page.on('pageerror',e=>errors.push(e.message))
 await page.route('https://**/*',r=>r.abort())
 await page.goto(pathToFileURL(prototype).href+'#overview')
 await page.waitForFunction(()=>typeof window.post21SyncPreview==='function')
 await page.waitForTimeout(1000)
 await page.evaluate(()=>{window.setEnv?.('ready');window.setRoute?.('overview',true)})
 for(const state of ['unread','absent','failed','unknown','known','matched']){
  await page.evaluate(state=>window.post21SyncPreview(state),state)
  await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))))
  const card=page.locator('#card-overview-sync')
  const visible=await card.isVisible(),text=await card.innerText()
  const expected={unread:'尚未读取',absent:'未发现快照',failed:'读取失败',unknown:'已读取 · 保存时间未知',known:'上次观察',matched:'最近检查一致'}[state]
  checks.push({state,visible,expected,text,passed:visible&&text.includes(expected)})
  if(visible)await card.screenshot({path:path.join(out,'prototype-'+engine.name+'-'+state+'.png')})
 }
 const controls=await page.locator('[data-post21-version]').count();checks.push({name:'all seven review controls',passed:controls===7,controls})
}finally{await browser.close();await fs.writeFile(path.join(out,'prototype-'+engine.name+'.json'),JSON.stringify({engine:engine.name,scope:'mock-only applicable post021 state increment; working entry contains excluded future work',checks,errors},null,2))}
if(errors.length||checks.some(x=>!x.passed))throw Error(JSON.stringify({errors,failed:checks.filter(x=>!x.passed)}))
console.log(JSON.stringify({engine:engine.name,passed:checks.length}))
