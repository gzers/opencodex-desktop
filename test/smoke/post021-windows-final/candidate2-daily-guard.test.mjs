import {test} from 'node:test'
import assert from 'node:assert/strict'
import {generateKeyPairSync, sign} from 'node:crypto'
import {verifyDailySnapshot} from './candidate2-daily-guard.mjs'

test('Cross-account daily guard authenticates fresh owner-bound hash evidence and rejects replay/tamper', () => {
  const {privateKey, publicKey} = generateKeyPairSync('ed25519')
  const guard = {runId: 'isolated-test', publicKey}, nonce = 'fresh-challenge', now = Date.now()
  const paths = ['C:/Users/15119/.codex/config.toml']
  const value = {runId: guard.runId, nonce, owner: 'EZIO-MINI\\15119', sid: 'S-1-5-21-587408064-856152790-2583328096-1001', sessionId: 2, capturedAtUtc: new Date(now).toISOString(), configs: [{path: paths[0], bytes: 3340, sha256: 'a'.repeat(64)}]}
  const envelope = input => {const payload = JSON.stringify(input); return {payload, signature: sign(null, Buffer.from(payload), privateKey).toString('base64')}}
  assert.deepEqual(verifyDailySnapshot(envelope(value), guard, nonce, paths, now), value)
  assert.throws(() => verifyDailySnapshot(envelope(value), guard, 'different-challenge', paths, now), /challenge/)
  assert.throws(() => verifyDailySnapshot(envelope(value), guard, nonce, paths, now + 20001), /expired/)
  assert.throws(() => verifyDailySnapshot(envelope({...value, sid: 'S-1-5-18'}), guard, nonce, paths, now))
  assert.throws(() => verifyDailySnapshot(envelope({...value, sessionId: 1}), guard, nonce, paths, now))
  assert.throws(() => verifyDailySnapshot(envelope({...value, runId: 'prior-attempt'}), guard, nonce, paths, now))
  assert.throws(() => verifyDailySnapshot(envelope(value), guard, nonce, ['C:/different/config.toml'], now))
  const changed = envelope(value); changed.payload = changed.payload.replace('3340', '3341')
  assert.throws(() => verifyDailySnapshot(changed, guard, nonce, paths, now), /signature/)
})
