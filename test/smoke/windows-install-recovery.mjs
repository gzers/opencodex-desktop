// Real WebView2 + production IPC. The launcher provisions a fresh isolated root.
import assert from 'node:assert/strict'
import {readFile, writeFile} from 'node:fs/promises'
import {join, win32} from 'node:path'
import {discoverMainTarget} from './windows-cdp.mjs'
import {verifyWindowsInstallUi} from './windows-install-ui.mjs'

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
async function click(text,root='[data-testid=runtime-install-modal]'){
  assert(await evaluate(`(()=>{const root=document.querySelector(${JSON.stringify(root)});const b=[...root?.querySelectorAll('button')??[]].find(b=>b.textContent.trim()===${JSON.stringify(text)}&&!b.disabled);if(!b)return false;b.click();return true})()`),'missing enabled button: '+text)
}
async function screenshot(name){
  const image=await call('Page.captureScreenshot',{format:'png'})
  await writeFile(join(output,name+'.png'),Buffer.from(image.data,'base64'))
}
async function source(){return evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")}
async function install(version){
  await click('安装 OpenCodex','.route-section.active')
  await wait("!!document.querySelector('#wiz-prefix')")
  await click('下一步');await click('下一步')
  await evaluate("(()=>{const boxes=[...document.querySelectorAll('[data-testid=runtime-install-modal] input[type=checkbox]')];boxes[0].click();boxes.at(-1).click()})()")
  await wait('!!document.querySelector(\'input[placeholder="例如 2.50.0"]\')')
  await evaluate(`(()=>{const i=document.querySelector('input[placeholder="例如 2.50.0"]');i.value=${JSON.stringify(version)};i.dispatchEvent(new Event('input',{bubbles:true}))})()`)
  await click('下一步')
  return wait("(()=>{const d=document.querySelector('[data-testid=runtime-install-modal]');const error=d?.querySelector('[data-testid=wiz-error]')?.textContent;const done=d?.querySelector('[data-testid=wiz-status]')?.textContent.includes('安装完成')&&d?.querySelector('[data-testid=wiz-outcome]')?.textContent;if(!error&&!done)return false;return {error,done,console:d.querySelector('[data-testid=wiz-console]')?.textContent,offlineSteps:!!d.querySelector('[data-testid=wiz-substeps]'),scriptsEnabled:!d.querySelector('[data-testid=wiz-allow-scripts]')?.disabled}})()",300_000)
}
const report={result:'fail',screenshots:[],checks:[]}
try{
  await wait("!!window.__TAURI_INTERNALS__?.invoke&&!!document.querySelector('.side')")
  const config=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")
  const equal=(a,b)=>win32.normalize(a).replace(/^\\\\\?\\/,'').toLowerCase()===win32.normalize(b).replace(/^\\\\\?\\/,'').toLowerCase()
  assert(equal(config.activeDataRoot,sandbox)&&equal(config.currentDataRoot,sandbox)&&config.runtimeActive,'immutable sandbox mismatch')
  report.about=await evaluate("window.__TAURI_INTERNALS__.invoke('app_about')")
  await verifyWindowsInstallUi(call,output,false)
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
  await click('完成')
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
  await click('关闭')
  const recovered=await install('2.93.0')
  assert(recovered.done&&!recovered.error,JSON.stringify(recovered))
  assert(!recovered.offlineSteps&&recovered.console?.includes('日志：'))
  const final=await source()
  assert.equal(final.version,'2.93.0');assert.equal(final.kind,'managed')
  await screenshot('recovered-upgraded-2.93.0')
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
