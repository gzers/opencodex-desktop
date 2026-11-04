// Owned native WebView2 acceptance. No DEV bridge, daily config or client restart.
import assert from 'node:assert/strict'
import {mkdir,writeFile,readFile} from 'node:fs/promises'
import {join,win32} from 'node:path'
import {discoverMainTarget} from '../windows-cdp.mjs'
const [endpoint,root,output,scenario,argument]=process.argv.slice(2)
assert(endpoint&&root&&output&&scenario);assert.equal(new URL(endpoint).hostname,'127.0.0.1')
assert(win32.isAbsolute(root)&&root.includes('post021-windows-final'),'explicit isolated Windows root required')
await mkdir(output,{recursive:true})
const target=await discoverMainTarget(endpoint),socket=new WebSocket(target.webSocketDebuggerUrl)
await new Promise((r,j)=>{socket.addEventListener('open',r,{once:true});socket.addEventListener('error',j,{once:true})})
let serial=0;const pending=new Map()
socket.addEventListener('message',event=>{const m=JSON.parse(event.data),p=pending.get(m.id);if(p){pending.delete(m.id);m.error?p.reject(Error(m.error.message)):p.resolve(m.result)}})
function call(method,params={}){return new Promise((resolve,reject)=>{const id=++serial,t=setTimeout(()=>{pending.delete(id);reject(Error('bounded CDP timeout: '+method))},30000);pending.set(id,{resolve:v=>{clearTimeout(t);resolve(v)},reject});socket.send(JSON.stringify({id,method,params}))})}
async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
const js=(fn,...args)=>'('+fn.toString()+')('+args.map(a=>JSON.stringify(a)).join(',')+')'
const invoke=(command,args={})=>evaluate(js((c,a)=>window.__TAURI_INTERNALS__.invoke(c,a),command,args))
async function wait(expression,label){for(let i=0;i<150;i++){const v=await evaluate(expression);if(v)return v;await new Promise(r=>setTimeout(r,100))}throw Error('native wait: '+label)}
const click=(text,selector='body')=>wait(js((text,selector)=>{const b=[...document.querySelector(selector)?.querySelectorAll('button')??[]].find(b=>(b.textContent.trim()===text||b.getAttribute('aria-label')===text)&&b.getClientRects().length&&!b.disabled);if(!b)return false;b.click();return true},text,selector),'visible '+text)
const input=(label,value)=>evaluate(js((label,value)=>{const l=[...document.querySelectorAll('dialog[open] label')].find(l=>l.querySelector('span')?.textContent.trim()===label),e=l?.querySelector('input');if(!e||e.disabled)throw Error('input unavailable: '+label);e.value=value;e.dispatchEvent(new Event('input',{bubbles:true}));return true},label,value))
const report={scenario,result:'fail',root,checks:[]}
function check(name,value){assert(value,name);report.checks.push(name)}
async function capture(name){const shot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(shot.data,'base64'))}
try{
 report.facts={about:await invoke('app_about'),dataRoot:await invoke('get_data_root_config')}
 const normalize=s=>win32.resolve(s).toLowerCase()
 check('real isolated Tauri binding',report.facts.about.platform==='Windows'&&normalize(report.facts.dataRoot.activeDataRoot)===normalize(root))
 if(scenario==='sync-setup'){
  report.environment=await invoke('discover_environment',{request:null})
  check('real controlled Node/npm/ocx available',report.environment.gate==='ready')
  await click('同步','.side');await click('WebDAV 同步','.sync-route');await click('连接设置','.sync-route');await wait("!!document.querySelector('dialog[open]')",'connection dialog')
  for(const [label,value]of [['服务地址','https://127.0.0.1:60441'],['独立配置同步目录','native021-only'],['用户名','synthetic-post021'],['连接密码','synthetic-post021-password']])await input(label,value)
  await click('保存连接','dialog[open]');await wait("!document.querySelector('dialog[open]')",'saved connection')
  report.address=await wait("document.querySelector('.sync-connection-summary small')?.textContent",'saved address')
  check('one boundary slash',report.address==='https://127.0.0.1:60441/native021-only')
  await wait("document.querySelectorAll('#sync-webdav-panel input[type=checkbox]').length>0",'actual scope controls')
  let scopeBytes;for(let n=0;n<30;n++){try{scopeBytes=await readFile(join(root,'manager-state/config-sync-scope.json'),'utf8');break}catch{await new Promise(r=>setTimeout(r,200))}}assert(scopeBytes,'scope file missing after bounded wait');report.scope=JSON.parse(scopeBytes)
  check('first full catalogue selects channels and templates',report.scope.channelIds.length+report.scope.templateIds.length>0)
  report.nativeScope=await evaluate("[...document.querySelectorAll('#sync-webdav-panel input[type=checkbox]')].map(e=>({label:e.getAttribute('aria-label'),checked:e.checked}))")
  check('actual scope checkboxes selected',report.nativeScope.some(x=>x.checked))
 }else if(scenario==='sync-version'){
  await click('概览','.side');await wait("!!document.querySelector('[data-testid=overview-remote-version]')",'ready native overview')
  await click('检测版本','.mods')
  const expected={unknown:'已读取 · 保存时间未知',known:'上次观察',absent:'未发现快照',failed:'读取失败'}[argument];assert(expected)
  report.version=await wait(js(expected=>{const e=document.querySelector('[data-testid=overview-remote-version]');return e?.textContent.includes(expected)?e.textContent:null},expected),'actual readonly '+argument)
  check('distinct version '+argument,report.version.includes(expected))
 }else if(scenario==='sync-startup'){
  await call('Page.reload',{ignoreCache:true});await wait("!!document.querySelector('.side')",'new mount');await click('概览','.side')
  report.version=await wait("(()=>{const e=document.querySelector('[data-testid=overview-remote-version]');return e?.textContent.includes('读取失败')?e.textContent:null})()",'adopted automatic readonly startup observation')
  check('startup performs readonly version observation',!!report.version)
 }else if(scenario==='restart-missing'){
  const before=await readFile(join(root,'opencodex-home/config.json'))
  await evaluate("(()=>{const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;p._s.get('model-save-feedback').showSaved();return true})()")
  await click('立即重启 Codex','.notif-layer');await click('确认重启','dialog[open]')
  report.error=await wait("document.querySelector('dialog[open] [role=alert]')?.textContent",'concrete restart precondition')
  check('missing runtime names file and no panel precondition',report.error.includes('runtime-port.json')&&report.error.includes('无需先打开面板')&&!report.error.includes('manage backups'))
  check('retry preserved',await evaluate("[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='重试重启'&&!b.disabled)"))
  check('saved config byte invariant',(await readFile(join(root,'opencodex-home/config.json'))).equals(before))
  await capture('restart-missing-error');await click('取消','dialog[open]');await click('稍后','.notif-layer')
 }else throw Error('unknown scenario '+scenario)
 await capture(scenario+'-'+(argument??'result'));report.result='pass'
}catch(error){report.error={message:error.message,stack:error.stack};try{report.visible=await evaluate('document.body.innerText.slice(-1800)');await capture('failure-'+scenario)}catch{};process.exitCode=1}
finally{await writeFile(join(output,scenario+(argument?'-'+argument:'')+'.json'),JSON.stringify(report,null,2)+'\n');socket.close()}
console.log(JSON.stringify({scenario,result:report.result,checks:report.checks,error:report.error?.message}))
