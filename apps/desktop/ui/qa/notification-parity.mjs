// Compare result-card recipes with the sole prototype, not just DOM existence.
import fs from 'node:fs'
import path from 'node:path'
import {spawn} from 'node:child_process'
import {fileURLToPath,pathToFileURL} from 'node:url'
import {pickEngine} from './engine.mjs'
const ui=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..')
const proto=path.resolve(ui,'../../../../governance-main/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html')
const out=process.env.AUDIT_OUT_DIR||'/tmp/ocx-v14-notification-parity';fs.mkdirSync(out,{recursive:true})
if(!fs.existsSync(proto)){console.log('BLOCKED: sole prototype not available');process.exit(2)}
const server=spawn(path.join(ui,'node_modules/.bin/vite'),['--host','127.0.0.1','--port','5219','--strictPort'],{cwd:ui,stdio:'ignore'})
const results=[];let browser
function metric(arg){const card=document.querySelector(arg.card),title=card?.querySelector(arg.title),button=card?.querySelector(arg.button);if(!card||!title||!button)return null;const c=getComputedStyle(card),t=getComputedStyle(title),b=getComputedStyle(button);return{padding:c.padding,borderRadius:c.borderRadius,titleFont:t.fontSize,titleWeight:t.fontWeight,buttonHeight:b.minHeight,buttonFont:b.fontSize,buttonPadding:b.padding,buttonBackground:b.backgroundColor,actionsAlign:getComputedStyle(card.querySelector(arg.actions)).justifyContent,background:c.backgroundImage}}
try{
 for(let i=0;i<300;i++){try{if((await fetch('http://127.0.0.1:5219/')).ok)break}catch{}await new Promise(r=>setTimeout(r,100))}
 const engine=await pickEngine();browser=await engine.launch()
 for(const theme of ['light','dark']){
  const p=await browser.newPage({viewport:{width:1280,height:900}}),s=await browser.newPage({viewport:{width:1280,height:900}})
  await p.goto(pathToFileURL(proto).href+'#models');await p.waitForFunction(()=>typeof window.showTask==='function')
  await p.evaluate(theme=>{document.documentElement.dataset.theme=theme;showTask({notificationId:'qa-parity-result',title:'已写入',phase:'',steps:[],acts:[['立即重启 Codex','model-restart'],['稍后','model-later']],running:false,kind:'info'})},theme)
  await s.goto('http://127.0.0.1:5219/?__audit=1&route=models&env=ready&runtime=running&effects=high&theme='+theme);await s.waitForSelector('.channel-table')
  await s.evaluate(async()=>{(await import('/src/app/feedback/store.ts')).useFeedbackStore().clearToast();(await import('/src/app/feedback/modelSave.ts')).useModelSaveFeedback().showSaved()})
  await s.waitForSelector('.notif-layer .process-progress')
  const a=await p.evaluate(metric,{card:'.task-card',title:'h4',button:'.tc-acts button',actions:'.tc-acts'}),b=await s.evaluate(metric,{card:'.process-progress',title:'h3',button:'.process-progress-actions button',actions:'.process-progress-actions'})
  for(const key of ['padding','borderRadius','titleFont','titleWeight','buttonHeight','buttonFont','buttonPadding','buttonBackground','actionsAlign'])results.push({name:theme+' '+key,ok:a?.[key]===b?.[key],prototype:a?.[key],implementation:b?.[key]})
  results.push({name:theme+' original glass recipe',ok:Boolean(a?.background.includes('linear-gradient')&&b?.background.includes('linear-gradient'))})
  await p.screenshot({path:path.join(out,theme+'-prototype.png')});await s.screenshot({path:path.join(out,theme+'-implementation.png')});await p.close();await s.close()
 }
}finally{if(browser)await browser.close();server.kill('SIGTERM');fs.writeFileSync(path.join(out,'report.json'),JSON.stringify({method:'computed recipe plus paired screenshots; no product writes',results},null,2))}
const failed=results.filter(r=>!r.ok);for(const r of failed)console.log('FAIL '+JSON.stringify(r));console.log('notification recipe parity failed='+failed.length+'/'+results.length);process.exitCode=failed.length?1:0
