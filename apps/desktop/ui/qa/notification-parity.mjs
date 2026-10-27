// Compare result-card recipes with the sole prototype, not just DOM existence.
import fs from 'node:fs'
import path from 'node:path'
import {spawn} from 'node:child_process'
import {fileURLToPath} from 'node:url'
import http from 'node:http'
import {pickEngine} from './engine.mjs'
const ui=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'..')
const proto=process.env.PROTOTYPE_FILE||path.resolve(ui,'../../../../governance-main/docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html')
const out=process.env.AUDIT_OUT_DIR||'/tmp/ocx-v16-notification-parity';fs.mkdirSync(out,{recursive:true})
if(!fs.existsSync(proto)){console.log('BLOCKED: sole prototype not available');process.exit(2)}
const server=spawn(path.join(ui,'node_modules/.bin/vite'),['--host','127.0.0.1','--port','5219','--strictPort'],{cwd:ui,stdio:'ignore'})
// Serve the parent directory too: the sole prototype consumes shared glass CSS.
const protoRoot=path.dirname(path.dirname(proto))
const prototypeServer=http.createServer((req,res)=>{try{const target=path.resolve(protoRoot,'.'+decodeURIComponent(new URL(req.url,'http://localhost').pathname));if(!target.startsWith(protoRoot+path.sep)||!fs.statSync(target).isFile()){res.writeHead(404);res.end();return}const ext=path.extname(target);res.setHeader('Content-Type',({'.html':'text/html','.css':'text/css','.js':'text/javascript','.svg':'image/svg+xml','.json':'application/json'})[ext]||'application/octet-stream');res.end(fs.readFileSync(target))}catch{res.writeHead(404);res.end()}})
const results=[];let browser
function metric(arg){const card=document.querySelector(arg.card),title=card?.querySelector(arg.title),button=card?.querySelector(arg.button);if(!card||!title||!button)return null;const c=getComputedStyle(card),t=getComputedStyle(title),b=getComputedStyle(button);return{padding:c.padding,borderRadius:c.borderRadius,titleFont:t.fontSize,titleWeight:t.fontWeight,buttonHeight:b.minHeight,buttonFont:b.fontSize,buttonPadding:b.padding,buttonBackground:b.backgroundColor,actionsAlign:getComputedStyle(card.querySelector(arg.actions)).justifyContent,background:c.backgroundImage,backgroundColor:c.backgroundColor,backdropFilter:c.backdropFilter||c.webkitBackdropFilter,glassFill:c.getPropertyValue('--glass-fill').trim()}}
try{
 await new Promise(resolve=>prototypeServer.listen(0,'127.0.0.1',resolve))
 const prototypeOrigin='http://127.0.0.1:'+prototypeServer.address().port
 for(let i=0;i<300;i++){try{if((await fetch('http://127.0.0.1:5219/')).ok)break}catch{}await new Promise(r=>setTimeout(r,100))}
 const engine=await pickEngine();browser=await engine.launch()
 for(const theme of ['light','dark'])for(const effects of ['high','mid','low']){
  const label=theme+'-'+effects
  const p=await browser.newPage({viewport:{width:1280,height:900}}),s=await browser.newPage({viewport:{width:1280,height:900}})
  const dependencies=[]
  p.on('response',response=>{const name=new URL(response.url()).pathname.split('/').at(-1);if(['glass-tokens.css','glass-components.css','glass-effects.css'].includes(name))dependencies.push({name,status:response.status()})})
  await p.goto(prototypeOrigin+'/'+encodeURIComponent(path.basename(path.dirname(proto)))+'/index.html#models');await p.waitForFunction(()=>typeof window.showTask==='function')
  await p.evaluate(({theme,effects})=>{document.documentElement.dataset.theme=theme;document.documentElement.dataset.effects=effects;showTask({notificationId:'qa-parity-result',title:'已写入',phase:'',steps:[],acts:[['立即重启 Codex','model-restart'],['稍后','model-later']],running:false,kind:'info'})},{theme,effects})
  await s.goto('http://127.0.0.1:5219/?__audit=1&route=models&env=ready&runtime=running&effects='+effects+'&theme='+theme);await s.waitForSelector('.channel-table')
  await s.evaluate(async()=>{(await import('/src/app/feedback/store.ts')).useFeedbackStore().clearToast();(await import('/src/app/feedback/modelSave.ts')).useModelSaveFeedback().showSaved()})
  await s.waitForSelector('.notif-layer .process-progress')
  const a=await p.evaluate(metric,{card:'.task-card',title:'h4',button:'.tc-acts button',actions:'.tc-acts'}),b=await s.evaluate(metric,{card:'.process-progress',title:'h3',button:'.process-progress-actions button',actions:'.process-progress-actions'})
  for(const key of ['padding','borderRadius','titleFont','titleWeight','buttonHeight','buttonFont','buttonPadding','buttonBackground','actionsAlign','backgroundColor','backdropFilter'])results.push({name:label+' '+key,ok:a?.[key]===b?.[key],prototype:a?.[key],implementation:b?.[key]})
  results.push({name:label+' original glass recipe',ok:effects==='low'?a?.background==='none'&&b?.background==='none':Boolean(a?.background.includes('linear-gradient')&&b?.background.includes('linear-gradient')),prototype:a,implementation:b})
  results.push({name:label+' prototype shared tokens loaded',ok:await p.evaluate(()=>[...document.styleSheets].some(sheet=>sheet.href&&new URL(sheet.href).pathname.endsWith('/glass-tokens.css')&&sheet.cssRules.length>0))})
  results.push({name:label+' shared dependencies HTTP 200',ok:['glass-tokens.css','glass-components.css','glass-effects.css'].every(name=>dependencies.some(item=>item.name===name&&item.status===200)),dependencies})
  await p.screenshot({path:path.join(out,label+'-prototype.png')});await s.screenshot({path:path.join(out,label+'-implementation.png')});await p.close();await s.close()
 }
}finally{if(browser)await browser.close();server.kill('SIGTERM');await new Promise(resolve=>prototypeServer.close(resolve));fs.writeFileSync(path.join(out,'report.json'),JSON.stringify({method:'computed recipe plus paired screenshots; no product writes',results},null,2))}
const failed=results.filter(r=>!r.ok);for(const r of failed)console.log('FAIL '+JSON.stringify(r));console.log('notification recipe parity failed='+failed.length+'/'+results.length);process.exitCode=failed.length?1:0
