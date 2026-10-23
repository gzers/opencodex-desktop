// 导出确认页实屏（WP-56 / WP-58）：加载模型夹具后切到文件同步页，验证导出界面。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-export-shots-'))
fs.mkdirSync(outDir, { recursive: true })
const PORT = 5199, BASE = 'http://127.0.0.1:' + PORT
function pw(){const n=path.join(os.homedir(),'.npm','_npx');if(fs.existsSync(n))for(const d of fs.readdirSync(n)){const c=path.join(n,d,'node_modules','playwright-core');if(fs.existsSync(path.join(c,'package.json')))return c}return null}
function exe(){const b=path.join(os.homedir(),'Library','Caches','ms-playwright');if(!fs.existsSync(b))return null;for(const d of fs.readdirSync(b).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()){const p=path.join(b,d,'chrome-headless-shell-mac-arm64','chrome-headless-shell');if(fs.existsSync(p))return p}return null}
const {chromium}=createRequire(pw()+'/')('playwright-core')
const waitPort=(port,t=40000)=>new Promise((res,rej)=>{const s=Date.now();const tick=()=>{const k=net.connect(port,'127.0.0.1');k.once('connect',()=>{k.destroy();res()});k.once('error',()=>{k.destroy();if(Date.now()-s>t)rej(new Error('vite'));else setTimeout(tick,200)})};tick()})
const results=[]
const check=(n,ok,d='')=>results.push({n,ok:!!ok,d})
async function main(){
  const server=spawn(path.resolve(uiDir,'node_modules','.bin','vite'),['--host','127.0.0.1','--port',String(PORT),'--strictPort'],{cwd:uiDir,stdio:['ignore','pipe','pipe']})
  process.on('exit',()=>{try{server.kill('SIGTERM')}catch{}})
  let browser
  try{
    await waitPort(PORT)
    browser=await chromium.launch({executablePath:exe()})
    for(const c of [{name:'export-1180x760-light',w:1180,h:760,theme:'light'},{name:'export-900x600-dark',w:900,h:600,theme:'dark'}]){
      const page=await browser.newPage({viewport:{width:c.w,height:c.h},deviceScaleFactor:2})
      const errors=[]
      page.on('console',m=>{if(m.type()==='error')errors.push(m.text())})
      page.on('pageerror',e=>errors.push('pageerror: '+e.message))
      // 先加载模型夹具（route=models），再切到同步页，夹具保持已安装。
      await page.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme='+c.theme)
      await page.waitForSelector('#app .app-window',{timeout:20000}).catch(()=>{})
      await page.waitForTimeout(1200)
      await page.evaluate(()=>{location.hash='#sync?tab=file'})
      await page.waitForTimeout(1500)
      // 打开导出确认：先展开资源分组并完成复核，走真实交互而不是绕过校验。
      const expanders = page.locator('[data-testid="sync-scope-workspace"] summary, [data-testid="sync-scope-workspace"] .scope-group-toggle')
      const groupCount = await expanders.count()
      for (let i = 0; i < groupCount; i++) { await expanders.nth(i).click().catch(()=>{}) }
      const reviewBox = page.locator('input[type=checkbox][aria-label*="复核"], label:has-text("已复核") input')
      if (await reviewBox.count()) await reviewBox.first().check().catch(()=>{})
      await page.waitForTimeout(600)
      const exportButton = page.locator('button', { hasText: '导出配置' }).first()
      const disabled = await exportButton.isDisabled().catch(()=>true)
      check(c.name + ' 导出按钮可点', !disabled)
      if (!disabled) await exportButton.click().catch(()=>{})
      await page.waitForSelector('dialog[open], .ui-dialog[open], .transfer-review', { timeout: 8000 }).catch(()=>{})
      await page.waitForTimeout(1200)
      const info=await page.evaluate(()=>{
        const text=document.body.innerText
        return {
          hasSync: text.includes('导入与导出'),
          hasExportButton: !!Array.from(document.querySelectorAll('button')).find(b=>b.innerText.trim()==='导出配置'),
          hasScope: !!document.querySelector('[data-testid="sync-scope-workspace"]'),
          hasOldSaveLocation: text.includes('未选择目录时保存到默认导出目录'),
          hasDatePackage: text.includes('日期 / 单次包') || text.includes('每次导出生成一个独立备份包'),
          dialogOpen: !!(document.querySelector('dialog[open]') || document.querySelector('.ui-dialog[open]') || document.querySelector('.transfer-review')),
          overflow: Math.max(0, document.documentElement.scrollWidth - document.documentElement.clientWidth)
        }
      })
      check(c.name+' 同步页已加载',info.hasSync)
      check(c.name+' 导出配置入口存在',info.hasExportButton)
      check(c.name+' 范围工作区存在',info.hasScope)
      check(c.name+' 旧保存位置文案已移除',!info.hasOldSaveLocation)
      check(c.name+' 导出确认已打开',info.dialogOpen)
      check(c.name+' 备份包说明已就位',info.hasDatePackage)
      check(c.name+' 无横向溢出',info.overflow<=1,'overflow='+info.overflow)
      check(c.name+' 无未捕获错误',errors.length===0,errors.slice(0,2).join(' | '))
      await page.screenshot({path:path.join(outDir,c.name+'.png'),fullPage:false})
      await page.close()
    }
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  fs.writeFileSync(path.join(outDir,'export-report.json'),JSON.stringify({total:results.length,failed:failed.length,results},null,2))
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{console.error(e);process.exit(3)})

