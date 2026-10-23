// 更新中心结果页实屏（WP-63）：注入 IPC 桩复现「有可用更新」，检查一屏看完与两级入口。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-update-shots-'))
fs.mkdirSync(outDir, { recursive: true })
const PORT = 5199, BASE = 'http://127.0.0.1:' + PORT
function pw(){const n=path.join(os.homedir(),'.npm','_npx');if(fs.existsSync(n))for(const d of fs.readdirSync(n)){const c=path.join(n,d,'node_modules','playwright-core');if(fs.existsSync(path.join(c,'package.json')))return c}return null}
function exe(){const b=path.join(os.homedir(),'Library','Caches','ms-playwright');if(!fs.existsSync(b))return null;for(const d of fs.readdirSync(b).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()){const p=path.join(b,d,'chrome-headless-shell-mac-arm64','chrome-headless-shell');if(fs.existsSync(p))return p}return null}
const {chromium}=createRequire(pw()+'/')('playwright-core')
const waitPort=(port,t=40000)=>new Promise((res,rej)=>{const s=Date.now();const tick=()=>{const k=net.connect(port,'127.0.0.1');k.once('connect',()=>{k.destroy();res()});k.once('error',()=>{k.destroy();if(Date.now()-s>t)rej(new Error('vite'));else setTimeout(tick,200)})};tick()})

// 用独立挂载页（qa/update-harness）只渲染更新中心，避免完整应用启动期的其它依赖。
const results=[]
const check=(n,ok,d='')=>results.push({n,ok:!!ok,d})
async function main(){
  const server=spawn(path.resolve(uiDir,'node_modules','.bin','vite'),['--host','127.0.0.1','--port',String(PORT),'--strictPort'],{cwd:uiDir,stdio:['ignore','pipe','pipe']})
  process.on('exit',()=>{try{server.kill('SIGTERM')}catch{}})
  let browser
  try{
    await waitPort(PORT)
    browser=await chromium.launch({executablePath:exe()})
    for(const c of [{name:'update-1180x760-light',w:1180,h:760,theme:'light'},{name:'update-900x600-dark',w:900,h:600,theme:'dark'}]){
      const page=await browser.newPage({viewport:{width:c.w,height:c.h},deviceScaleFactor:2})
      const errors=[]
      page.on('console',m=>{if(m.type()==='error')errors.push(m.text())})
      page.on('pageerror',e=>errors.push('pageerror: '+e.message))
      await page.goto(BASE+'/qa/update-harness/index.html?theme='+c.theme)
      await page.waitForSelector('[data-testid="update-center"]',{timeout:20000}).catch(()=>{})
      await page.waitForTimeout(1500)
      if (c.theme === 'dark') await page.evaluate(() => document.documentElement.dataset.theme = 'dark')
      const info=await page.evaluate(()=>{
        const root=document.querySelector('[data-testid="update-center"]')
        if(!root) return {open:false}
        const text=root.innerText
        const body=root.querySelector('.update-center-body')
        const rows=root.querySelectorAll('.update-result-row').length
        const box=root.querySelector('.update-center')?.getBoundingClientRect()
        return {
          open:true,
          title:text.includes('检查更新结果'),
          rows,
          hasDetailButtons:[...root.querySelectorAll('button')].filter(b=>b.innerText.trim()==='查看详情').length,
          hasInstallOnResult:[...root.querySelectorAll('button')].some(b=>/^确认更新$|^备份并更新面板$|^安装/.test(b.innerText.trim())),
          hasCheckboxOnResult:root.querySelectorAll('input[type=checkbox]').length,
          hasNotesDisclosure:[...root.querySelectorAll('.update-result-row details')].length,
          bodyScroll: body ? { client: body.clientHeight, scroll: body.scrollHeight } : null,
          footerButtons:[...root.querySelectorAll('.update-center-foot button')].map(b=>b.innerText.trim()),
          box: box ? {w:Math.round(box.width),h:Math.round(box.height),top:Math.round(box.top),bottom:Math.round(box.bottom)}:{},
          viewportH: window.innerHeight
        }
      })
      check(c.name+' 结果页已打开',info.open)
      if(info.open){
        check(c.name+' 标题为检查更新结果',info.title)
        check(c.name+' 每个对象一行',info.rows===2,'rows='+info.rows)
        check(c.name+' 只留查看详情',info.hasDetailButtons===2,'detail='+info.hasDetailButtons)
        check(c.name+' 结果页无安装按钮',!info.hasInstallOnResult)
        check(c.name+' 结果页无备份勾选',info.hasCheckboxOnResult===0,'checkbox='+info.hasCheckboxOnResult)
        check(c.name+' 结果页无更新内容折叠',info.hasNotesDisclosure===0,'details='+info.hasNotesDisclosure)
        check(c.name+' 正文一屏不滚动',!!info.bodyScroll&&info.bodyScroll.scroll<=info.bodyScroll.client+1,JSON.stringify(info.bodyScroll))
        check(c.name+' 底部只有关闭',info.footerButtons.length===1&&info.footerButtons[0]==='关闭','footer='+JSON.stringify(info.footerButtons))
        check(c.name+' 弹窗不越界',info.box.top>=0&&info.box.bottom<=info.viewportH+1,JSON.stringify(info.box))
      }
      check(c.name+' 无未捕获错误',errors.length===0,errors.slice(0,2).join(' | '))
      await page.screenshot({path:path.join(outDir,c.name+'.png'),fullPage:false})
      await page.close()
    }
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  fs.writeFileSync(path.join(outDir,'update-report.json'),JSON.stringify({total:results.length,failed:failed.length,results},null,2))
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{console.error(e);process.exit(3)})

