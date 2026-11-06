import assert from 'node:assert/strict'
import {readFile, writeFile, rename, mkdir, access} from 'node:fs/promises'
import {createHash, generateKeyPairSync, randomUUID, sign, verify} from 'node:crypto'
import {execFileSync} from 'node:child_process'
import {join, resolve} from 'node:path'
import {fileURLToPath} from 'node:url'

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms))
const sha = bytes => createHash('sha256').update(bytes).digest('hex')
export function verifyDailySnapshot(envelope, guard, nonce, paths, now = Date.now()) {
  assert(verify(null, Buffer.from(envelope.payload), guard.publicKey, Buffer.from(envelope.signature, 'base64')), 'Daily observer signature rejected')
  const value = JSON.parse(envelope.payload)
  assert.equal(value.runId, guard.runId)
  assert.equal(value.nonce, nonce, 'Stale daily observer challenge')
  assert.equal(value.owner.toLowerCase(), 'ezio-mini\\15119')
  assert.equal(value.sid, 'S-1-5-21-587408064-856152790-2583328096-1001')
  assert.equal(value.sessionId, 2)
  const age = now - Date.parse(value.capturedAtUtc)
  assert(Number.isFinite(age) && age >= -2000 && age < 20000, 'Daily observer snapshot expired')
  assert.deepEqual(value.configs.map(row => resolve(row.path).toLowerCase()), paths.map(path => resolve(path).toLowerCase()))
  assert(value.configs.every(row => Number.isInteger(row.bytes) && row.bytes > 0 && /^[a-f0-9]{64}$/.test(row.sha256)))
  return value
}

export async function requestDailySnapshot(guard, paths) {
  const nonce = randomUUID(), request = {runId: guard.runId, nonce, atUtc: new Date().toISOString()}
  await writeFile(join(guard.root, 'request.next.json'), JSON.stringify(request), {flag: 'wx'})
  await rename(join(guard.root, 'request.next.json'), join(guard.root, 'request.json'))
  const until = Date.now() + 30000
  while (Date.now() < until) {
    let envelope
    try { envelope = JSON.parse(await readFile(join(guard.root, 'response.json'), 'utf8')) }
    catch (error) { if (error.code !== 'ENOENT') throw error }
    if (envelope && JSON.parse(envelope.payload).nonce === nonce) {
      return {envelope, value: verifyDailySnapshot(envelope, guard, nonce, paths)}
    }
    await sleep(250)
  }
  throw Error('15119 daily readonly observer did not answer a fresh challenge')
}

async function observe(planFile) {
  const plan = JSON.parse(await readFile(planFile, 'utf8'))
  assert.equal(resolve(plan.runRoot), resolve('D:/Users/Administrator/WorkSpace/Test/post021-candidate2-targeted-20261105'))
  assert.equal(resolve(plan.dailyGuard.root), resolve(join(plan.runRoot, 'daily-guard-attempt2')))
  const ps = join(process.env.SystemRoot, 'System32/WindowsPowerShell/v1.0/powershell.exe')
  const account = JSON.parse(execFileSync(ps, ['-NoProfile', '-NonInteractive', '-Command', '[ordered]@{owner=[Security.Principal.WindowsIdentity]::GetCurrent().Name;sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;sessionId=(Get-Process -Id $PID).SessionId}|ConvertTo-Json -Compress'], {encoding: 'utf8', windowsHide: true, timeout: 15000}))
  assert.equal(account.owner.toLowerCase(), 'ezio-mini\\15119')
  assert.equal(account.sid, 'S-1-5-21-587408064-856152790-2583328096-1001')
  assert.equal(account.sessionId, 2)
  assert.deepEqual(plan.protectedConfigPaths, ['C:/Users/15119/.codex/config.toml', 'D:/Software/Develop/OpenCodeX Desktop/data/opencodex-home/config.json'])
  const root = plan.dailyGuard.root
  await mkdir(root, {recursive: false})
  const {privateKey, publicKey} = generateKeyPairSync('ed25519')
  const publicPem = publicKey.export({type: 'spki', format: 'pem'})
  await writeFile(join(root, 'ready.json'), JSON.stringify({runId: plan.dailyGuard.runId, publicKey: publicPem, ...account, pid: process.pid, atUtc: new Date().toISOString()}))
  const until = Date.now() + 12 * 60 * 60 * 1000
  let lastNonce
  while (Date.now() < until) {
    try { await access(join(plan.runRoot, 'evidence-attempt2/result.json')); break } catch (error) { if (error.code !== 'ENOENT') throw error }
    let request
    try { request = JSON.parse(await readFile(join(root, 'request.json'), 'utf8')) } catch (error) { if (error.code !== 'ENOENT') throw error }
    if (request && request.nonce !== lastNonce) {
      assert.equal(request.runId, plan.dailyGuard.runId)
      assert(/^[a-f0-9-]{36}$/.test(request.nonce))
      const age = Date.now() - Date.parse(request.atUtc)
      assert(Number.isFinite(age) && age >= -2000 && age < 30000)
      const configs = []
      for (const path of plan.protectedConfigPaths) {
        const bytes = await readFile(path)
        configs.push({path, bytes: bytes.length, sha256: sha(bytes)})
      }
      const payload = JSON.stringify({runId: plan.dailyGuard.runId, nonce: request.nonce, ...account, pid: process.pid, capturedAtUtc: new Date().toISOString(), configs})
      const envelope = {payload, signature: sign(null, Buffer.from(payload), privateKey).toString('base64')}
      await writeFile(join(root, 'response.next.json'), JSON.stringify(envelope), {flag: 'wx'})
      await rename(join(root, 'response.next.json'), join(root, 'response.json'))
      lastNonce = request.nonce
    }
    await sleep(250)
  }
  await writeFile(join(root, 'ended.json'), JSON.stringify({pid: process.pid, atUtc: new Date().toISOString(), scope: 'Only read the two fixed daily config paths and wrote hash-only test evidence. No ACL, config, process or runtime changes.'}))
}
if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) await observe(process.argv[2])
