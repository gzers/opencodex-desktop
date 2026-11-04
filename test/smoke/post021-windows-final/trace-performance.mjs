// Real Chromium main-thread/compositor/GPU trace; this does not alter rendering.
import assert from 'node:assert/strict'
import { mkdir, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { setTimeout as delay } from 'node:timers/promises'
import { connect } from './cdp-session.mjs'
import { assertModelSandbox } from '../windows-model-ui.mjs'

const [endpoint, root, output, seconds = '40'] = process.argv.slice(2), duration = Number(seconds) * 1000
assert(endpoint && root && output, 'usage: trace-performance.mjs ENDPOINT SANDBOX_ROOT OUTPUT [SECONDS]')
assert(duration >= 1000 && duration <= 60000, 'bounded trace duration required')
await mkdir(output, { recursive: true })
const summary = { root, durationMs: duration, result: 'fail', acceptance: false }
let session, tracing = false, completionTimer, unsubscribe
try {
  session = await connect(endpoint)
  summary.target = { id: session.target.id, url: session.target.url }
  assertModelSandbox({ about: await session.invoke('app_about'), dataRoot: await session.invoke('get_data_root_config') }, root)
  summary.foregroundBefore = await session.invoke('app_foreground')
  assert(summary.foregroundBefore, 'native foreground required before trace; reconnect/unlock RDP first')
  const done = new Promise(resolve => { unsubscribe = session.on('Tracing.tracingComplete', resolve) })
  await session.call('Tracing.start', {
    categories: 'toplevel,blink,cc,gpu,viz,renderer.scheduler,devtools.timeline,disabled-by-default-devtools.timeline.frame,disabled-by-default-gpu.service',
    options: 'record-as-much-as-possible', transferMode: 'ReturnAsStream',
  })
  tracing = true
  await writeFile(join(output, 'trace-started.json'), JSON.stringify({ at: new Date().toISOString(), durationMs: duration }) + '\n')
  await delay(duration)
  await session.call('Tracing.end')
  tracing = false
  const { stream } = await Promise.race([done, new Promise((_, reject) => {
    completionTimer = setTimeout(() => reject(Error('trace completion deadline')), 30000)
  })])
  clearTimeout(completionTimer)
  const chunks = []; let size = 0
  try {
    while (true) {
      const part = await session.call('IO.read', { handle: stream, size: 1024 * 1024 })
      const bytes = Buffer.from(part.data, part.base64Encoded ? 'base64' : 'utf8')
      size += bytes.length
      assert(size <= 256 * 1024 * 1024, 'trace exceeds bounded size')
      chunks.push(bytes)
      if (part.eof) break
    }
  } finally { await session.call('IO.close', { handle: stream }) }
  const data = Buffer.concat(chunks)
  await writeFile(join(output, 'chromium-trace.json'), data)
  summary.foregroundAfter = await session.invoke('app_foreground')
  assert(summary.foregroundAfter, 'trace lost native foreground; retain as invalid diagnostic')
  const trace = JSON.parse(data), aggregates = new Map()
  for (const event of trace.traceEvents ?? []) if (event.ph === 'X' && event.dur) {
    const key = [event.pid, event.tid, event.name].join(':')
    const aggregate = aggregates.get(key) ?? { pid: event.pid, tid: event.tid, name: event.name, count: 0, totalMs: 0, maxMs: 0 }
    aggregate.count++; aggregate.totalMs += event.dur / 1000; aggregate.maxMs = Math.max(aggregate.maxMs, event.dur / 1000)
    aggregates.set(key, aggregate)
  }
  summary.result = 'captured-diagnostic-only'
  summary.traceEvents = trace.traceEvents?.length
  summary.processes = trace.traceEvents?.filter(event => event.ph === 'M' && ['process_name', 'thread_name'].includes(event.name))
  summary.longest = [...aggregates.values()].sort((a, b) => b.maxMs - a.maxMs).slice(0, 40)
  summary.totalWork = [...aggregates.values()].sort((a, b) => b.totalMs - a.totalMs).slice(0, 40)
} catch (error) {
  summary.error = { message: error.message, stack: error.stack }; process.exitCode = 1
} finally {
  clearTimeout(completionTimer); unsubscribe?.()
  if (tracing) await session.call('Tracing.end').catch(() => {})
  session?.close()
  await writeFile(join(output, 'trace-summary.json'), JSON.stringify(summary, null, 2) + '\n')
}
