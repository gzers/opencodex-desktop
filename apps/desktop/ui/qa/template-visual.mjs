// 模版详情实屏（WP-54）：进入模版编辑页，核对侧栏只有三项操作、匹配预览在匹配分区。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
import {pickEngine, Blocked} from './engine.mjs'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-template-shots-'))
fs.mkdirSync(outDir, { recursive: true })
const PORT = 5199, BASE = 'http://127.0.0.1:' + PORT
function pw(){const n=path.join(os.homedir(),'.npm','_npx');if(fs.existsSync(n))for(const d of fs.readdirSync(n)){const c=path.join(n,d,'node_modules','playwright-core');if(fs.existsSync(path.join(c,'package.json')))return c}return null}
function exe(){const b=path.join(os.homedir(),'Library','Caches','ms-playwright');if(!fs.existsSync(b))return null;for(const d of fs.readdirSync(b).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()){const p=path.join(b,d,'chrome-headless-shell-mac-arm64','chrome-headless-shell');if(fs.existsSync(p))return p}return null}
const engine=await pickEngine()
const waitPort=(port,t=40000)=>new Promise((res,rej)=>{const s=Date.now();const tick=()=>{const k=net.connect(port,'127.0.0.1');k.once('connect',()=>{k.destroy();res()});k.once('error',()=>{k.destroy();if(Date.now()-s>t)rej(new Error('vite'));else setTimeout(tick,200)})};tick()})
const results=[]
const check=(n,ok,d='')=>results.push({n,ok:!!ok,d})
async function main(){
  const server=spawn(path.resolve(uiDir,'node_modules','.bin','vite'),['--host','127.0.0.1','--port',String(PORT),'--strictPort'],{cwd:uiDir,stdio:['ignore','pipe','pipe']})
  process.on('exit',()=>{try{server.kill('SIGTERM')}catch{}})
  let browser
  try{
    await waitPort(PORT)
    browser=await engine.launch()
    for(const c of [{name:'template-1180x760-light',w:1180,h:760,theme:'light'},{name:'template-900x600-dark',w:900,h:600,theme:'dark'}]){
      const page=await browser.newPage({viewport:{width:c.w,height:c.h},deviceScaleFactor:2})
      const errors=[]
      page.on('console',m=>{if(m.type()==='error')errors.push(m.text())})
      page.on('pageerror',e=>errors.push('pageerror: '+e.message))
      await page.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme='+c.theme)
      await page.waitForSelector('#app .app-window',{timeout:20000}).catch(()=>{})
      await page.waitForTimeout(1000)
      // 切到模型模版，再点第一条模版的「编辑」进入详情。
      const tabs=page.locator('button[role=tab]', { hasText: '模型模版' }).first()
      if(await tabs.count()) await tabs.click().catch(()=>{})
      await page.waitForTimeout(900)
      // 精确定位模版行内的「编辑」，避免命中别的同名按钮。
      const row = page.locator('tbody tr', { hasText: 'GPT' }).first()
      const editBtn = row.locator('button', { hasText: '编辑' }).first()
      if (await editBtn.count()) await editBtn.click()
      await page.waitForSelector('.detail-navigation', { timeout: 10000 }).catch(()=>{})
      await page.waitForTimeout(1000)
      const info=await page.evaluate(()=>{
        const text=document.body.innerText
        const nav=document.querySelector('.detail-navigation')
        const navText=nav?nav.innerText:''
        const matching=document.querySelector('#model-matching')
        return {
          editorOpen: !!nav && text.includes('基本信息'),
          navHasAgent: navText.includes('Agent自动填充') || navText.includes('Agent 自动填充'),
          navHasExternal: navText.includes('外部任务'),
          navHasCopy: navText.includes('复制模版'),
          navHasPreview: navText.includes('匹配预览'),
          previewInMatching: !!(matching && matching.innerText.includes('匹配预览')),
          navHasSave: navText.includes('保存'),
          sections: Array.from(document.querySelectorAll('.detail-workspace-content section[id]')).length,
          saveDisabled: (() => { const b = Array.from(document.querySelectorAll('.detail-navigation button')).find(x => x.innerText.trim() === '保存'); return b ? b.disabled : null })(),
          resetDisabled: (() => { const b = Array.from(document.querySelectorAll('.detail-navigation button')).find(x => x.innerText.trim() === '还原修改'); return b ? b.disabled : null })(),
          overflow: Math.max(0, document.documentElement.scrollWidth - document.documentElement.clientWidth)
        }
      })
      check(c.name+' 模版编辑器已打开',info.editorOpen)
      check(c.name+' 侧栏含 Agent 自动填充',info.navHasAgent)
      check(c.name+' 侧栏含外部任务',info.navHasExternal)
      check(c.name+' 侧栏含复制模版',info.navHasCopy)
      check(c.name+' 侧栏不再有匹配预览',!info.navHasPreview)
      check(c.name+' 匹配预览在匹配分区',info.previewInMatching)
      check(c.name+' 侧栏保留保存',info.navHasSave)
      check(c.name+' 分区数量正确',info.sections>=5,'sections='+info.sections)
      check(c.name+' 无横向溢出',info.overflow<=1,'overflow='+info.overflow)
      check(c.name+' 无未捕获错误',errors.length===0,errors.slice(0,2).join(' | '))
      // 未修改时保存与还原应为禁用（真实禁用态，不是靠配色装作可用）。
      check(c.name+' 未修改时保存禁用',info.saveDisabled)
      check(c.name+' 未修改时还原禁用',info.resetDisabled)
      // 真实修改后两键必须恢复可用，否则用户改完无法保存。
      const desc = page.locator('#model-basic textarea').first()
      if (await desc.count()) await desc.fill('实屏验证：模版说明修改')
      await page.waitForTimeout(700)
      const dirty = await page.evaluate(() => {
        const nav = document.querySelector('.detail-navigation')
        const find = t => Array.from(nav.querySelectorAll('button')).find(b => b.innerText.trim() === t)
        return { save: find('保存')?.disabled ?? null, reset: find('还原修改')?.disabled ?? null }
      })
      check(c.name+' 修改后保存可用',dirty.save === false)
      check(c.name+' 修改后还原可用',dirty.reset === false)
      await page.screenshot({path:path.join(outDir,c.name+'.png'),fullPage:false})
      await page.close()
    }
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  fs.writeFileSync(path.join(outDir,'template-report.json'),JSON.stringify({total:results.length,failed:failed.length,results},null,2))
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{if(e instanceof Blocked){console.log('BLOCKED: '+e.message);process.exit(2)}console.error(e);process.exit(3)})

