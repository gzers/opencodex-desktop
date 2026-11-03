import test from 'node:test';
import assert from 'node:assert/strict';
import { verifyBundleIdentity,verifyInstalledBundleHash } from './windows-bundle-identity.mjs';
import {createHash} from 'node:crypto';

const original = Buffer.from('prefix\0__TAURI_BUNDLE_TYPE_VAR_UNK\0payload');
const bundled = kind => Buffer.from('prefix\0__TAURI_BUNDLE_TYPE_VAR_' + kind + '\0payload');
test('accepts exact NSIS and MSI package bytes', () => {
  assert.equal(verifyBundleIdentity(original, bundled('NSS'), 'nsis').result, 'pass');
  assert.equal(verifyBundleIdentity(original, bundled('MSI'), 'msi').result, 'pass');
});
test('rejects changed application payload', () => {
  const changed = bundled('NSS');
  changed[changed.length - 1] ^= 1;
  assert.throws(() => verifyBundleIdentity(original, changed, 'nsis'), /exact expected/);
});
test('rejects wrong package marker or unchanged EXE', () => {
  assert.throws(() => verifyBundleIdentity(original, bundled('MSI'), 'nsis'), /exact expected/);
  assert.throws(() => verifyBundleIdentity(original, original, 'nsis'), /exact expected/);
});
test('rejects missing or ambiguous original marker', () => {
  assert.throws(() => verifyBundleIdentity(Buffer.from('payload'), bundled('NSS'), 'nsis'), /exactly one/);
  assert.throws(() => verifyBundleIdentity(Buffer.concat([original, original]), bundled('NSS'), 'nsis'), /exactly one/);
});
test('rejects unsupported bundle kind', () => {
  assert.throws(() => verifyBundleIdentity(original, bundled('NSS'), 'other'), /Unsupported/);
});
test('OTA recovers only the frozen original hash from its exact installer marker',()=>{
  const expected=createHash('sha256').update(original).digest('hex');
  for(const [kind,suffix] of [['msi','MSI'],['nsis','NSS']]){
    const result=verifyInstalledBundleHash(bundled(suffix),expected,kind);
    assert.equal(result.builtSha256,expected);assert.equal(result.result,'pass');
  }
});
test('OTA rejects wrong package kind, changed payload, missing marker and another source EXE',()=>{
  const expected=createHash('sha256').update(original).digest('hex'),changed=bundled('MSI');changed[0]^=1;
  for(const bytes of [changed,original,bundled('NSS'),Buffer.from('no marker')])assert.throws(()=>verifyInstalledBundleHash(bytes,expected,'msi'));
  assert.throws(()=>verifyInstalledBundleHash(bundled('MSI'),'0'.repeat(64),'msi'));
  assert.throws(()=>verifyInstalledBundleHash(bundled('MSI'),'bad','msi'));
});
