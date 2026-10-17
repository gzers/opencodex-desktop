// A local diagnostic workload, never the product or a query-response stub.
import assert from 'node:assert/strict'
import { spawnSync } from 'node:child_process'
import { once } from 'node:events'
import { createServer, get } from 'node:http'
import { dirname, isAbsolute, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { writeFreshObservationJson } from './windows-observation-files.mjs'

if (process.argv[2] !== '--probe' || !process.argv[3]) {
  console.log(JSON.stringify({ status: 'prepared-only', workloadStarted: false, productStarted: false }))
} else {
  assert.equal(process.platform, 'win32')
  const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
  const output = resolve(process.argv[3])
  const owned = relative(join(repo, 'test/runtime'), output)
  assert(owned && !owned.startsWith('..') && !isAbsolute(owned), 'Owned diagnostic output required')
  const children = []
  for (let index = 0; index < 20; index++) {
    const started = performance.now()
    const child = spawnSync(join(process.env.SystemRoot, 'System32/cmd.exe'), ['/d', '/c', 'exit', '0'], {
      windowsHide: true, timeout: 5000, stdio: 'pipe',
    })
    assert(!child.error && child.status === 0 && !child.signal, 'Owned diagnostic child must exit normally')
    children.push({ pid: child.pid, exitCode: child.status, elapsedMs: performance.now() - started })
  }
  const body = Buffer.alloc(256, 0x61)
  const server = createServer((_request, response) => {
    response.setHeader('Content-Length', body.length)
    response.end(body)
  })
  let networkBytes = 0
  try {
    await new Promise(resolveListen => server.listen(0, '127.0.0.1', resolveListen))
    for (let index = 0; index < 4; index++) {
      await new Promise((resolveRequest, reject) => {
        const request = get({ hostname: '127.0.0.1', port: server.address().port, path: '/probe', agent: false },
          response => {
            assert.equal(response.statusCode, 200)
            response.on('data', chunk => { networkBytes += chunk.length })
            response.on('end', resolveRequest)
            response.on('error', reject)
          })
        request.setTimeout(5000, () => request.destroy(Error('Owned loopback deadline')))
        request.on('error', reject)
      })
    }
  } finally {
    await new Promise(resolveClose => server.close(resolveClose))
  }
  assert.equal(networkBytes, 4 * body.length)
  await writeFreshObservationJson(output, {
    atUtc: new Date().toISOString(), pid: process.pid, children,
    loopbackHttpRequests: 4, loopbackBodyBytes: networkBytes,
    externalNetworkUsed: false, productStarted: false, awaitingOwnedStdinExit: true,
  })
  // Keep the root alive for the ETW consumer's live-image check. The ordinary
  // controller releases only this owned stdin after observing the root event.
  process.stdin.resume()
  await Promise.race([once(process.stdin, 'data'), once(process.stdin, 'end')])
  process.stdin.pause()
}
