import assert from 'node:assert/strict'
import { mkdtemp, readFile, readdir } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import test from 'node:test'
import { writeFreshObservationJson } from './windows-observation-files.mjs'

test('concurrent marker publication keeps exactly one complete observation without overwrite', async () => {
  const root = await mkdtemp(join(tmpdir(), 'ocx-observation-marker-'))
  const path = join(root, 'ready.json')
  const contenders = [{ owner: 1, fields: Array.from({ length: 1000 }, (_, i) => i) },
    { owner: 2, fields: Array.from({ length: 1000 }, (_, i) => i + 1000) }]
  const results = await Promise.allSettled(contenders.map(value => writeFreshObservationJson(path, value)))
  assert.equal(results.filter(value => value.status === 'fulfilled').length, 1)
  const rejected = results.find(value => value.status === 'rejected')
  assert.equal(rejected.reason.code, 'EEXIST')
  const winner = JSON.parse(await readFile(path, 'utf8'))
  assert.deepEqual(winner, contenders[winner.owner - 1])
  const bytes = await readFile(path)
  await assert.rejects(writeFreshObservationJson(path, { owner: 3 }), { code: 'EEXIST' })
  assert.deepEqual(await readFile(path), bytes)
  assert.deepEqual(await readdir(root), ['ready.json'])
})
