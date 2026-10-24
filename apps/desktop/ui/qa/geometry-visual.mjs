// 渠道与模型行的几何验收（WP-40 / 41 / 47）：在真实渲染里量列宽、裁切与基线。
import fs from 'fs'; import path from 'path'; import os from 'os'; import net from 'net'
import { spawn } from 'child_process'; import { fileURLToPath } from 'url'; import { createRequire } from 'module'
const here = path.dirname(fileURLToPath(import.meta.url)), uiDir = path.resolve(here, '..')
const PORT = 5199, BASE = 'http://127.0.0.1:' + PORT
function pw(){const n=path.join(os.homedir(),'.npm','_npx');if(fs.existsSync(n))for(const d of fs.readdirSync(n)){const c=path.join(n,d,'node_modules','playwright-core');if(fs.existsSync(path.join(c,'package.json')))return c}return null}
function exe(){const b=path.join(os.homedir(),'Library','Caches','ms-playwright');if(!fs.existsSync(b))return null;for(const d of fs.readdirSync(b).filter(x=>x.startsWith('chromium_headless_shell-')).sort().reverse()){const p=path.join(b,d,'chrome-headless-shell-mac-arm64','chrome-headless-shell');if(fs.existsSync(p))return p}return null}
const {chromium}=createRequire(pw()+'/')('playwright-core')
const waitPort=(port,t=40000)=>new Promise((res,rej)=>{const s=Date.now();const tick=()=>{const k=net.connect(port,'127.0.0.1');k.once('connect',()=>{k.destroy();res()});k.once('error',()=>{k.destroy();if(Date.now()-s>t)rej(new Error('vite'));else setTimeout(tick,200)})};tick()})
const results=[]
const check=(n,ok,d='')=>results.push({n,ok:!!ok,d})
async function open(browser, width, height, theme){
  const page=await browser.newPage({viewport:{width,height},deviceScaleFactor:2})
  const errors=[]
  page.on('console',m=>{if(m.type()==='error')errors.push(m.text())})
  page.on('pageerror',e=>errors.push('pageerror: '+e.message))
  await page.goto(BASE+'/?__audit=1&report=1&route=models&env=ready&runtime=running&effects=low&theme='+theme)
  await page.waitForSelector('.channel-table',{timeout:20000}).catch(()=>{})
  await page.waitForTimeout(1200)
  return {page,errors}
}
async function main(){
  const server=spawn(path.resolve(uiDir,'node_modules','.bin','vite'),['--host','127.0.0.1','--port',String(PORT),'--strictPort'],{cwd:uiDir,stdio:['ignore','pipe','pipe']})
  process.on('exit',()=>{try{server.kill('SIGTERM')}catch{}})
  let browser
  try{
    await waitPort(PORT)
    browser=await chromium.launch({executablePath:exe()})

    // —— WP-40：窄容器下的渠道表不重叠、操作列内容不越界 ——
    for (const c of [{w:900,h:600,name:'narrow'},{w:1180,h:760,name:'wide'}]) {
      const {page,errors}=await open(browser,c.w,c.h,'light')
      const geo=await page.evaluate(()=>{
        const scroll=document.querySelector('.model-table-scroll')
        const table=document.querySelector('.channel-table')
        if(!scroll||!table) return null
        const row=table.querySelector('tbody tr')
        const cells=row?[...row.children].map(td=>{const r=td.getBoundingClientRect();return {l:Math.round(r.left),r:Math.round(r.right),w:Math.round(r.width)}}):[]
        // 相邻列不重叠
        let overlap=0
        for(let i=1;i<cells.length;i++) if(cells[i].l < cells[i-1].r-1) overlap++
        const acts=row?.children[5]
        const ab=acts?acts.getBoundingClientRect():null
        const buttons=acts?[...acts.querySelectorAll('button')].map(b=>{const r=b.getBoundingClientRect();return {l:Math.round(r.left),r:Math.round(r.right)}}):[]
        const maxRight=buttons.length?Math.max(...buttons.map(b=>b.r)):0
        return {
          containerW: scroll.clientWidth, tableW: Math.round(table.getBoundingClientRect().width),
          horizontalScroll: scroll.scrollWidth > scroll.clientWidth,
          overlap, actionsW: ab?Math.round(ab.width):0,
          actionsOverflow: ab? Math.max(0, maxRight - Math.round(ab.right)) : 0,
          rowH: row?Math.round(row.getBoundingClientRect().height):0
        }
      })
      check('WP40 '+c.name+' 渠道表已渲染',!!geo)
      if(geo){
        check('WP40 '+c.name+' 相邻列不重叠',geo.overlap===0,'overlap='+geo.overlap)
        check('WP40 '+c.name+' 操作按钮不越出单元格',geo.actionsOverflow<=1,'overflow='+geo.actionsOverflow)
        check('WP40 '+c.name+' 行高未被撑破',geo.rowH<=96,'rowH='+geo.rowH)
        check('WP40 '+c.name+' 容器可横向滚动而非裁切',geo.horizontalScroll||geo.tableW<=geo.containerW+1, 'tableW='+geo.tableW+' container='+geo.containerW)
      }
      check('WP40 '+c.name+' 无控制台错误',errors.length===0,errors.slice(0,2).join(' | '))
      await page.close()
    }

    // —— WP-41 节点下拉基线 + WP-47 模型行列宽 ——
    const {page,errors}=await open(browser,1180,760,'light')
    const trigger=await page.evaluate(()=>{
      const t=document.querySelector('.channel-node-select .select-trigger')
      if(!t) return null
      const label=t.querySelector('.node-current-label > span')
      const small=t.querySelector('.node-current-label small')
      if(!label||!small) return null
      const a=label.getBoundingClientRect(), b=small.getBoundingClientRect()
      const fs1=parseFloat(getComputedStyle(label).fontSize), fs2=parseFloat(getComputedStyle(small).fontSize)
      return {nameBottom:Math.round(a.bottom), statusBottom:Math.round(b.bottom), delta:Math.abs(Math.round(a.bottom)-Math.round(b.bottom)), nameFont:fs1, statusFont:fs2}
    })
    check('WP41 节点触发器存在',!!trigger)
    if(trigger){
      check('WP41 名称与状态字号一致',trigger.nameFont===trigger.statusFont,'name='+trigger.nameFont+' status='+trigger.statusFont)
      check('WP41 文字基线对齐',trigger.delta<=1,'delta='+trigger.delta)
    }
    // 进入渠道详情量模型列
    const editBtn=page.locator('.channel-table tbody tr').first().locator('button', {hasText:'编辑'}).first()
    if(await editBtn.count()) await editBtn.click()
    await page.waitForSelector('.channel-model-table',{timeout:15000}).catch(()=>{})
    await page.waitForTimeout(900)
    const cols=await page.evaluate(()=>{
      const table=document.querySelector('.channel-model-table table')
      if(!table) return null
      const ths=[...table.querySelectorAll('thead th')].map(th=>Math.round(th.getBoundingClientRect().width))
      const firstRow=table.querySelector('tbody tr[data-model]')
      const cells=firstRow?[...firstRow.children].map(td=>Math.round(td.getBoundingClientRect().width)):[]
      const actions=firstRow?.children[3]
      const ab=actions?actions.getBoundingClientRect():null
      const maxRight=actions?Math.max(...[...actions.querySelectorAll('button')].map(b=>b.getBoundingClientRect().right), ab.left):0
      return {ths, cells, actionsW:ab?Math.round(ab.width):0, actionsOverflow:ab?Math.max(0,Math.round(maxRight-ab.right)):0}
    })
    check('WP47 模型表已渲染',!!cols)
    if(cols){
      check('WP47 选择列 36px',Math.abs(cols.ths[0]-36)<=1,'th0='+cols.ths[0])
      check('WP47 启停列 62px',Math.abs(cols.ths[2]-62)<=1,'th2='+cols.ths[2])
      check('WP47 操作列 182px',Math.abs(cols.ths[3]-182)<=1,'th3='+cols.ths[3])
      check('WP47 行内按钮不越出单元格',cols.actionsOverflow<=1,'overflow='+cols.actionsOverflow)
    }
    // WP-58 模型行垂直居中与摘要分组（2026-10-24 实屏回归）：
    // 旧实现用 vertical-align:top 且摘要 display:block，勾选/开关/操作组各自贴顶、五段摘要挤成一行。
    // 这里按原型 .mp-model-row 量四个单元格的中心是否与整行中心同轴，并确认摘要为两行分组。
    const align=await page.evaluate(()=>{
      const rows=[...document.querySelectorAll('.channel-model-table tbody tr[data-model]')].slice(0,3)
      if(!rows.length) return null
      return rows.map(tr=>{
        const rr=tr.getBoundingClientRect(), mid=rr.top+rr.height/2
        const delta=el=>{if(!el)return null;const r=el.getBoundingClientRect();return Math.round(r.top+r.height/2-mid)}
        const summary=tr.querySelector('.model-row-summary')
        return {
          rowH:Math.round(rr.height),
          cells:[...tr.children].map(td=>getComputedStyle(td).verticalAlign),
          pick:delta(tr.querySelector('.model-row-pick')), identity:delta(tr.querySelector('.model-identity')),
          enable:delta(tr.querySelector('.model-enabled-cell')), actions:delta(tr.querySelector('.model-row-actions')),
          summaryLines:summary?summary.children.length:0,
          summaryDisplay:summary?getComputedStyle(summary).display:''
        }
      })
    })
    check('WP58 模型行已渲染',!!align)
    if(align){
      for(const [i,row] of align.entries()){
        check('WP58 行'+(i+1)+' 单元格垂直居中',row.cells.every(v=>v==='middle'),'cells='+row.cells.join(','))
        check('WP58 行'+(i+1)+' 勾选列与整行同轴',Math.abs(row.pick??99)<=2,'delta='+row.pick)
        check('WP58 行'+(i+1)+' 启停列与整行同轴',Math.abs(row.enable??99)<=2,'delta='+row.enable)
        check('WP58 行'+(i+1)+' 操作组与整行同轴',Math.abs(row.actions??99)<=2,'delta='+row.actions)
        check('WP58 行'+(i+1)+' 摘要分两行显示',row.summaryDisplay==='grid'&&row.summaryLines===2,'display='+row.summaryDisplay+' lines='+row.summaryLines)
      }
      check('WP58 模型行高度未被摘要撑破',align.every(r=>r.rowH<=112),'rowH='+align.map(r=>r.rowH).join('/'))
    }
    // WP-58 过滤区：公开范围 / 搜索 / 族筛选 / 仅启用 / 计数同一行、同一高度，勾选框不被撑宽。
    // 旧实现 `.model-toolbar input{flex:1;min-width:180px}` 命中了复选框，把 13px 勾选框撑成 180px。
    const bar=await page.evaluate(()=>{
      const bar=document.querySelector('.model-toolbar')
      if(!bar) return null
      const items=[...bar.children].map(el=>{const r=el.getBoundingClientRect();return {tag:el.tagName,h:Math.round(r.height),w:Math.round(r.width),cy:Math.round(r.top+r.height/2)}})
      const box=bar.querySelector('.ui-inline-check input[type=checkbox]')
      const boxR=box?box.getBoundingClientRect():null
      const search=bar.querySelector('input[type=search]')
      return {barH:Math.round(bar.getBoundingClientRect().height),items,
        checkboxW:boxR?Math.round(boxR.width):null,
        searchW:search?Math.round(search.getBoundingClientRect().width):null,
        centers:[...new Set(items.filter(i=>i.h>=28).map(i=>i.cy))]}
    })
    check('WP58 过滤区已渲染',!!bar)
    if(bar){
      const controls=bar.items.filter(i=>i.h>=28)
      check('WP58 过滤区控件同高',new Set(controls.map(i=>i.h)).size===1,'h='+controls.map(i=>i.h).join('/'))
      check('WP58 过滤区控件同轴',bar.centers.length===1,'centers='+bar.centers.join('/'))
      check('WP58 仅启用勾选框未被撑宽',bar.checkboxW!==null&&bar.checkboxW<=20,'w='+bar.checkboxW)
      check('WP58 搜索框吃剩余宽度',bar.searchW!==null&&bar.searchW>=200,'w='+bar.searchW)
      check('WP58 过滤区不高出单行',bar.barH<=38,'barH='+bar.barH)
    }
    check('WP47 无控制台错误',errors.length===0,errors.slice(0,2).join(' | '))
    await page.close()
  } finally { if(browser) await browser.close().catch(()=>{}); try{server.kill('SIGTERM')}catch{} }
  const failed=results.filter(r=>!r.ok)
  for(const r of results) console.log((r.ok?'PASS ':'FAIL ')+r.n+(r.d?' | '+r.d:''))
  console.log('failed='+failed.length+'/'+results.length)
  process.exit(failed.length?1:0)
}
main().catch(e=>{console.error(e);process.exit(3)})

