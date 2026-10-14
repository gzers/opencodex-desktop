// Applies only to the already-running, SHA-bound sandbox instance. No install.
import { createRequire } from 'node:module'
import { readFile, writeFile, mkdir } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createHash } from 'node:crypto'
import assert from 'node:assert/strict'
import { discoverMainTarget } from 'file:///C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop/test/smoke/windows-cdp.mjs'

const require = createRequire('C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/package.json')
const sharp = require('sharp')
const root = dirname(fileURLToPath(import.meta.url))
const code = 'C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop'
const session = JSON.parse((await readFile(code + '/test/runtime/windows0110/control/current-session.json', 'utf8')).replace(/^\uFEFF/, ''))
assert.equal(session.status, 'running')
assert.equal(session.applicationPid, 10904)
assert.equal(session.executableSha256, '3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1')
assert.equal(session.applicationToken.creationTimeUtc, '2026-10-13T14:57:31.5757733Z')
assert(Date.now() - Date.parse(session.lastLiveAtUtc) < 30_000)
const restore = process.argv.includes('--restore')
const revision = process.argv.includes('--normalized-colour') ? '-normalized-colour' : ''
const source = restore ? code + '/apps/desktop/tauri/icons/32x32.png' : join(root, 'tray-light/32.png')
const raw = await sharp(await readFile(source)).ensureAlpha().raw().toBuffer({ resolveWithObject: true })
const target = await discoverMainTarget(session.endpoint)
const socket = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  const timer = setTimeout(() => reject(Error('CDP connection deadline')), 5000)
  socket.addEventListener('open', () => { clearTimeout(timer); resolve() }, { once: true })
  socket.addEventListener('error', () => { clearTimeout(timer); reject(Error('CDP connection failed')) }, { once: true })
})
let sequence = 0
const pending = new Map()
socket.addEventListener('message', event => {
  const message = JSON.parse(event.data), waiter = pending.get(message.id)
  if (!waiter) return
  pending.delete(message.id)
  clearTimeout(waiter.timer)
  message.error ? waiter.reject(Error(message.error.message)) : waiter.resolve(message.result)
})
function call(method, params) {
  return new Promise((resolve, reject) => {
    const id = ++sequence
    const timer = setTimeout(() => { pending.delete(id); reject(Error('CDP request deadline')) }, 10_000)
    pending.set(id, { resolve, reject, timer })
    socket.send(JSON.stringify({ id, method, params }))
  })
}
try {
  const expression = `(async()=>{
    const invoke=window.__TAURI_INTERNALS__.invoke;
    const about=await invoke('app_about'), root=await invoke('get_data_root_config');
    let normalizedRoot=root.currentDataRoot.split(String.fromCharCode(92)).join('/').toLowerCase();
    if(normalizedRoot.startsWith('//?/')) normalizedRoot=normalizedRoot.slice(4);
    if(about.platform!=='Windows'||about.version!=='0.1.10'
      ||!normalizedRoot.startsWith(${JSON.stringify((code + '/test/runtime/windows0110/').toLowerCase())}))
      throw Error('Refuse to alter an unverified instance');
    const rid=await invoke('plugin:tray|get_by_id',{id:'opencodex-main'});
    if(rid===null) throw Error('Existing tray icon not found; do not create another');
    const image=await invoke('plugin:image|new',{rgba:${JSON.stringify([...raw.data])},width:${raw.info.width},height:${raw.info.height}});
    try {
      await invoke('plugin:tray|set_icon',{rid,icon:image});
      await invoke('plugin:tray|set_icon_as_template',{rid,asTemplate:false});
    } finally { await invoke('plugin:resources|close',{rid:image}); }
    return {existingTrayId:'opencodex-main',existingTrayResource:rid,width:${raw.info.width},height:${raw.info.height},
      previewApplied:${!restore},originalIconRestored:${restore},nativeInstanceStarted:false,dataRoot:root.currentDataRoot,
      scope:'live visual preview only; original EXE unchanged; not a rebuilt-candidate acceptance'};
  })()`
  const result = await call('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true })
  assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails))
  await mkdir(join(root, 'live-preview'), { recursive: true })
  const report = {
    atUtc: new Date().toISOString(), applicationPid: session.applicationPid, sourceCommit: session.sourceCommit,
    executableSha256: session.executableSha256, imagePath: source,
    rawRgbaSha256: createHash('sha256').update(raw.data).digest('hex'), result: result.result.value,
  }
  await writeFile(join(root, 'live-preview', restore ? 'tray-restored.json' : `tray-applied${revision}.json`), JSON.stringify(report, null, 2) + '\n', { flag: 'wx' })
  console.log(JSON.stringify(report.result))
} finally { socket.close() }
