import fs from 'node:fs/promises';import path from 'node:path';import {fileURLToPath,pathToFileURL} from 'node:url'
import {pickEngine} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/qa/engine.mjs'
const out=path.dirname(fileURLToPath(import.meta.url)),source=JSON.parse(await fs.readFile(path.join(out,'prototype-adoption.json'),'utf8')),engine=await pickEngine(),browser=await engine.launch(),checks=[],errors=[]
const check=(name,passed)=>{checks.push({name,passed:!!passed});if(!passed)throw Error(name)}
try{
 const page=await browser.newPage({viewport:{width:1559,height:1100}});page.on('pageerror',e=>errors.push(e.message));await page.route('https://**/*',r=>r.abort())
 await page.goto(pathToFileURL(source.entry).href+'#models');await page.locator('[data-mp=new-channel]').waitFor();await page.locator('[data-mp=new-channel]').click()
 check('new channel has no runtime-ID input',await page.locator('#mpChannelId').count()===0)
 await page.locator('#mpChannelName').fill('中文 渠道 / one');await page.locator('#mpChannelURL').fill('http://127.0.0.1:11434/v1');await page.locator('#mpChannelKey').fill('mock-only-key')
 await page.locator('#modal').screenshot({path:path.join(out,'prototype-'+engine.name+'-new.png')});await page.locator('#modalConfirm').click()
 await page.locator('[data-mp=channel-fetch]').click();await page.waitForFunction(()=>document.querySelector('#modalBody')?.textContent.includes('已获取'))
 check('local discovery without permission control',!(await page.locator('#modalBody').innerText()).includes('私有网络'))
 await page.locator('[data-mp=channel-select]').click();await page.locator('#modalConfirm').click()
 await page.locator('#mpChannelInline-name').waitFor();check('detail has no second identifier',await page.locator('#mpChannelInline-providerId').count()===0)
 await page.locator('#mpChannelInline-name').fill('openai');check('reserved word display name valid',(await page.locator('#mpChannelRenameError').innerText())==='')
 await page.screenshot({path:path.join(out,'prototype-'+engine.name+'-rename.png')})
}finally{await browser.close();await fs.writeFile(path.join(out,'prototype-'+engine.name+'.json'),JSON.stringify({engine:engine.name,checks,errors,scope:'read-only adopted prototype snapshot; memory mock only'},null,2))}
if(errors.length||checks.some(c=>!c.passed))throw Error(JSON.stringify({errors,checks}));console.log(JSON.stringify({engine:engine.name,passed:checks.length}))
