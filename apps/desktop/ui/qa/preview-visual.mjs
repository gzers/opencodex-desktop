// 保存预览弹窗实屏（§38）：点击保存入口后截图并断言关键结构。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
import {pickEngine, Blocked} from './engine.mjs'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.AUDIT_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-preview-shots-'))
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
    for(const c of [{name:'preview-1180x760-light',w:1180,h:760,theme:'light'},{name:'preview-1180x760-dark',w:1180,h:760,theme:'dark'},{name:'preview-900x600-light',w:900,h:600,theme:'light'}]){
      const page=await browser.newPage({viewport:{width:c.w,height:c.h},deviceScaleFactor:2})
      const errors=[]
      page.on('console',m=>{if(m.type()==='error')errors.push(m.text())})
      page.on('pageerror',e=>errors.push('pageerror: '+e.message))
      await page.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme='+c.theme)
      await page.waitForSelector('#app .app-window',{timeout:20000}).catch(()=>{})
      await page.waitForTimeout(1200)
      // 先制造一次草稿修改，让预览里出现「新 / 停」内联变化。
      const toggle=page.locator('.channel-enable-cell button, .channel-enable-cell [role=switch]').first()
      if(await toggle.count()) await toggle.click().catch(()=>{})
      await page.waitForTimeout(400)
      const save=page.locator('button[aria-label="保存渠道草稿"], button:has-text("保存")').first()
      await save.click().catch(()=>{})
      await page.waitForSelector('dialog[open], .ui-dialog[open]',{timeout:8000}).catch(()=>{})
      await page.waitForTimeout(900)
      const info=await page.evaluate(()=>{
        const dlg=document.querySelector('dialog[open]')||document.querySelector('.ui-dialog[open]')
        const text=dlg?dlg.innerText:''
        const groups=dlg?dlg.querySelectorAll('.preview-group').length:0
        const items=dlg?dlg.querySelectorAll('.preview-item').length:0
        const box=dlg?dlg.getBoundingClientRect():null
        const order=dlg?.querySelector('.selector-preview-order')
        return {
          open:!!dlg, title:text.split('\n')[0]||'',
          hasSource:text.includes('预览来源'), hasCounts:text.includes('按渠道查看数量'),
          hasChangeTree:text.includes('本次变更与写入'), groups, items,
          box:box?{w:Math.round(box.width),h:Math.round(box.height),top:Math.round(box.top),bottom:Math.round(box.bottom)}:null,
          orderScroll:order?{client:order.clientHeight,scroll:order.scrollHeight}:null,
          viewportH:window.innerHeight, bodyOverflow:Math.max(0,document.documentElement.scrollWidth-document.documentElement.clientWidth)
        }
      })
      check(c.name+' 弹窗打开',info.open,'title='+info.title)
      check(c.name+' 标题为保存预览',info.title.includes('保存预览'),info.title)
      check(c.name+' 无预览来源',!info.hasSource)
      check(c.name+' 无按渠道查看数量',!info.hasCounts)
      check(c.name+' 无变更差异树',!info.hasChangeTree)
      check(c.name+' 渠道卡渲染',info.groups>0,'groups='+info.groups)
      check(c.name+' 型号标签渲染',info.items>0,'items='+info.items)
      check(c.name+' 弹窗不越界',!!info.box&&info.box.top>=0&&info.box.bottom<=info.viewportH+1,JSON.stringify(info.box))
      // §38.1：宽度上限 980px；窄窗按可用宽度收缩，不得超过视口。
      if(info.box) check(c.name+' 宽度按规格',info.box.w<=980&&info.box.w<=c.w, 'w='+info.box.w)
      check(c.name+' 无未捕获错误',errors.length===0,errors.slice(0,2).join(' | '))
      await page.screenshot({path:path.join(outDir,c.name+'.png'),fullPage:false})
      await page.close()
    }
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  fs.writeFileSync(path.join(outDir,'preview-report.json'),JSON.stringify({total:results.length,failed:failed.length,results},null,2))
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{if(e instanceof Blocked){console.log('BLOCKED: '+e.message);process.exit(2)}console.error(e);process.exit(3)})

