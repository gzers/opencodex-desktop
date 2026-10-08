import assert from 'node:assert/strict'
import { createServer } from 'node:http'
import test from 'node:test'
import { discoverMainTarget } from './windows-cdp.mjs'

const main = { type: 'page', url: 'http://tauri.localhost/', webSocketDebuggerUrl: 'ws://127.0.0.1:1234/devtools/page/main' }
const fast = { timeoutMs: 150, requestTimeoutMs: 30, pollIntervalMs: 5 }
async function serve(t, handler) {
  const server = createServer(handler)
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
  t.after(() => { server.closeAllConnections(); server.close() })
  return `http://127.0.0.1:${server.address().port}`
}
function json(response, value) {
  response.setHeader('Content-Type', 'application/json')
  response.end(JSON.stringify(value))
}

test('retries until the main page appears; ignores embedded or unrelated pages', async t => {
  let lists = 0
  const endpoint = await serve(t, (request, response) => {
    if (request.url === '/json/version') return json(response, { Browser: 'WebView2/test' })
    json(response, ++lists === 1 ? [] : [
      { ...main, url: 'https://chatgpt.com/' }, { ...main, type: 'worker' }, main,
    ])
  })
  const diagnostics = {}
  const snapshots = []
  assert.deepEqual(await discoverMainTarget(endpoint, {
    ...fast, timeoutMs: 1000, diagnostics, saveDiagnostics: async value => snapshots.push(structuredClone(value)),
  }), main)
  assert.equal(diagnostics.attempts.length, 2)
  assert.equal(diagnostics.lastVersion.Browser, 'WebView2/test')
  assert.equal(snapshots.at(-1).result, 'target-found')
})

test('HTTP errors are preserved instead of being swallowed', async t => {
  const endpoint = await serve(t, (_request, response) => { response.writeHead(503); response.end('unavailable') })
  const diagnostics = {}
  // The final deadline can expire during a later request. Require the observed
  // HTTP error in retained attempts rather than assuming it is the last error.
  await assert.rejects(discoverMainTarget(endpoint, {
    timeoutMs: 1000, requestTimeoutMs: 250, pollIntervalMs: 50, diagnostics,
  }), /did not become available/)
  assert.equal(diagnostics.result, 'fail')
  assert(diagnostics.attempts.some(attempt =>
    attempt.list?.status === 503 && attempt.list.error?.includes('HTTP 503')))
})

test('malformed target JSON fails with parse evidence', async t => {
  const endpoint = await serve(t, (request, response) => {
    if (request.url === '/json/version') return json(response, {})
    response.end('not json')
  })
  const diagnostics = {}
  await assert.rejects(discoverMainTarget(endpoint, { ...fast, diagnostics }), error => error.message.includes('/json/list:'))
  assert(diagnostics.attempts[0].list.error)
})

test('non-array responses and empty target lists never count as rendered UI', async t => {
  for (const value of [{ unexpected: true }, []]) {
    const endpoint = await serve(t, (request, response) => json(response, request.url === '/json/version' ? {} : value))
    await assert.rejects(discoverMainTarget(endpoint, fast), /did not become available/)
  }
})

test('stalled responses are bounded by both request and discovery deadlines', async t => {
  const endpoint = await serve(t, () => {})
  const diagnostics = {}
  const start = performance.now()
  await assert.rejects(discoverMainTarget(endpoint, { ...fast, diagnostics }), /did not become available/)
  assert(performance.now() - start < 1500)
  assert(diagnostics.attempts[0].version.error)
  assert(diagnostics.attempts[0].list.error)
})

test('connection refusal retains the underlying network error', async t => {
  const server = createServer()
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve))
  const endpoint = `http://127.0.0.1:${server.address().port}`
  await new Promise(resolve => server.close(resolve))
  const diagnostics = {}
  await assert.rejects(discoverMainTarget(endpoint, { ...fast, diagnostics }), /ECONNREFUSED/)
  assert.equal(diagnostics.result, 'fail')
})

test('refuses non-loopback endpoints before making a request', async () => {
  for (const endpoint of ['http://example.com:9222', 'http://127.0.0.1', 'https://127.0.0.1:9222']) {
    await assert.rejects(discoverMainTarget(endpoint, fast), /loopback endpoint/)
  }
})
