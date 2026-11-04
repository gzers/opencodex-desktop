import assert from 'node:assert/strict'
import { discoverMainTarget } from '../windows-cdp.mjs'

// Real native WebView2 CDP transport, shared by owned diagnostic drivers.
export async function connect(endpoint) {
  const target = await discoverMainTarget(endpoint)
  const debuggerUrl = new URL(target.webSocketDebuggerUrl)
  assert.equal(debuggerUrl.protocol, 'ws:')
  assert.equal(debuggerUrl.hostname, '127.0.0.1')
  assert.equal(debuggerUrl.port, new URL(endpoint).port)
  const socket = new WebSocket(debuggerUrl), pending = new Map(), listeners = new Map()
  let serial = 0
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => { socket.close(); reject(Error('CDP connection deadline')) }, 10000)
    const finish = callback => value => { clearTimeout(timer); callback(value) }
    socket.addEventListener('open', finish(resolve), { once: true })
    socket.addEventListener('error', finish(reject), { once: true })
  })
  socket.addEventListener('message', event => {
    const message = JSON.parse(event.data)
    if (message.id) {
      const request = pending.get(message.id)
      if (request) {
        pending.delete(message.id)
        message.error ? request.reject(Error(message.error.message)) : request.resolve(message.result)
      }
    } else {
      for (const callback of listeners.get(message.method) ?? []) callback(message.params)
    }
  })
  const rejectPending = () => {
    for (const request of pending.values()) request.reject(Error('CDP closed'))
    pending.clear()
  }
  socket.addEventListener('close', rejectPending)
  const call = (method, params = {}) => new Promise((resolve, reject) => {
    const id = ++serial
    const timer = setTimeout(() => { pending.delete(id); reject(Error('bounded CDP timeout: ' + method)) }, 45000)
    pending.set(id, {
      resolve: value => { clearTimeout(timer); resolve(value) },
      reject: error => { clearTimeout(timer); reject(error) },
    })
    try { socket.send(JSON.stringify({ id, method, params })) }
    catch (error) { pending.get(id).reject(error); pending.delete(id) }
  })
  const evaluate = async expression => {
    const result = await call('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true })
    if (result.exceptionDetails) throw Error(JSON.stringify(result.exceptionDetails))
    return result.result.value
  }
  const invoke = (command, args = {}) => evaluate('window.__TAURI_INTERNALS__.invoke(' + JSON.stringify(command) + ',' + JSON.stringify(args) + ')')
  const on = (method, callback) => {
    if (!listeners.has(method)) listeners.set(method, new Set())
    listeners.get(method).add(callback)
    return () => listeners.get(method).delete(callback)
  }
  return { call, evaluate, invoke, on, close: () => { rejectPending(); socket.close() }, target }
}
