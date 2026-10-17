// Test-only V8 target, never a product or acceptance substitute.
globalThis.window = globalThis
globalThis.location = { hash: '#overview' }
const listeners = new Map()
globalThis.document = { visibilityState: 'visible', hasFocus: () => true }
globalThis.addEventListener = (kind, callback) => listeners.set(kind, callback)
globalThis.removeEventListener = kind => listeners.delete(kind)
globalThis.requestAnimationFrame = callback => setTimeout(() => callback(performance.now()), 16)
globalThis.cancelAnimationFrame = clearTimeout
globalThis.PerformanceObserver = class {
  static supportedEntryTypes = ['longtask']
  observe() {}
  disconnect() {}
}
const bridge = {}
Object.defineProperties(bridge, {
  postMessage: { value: function postMessage(message) { return message.callback } },
  runCallback: { value: function runCallback(id, value) { return value } },
  invoke: { value: function invoke() { throw Error('Observer must never invoke a native command') } },
})
Object.defineProperty(globalThis, '__TAURI_INTERNALS__', { value: bridge })
process.on('message', message => {
  if (message === 'exit') { process.exitCode = 0; process.disconnect(); return }
  if (message === 'begin-only') {
    bridge.postMessage({ cmd: 'check_for_update', callback: 901, error: 902 })
    process.send('begin-complete')
  }
  if (message === 'end-only') {
    bridge.runCallback(901, {})
    process.send('end-complete')
  }
  if (message === 'calls') {
    for (let index = 1; index <= 25; index++) {
      bridge.postMessage({ cmd: 'check_for_update', callback: index, error: index + 100,
        payload: { trigger: 'automatic', secret: 'NOT_TO_BE_SERIALIZED' } })
      bridge.runCallback(index, { secret: 'NOT_TO_BE_SERIALIZED' })
    }
    bridge.postMessage({ cmd: 'official_remote_latest', callback: 201, error: 202 })
    bridge.runCallback(202, { secret: 'NOT_TO_BE_SERIALIZED' })
    bridge.postMessage({ cmd: 'update_schedule_plan', callback: 301, error: 302 })
    bridge.runCallback(301, { targets: [] })
    bridge.postMessage({ cmd: 'unrelated_private_command', callback: 401, error: 402 })
    bridge.runCallback(401, {})
    process.send('calls-complete')
  }
})
process.send('ready')
