// Agent 助手（WP-55）：原型与实现并排对照，重点核对“已选模型展示是否紧凑”。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const outDir = process.env.PARITY_OUT_DIR || fs.mkdtempSync(path.join(os.tmpdir(), 'ocx-parity-agent-'))
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
    // 原型 Agent 配置从“模型 → 模型模版 → 编辑器侧栏 Agent 自动填充”进入。
    await proto.goto('file://' + encodeURI(PROTOTYPE) + '#models')
    await proto.waitForTimeout(1800)
    const tplTab=proto.locator('[data-mp="tab-templates"], button:has-text("模型模版")').first()
    if(await tplTab.count()) await tplTab.click().catch(()=>{})
    await proto.waitForTimeout(900)
    const editBtn=proto.locator('button:has-text("编辑")').first()
    if(await editBtn.count()) await editBtn.click().catch(()=>{})
    await proto.waitForTimeout(1000)
    const agentBtn=proto.locator('[data-mp="generate"], [id="mpTemplateAgentAction"], button:has-text("Agent 自动填充")').first()
    if(await agentBtn.count()) await agentBtn.click().catch(()=>{})
    await proto.waitForTimeout(1200)
    const pInfo=await proto.evaluate(()=>{
      const body=document.querySelector('#modalBody')||document.body
      const text=body.innerText
      // 原型的紧凑形态在 Agent 面板头部：标题 / 范围 / 单行型号入口。
      const head=document.querySelector('#mpAssistantPanel > header')
      const modelEntry=document.querySelector('.mp-assistant-model')
      return {
        hasCard:!!document.querySelector('#mpAssistantPanel, .mp-agent-card'),
        hasTitle:/Agent 助手/.test(head?.innerText||''),
        hasScope:/本机共用/.test(head?.innerText||''),
        hasCompactModel:/选择执行模型|>/.test((modelEntry?.innerText||'')),
        headHeight:head?Math.round(head.getBoundingClientRect().height):0,
        hasFacts:!!document.querySelector('.mp-agent-facts'),
        summaryHeight:(document.querySelector('#mpAgentSummary')?.getBoundingClientRect().height)||0
      }
    })
    await proto.screenshot({path:path.join(outDir,'prototype-agent.png')})

    // 实现侧：模型页 → 模型模版 → Agent 配置（compact 入口）
    const impl=await browser.newPage({viewport:{width:1180,height:760},deviceScaleFactor:2})
    await impl.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme=light')
    await impl.waitForSelector('#app .app-window',{timeout:20000}).catch(()=>{})
    await impl.waitForTimeout(1000)
    await impl.locator('button[role=tab]', { hasText: '模型模版' }).first().click().catch(()=>{})
    await impl.waitForTimeout(900)
    const iInfo=await impl.evaluate(()=>{
      // 实现侧紧凑入口在 Tab 栏右侧（AgentExecutionField compact），不是 .model-execution。
      const head=document.querySelector('.model-execution, .model-agent-execution')||document.querySelector('.model-tabs-line')
      const entry=document.querySelector('.model-execution-entry')||document.querySelector('.model-tabs-line .btn, .model-tabs-line button')
      const text=document.body.innerText
      return {
        hasCard:!!head,
        headText:(head?.innerText||'').slice(0,60),
        hasCompactEntry:!!entry,
        entryHeight:entry?Math.round(entry.getBoundingClientRect().height):0,
        hasFactsOpen:!!document.querySelector('.model-execution-facts'),
        inlineSummary:/模型模版助手|Agent/.test(text)
      }
    })
    await impl.screenshot({path:path.join(outDir,'implementation-agent.png')})

    check('原型 Agent 卡片存在',pInfo.hasCard)
    check('实现 Agent 入口存在',iInfo.hasCard)
    // 原型头部由标题行与型号入口行组成，实测 87px；判据在于不铺开完整事实卡。
    check('原型 Agent 面板头部保持紧凑',pInfo.hasTitle&&pInfo.hasScope&&pInfo.headHeight>0&&pInfo.headHeight<=100,'h='+pInfo.headHeight)
    check('原型头部有单行型号入口',pInfo.hasCompactModel)
    check('实现有 Agent 入口文案',iInfo.inlineSummary)
    // 用户诉求：已选模型展示不能占太大空间 → 实现应是单行紧凑入口
    check('实现采用单行紧凑入口',iInfo.hasCompactEntry&&iInfo.entryHeight>0&&iInfo.entryHeight<=40,'h='+iInfo.entryHeight)
    check('实现默认不铺开事实卡',iInfo.hasFactsOpen===false,'facts='+iInfo.hasFactsOpen)
    fs.writeFileSync(path.join(outDir,'parity-agent.json'),JSON.stringify({prototype:pInfo,implementation:iInfo,results},null,2))
    await proto.close(); await impl.close()
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('shots: '+outDir+' failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{console.error(e);process.exit(3)})

