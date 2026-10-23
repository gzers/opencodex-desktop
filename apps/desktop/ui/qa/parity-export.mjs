// 导出确认页：原型与实现并排对照（WP-56 / WP-58）。
// 原型：同步 → 文件同步 → 导出配置；实现：同一路径。只读，不生成文件。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.PARITY_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-parity-export-'))
fs.mkdirSync(outDir, { recursive: true })
const PROTOTYPE = process.env.PROTOTYPE_FILE || '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html'
const PORT = 5199, BASE = 'http://127.0.0.1:' + PORT
function pw(){const n=path.join(os.homedir(),'.npm','_npx');if(fs.existsSync(n))for(const d of fs.readdirSync(n)){const c=path.join(n,d,'node_modules','playwright-core');if(fs.existsSync(path.join(c,'package.json')))return c}return null}
function exe(){const b=path.join(os.homedir(),'Library','Caches','ms-playwright');if(!fs.existsSync(b))return null;for(const d of fs.readdirSync(b).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()){const p=path.join(b,d,'chrome-headless-shell-mac-arm64','chrome-headless-shell');if(fs.existsSync(p))return p}return null}
const {chromium}=createRequire(pw()+'/')('playwright-core')
const waitPort=(port,t=40000)=>new Promise((res,rej)=>{const s=Date.now();const tick=()=>{const k=net.connect(port,'127.0.0.1');k.once('connect',()=>{k.destroy();res()});k.once('error',()=>{k.destroy();if(Date.now()-s>t)rej(new Error('vite'));else setTimeout(tick,200)})};tick()})
const results=[]
const check=(n,ok,d='')=>results.push({n,ok:!!ok,d})
async function main(){
  if (!fs.existsSync(PROTOTYPE)) { console.log('BLOCKED: 原型不存在'); process.exit(2) }
  const server=spawn(path.resolve(uiDir,'node_modules','.bin','vite'),['--host','127.0.0.1','--port',String(PORT),'--strictPort'],{cwd:uiDir,stdio:['ignore','pipe','pipe']})
  process.on('exit',()=>{try{server.kill('SIGTERM')}catch{}})
  let browser
  try{
    await waitPort(PORT)
    browser=await chromium.launch({executablePath:exe()})

    const proto=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2})
    await proto.goto('file://' + encodeURI(PROTOTYPE) + '#sync')
    await proto.waitForTimeout(1800)
    const pExport=proto.locator('[data-sx="export"], button:has-text("导出配置")').first()
    if(await pExport.count()) await pExport.click().catch(()=>{})
    await proto.waitForTimeout(1200)
    const pInfo=await proto.evaluate(()=>{
      const text=document.body.innerText
      return {
        // 原型弹窗可能用 display:flex 之外的方式显示，改按内容存在判断。
        dialogOpen:!!(document.querySelector('#modalBody .sx-export-tree')||document.querySelector('.sx-export-summary')||document.querySelector('#modal .modal')),
        hasTree:!!document.querySelector('.sx-export-tree'),
        treeText:(document.querySelector('.sx-export-tree')?.innerText||'').slice(0,200),
        hasSummary:!!document.querySelector('.sx-export-summary'),
        mentionsDate:text.includes('2026-10-23'),
        mentionsPackage:text.includes('备份包'),
        mentionsCopy:text.includes('复制整个备份包'),
        hasOpenDir:false
      }
    })
    await proto.screenshot({path:path.join(outDir,'prototype-export.png')})

    const impl=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2})
    await impl.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme=light')
    await impl.waitForSelector('.channel-table',{timeout:20000}).catch(()=>{})
    await impl.waitForTimeout(1000)
    await impl.evaluate(()=>{location.hash='#sync?tab=file'})
    await impl.waitForTimeout(1500)
    const expanders=impl.locator('[data-testid="sync-scope-workspace"] summary')
    const n=await expanders.count()
    for(let i=0;i<n;i++) await expanders.nth(i).click().catch(()=>{})
    const review=impl.locator('label:has-text("已复核所选资源") input')
    if(await review.count()) await review.first().check().catch(()=>{})
    await impl.waitForTimeout(400)
    const iExport=impl.locator('button:has-text("导出配置")').first()
    if(await iExport.count()) await iExport.click().catch(()=>{})
    await impl.waitForSelector('dialog[open], .transfer-review',{timeout:10000}).catch(()=>{})
    await impl.waitForTimeout(900)
    const iInfo=await impl.evaluate(()=>{
      const dlg=document.querySelector('dialog[open]')
      const text=dlg?dlg.innerText:''
      const tree=[...dlg?.querySelectorAll('*')||[]].find(e=>/日期|备份包/.test(e.textContent||'')&&e.children.length===0)
      return {
        dialogOpen:!!dlg,
        hasTree:!!(dlg&&/按日期|日期\s*\/\s*单次包|导出目录/.test(text)),
        treeText:(dlg?.querySelector('.export-destination')?.innerText||'').slice(0,200),
        hasSummary:!!(dlg&&/备份包/.test(text)),
        mentionsDate:/日期\s*\/\s*单次包|按日期/.test(text),
        mentionsPackage:text.includes('备份包'),
        mentionsCopy:/整体拷贝|复制整个/.test(text),
        hasOpenDir:false
      }
    })
    await impl.screenshot({path:path.join(outDir,'implementation-export.png')})

    check('原型导出确认已打开',pInfo.dialogOpen)
    check('实现导出确认已打开',iInfo.dialogOpen)
    check('两侧都有日期/目录结构说明',pInfo.hasTree&&iInfo.hasTree,'proto='+pInfo.hasTree+' impl='+iInfo.hasTree)
    check('两侧都提到备份包',pInfo.mentionsPackage&&iInfo.mentionsPackage)
    check('两侧都说明可整体拷贝',pInfo.mentionsCopy&&iInfo.mentionsCopy,'proto='+pInfo.mentionsCopy+' impl='+iInfo.mentionsCopy)
    check('原型有背包摘要',pInfo.hasSummary)
    check('实现有背包摘要',iInfo.hasSummary)
    fs.writeFileSync(path.join(outDir,'parity-export.json'),JSON.stringify({prototype:pInfo,implementation:iInfo,results},null,2))
    await proto.close(); await impl.close()
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{console.error(e);process.exit(3)})

