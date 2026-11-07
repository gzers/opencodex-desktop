// Production WebView path surfaces. This operates only on the launcher's isolated root.
import assert from 'node:assert/strict'
import {writeFile} from 'node:fs/promises'
import {join, win32} from 'node:path'
import {nativeModelUiContract} from './windows-model-ui.mjs'

export async function confirmNativeBackupDirectory({wait,click},version){
 const {publicDialogs,confirmation}=nativeModelUiContract(version)
 const selector=confirmation+(publicDialogs?'':' [role=dialog]')
 await wait('(()=>{const d=document.querySelector('+JSON.stringify(selector)+');return !!d&&d.textContent.includes("打开备份目录")'+(publicDialogs?'&&d instanceof HTMLDialogElement&&d.open&&document.querySelectorAll("dialog[open]").length===1':'')+'})()')
 await click('继续打开',selector)
 await wait('!document.querySelector('+JSON.stringify(selector)+')')
 return {version,selector,confirmed:true}
}

export async function verifyWindowsPathUi(call, output, sandbox){
  async function evaluate(expression){
    const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true,userGesture:true})
    assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails))
    return r.result.value
  }
  async function wait(expression){
    for(let n=0;n<100;n++){
      const r=await evaluate(expression)
      if(r)return r
      await new Promise(resolve=>setTimeout(resolve,100))
    }
    throw Error('native path UI wait: '+expression)
  }
  async function click(text,selector='.route-section.active'){
    assert(await evaluate(`(()=>{const root=document.querySelector(${JSON.stringify(selector)});const b=[...root?.querySelectorAll('button')??[]].find(e=>e.textContent.trim()===${JSON.stringify(text)}&&!e.disabled);if(!b)return false;b.click();return true})()`),'missing native path action: '+text)
  }
  async function screenshot(name){
    const image=await call('Page.captureScreenshot',{format:'png'})
    await writeFile(join(output,name+'.png'),Buffer.from(image.data,'base64'))
  }
  const ordinary=value=>win32.normalize(value).replace(/^\\\\\?\\UNC\\/i,'\\\\').replace(/^\\\\\?\\(?=[a-z]:\\)/i,'')
  const config=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")
  assert.equal(ordinary(config.currentDataRoot).toLowerCase(),ordinary(sandbox).toLowerCase())
  const about=await evaluate("window.__TAURI_INTERNALS__.invoke('app_about')")
  nativeModelUiContract(about.version)
  const source=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")
  const pathTargets=await evaluate("window.__TAURI_INTERNALS__.invoke('get_managed_path_targets')")
  const report={result:'fail',checks:[],rawDataRoot:config.currentDataRoot}
  let clipboard
  try{
    await click('安装配置')
    let expectedClipboard
    if(about.version==='0.2.23'){
      const snapshot=await evaluate("window.__TAURI_INTERNALS__.invoke('model_paths_v2_snapshot')")
      const keys=['aggregateRoot','desktopRoot','opencodexHome','configPath','installationRoot','managerExecutable','runtimePrefix','runtimeEntry']
      await wait("document.querySelectorAll('.paths-facts dd[data-path-key] code').length===8")
      const paths=await evaluate("[...document.querySelectorAll('.paths-facts dd[data-path-key]')].map(e=>({key:e.dataset.pathKey,path:e.querySelector('code')?.textContent,status:e.querySelector('[data-path-status]')?.dataset.pathStatus}))")
      assert.equal(snapshot.schemaVersion,1)
      for(const key of keys){
        const visible=paths.find(p=>p.key===key)
        assert(visible,'missing actual 023 path: '+key)
        assert.equal(visible.path,ordinary(snapshot[key].path))
        assert.equal(visible.status,snapshot[key].status)
        assert(!visible.path.startsWith('\\\\?\\'),'unreadable extended Windows prefix')
      }
      assert.equal(ordinary(snapshot.desktopRoot.path).toLowerCase(),ordinary(config.currentDataRoot).toLowerCase())
      assert.equal(ordinary(snapshot.opencodexHome.path).toLowerCase(),ordinary(config.currentOpencodexHome).toLowerCase())
      await evaluate("document.querySelector('[data-testid=paths-details]').click()")
      await wait("!!document.querySelector('dialog.paths-v2-dialog[open] [data-testid=paths-copy]')")
      const detailed=await evaluate("[...document.querySelectorAll('dialog.paths-v2-dialog[open] code')].map(e=>e.textContent)")
      expectedClipboard=keys.map(key=>ordinary(snapshot[key].path)).join('\n')
      assert.deepEqual(detailed,expectedClipboard.split('\n'))
      report.pathSnapshot=snapshot;report.openedTargets=[]
      report.checks.push('023 complete U/D/H/program paths and actual statuses agree with production snapshot; no retired editable root input')
    }else{
    await wait("!!document.querySelector('#data-root-input')")
    await wait(`document.querySelectorAll('.data-root-table code').length>=${pathTargets.managed.length+2}`)
    const paths=await evaluate("({values:[...document.querySelectorAll('.data-root-table code')].map(e=>e.textContent),placeholder:document.querySelector('#data-root-input').placeholder})")
    assert(paths.values.includes(ordinary(config.currentDataRoot)),JSON.stringify(paths))
    assert(paths.values.includes(ordinary(config.currentOpencodexHome)),JSON.stringify(paths))
    for(const target of pathTargets.managed) assert(paths.values.includes(ordinary(target.path)),'missing managed partition: '+target.label)
    assert(paths.values.every(value=>!value.startsWith('\\\\?\\')))
    assert.equal(paths.placeholder,'D:\\OpenCodexData')
    report.checks.push('installation data root/home/all six managed partitions/native placeholder use readable ordinary paths')
      expectedClipboard=ordinary(config.currentDataRoot)
    }
    // Loopback CDP smoke windows may lack native foreground focus. Explicitly
    // emulate document focus; clipboard calls still use the real platform API.
    await call('Emulation.setFocusEmulationEnabled',{enabled:true})
    report.documentFocusEmulated=true
    await call('Browser.grantPermissions',{origin:'http://tauri.localhost',permissions:['clipboardReadWrite','clipboardSanitizedWrite']})
    report.isolatedClipboardPermissionGranted=true
    const readClipboard="Promise.race([navigator.clipboard.readText(),new Promise((_,reject)=>setTimeout(()=>reject(new Error('isolated clipboard permission deadline')),5000))])"
    try{clipboard=await evaluate(readClipboard)}catch{}
    await evaluate(about.version==='0.2.23'?"document.querySelector('[data-testid=paths-copy]').click()":"document.querySelector('.data-root-table .cli-copy-btn').click()")
    // The Vue click handler awaits the native clipboard write. Observe its
    // actual result instead of racing that promise with an immediate read.
    const copied=await wait(`(async()=>{const text=await ${readClipboard};return text===${JSON.stringify(expectedClipboard)}?text:false})()`)
    assert.equal(copied,expectedClipboard)
    report.checks.push('real clipboard receives readable same-identity data root')
    if(about.version==='0.2.23'){
      await click('关闭','dialog.paths-v2-dialog[open] > .ui-dialog-header')
      await wait("!document.querySelector('dialog.paths-v2-dialog[open]')")
    }else{
      await click('打开','.data-root-table tbody tr')
      report.openedTargets=[config.currentDataRoot]
    }
    await click('数据与备份')
    const backup=await wait("document.querySelector('.backup-root')?.textContent")
    assert.equal(backup,ordinary(win32.join(config.currentDataRoot,'backups')))
    const typography=await evaluate("({font:getComputedStyle(document.querySelector('.backup-root')).fontFamily,hasBackslash:document.querySelector('.backup-root').textContent.includes(String.fromCharCode(92)),hasWon:document.querySelector('.backup-root').textContent.includes(String.fromCharCode(0x20a9))})")
    assert(typography.font.startsWith('Consolas')&&typography.hasBackslash&&!typography.hasWon,JSON.stringify(typography))
    report.typography=typography
    await screenshot('paths-backup-before')
    // A fresh isolated profile has no preferences file to back up. Persist its
    // actual defaults through the production API, rather than inventing a source.
    await evaluate("(async()=>{const invoke=window.__TAURI_INTERNALS__.invoke;const preferences=await invoke('get_preferences');await invoke('save_preferences',{preferences});return true})()")
    report.freshPreferencesSavedThroughProductionApi=true
    await click('生成备份')
    await wait("!!document.querySelector('.backup-records li')")
    await click('打开目录','.backup-files tbody')
    report.backupConfirmation=await confirmNativeBackupDirectory({wait,click},about.version)
    report.openedTargets.push(win32.join(config.currentDataRoot,'backups'))
    report.checks.push('backup root readable, actual backup generated, production open action uses original target')
    await screenshot('paths-backup-created')
    await click('扩展管理')
    const agents=await wait(`(()=>{const p=[...document.querySelectorAll('.agent-path-copy code,.skills-source code')].map(e=>e.textContent);return p.length===${pathTargets.agent.length}&&!p.includes('—')?p:false})()`)
    assert(agents.every(value=>!value.startsWith('\\\\?\\')&&!value.includes('/')),JSON.stringify(agents))
    for(const target of pathTargets.agent) assert(agents.includes(ordinary(target.path)),'missing client path: '+target.label)
    report.checks.push('Skills source and all client Skills/MCP paths use shared readable presentation')
    report.agentPathCount=agents.length
    await screenshot('paths-extension-directories')
    const after=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")
    assert.deepEqual(after,config,'display/copy/open mutated data binding')
    const sourceAfter=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")
    assert.deepEqual(sourceAfter,source,'display/copy/open mutated runtime source')
    report.checks.push('path display, copy and open leave stored data-root and source identities unchanged')
    await click('安装配置')
    report.result='pass'
    return report
  }catch(error){report.error=error.stack;throw error}
  finally{
    if(clipboard!==undefined){
      try{await evaluate(`navigator.clipboard.writeText(${JSON.stringify(clipboard)})`);report.clipboardRestored=true}catch{report.clipboardRestored=false}
    }
    if(report.isolatedClipboardPermissionGranted){try{await call('Browser.resetPermissions')}catch{}}
    await writeFile(join(output,'windows-path-ui.json'),JSON.stringify(report,null,2)+'\n')
  }
}
