// Owned native WebView2 acceptance. No DEV bridge, daily config or client restart.
import assert from 'node:assert/strict'
import {mkdir,writeFile,readFile} from 'node:fs/promises'
import {join,win32} from 'node:path'
import {discoverMainTarget} from '../windows-cdp.mjs'
import {assertModelSandbox,modelConfigPath} from '../windows-model-ui.mjs'
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
const suffix=['unknown','known','absent','failed'].includes(argument)?argument:'result'
const report={scenario,result:'fail',root,checks:[]}
function check(name,value){assert(value,name);report.checks.push(name)}
async function capture(name){const shot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(shot.data,'base64'))}
try{
 report.facts={about:await invoke('app_about'),dataRoot:await invoke('get_data_root_config')}
 assertModelSandbox(report.facts,root)
 modelConfigPath(report.facts,root)
 check('real isolated Tauri binding',true)
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
  const snapshot=await invoke('model_file_snapshot')
  const catalog=JSON.parse(await readFile(join(root,'manager-state/config-sync-catalog.json'),'utf8'))
  const channelIds=[...new Set([...snapshot.policy.channels.map(c=>c.id),...catalog.objects.filter(o=>o.kind==='channel').map(o=>o.id)])].sort()
  const templateIds=[...new Set([...snapshot.library.templates.map(t=>t.id),...catalog.objects.filter(o=>o.kind==='template').map(o=>o.id)])].sort()
  assert.deepEqual([...report.scope.channelIds].sort(),channelIds,'first catalogue omitted a channel')
  assert.deepEqual([...report.scope.templateIds].sort(),templateIds,'first catalogue omitted a template')
  report.catalogue={channelIds,templateIds,remoteObjects:catalog.objects.length}
  check('scope selects every available local/remote channel and template',channelIds.length+templateIds.length>0)
  report.nativeScope=await evaluate("[...document.querySelectorAll('#sync-webdav-panel input[type=checkbox]')].map(e=>({label:e.getAttribute('aria-label'),checked:e.checked}))")
  check('actual scope checkboxes selected',report.nativeScope.some(x=>x.checked))
 }else if(scenario==='channels-reopen'){
  const configFile=join(root,'opencodex-home/config.json'),before=JSON.parse(await readFile(configFile))
  const key=Object.keys(before.providers).find(k=>before.providers[k].name==='中文 渠道 / one');assert(key,'existing synthetic created channel required')
  await evaluate(js(()=>{const e=document.querySelector('#channel-config label input');if(!e)throw Error('current channel editor missing');e.value='openai';e.dispatchEvent(new Event('input',{bubbles:true}));return true}))
  await click('保存渠道草稿','.models-page');await wait("!!document.querySelector('.channel-table')",'reserved-name saved')
  const renamed=JSON.parse(await readFile(configFile));check('reserved name and raw model persisted',renamed.providers[key]?.name==='openai'&&renamed.providers[key].selectedModels[0]==='vendor/native-model')
  for(const [id,row]of Object.entries(before.providers))if(id!==key)assert.deepEqual(renamed.providers[id],row)
  await click('稍后','.notif-layer');await call('Page.reload',{ignoreCache:true});await wait("!!document.querySelector('.channel-table')",'full native reload')
  await evaluate(js(()=>{const row=[...document.querySelectorAll('.channel-table tr')].find(r=>r.textContent.includes('openai'));[...row.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑').click();return true}))
  await wait("document.querySelector('#channel-config label input')?.value==='openai'",'reserved-name reopened')
  check('reserved name survives full reopen',true)
  await click('检测全部节点','.models-page')
  await wait("document.querySelector('.models-page').innerText.includes('发现 1')||document.querySelector('.models-page').innerText.includes('1 个模型')",'actual node response')
  check('measurement leaves config bytes unchanged',JSON.stringify(JSON.parse(await readFile(configFile)))===JSON.stringify(renamed));report.internalKey=key
 }else if(scenario==='channels'){
  const configFile=join(root,'opencodex-home/config.json'),beforeBytes=await readFile(configFile),before=JSON.parse(beforeBytes)
  await click('模型','.side');await click('渠道模型');await click('新增渠道','.models-page')
  await wait("!!document.querySelector('dialog[open] input[type=url]')",'name-only channel dialog')
  check('no runtime identifier or private-network switch',await evaluate("!document.querySelector('dialog[open] input[aria-label=\"运行渠道标识\"]')&&!document.querySelector('dialog[open]').innerText.includes('允许访问可信私有网络')"))
  const fill=(label,value)=>evaluate(js((label,value)=>{const l=[...document.querySelectorAll('dialog[open] label')].find(l=>l.textContent.trim()===label);const e=l?.querySelector('input');if(!e||e.disabled)throw Error('channel field missing '+label);e.value=value;e.dispatchEvent(new Event('input',{bubbles:true}));return true},label,value))
  await fill('渠道名称','中文 渠道 / one');await fill('服务基地址','http://127.0.0.1:60443/v1');await fill('API Key','synthetic-l2-key')
  await click('下一步','dialog[open]');await click('获取模型列表','dialog[open]')
  await wait("!!document.querySelector('dialog[open] input[aria-label=\"选择新渠道型号 vendor/native-model\"]:not(:disabled)')",'real models response')
  await evaluate("document.querySelector('dialog[open] input[aria-label=\"选择新渠道型号 vendor/native-model\"]').click();true")
  check('discovery does not save config',(await readFile(configFile)).equals(beforeBytes))
  await click('加入草稿','dialog[open]');await click('保存渠道草稿','.models-page')
  await wait("!!document.querySelector('.channel-table')",'saved channel list')
  const saved=JSON.parse(await readFile(configFile)),keys=Object.keys(saved.providers).filter(k=>!Object.hasOwn(before.providers,k))
  assert.equal(keys.length,1);const key=keys[0]
  check('generated internal identity and exact model readback',key.startsWith('channel-')&&saved.providers[key].name==='中文 渠道 / one'&&JSON.stringify(saved.providers[key].selectedModels)==='[\"vendor/native-model\"]')
  for(const [id,row]of Object.entries(before.providers))assert.deepEqual(saved.providers[id],row,'new save changed adjacent provider')
  await click('稍后','.notif-layer')
  await evaluate(js(()=>{const row=[...document.querySelectorAll('.channel-table tr')].find(r=>r.textContent.includes('中文 渠道 / one'));const b=[...row.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑');b.click();return true}))
  await wait("!!document.querySelector('#channel-config label input')",'reopen new channel')
  await evaluate(js(()=>{const e=document.querySelector('#channel-config label input');e.value='openai';e.dispatchEvent(new Event('input',{bubbles:true}));return true}))
  await click('保存渠道草稿','.models-page');await wait("!!document.querySelector('.channel-table')",'renamed channel saved')
  const renamed=JSON.parse(await readFile(configFile));check('reserved display name preserves generated provider key',renamed.providers[key]?.name==='openai')
  await click('稍后','.notif-layer');await call('Page.reload',{ignoreCache:true});await wait("!!document.querySelector('.channel-table')",'full reopen')
  await evaluate(js(()=>{const row=[...document.querySelectorAll('.channel-table tr')].find(r=>r.textContent.includes('openai'));[...row.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑').click();return true}))
  await wait("document.querySelector('#channel-config label input')?.value==='openai'",'reserved display name persisted')
  await click('检测全部节点','.models-page')
  await wait("document.querySelector('.models-page').innerText.includes('发现 1')||document.querySelector('.models-page').innerText.includes('1 个模型')",'actual node measurement')
  check('node measurement does not save',JSON.stringify(JSON.parse(await readFile(configFile)))===JSON.stringify(renamed))
  report.internalKey=key;report.rawModelId='vendor/native-model'
 }else if(scenario==='panel'||scenario==='panel-recovery'){
  const before=await readFile(join(root,'opencodex-home/config.json'))
  assert(argument&&win32.isAbsolute(argument),'explicit owned diagnostic source executable required')
  assert(argument.includes('post021-windows-final'),'diagnostic source must be owned')
  await invoke('set_runtime_source',{path:argument})
  await evaluate("document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('app').loadStatusSnapshot()")
  report.status=await invoke('get_status_snapshot')
  if(scenario==='panel-recovery')await click('重试加载','.panel-frame')
  else {
   await click('概览','.side')
   report.reentry=await evaluate(js(async()=>{const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;const started=performance.now();p._s.get('routes').go('panel');await new Promise(r=>setTimeout(r,10));p._s.get('routes').go('overview');await new Promise(r=>setTimeout(r,30));p._s.get('routes').go('panel');return {elapsedMs:performance.now()-started,earlyReentryRequested:true}}))
  }
  await wait("document.querySelector('.panel-frame')?.innerText.includes('官方面板已嵌入主窗口')",'rapid navigation reentry ready')
  report.trace=await evaluate("JSON.parse(JSON.stringify(document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('panel').loadTrace))")
  check(scenario==='panel'?'early reentry remains alive and ready':'manual retry recovers failed origin change',report.trace.outcome==='visible-ready')
  report.reopens=[]
  for(let i=0;i<2;i++){await click('概览','.side');await new Promise(r=>setTimeout(r,200));await click('面板','.side');await wait("document.querySelector('.panel-frame')?.innerText.includes('官方面板已嵌入主窗口')",'reopen ready');report.reopens.push(await evaluate("JSON.parse(JSON.stringify(document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('panel').loadTrace))"))}
  check('two route reopens ready',report.reopens.every(t=>t.outcome==='visible-ready'))
  await evaluate("document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('notifications').panelOpen=true;true")
  await wait("!!document.querySelector('.notification-panel.show')",'notification surface opened')
  await capture('panel-overlay-open')
  await evaluate("document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('notifications').panelOpen=false;true")
  await wait("document.querySelector('.panel-frame')?.innerText.includes('官方面板已嵌入主窗口')",'overlay restore ready')
  check('panel observation preserves config bytes',(await readFile(join(root,'opencodex-home/config.json'))).equals(before))
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
 await capture(scenario+'-'+suffix);report.result='pass'
}catch(error){report.error={message:error.message,stack:error.stack};try{report.visible=await evaluate('document.body.innerText.slice(-1800)');await capture('failure-'+scenario)}catch{};process.exitCode=1}
finally{await writeFile(join(output,scenario+(suffix==='result'?'':'-'+suffix)+'.json'),JSON.stringify(report,null,2)+'\n');socket.close()}
console.log(JSON.stringify({scenario,result:report.result,checks:report.checks,error:report.error?.message}))
