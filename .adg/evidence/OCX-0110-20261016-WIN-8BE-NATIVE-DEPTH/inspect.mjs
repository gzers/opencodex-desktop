import assert from 'node:assert/strict'
import { readFile, writeFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
const root = dirname(fileURLToPath(import.meta.url))
const code = 'C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop'
const { discoverMainTarget } = await import(pathToFileURL(join(code, 'test/smoke/windows-cdp.mjs')))
const launch = JSON.parse((await readFile(join(root, 'native-depth-launch.json'), 'utf8')).replace(/^\uFEFF/, ''))
const target = await discoverMainTarget(`http://127.0.0.1:${launch.port}`)
const ws = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  const timer = setTimeout(() => reject(Error('Native WebSocket open deadline')), 5000)
  ws.addEventListener('open', () => { clearTimeout(timer); resolve() }, { once: true })
  ws.addEventListener('error', () => { clearTimeout(timer); reject(Error('Native WebSocket failed')) }, { once: true })
})
ws.send(JSON.stringify({ id: 1, method: 'Runtime.evaluate', params: {
  expression: `(async () => {
    for(let attempt=0;attempt<60&&!document.querySelector('canvas.motion-mesh');attempt++){
      await new Promise(resolve=>setTimeout(resolve,100));
    }
    const invoke=window.__TAURI_INTERNALS__.invoke;
    const about=await invoke('app_about');
    const preferences=await invoke('get_preferences');
    const root=await invoke('get_data_root_config');
    const canvases=[...document.querySelectorAll('canvas.motion-mesh')].map(canvas=>{
      const gl=canvas.getContext('webgl');const bounds=canvas.getBoundingClientRect();
      return{width:canvas.width,height:canvas.height,cssWidth:bounds.width,cssHeight:bounds.height,
        attributes:gl?.getContextAttributes(),depthBits:gl?.getParameter(gl.DEPTH_BITS),
        stencilBits:gl?.getParameter(gl.STENCIL_BITS),depthTestEnabled:gl?.isEnabled(gl.DEPTH_TEST),glError:gl?.getError()};});
    return{atUtc:new Date().toISOString(),about,preferences,root,canvases,dpr:devicePixelRatio,
      runtimePatched:false,forceScaleUsed:false};
  })()`, awaitPromise: true, returnByValue: true,
} }))
try {
  const response = await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(Error('Native depth inspection deadline')), 15000)
    ws.addEventListener('message', event => {
      const value = JSON.parse(event.data)
      if (value.id !== 1) return
      clearTimeout(timer); resolve(value)
    })
  })
  await writeFile(join(root, 'depth-inspection.original.json'), JSON.stringify(response, null, 2) + '\n', { flag: 'wx' })
  assert(!response.error && !response.result.exceptionDetails)
  const result = response.result.result.value
  assert.equal(result.about.version, '0.1.10')
  assert.equal(result.canvases.length, 1)
  const canvas = result.canvases[0]
  assert.equal(canvas.attributes.depth, false)
  assert.equal(canvas.attributes.stencil, false)
  assert.equal(canvas.depthBits, 0)
  assert.equal(canvas.stencilBits, 0)
  assert.equal(canvas.depthTestEnabled, false)
  assert.equal(canvas.glError, 0)
  assert(canvas.width > 0 && canvas.height > 0 && canvas.cssWidth > 0 && canvas.cssHeight > 0)
  console.log(JSON.stringify({ source: launch.source, exeSha256: launch.exeSha256,
    actualContextDepth: false, actualDepthBits: 0, actualStencilBits: 0,
    drawingBuffer: [canvas.width, canvas.height], actualAbout: result.about,
    memoryBudgetPassed: false }))
} finally { ws.close() }
