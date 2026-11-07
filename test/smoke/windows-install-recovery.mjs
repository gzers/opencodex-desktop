// Real WebView2 + production IPC. The launcher provisions a fresh isolated root.
import assert from 'node:assert/strict'
import {readFile, writeFile} from 'node:fs/promises'
import {join, win32} from 'node:path'
import {discoverMainTarget} from './windows-cdp.mjs'
import {verifyWindowsInstallUi} from './windows-install-ui.mjs'
import {verifyWindowsPathUi} from './windows-path-ui.mjs'
import {verifyWindowsQueryUi} from './windows-query-ui.mjs'
import {runtimeAcceptanceLayout,assert023RuntimeAdmission} from './windows023-runtime-layout.mjs'
import {runtimeInstallHost,runtimeInstallDomExpression,runtimeInstallExpression,closeRuntimeInstallTerminal} from './windows-021-runtime-host.mjs'

const [endpoint, output, sandbox] = process.argv.slice(2)
assert(output && win32.isAbsolute(sandbox), 'explicit isolated root required')
const target = await discoverMainTarget(endpoint)
const ws = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  ws.addEventListener('open', resolve, {once:true})
  ws.addEventListener('error', reject, {once:true})
})
let id=0
const pending=new Map()
ws.addEventListener('message', event => {
  const result=JSON.parse(event.data), request=pending.get(result.id)
  if(!request)return
  pending.delete(result.id)
  result.error?request.reject(Error(result.error.message)):request.resolve(result.result)
})
function call(method,params={}){
  return new Promise((resolve,reject)=>{
    const current=++id
    const timer=setTimeout(()=>{pending.delete(current);reject(Error('CDP timeout: '+method))},30_000)
    pending.set(current,{resolve:value=>{clearTimeout(timer);resolve(value)},reject:error=>{clearTimeout(timer);reject(error)}})
    ws.send(JSON.stringify({id:current,method,params}))
  })
}
async function evaluate(expression){
  const result=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true})
  assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails))
  return result.result.value
}
async function wait(expression,timeout=20_000){
  const end=Date.now()+timeout
  while(Date.now()<end){
    const result=await evaluate(expression)
    if(result)return result
    await new Promise(resolve=>setTimeout(resolve,200))
  }
  throw Error('UI wait timed out: '+expression)
}
async function click(text,root=runtimeInstallHost(report.about.version).selector){
  assert(await evaluate(`(()=>{const root=document.querySelector(${JSON.stringify(root)});const b=[...root?.querySelectorAll('button')??[]].find(b=>(b.textContent.trim()===${JSON.stringify(text)}||b.getAttribute('aria-label')===${JSON.stringify(text)})&&!b.disabled);if(!b)return false;b.click();return true})()`),'missing enabled button: '+text)
}
async function screenshot(name){
  const image=await call('Page.captureScreenshot',{format:'png'})
  await writeFile(join(output,name+'.png'),Buffer.from(image.data,'base64'))
}
async function source(){return evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")}
async function install(version){
  const host=runtimeInstallHost(report.about.version)
  const inInstall=(fn,...args)=>runtimeInstallDomExpression(report.about.version,fn,...args)
  await click(report.about.version==='0.2.23'?'安装程序':'安装 OpenCodex','.route-section.active')
  await wait(inInstall(root=>!!document.querySelector(root+' #wiz-prefix')))

  await evaluate(inInstall(root=>document.querySelector(root+' [data-testid=wiz-ack]').click()))
  await wait(inInstall(root=>!!document.querySelector(root+' input[aria-label="版本"]')))
  await evaluate(inInstall((root,version)=>{const i=document.querySelector(root+' input[aria-label="版本"]');i.value=version;i.dispatchEvent(new Event('input',{bubbles:true}));return true},version))
  await click('开始安装')
  const terminal=await wait('(()=>{const scene='+runtimeInstallExpression(report.about.version)+';return scene.error||scene.done?scene:false})()',300_000)
  if(terminal.error){
    const current=await source(),layout=runtimeAcceptanceLayout(report.about.version,sandbox)
    const diagnostic={version,terminal,source:current,paths:await evaluate("window.__TAURI_INTERNALS__.invoke('model_paths_v2_snapshot')")}
    for(const [key,path] of Object.entries({manifest:join(current.managedPrefix,'.runtime-manifest.json'),entry:join(layout.installationRoot??layout.dataRoot,'runtime','bin','ocx.cmd'),runtimeRecord:join(layout.dataRoot,'manager-state','runtime.json'),appLog:join(layout.dataRoot,'logs','app.log')})){
      try{diagnostic[key]={path,text:await readFile(path,'utf8')}}catch(error){diagnostic[key]={path,error:error.message}}
    }
    await writeFile(join(output,'install-terminal-'+version+'.json'),JSON.stringify(diagnostic,null,2)+'\n')
    await screenshot('install-terminal-'+version)
  }
  return terminal
}
const report={result:'fail',screenshots:[],checks:[]}
try{
  await wait("!!window.__TAURI_INTERNALS__?.invoke&&!!document.querySelector('.side')")
  const config=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")
  report.about=await evaluate("window.__TAURI_INTERNALS__.invoke('app_about')")
  const layout=runtimeAcceptanceLayout(report.about.version,sandbox)
  if(report.about.version==='0.2.23')assert023RuntimeAdmission({about:report.about,dataRoot:config},sandbox)
  else {
    const equal=(a,b)=>win32.normalize(a).replace(/^\\\\\?\\/,'').toLowerCase()===win32.normalize(b).replace(/^\\\\\?\\/,'').toLowerCase()
    assert(equal(config.activeDataRoot,sandbox)&&equal(config.currentDataRoot,sandbox)&&config.runtimeActive,'immutable sandbox mismatch')
  }
  report.layout=layout
  await verifyWindowsInstallUi(call,output,false,layout.dataRoot)
  await verifyWindowsPathUi(call,output,layout.dataRoot)
  report.checks.push('prefill/reset/readable native Windows path')
  const first=await install('2.91.0')
  assert(first.done&&!first.error,JSON.stringify(first))
  assert(first.console?.includes('日志：')&&!first.offlineSteps)
  await screenshot('installed-2.91.0')
  const before=await source()
  assert.equal(before.version,'2.91.0')
  const entryBytes=await readFile(before.managedEntry)
  const manifestPath=join(before.managedPrefix,'.runtime-manifest.json')
  const manifest=await readFile(manifestPath)
  report.firstClosure=await closeRuntimeInstallTerminal({evaluate,wait,click},report.about.version,'done')
  const failure=await install('0.0.0-ocx-missing-native-recovery')
  assert(failure.error?.includes('npm pack')&&failure.error.includes('日志：'),JSON.stringify(failure))
  assert(!failure.error.includes('managed runtime operation failed'))
  assert(failure.console?.includes('npm error')&&!failure.offlineSteps&&!failure.scriptsEnabled)
  const after=await source()
  assert.equal(after.version,before.version)
  assert.equal(after.path,before.path)
  assert.deepEqual(await readFile(before.managedEntry),entryBytes)
  assert.deepEqual(await readFile(manifestPath),manifest)
  assert(after.history.at(-1).reason.includes('npm pack')&&after.history.at(-1).reason.includes('日志：'))
  await screenshot('failure-retains-output')
  report.failureClosure=await closeRuntimeInstallTerminal({evaluate,wait,click},report.about.version,'failed')
  const recovered=await install('2.93.0')
  assert(recovered.done&&!recovered.error,JSON.stringify(recovered))
  assert(!recovered.offlineSteps&&recovered.console?.includes('日志：'))
  const final=await source()
  assert.equal(final.version,'2.93.0');assert.equal(final.kind,'managed')
  await screenshot('recovered-upgraded-2.93.0')
  report.recoveredClosure=await closeRuntimeInstallTerminal({evaluate,wait,click},report.about.version,'done')
  if(runtimeInstallHost(report.about.version).publicDialogs){
    const {prepare021ReadyWebdav,verify021ReadyWebdav}=await import('./windows-021-ready-webdav.mjs')
    const facts={about:report.about,dataRoot:await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")}
    const driver={call,evaluate,wait,click,invoke:(command,args={})=>evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+')')}
    const prepared=await prepare021ReadyWebdav(driver,facts,sandbox,true)
    report.readyWebdav=await verify021ReadyWebdav(driver,output,facts,sandbox,prepared,final,'recovery-installed')
  }
  await verifyWindowsQueryUi(call,output,sandbox,{dataRoot:layout.dataRoot})
  await click('概览','.side');await wait("!!document.querySelector('.ov .mods')")
  await click('导入','.ov')
  if(report.about.version==='0.2.23'){
    await wait("[...document.querySelectorAll('dialog[open]')].some(d=>d.querySelector('h2')?.textContent==='选择配置文件')")
    const picker=await evaluate("(()=>{const d=[...document.querySelectorAll('dialog[open]')].find(d=>d.querySelector('h2')?.textContent==='选择配置文件'),r=d.getBoundingClientRect(),z=Number(getComputedStyle(d).zoom)||1;return {native:d instanceof HTMLDialogElement,title:d.querySelector('h2').textContent,text:d.textContent,width:r.width/z,viewport:innerWidth/z,hash:location.hash,masked:d.querySelector('input[aria-label=导入读取口令]')?.type==='password',chooser:[...d.querySelectorAll('button')].some(b=>b.textContent.trim()==='选择本机文件'&&!b.disabled),confirm:[...d.querySelectorAll('button')].some(b=>b.textContent.includes('确认采用')&&!b.disabled),close:d.querySelectorAll('.ui-dialog-header .modal-close:not(:disabled)').length}})()")
    assert(picker.native&&picker.masked&&picker.chooser&&!picker.confirm&&picker.close===1&&picker.text.includes('尚未选择文件')&&picker.hash.startsWith('#overview'),JSON.stringify(picker))
    assert(Math.abs(picker.width-Math.min(560,picker.viewport-32))<=2,'023 chooser must use the adopted shared dialog width')
    report.importPicker=picker
    await screenshot('overview-import-dialog')
    await click('取消','dialog[open]');await wait("!document.querySelector('dialog[open]')")
  }else{
  await wait("!!document.querySelector('dialog[open].transfer-review')")
  const picker=await evaluate("(()=>{const d=document.querySelector('dialog[open].transfer-review');return {title:d.querySelector('h2').textContent,readDisabled:[...d.querySelectorAll('button')].find(b=>b.textContent.trim()==='读取配置').disabled,hash:location.hash}})()")
  assert.equal(picker.title,'导入配置');assert(picker.readDisabled&&picker.hash.startsWith('#overview'))
  const pickerLayout=await evaluate("(()=>{const d=document.querySelector('dialog[open].transfer-review'),f=d.querySelector('.transfer-file-row'),label=d.querySelector('.transfer-file-label');const b=d.getBoundingClientRect(),s=getComputedStyle(f);return {width:b.width,labelBottom:label.getBoundingClientRect().bottom,fieldTop:f.getBoundingClientRect().top,border:s.borderTopWidth,padding:s.paddingTop}})()")
  assert(Math.abs(pickerLayout.width-720)<=2,'native import dialog must retain the maintained prototype width')
  assert(pickerLayout.fieldTop>=pickerLayout.labelBottom+5&&pickerLayout.border==='1px'&&pickerLayout.padding==='12px','native import field must keep its separate label and frame')
  await screenshot('overview-import-dialog')
  await click('取消','dialog[open].transfer-review');await wait("!document.querySelector('dialog[open].transfer-review')")
  }
  assert(await evaluate("location.hash.startsWith('#overview')"))
  report.checks.push('overview shared import opens without navigation, requires actual file selection before adoption, cancels back to overview')

  report.result='pass'
  report.checks.push('actual registry install 2.91.0','actual missing-version failure with readable details and retained console','old package/launcher/source preserved','failure history retains npm stage and log path','reopen and recover upgrade to 2.93.0')
  report.screenshots=['installed-2.91.0.png','failure-retains-output.png','recovered-upgraded-2.93.0.png']
  report.failure=failure
  report.final={kind:final.kind,version:final.version,managedPrefix:final.managedPrefix}
}catch(error){report.error=error.stack;throw error}
finally{
  await writeFile(join(output,'native-install-recovery.json'),JSON.stringify(report,null,2)+'\n')
  ws.close()
}
