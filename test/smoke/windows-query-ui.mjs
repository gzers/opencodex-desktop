// Actual npm view failure/recovery through production WebView handlers. Only the
// launcher's fresh sandbox npmrc is changed, and its prior bytes are restored.
import assert from 'node:assert/strict'
import {createServer} from 'node:http'
import {readFile, readdir, writeFile, unlink} from 'node:fs/promises'
import {join, win32} from 'node:path'

export async function verifyWindowsQueryUi(call, output, sandbox) {
  const report = {result:'fail', checks:[], requests:[]}
  const npmrc = join(sandbox,'home','.npmrc')
  let original
  let mode='query-failure'
  try { original = await readFile(npmrc) } catch (error) { if (error.code !== 'ENOENT') throw error }
  const server = createServer((request,response) => {
    report.requests.push({method:request.method,url:request.url})
    if(mode==='install-failure'&&!request.url.endsWith('.tgz')) {
      const version='999.0.0-ocx-native-failure'
      response.writeHead(200,{'content-type':'application/json'})
      response.end(JSON.stringify({name:'@bitkyc08/opencodex','dist-tags':{latest:version},
        versions:{[version]:{name:'@bitkyc08/opencodex',version,dist:{tarball:registry+'blocked.tgz'}}}}))
      return
    }
    response.writeHead(404,{'content-type':'application/json'})
    response.end(JSON.stringify({error:'E404',reason:'OCX_QUERY_NATIVE_FAILURE'}))
  })
  await new Promise(resolve => server.listen(0,'127.0.0.1',resolve))
  const registry = `http://127.0.0.1:${server.address().port}/`
  async function evaluate(expression) {
    const result=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true})
    assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails))
    return result.result.value
  }
  async function wait(expression,timeout=90_000) {
    const end=Date.now()+timeout
    while(Date.now()<end) {
      const value=await evaluate(expression)
      if(value)return value
      await new Promise(resolve=>setTimeout(resolve,200))
    }
    throw Error('query UI wait: '+expression)
  }
  async function click(text,selector='.route-section.active') {
    assert(await evaluate(`(()=>{const b=[...document.querySelector(${JSON.stringify(selector)})?.querySelectorAll('button')??[]].find(e=>(e.textContent.trim()===${JSON.stringify(text)}||e.getAttribute('aria-label')===${JSON.stringify(text)})&&!e.disabled);if(!b)return false;b.click();return true})()`),'query UI action: '+text)
  }
  async function screenshot(name) {
    const image=await call('Page.captureScreenshot',{format:'png'})
    await writeFile(join(output,name+'.png'),Buffer.from(image.data,'base64'))
  }
  async function restoreNpmrc() {
    if(original!==undefined)await writeFile(npmrc,original)
    else { try { await unlink(npmrc) } catch(error) { if(error.code!=='ENOENT')throw error } }
  }
  const ordinary=value=>win32.normalize(value).replace(/^\\\\\?\\UNC\\/i,'\\\\').replace(/^\\\\\?\\(?=[a-z]:\\)/i,'')
  try {
    const config=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")
    assert.equal(ordinary(config.currentDataRoot).toLowerCase(),ordinary(sandbox).toLowerCase())
    const source=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")
    const entryBytes=await readFile(source.managedEntry)
    const manifestPath=join(source.managedPrefix,'.runtime-manifest.json')
    const manifestBytes=await readFile(manifestPath)
    const before=await evaluate("window.__TAURI_INTERNALS__.invoke('official_remote_latest',{trigger:'user'})")
    assert(/^\d/.test(before.version),'real registry metadata must contain version')
    report.before=before
    const directory=join(sandbox,'logs','runtime-query')
    const previous=new Set(await readdir(directory))
    await evaluate("(async()=>{const invoke=window.__TAURI_INTERNALS__.invoke;const p=await invoke('get_preferences');p.appUpdateAutoCheck=false;await invoke('save_preferences',{preferences:p});return true})()")
    await writeFile(npmrc,`registry=${registry}\nfetch-retries=0\n`)
    report.failureRegistry=registry
    await click('版本升级')
    await click('检查更新')
    await wait("document.querySelector('[data-testid=official-update-row]')?.textContent.includes('检查失败')")
    await wait("!document.querySelector('[data-testid=manager-update-row]')?.textContent.includes('正在检查')")
    await evaluate("document.querySelector('[data-testid=official-update-detail]').click()")
    const error=await wait("document.querySelector('[data-testid=official-query-error]')?.textContent")
    assert(error.includes('npm view')&&error.includes('退出码 1')&&error.includes('E404')&&error.includes('日志：'),error)
    assert(!error.includes('official metadata query failed'))
    const typography=await evaluate("({font:getComputedStyle(document.querySelector('[data-testid=official-query-error]')).fontFamily,hasWon:document.querySelector('[data-testid=official-query-error]').textContent.includes(String.fromCharCode(0x20a9))})")
    assert(typography.font.startsWith('Consolas')&&!typography.hasWon,JSON.stringify(typography))
    report.typography=typography
    assert(report.requests.some(request=>request.url.includes('opencodex')),'real npm did not contact the local fixture registry')
    report.error=error
    await screenshot('query-failure-details')
    // Check persisted command facts independently of the visible error.
    const added=(await readdir(directory)).filter(name=>!previous.has(name))
    const logs=await Promise.all(added.map(async name=>({
      name,records:(await readFile(join(directory,name),'utf8')).trim().split('\n').map(line=>JSON.parse(line)),
    })))
    const failure=logs.find(log=>log.records.at(-1).reason==='npm_view_exit')
    assert(failure,'missing durable npm view failure log')
    const [header,stdout,stderr,terminal]=failure.records
    assert.deepEqual(header.arguments,['view','@bitkyc08/opencodex@latest','version','dist.integrity','--json'])
    assert.equal(ordinary(header.workingDirectory).toLowerCase(),ordinary(join(sandbox,'home')).toLowerCase())
    assert.equal(ordinary(header.environment.OPENCODEX_HOME).toLowerCase(),ordinary(config.currentOpencodexHome).toLowerCase())
    assert.equal(ordinary(header.environment.npm_config_cache).toLowerCase(),ordinary(join(sandbox,'cache','npm')).toLowerCase())
    assert.equal(header.environment.COMSPEC.toLowerCase(),win32.join(header.environment.SystemRoot,'System32','cmd.exe').toLowerCase())
    assert.equal(terminal.exitCode,1)
    assert(stdout.text.includes('E404')&&stderr.text.includes('E404'))
    assert.equal(stdout.complete,true);assert.equal(stderr.complete,true)
    report.failureLog=failure
    report.checks.push('real npm E404 from isolated loopback registry; actual argv/cwd/selected HOME/cache/native COMSPEC and both streams recorded')
    await click('关闭更新中心','[data-testid=update-center]')
    // Opening existing details performs no new check and preserves the error.
    await click('查看面板更新')
    await wait("document.querySelector('[data-testid=official-query-error]')?.textContent.includes('E404')")
    report.checks.push('close/reopen retains query stderr and command log reference')
    await click('关闭更新中心','[data-testid=update-center]')
    mode='install-failure'
    await click('检查更新')
    await wait("document.querySelector('[data-testid=official-update-row]')?.textContent.includes('999.0.0-ocx-native-failure')")
    await wait("!document.querySelector('[data-testid=manager-update-row]')?.textContent.includes('正在检查')")
    await evaluate("document.querySelector('[data-testid=official-update-detail]').click()")
    await click('备份并更新面板','[data-testid=update-center]')
    const failedUpdate=await wait("(()=>{const d=document.querySelector('[data-testid=official-update-card]');const e=d?.querySelector('[role=alert]')?.textContent;const c=d?.querySelector('pre[aria-label=\"面板安装输出\"]')?.textContent;return e&&c?{error:e,console:c,text:d.textContent}:false})()",120_000)
    assert(failedUpdate.error.includes('npm pack')&&failedUpdate.error.includes('日志：'),JSON.stringify(failedUpdate))
    assert(failedUpdate.console.includes('E404')&&failedUpdate.text.includes('面板更新失败'),JSON.stringify(failedUpdate))
    assert(!failedUpdate.error.includes('managed runtime operation failed'))
    await screenshot('update-failure-retains-output')
    report.failedUpdate=failedUpdate
    await click('关闭更新中心','[data-testid=update-center]')
    await click('查看面板更新')
    await wait("document.querySelector('pre[aria-label=\"面板安装输出\"]')?.textContent.includes('E404')")
    report.checks.push('real npm pack tarball failure after production backup; failed update retains console through close/reopen')
    await click('关闭更新中心','[data-testid=update-center]')
    await restoreNpmrc()
    // The query and manager check share the center's checking state. Allow the
    // admitted check to finish before explicitly requesting recovery.
    await click('检查更新')
    // A retained installation failure can legitimately take precedence in the
    // row's status label. Require this query's durable cache to change back from
    // the known fixture version before treating recovery as complete.
    const after=await wait("(async()=>{const value=await window.__TAURI_INTERNALS__.invoke('official_remote_cache');return value&&value.version!=='999.0.0-ocx-native-failure'?value:false})()")
    assert.equal(after.version,before.version,'recovered metadata did not return to the actual registry version')
    await wait("(()=>{const row=document.querySelector('[data-testid=official-update-row]');return row&&!row.textContent.includes('999.0.0-ocx-native-failure')})()")
    await evaluate("document.querySelector('[data-testid=official-update-detail]').click()")
    assert(!await evaluate("!!document.querySelector('[data-testid=official-query-error]')"),'stale query failure remained after recovery')
    assert(/^\d/.test(after.version))
    const sourceAfter=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")
    assert.equal(sourceAfter.path,source.path)
    assert.equal(sourceAfter.kind,source.kind)
    assert.equal(sourceAfter.version,source.version)
    assert.deepEqual(await readFile(source.managedEntry),entryBytes)
    assert.deepEqual(await readFile(manifestPath),manifestBytes)
    report.after=after
    report.checks.push('actual registry metadata recovery clears failure; installed runtime source unchanged')
    await screenshot('query-recovered')
    await click('关闭更新中心','[data-testid=update-center]')
    await click('安装配置')
    report.result='pass'
    return report
  } catch(error) { report.errorStack=error.stack; throw error }
  finally {
    await restoreNpmrc()
    if(original!==undefined)assert.deepEqual(await readFile(npmrc),original)
    else await assert.rejects(readFile(npmrc),{code:'ENOENT'})
    report.sandboxNpmrcRestored=true
    await new Promise(resolve=>server.close(resolve))
    await writeFile(join(output,'native-query-recovery.json'),JSON.stringify(report,null,2)+'\n')
  }
}
