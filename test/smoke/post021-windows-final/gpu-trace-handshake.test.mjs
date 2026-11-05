import assert from 'node:assert/strict'
import { mkdtemp, writeFile, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test } from 'node:test'
import { readCollectorStart, assertCaptureOverlap } from './gpu-trace-handshake.mjs'

test('PS5 BOM, PS7 UTF-8 and incomplete handshake writes', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'ocx-gpu-handshake-')), file = join(dir, 'started.json')
  try {
    assert.equal(await readCollectorStart(file), null)
    for (const prefix of ['', '﻿']) {
      await writeFile(file, prefix + JSON.stringify({ recorderStarted: true }))
      assert.equal((await readCollectorStart(file)).recorderStarted, true)
    }
    await writeFile(file, '﻿{')
    assert.equal(await readCollectorStart(file), null)
    await assert.rejects(readCollectorStart(dir))
  } finally { await rm(dir, { recursive: true }) }
})

test('late or invalid starts cannot be accepted as a complete sample', () => {
  const at = Date.parse('2026-11-05T04:21:23Z')
  const started = { recorderStarted: true, startedAtUtc: new Date(at).toISOString(), durationSeconds: 45, instanceName: 'ocx-post021-gpu-' + 'a'.repeat(32) }
  assertCaptureOverlap(started, at + 9999)
  for (const now of [at - 1, at + 10000, at + 45000]) assert.throws(() => assertCaptureOverlap(started, now))
  assert.throws(() => assertCaptureOverlap({ ...started, recorderStarted: false }, at))
  assert.throws(() => assertCaptureOverlap({ ...started, startedAtUtc: 'invalid' }, at))
})
