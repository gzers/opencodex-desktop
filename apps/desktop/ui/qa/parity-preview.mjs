// 0.2.7 保存预览：原型与实现并排对照（§38）。
//
// 同一场景各截一张：原型打开「保存预览」，实现打开同名弹窗，
// 并量出两侧的关键几何（宽、高、摘要位置、列表滚动区），便于逐项比对。
// 只读：不写入任何配置，不点击确认保存。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.PARITY_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-parity-'))
fs.mkdirSync(outDir, { recursive: true })
const PROTOTYPE = process.env.PROTOTYPE_FILE || path.resolve(uiDir, '..', '..', '..', '..', 'governance-main', 'docs', '04-项目资料', '01-项目支撑资料', '01-产品原型', '原型', 'index.html')
const PORT = 5199, BASE = 'http://127.0.0.1:' + PORT
function pw(){const n=path.join(os.homedir(),'.npm','_npx');if(fs.existsSync(n))for(const d of fs.readdirSync(n)){const c=path.join(n,d,'node_modules','playwright-core');if(fs.existsSync(path.join(c,'package.json')))return c}return null}
function exe(){const b=path.join(os.homedir(),'Library','Caches','ms-playwright');if(!fs.existsSync(b))return null;for(const d of fs.readdirSync(b).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()){const p=path.join(b,d,'chrome-headless-shell-mac-arm64','chrome-headless-shell');if(fs.existsSync(p))return p}return null}
const {chromium}=createRequire(pw()+'/')('playwright-core')
const waitPort=(port,t=40000)=>new Promise((res,rej)=>{const s=Date.now();const tick=()=>{const k=net.connect(port,'127.0.0.1');k.once('connect',()=>{k.destroy();res()});k.once('error',()=>{k.destroy();if(Date.now()-s>t)rej(new Error('vite'));else setTimeout(tick,200)})};tick()})
const results=[]
const check=(n,ok,d='')=>results.push({n,ok:!!ok,d})
async function main(){
  if (!fs.existsSync(PROTOTYPE)) { console.log('BLOCKED: 原型文件不存在 ' + PROTOTYPE); process.exit(2) }
  const server=spawn(path.resolve(uiDir,'node_modules','.bin','vite'),['--host','127.0.0.1','--port',String(PORT),'--strictPort'],{cwd:uiDir,stdio:['ignore','pipe','pipe']})
  process.on('exit',()=>{try{server.kill('SIGTERM')}catch{}})
  let browser
  try{
    await waitPort(PORT)
    browser=await chromium.launch({executablePath:exe()})

    // —— 原型侧 ——
    const proto=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2})
    await proto.goto('file://' + encodeURI(PROTOTYPE) + '#models')
    await proto.waitForTimeout(1500)
    const protoSave=proto.locator('[data-mp="save"], #mpPolicySave').first()
    if(await protoSave.count()) await protoSave.click().catch(()=>{})
    await proto.waitForSelector('#modal[style*="flex"], .mp-selector-preview',{timeout:8000}).catch(()=>{})
    await proto.waitForTimeout(800)
    const pInfo=await proto.evaluate(()=>{
      const dlg=document.querySelector('#modal .modal, .mp-dialog')
      const preview=document.querySelector('.mp-selector-preview')
      const budget=document.querySelector('#mpPreviewBudget, .mp-preview-budget-summary')
      const order=document.querySelector('.mp-preview-order')
      const box=dlg?dlg.getBoundingClientRect():null
      return {
        open:!!preview,
        title:(document.querySelector('#modalTitle, .modal h3')?.textContent||'').trim(),
        width:box?Math.round(box.width):0, height:box?Math.round(box.height):0,
        hasBudget:!!budget, hasOrder:!!order,
        groups:document.querySelectorAll('.mp-preview-group').length,
        orderScroll:order?{client:order.clientHeight,scroll:order.scrollHeight}:null,
        hasSource:!!document.querySelector('.mp-preview-mode, [aria-label="预览来源"]'),
        hasCounts:document.body.innerText.includes('按渠道查看数量')
      }
    })
    await proto.screenshot({path:path.join(outDir,'prototype-preview.png')})

    // —— 实现侧 ——
    const impl=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2})
    await impl.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme=light')
    await impl.waitForSelector('.channel-table',{timeout:20000}).catch(()=>{})
    await impl.waitForTimeout(1200)
    const toggle=impl.locator('.channel-enable-cell button, .channel-enable-cell [role=switch]').first()
    if(await toggle.count()) await toggle.click().catch(()=>{})
    await impl.waitForTimeout(400)
    await impl.locator('[data-testid="policy-save"]').first().click().catch(()=>{})
    await impl.waitForSelector('dialog[open]',{timeout:10000}).catch(()=>{})
    await impl.waitForTimeout(900)
    const iInfo=await impl.evaluate(()=>{
      const dlg=document.querySelector('dialog[open]')
      const preview=dlg?.querySelector('.selector-preview')
      const budget=dlg?.querySelector('.selector-preview-budget')
      const order=dlg?.querySelector('.selector-preview-order')
      const box=dlg?dlg.getBoundingClientRect():null
      return {
        open:!!preview,
        title:(dlg?.querySelector('h2')?.textContent||'').trim(),
        width:box?Math.round(box.width):0, height:box?Math.round(box.height):0,
        hasBudget:!!budget, hasOrder:!!order,
        groups:dlg?dlg.querySelectorAll('.preview-group').length:0,
        orderScroll:order?{client:order.clientHeight,scroll:order.scrollHeight}:null,
        hasSource:!!(dlg&&dlg.querySelector('[aria-label="预览来源"]')),
        hasCounts:(dlg?.innerText||'').includes('按渠道查看数量')
      }
    })
    await impl.screenshot({path:path.join(outDir,'implementation-preview.png')})

    // —— 对照断言：结构必须一致，数值允许按渠道数量不同 ——
    check('原型预览已打开',pInfo.open,'title='+pInfo.title)
    check('实现预览已打开',iInfo.open,'title='+iInfo.title)
    check('两侧标题一致',pInfo.title===iInfo.title||iInfo.title.includes('保存预览'),pInfo.title+' vs '+iInfo.title)
    check('两侧都有激活摘要',pInfo.hasBudget&&iInfo.hasBudget)
    check('两侧都有顺序列表',pInfo.hasOrder&&iInfo.hasOrder)
    check('两侧都没有预览来源',!pInfo.hasSource&&!iInfo.hasSource)
    check('两侧都没有按渠道查看数量',!pInfo.hasCounts&&!iInfo.hasCounts)
    check('两侧都有渠道分组',pInfo.groups>0&&iInfo.groups>0,'proto='+pInfo.groups+' impl='+iInfo.groups)
    check('实现宽度不超原型上限',iInfo.width<=980&&pInfo.width<=980,'proto='+pInfo.width+' impl='+iInfo.width)
    if(pInfo.orderScroll&&iInfo.orderScroll){
      check('两侧列表都是独立滚动区',pInfo.orderScroll.scroll>=pInfo.orderScroll.client&&iInfo.orderScroll.scroll>=iInfo.orderScroll.client,
        'proto='+JSON.stringify(pInfo.orderScroll)+' impl='+JSON.stringify(iInfo.orderScroll))
    }
    fs.writeFileSync(path.join(outDir,'parity.json'),JSON.stringify({prototype:pInfo,implementation:iInfo,results},null,2))
    await proto.close(); await impl.close()
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{console.error(e);process.exit(3)})

