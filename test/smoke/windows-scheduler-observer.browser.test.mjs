// Opt-in real Chromium protocol check, using an isolated headless fixture.
// This is instrumentation validation, never product/performance acceptance.
import assert from 'node:assert/strict'
import { spawn } from 'node:child_process'
import { once } from 'node:events'
import { createServer } from 'node:http'
import { mkdir, readFile } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import test from 'node:test'
import { attachSchedulerObserver, ObservationCdp } from './windows-scheduler-observer.mjs'

test('real Chromium observes frozen bridge calls and an actual >50ms task without page reload',
  { skip: !process.env.OPENCODEX_OBSERVER_TEST_EDGE, timeout: 30_000 }, async t => {
    const output = resolve(dirname(fileURLToPath(import.meta.url)),
      '../runtime', `observer-browser-method-${new Date().toISOString().replace(/[:.]/g, '-')}`)
    await mkdir(output, { recursive: true })
    const server = createServer((request, response) => {
      if (request.url !== '/fixture') { response.writeHead(404); response.end(); return }
      response.setHeader('Content-Type', 'text/html')
      response.end(`<!doctype html><title>Isolated observer fixture</title><script>
        const b={};Object.defineProperties(b,{
          postMessage:{value:function postMessage(message){return message.callback}},
          runCallback:{value:function runCallback(id,value){return value}},
          invoke:{value:function invoke(){throw Error('Observer must not invoke product commands')}}});
        Object.defineProperty(window,'__TAURI_INTERNALS__',{value:b});
        window.probe=()=>{
          setTimeout(()=>{
          b.postMessage({cmd:'check_for_update',callback:1,error:2,payload:{trigger:'automatic'}});
          b.runCallback(1,{});
          b.postMessage({cmd:'official_remote_latest',callback:3,error:4});
          b.runCallback(4,{});
          const begin=performance.now();while(performance.now()-begin<75){}
          window.probeFinished=true;
          },0);
          return true;
        };
      </script>`)
    })
    await new Promise(resolveListen => server.listen(0, '127.0.0.1', resolveListen))
    t.after(() => { server.closeAllConnections(); server.close() })
    const page = `http://127.0.0.1:${server.address().port}/fixture`
    const child = spawn(process.env.OPENCODEX_OBSERVER_TEST_EDGE, [
      '--headless=new', '--no-first-run', '--no-default-browser-check',
      '--disable-background-networking', '--disable-component-update', '--disable-sync',
      '--remote-debugging-port=0', `--user-data-dir=${join(output, 'profile')}`, page,
    ], { windowsHide: true, stdio: 'ignore' })
    const exited = once(child, 'exit')
    let client, observer
    t.after(async () => {
      if (observer) await observer.cleanup()
      if (client) {
        try { await client.call('Browser.close') } catch {}
        client.socket.close()
      }
      if (child.exitCode === null) {
        const result = await Promise.race([
          exited.then(value => ({ value })),
          new Promise(resolveDeadline => setTimeout(() => resolveDeadline({ timeout: true }), 5000)),
        ])
        assert(!result.timeout, 'Headless fixture did not exit after Browser.close')
      }
    })
    let port
    const deadline = Date.now() + 10_000
    while (Date.now() < deadline) {
      try { port = Number((await readFile(join(output, 'profile/DevToolsActivePort'), 'utf8')).split('\n')[0]) }
      catch {}
      if (port) break
      await new Promise(resolveDelay => setTimeout(resolveDelay, 50))
    }
    assert(port, 'Isolated headless debugger port not published')
    let target
    while (Date.now() < deadline) {
      const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()
      target = targets.find(item => item.type === 'page' && item.url === page)
      if (target) break
      await new Promise(resolveDelay => setTimeout(resolveDelay, 50))
    }
    assert(target, 'Only the exact fixture target is admissible')
    client = await ObservationCdp.open(target.webSocketDebuggerUrl)
    let fixtureReady = false
    while (Date.now() < deadline) {
      fixtureReady = await client.evaluate("typeof window.probe==='function' && typeof window.__TAURI_INTERNALS__?.postMessage==='function'")
      if (fixtureReady) break
      await new Promise(resolveDelay => setTimeout(resolveDelay, 25))
    }
    assert(fixtureReady, 'The actual fixture bridge script must execute before instrumentation')
    observer = await attachSchedulerObserver(client)
    const before = await observer.snapshot()
    assert.equal(before.summary.queryCalls, 0)
    await client.evaluate('window.probe()')
    const taskDeadline = Date.now() + 5000
    let taskFinished = false
    while (Date.now() < taskDeadline) {
      taskFinished = await client.evaluate('window.probeFinished===true')
      if (taskFinished) break
      await new Promise(resolveDelay => setTimeout(resolveDelay, 25))
    }
    assert(taskFinished, 'Fixture must run as a browser event-loop task, not a DevTools evaluation')
    await new Promise(resolveDelay => setTimeout(resolveDelay, 150))
    const { raw, summary } = await observer.snapshot()
    assert.equal(summary.queryCalls, 2)
    assert.equal(summary.pendingObservedCalls, 0)
    assert(summary.mainThreadTasksOver50Ms >= 1)
    assert(raw.frames.length > 0)
    assert.equal(client.pauses, 0)
    assert.equal(raw.bridgeUnchanged, true)
    assert.equal(raw.clockAndRafUnchanged, true)
    assert.equal((await observer.cleanup()).observerRemoved, true)
    observer = null
    assert.equal(await client.evaluate("Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__,'postMessage').writable"), false)
  })
