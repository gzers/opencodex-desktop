// Run after npm run build: source declarations alone do not prove that the
// production CSS still contains a filter understood by current WebView2.
import assert from 'node:assert/strict'
import { readdir, readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { join } from 'node:path'

const directory = fileURLToPath(new URL('../dist/assets/', import.meta.url))
const expected = ['.ui-dialog', '.detail-workspace-head', '.detail-collapse',
  '.process-progress .notif-ctl', '.notification-panel', '.toast', '.select-menu', '.card']
const seen = new Set(), failures = []
let filters = 0
for (const file of await readdir(directory)) {
  if (!file.endsWith('.css')) continue
  const css = (await readFile(join(directory, file), 'utf8')).replace(/\/\*[\s\S]*?\*\//g, '')
  // Built declarations contain no nested blocks. Iterate leaf rules so @supports
  // and @media children are checked as well, across every lazy-loaded CSS chunk.
  for (const [, selector, body] of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
    const standard = body.match(/(?:^|;)backdrop-filter:([^;]+)/)?.[1]
    const webkit = body.match(/(?:^|;)-webkit-backdrop-filter:([^;]+)/)?.[1]
    if (webkit && !standard) failures.push({ file, selector, reason: 'prefixed filter has no standard declaration' })
    if (!standard) continue
    filters++
    for (const consumer of expected) if (selector.includes(consumer)) seen.add(consumer)
  }
}
assert.deepEqual(failures, [], JSON.stringify(failures, null, 2))
assert.deepEqual(expected.filter(selector => !seen.has(selector)), [], 'missing production material consumers')
console.log(JSON.stringify({ result: 'passed', filters, consumers: [...seen] }, null, 2))
