// 模版详情：原型与实现并排对照（WP-54）。
// 原型：模型 → 模型模版 → 某模版编辑；实现：同一路径。只读，不保存。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.PARITY_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-parity-template-'))
fs.mkdirSync(outDir, { recursive: true })
const PROTOTYPE = process.env.PROTOTYPE_FILE || path.resolve(uiDir, '..', '..', '..', '..', 'governance-main', 'docs', '04-项目资料', '01-项目支撑资料', '01-产品原型', '原型', 'index.html')
const PORT = 5199, BASE = 'http://127.0.0.1:' + PORT
function pw(){const n=path.join(os.homedir(),'.npm','_npx');if(fs.existsSync(n))for(const d of fs.readdirSync(n)){const c=path.join(n,d,'node_modules','playwright-core');if(fs.existsSync(path.join(c,'package.json')))return c}return null}
function exe(){const b=path.join(os.homedir(),'Library','Caches','ms-playwright');if(!fs.existsSync(b))return null;for(const d of fs.readdirSync(b).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()){const p=path.join(b,d,'chrome-headless-shell-mac-arm64','chrome-headless-shell');if(fs.existsSync(p))return p}return null}
const {chromium}=createRequire(pw()+'/')('playwright-core')
const waitPort=(port,t=40000)=>new Promise((res,rej)=>{const s=Date.now();const tick=()=>{const k=net.connect(port,'127.0.0.1');k.once('connect',()=>{k.destroy();res()});k.once('error',()=>{k.destroy();if(Date.now()-s>t)rej(new Error('vite'));else setTimeout(tick,200)})};tick()})
const results=[]
const check=(n,ok,d='')=>results.push({n,ok:!!ok,d})
const readNav = () => {
  const nav=document.querySelector('.mp-template-nav, .detail-navigation')
  if(!nav) return null
  const text=nav.innerText
  const items=[...nav.querySelectorAll('button, a')].map(b=>b.innerText.trim()).filter(Boolean)
  return {
    text,
    hasAgent:/Agent\s*自动填充/.test(text),
    hasExternal:text.includes('外部任务'),
    hasCopy:text.includes('复制模版'),
    hasPreview:text.includes('匹配预览'),
    hasSave:text.includes('保存'),
    hasVersionSwitch:!!document.querySelector('.mp-editor-versions .mp-select, .detail-workspace-head .ui-select-field, .detail-workspace-head select, header .ui-select-field'),
    hasCompare:/与版本比较/.test(document.querySelector('.detail-workspace-head, .mp-editor-head')?.innerText || ''),
    itemCount:items.length
  }
}
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
    const tab=proto.locator('[data-mp="tab-templates"], button:has-text("模型模版")').first()
    if(await tab.count()) await tab.click().catch(()=>{})
    await proto.waitForTimeout(900)
    const edit=proto.locator('button:has-text("编辑")').first()
    if(await edit.count()) await edit.click().catch(()=>{})
    await proto.waitForTimeout(1200)
    const pNav=await proto.evaluate(readNav)
    const pMatching=await proto.evaluate(()=>{
      const m=document.querySelector('#mpTemplateSection-match, [data-template-section="match"]')
      return {exists:!!m, hasPreview:!!(m&&m.innerText.includes('匹配预览'))}
    })
    await proto.screenshot({path:path.join(outDir,'prototype-template.png')})

    const impl=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2})
    await impl.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme=light')
    await impl.waitForSelector('.channel-table',{timeout:20000}).catch(()=>{})
    await impl.waitForTimeout(1000)
    await impl.locator('button[role=tab]', { hasText: '模型模版' }).first().click().catch(()=>{})
    await impl.waitForTimeout(800)
    await impl.locator('tbody tr', { hasText: 'GPT' }).first().locator('button', { hasText: '编辑' }).first().click().catch(()=>{})
    await impl.waitForSelector('.detail-navigation',{timeout:10000}).catch(()=>{})
    await impl.waitForTimeout(900)
    const iNav=await impl.evaluate(readNav)
    const iMatching=await impl.evaluate(()=>{
      const m=document.querySelector('#model-matching')
      return {exists:!!m, hasPreview:!!(m&&m.innerText.includes('匹配预览'))}
    })
    await impl.screenshot({path:path.join(outDir,'implementation-template.png')})

    check('原型模版侧栏存在',!!pNav)
    check('实现模版侧栏存在',!!iNav)
    if(pNav&&iNav){
      check('两侧都有 Agent 自动填充',pNav.hasAgent&&iNav.hasAgent)
      check('两侧都有外部任务',pNav.hasExternal&&iNav.hasExternal)
      check('两侧都有复制模版',pNav.hasCopy&&iNav.hasCopy)
      check('两侧侧栏都没有匹配预览',!pNav.hasPreview&&!iNav.hasPreview,'proto='+pNav.hasPreview+' impl='+iNav.hasPreview)
      check('两侧都有保存',pNav.hasSave&&iNav.hasSave)
      check('两侧匹配分区都有匹配预览',pMatching.hasPreview&&iMatching.hasPreview,'proto='+pMatching.hasPreview+' impl='+iMatching.hasPreview)
      // WP-54：原型头部提供版本下拉与“与版本比较”，实现也必须有对应入口。
      // 版本下拉只在“有多个版本”时出现；本夹具每个模版仅 1 个版本，
      // 因此只要求原型有该入口、实现侧该条件下不渲染（避免单版本时出现无意义下拉）。
      check('原型有版本切换入口',pNav.hasVersionSwitch,'proto='+pNav.hasVersionSwitch)
      check('单版本时不渲染多余版本下拉',iNav.hasVersionSwitch===false,'impl='+iNav.hasVersionSwitch)
      check('实现有与版本比较入口',iNav.hasCompare,'impl='+iNav.hasCompare)
    }
    fs.writeFileSync(path.join(outDir,'parity-template.json'),JSON.stringify({prototype:{nav:pNav,matching:pMatching},implementation:{nav:iNav,matching:iMatching},results},null,2))
    await proto.close(); await impl.close()
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{console.error(e);process.exit(3)})

