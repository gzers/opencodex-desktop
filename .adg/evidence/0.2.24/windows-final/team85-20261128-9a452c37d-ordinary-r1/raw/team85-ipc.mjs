import assert from 'node:assert/strict'
import { appendFileSync } from 'node:fs'
import { setTimeout as delay } from 'node:timers/promises'
import { discoverMainTarget } from './observer/test/smoke/windows-cdp.mjs'

const [endpoint, output] = process.argv.slice(2)
const log = (event, fields = {}) => appendFileSync(output, JSON.stringify({
  event, utc: new Date().toISOString(), ...fields,
}) + '\n')
const watched = /^(get_status_snapshot|run_doctor|.*runtime_preparation.*)$/
const tracked = new Map()
const pending = new Map()
let socket
let nextId = 0
function call(method, params = {}) {
  const id = ++nextId
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id)
      reject(Error(method + ' timed out'))
    }, 1500)
    pending.set(id, reply => {
      clearTimeout(timer)
      if (reply.error) reject(Error(reply.error.message))
      else resolve(reply.result)
    })
    socket.send(JSON.stringify({ id, method, params }))
  })
}
try {
  const target = await discoverMainTarget(endpoint, { pollIntervalMs: 50 })
  const ws = new URL(target.webSocketDebuggerUrl)
  assert.equal(ws.protocol, 'ws:')
  assert(['127.0.0.1', 'localhost'].includes(ws.hostname))
  assert.equal(ws.port, new URL(endpoint).port)
  socket = new WebSocket(ws)
  await Promise.race([
    new Promise((resolve, reject) => {
      socket.addEventListener('open', resolve, { once: true })
      socket.addEventListener('error', () => reject(Error('CDP connection failed')), { once: true })
    }),
    delay(5000).then(() => { throw Error('CDP connection timed out') }),
  ])
  socket.addEventListener('message', ({ data }) => {
    const reply = JSON.parse(data)
    const { id, method, params } = reply
    if (id) {
      const complete = pending.get(id)
      pending.delete(id)
      complete?.(reply)
      return
    }
    if (method === 'Network.requestWillBeSent') {
      if (!params.request.url.includes('://ipc.localhost/')) return
      const command = new URL(params.request.url).pathname.split('/').at(-1)
      if (!watched.test(command)) return
      const { requestId, timestamp, wallTime } = params
      tracked.set(requestId, { command, timestamp })
      let input
      try { input = JSON.parse(params.request.postData ?? '{}') } catch {}
      const request = input?.request ?? input
      log('request', { requestId, command, timestamp, wallTime, downloadConfirmed: request?.downloadConfirmed, attemptId: request?.attemptId })
    } else if (tracked.has(params?.requestId)) {
      const { requestId, timestamp } = params
      const original = tracked.get(requestId)
      const meta = { requestId, command: original.command, elapsedMs: Math.round((timestamp - original.timestamp) * 1000) }
      if (method === 'Network.responseReceived') {
        log('response', { ...meta, status: params.response.status })
      } else if (method === 'Network.loadingFailed') {
        log('loading-failed', { ...meta, error: params.errorText })
        tracked.delete(requestId)
      } else if (method === 'Network.loadingFinished') {
        tracked.delete(requestId)
        void call('Network.getResponseBody', { requestId }).then(({ body, base64Encoded }) => {
          const value = JSON.parse(base64Encoded ? Buffer.from(body, 'base64').toString('utf8') : body)
          log('result', { ...meta, code: value?.code, message: value?.message, state: value?.state,
            phase: value?.phase, service: value?.service?.state, attemptId: value?.attemptId })
        }).catch(error => log('body-unavailable', { ...meta, error: error.message }))
      }
    }
  })
  await call('Network.enable')
  log('observing')
  let previous
  const deadline = performance.now() + 300_000
  while (performance.now() < deadline && socket.readyState === WebSocket.OPEN) {
    const frame = await call('Runtime.evaluate', { returnByValue: true, expression:
      '({ text: (document.body?.innerText ?? "").split("\\n").filter(x => /运行状态|等待首次|重试状态|查看诊断|准备|无需联网|下载|稍后处理|Doctor|已停止/.test(x)).slice(0,40), start: [...document.querySelectorAll("button")].filter(b => /^启动(?: OpenCodex)?$/.test(b.textContent.trim())).map(b => ({disabled: b.disabled, ariaDisabled: b.getAttribute("aria-disabled")})) })' })
    const value = frame.result.value
    const encoded = JSON.stringify(value)
    if (encoded !== previous) log('view', value)
    previous = encoded
    await delay(100)
  }
  log('observation-ended')
} catch (error) {
  log('observer-error', { message: error.message })
  process.exitCode = 1
} finally {
  socket?.close()
}
