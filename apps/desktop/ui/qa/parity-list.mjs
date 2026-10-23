// 渠道列表：原型与实现并排对照（WP-40 / WP-47 结构层）。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.PARITY_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-parity-list-'))
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
  if (!fs.existsSync(PROTOTYPE)) { console.log('BLOCKED'); process.exit(2) }
  const server=spawn(path.resolve(uiDir,'node_modules','.bin','vite'),['--host','127.0.0.1','--port',String(PORT),'--strictPort'],{cwd:uiDir,stdio:['ignore','pipe','pipe']})
  process.on('exit',()=>{try{server.kill('SIGTERM')}catch{}})
  let browser
  try{
    await waitPort(PORT)
    browser=await chromium.launch({executablePath:exe()})
    const proto=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2})
    await proto.goto('file://' + encodeURI(PROTOTYPE) + '#models')
    await proto.waitForTimeout(1800)
    const pInfo=await proto.evaluate(()=>{
      const head=document.querySelector('.mp-provider-heading')
      const row=document.querySelector('.mp-provider')
      const text=document.body.innerText
      return {
        headers:head?[...head.children].map(c=>c.innerText.trim()).filter(Boolean):[],
        hasCountHint:!!(row&&/已选\s*\/\s*发现/.test(row.innerText)),
        hasGrip:!!document.querySelector('.mp-grip'),
        actions:row?[...row.querySelectorAll('.mp-provider-actions button')].map(b=>b.innerText.trim()):[],
        hasTop:!![...document.querySelectorAll('button')].find(b=>b.innerText.trim()==='置顶'),
        saveText: text.includes('预览选择器')
      }
    })
    await proto.screenshot({path:path.join(outDir,'prototype-list.png')})

    const impl=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2})
    await impl.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme=light')
    await impl.waitForSelector('.channel-table',{timeout:20000}).catch(()=>{})
    await impl.waitForTimeout(1200)
    const iInfo=await impl.evaluate(()=>{
      const head=document.querySelector('.channel-table thead tr')
      const row=document.querySelector('.channel-table tbody tr')
      const text=document.body.innerText
      return {
        headers:head?[...head.children].map(c=>c.innerText.trim()).filter(Boolean):[],
        hasCountHint:!!(row&&/已选\s*\/\s*发现/.test(row.innerText)),
        hasGrip:!!document.querySelector('.channel-table tbody .channel-order'),
        actions:row?[...row.querySelectorAll('.channel-row-actions button')].map(b=>b.innerText.trim()):[],
        hasTop:!![...document.querySelectorAll('button')].find(b=>b.innerText.trim()==='置顶'),
        saveText: text.includes('预览选择器')
      }
    })
    await impl.screenshot({path:path.join(outDir,'implementation-list.png')})

    check('原型渠道表存在',pInfo.headers.length>=5,'headers='+JSON.stringify(pInfo.headers))
    check('实现渠道表存在',iInfo.headers.length>=5,'headers='+JSON.stringify(iInfo.headers))
    check('两侧列头一致',JSON.stringify(pInfo.headers)===JSON.stringify(iInfo.headers),JSON.stringify(pInfo.headers)+' vs '+JSON.stringify(iInfo.headers))
    check('两侧都无独立预览选择器',!pInfo.saveText&&!iInfo.saveText)
    check('两侧都无置顶按钮',!pInfo.hasTop&&!iInfo.hasTop)
    check('两侧都有排序入口',pInfo.hasGrip&&iInfo.hasGrip,'proto='+pInfo.hasGrip+' impl='+iInfo.hasGrip)
    check('两侧操作组都含编辑/复制/删除',['编辑','复制','删除'].every(t=>pInfo.actions.includes(t)&&iInfo.actions.includes(t)),'proto='+JSON.stringify(pInfo.actions)+' impl='+JSON.stringify(iInfo.actions))
    check('模型列都存在已选/发现小字',pInfo.hasCountHint===iInfo.hasCountHint,'proto='+pInfo.hasCountHint+' impl='+iInfo.hasCountHint)
    fs.writeFileSync(path.join(outDir,'parity-list.json'),JSON.stringify({prototype:pInfo,implementation:iInfo,results},null,2))
    await proto.close(); await impl.close()
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{console.error(e);process.exit(3)})

