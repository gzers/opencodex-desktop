// Bounded discovery shared by the Windows EXE smoke and local protocol regression.
import { setTimeout as delay } from 'node:timers/promises'

export function describeError(error) {
  const cause = error.cause?.code ?? error.cause?.message
  return `${error.message ?? String(error)}${cause ? ` (${cause})` : ''}`
}

function isMainPage(target) {
  if (target.type !== 'page' || !target.webSocketDebuggerUrl) return false
  try {
    const url = new URL(target.url)
    return ['http:', 'https:'].includes(url.protocol) && url.hostname === 'tauri.localhost'
  } catch { return false }
}

export async function discoverMainTarget(endpoint, {
  timeoutMs = 20_000, requestTimeoutMs = 1000, pollIntervalMs = 250,
  diagnostics = {}, saveDiagnostics = async () => {},
} = {}) {
  const url = new URL(endpoint)
  if (url.protocol !== 'http:' || url.hostname !== '127.0.0.1' || !url.port || url.username || url.password) {
    throw Error('CDP discovery requires an HTTP loopback endpoint with an explicit port')
  }
  const deadline = performance.now() + timeoutMs
  Object.assign(diagnostics, { endpoint, startedAt: new Date().toISOString(), phase: 'discovery', attempts: [] })
  while (performance.now() < deadline) {
    const attempt = { at: new Date().toISOString() }
    for (const route of ['version', 'list']) {
      const remaining = deadline - performance.now()
      if (remaining <= 0) break
      try {
        const response = await fetch(`${url.origin}/json/${route}`, {
          signal: AbortSignal.timeout(Math.max(1, Math.ceil(Math.min(requestTimeoutMs, remaining)))),
        })
        attempt[route] = { status: response.status }
        if (!response.ok) throw Error(`HTTP ${response.status} on /json/${route}`)
        const value = await response.json()
        if (route === 'version') {
          if (!value || typeof value !== 'object' || Array.isArray(value)) throw Error('/json/version did not return an object')
          diagnostics.lastVersion = value
          delete diagnostics.lastVersionError
        } else {
          if (!Array.isArray(value)) throw Error('/json/list did not return an array')
          diagnostics.lastTargets = value
          const target = value.find(isMainPage)
          if (target) {
            diagnostics.attempts.push(attempt)
            diagnostics.result = 'target-found'
            delete diagnostics.lastError
            delete diagnostics.lastListError
            diagnostics.target = target
            await saveDiagnostics(diagnostics)
            return target
          }
          diagnostics.lastListError = 'No main tauri.localhost page with a WebSocket debugger URL'
        }
      } catch (error) {
        const message = describeError(error)
        attempt[route] = { ...attempt[route], error: message }
        diagnostics[route === 'list' ? 'lastListError' : 'lastVersionError'] = `/json/${route}: ${message}`
      }
    }
    diagnostics.attempts.push(attempt)
    diagnostics.lastError = diagnostics.lastListError ?? diagnostics.lastVersionError
    await saveDiagnostics(diagnostics)
    await delay(Math.max(0, Math.min(pollIntervalMs, deadline - performance.now())))
  }
  diagnostics.result = 'fail'
  const error = Error(`Main WebView CDP target did not become available: ${diagnostics.lastError ?? 'discovery deadline expired'}`)
  diagnostics.error = error.message
  await saveDiagnostics(diagnostics)
  throw error
}
