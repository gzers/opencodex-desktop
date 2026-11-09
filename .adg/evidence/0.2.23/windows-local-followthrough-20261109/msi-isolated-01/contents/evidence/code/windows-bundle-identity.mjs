import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const marker = Buffer.from('__TAURI_BUNDLE_TYPE_VAR_UNK');
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

// tauri-bundler 2.9.4 patches this one marker per package, then restores
// target/release's EXE. Compare every byte through the expected patched hash.
export function verifyBundleIdentity(original, installed, kind) {
  const suffix = { nsis: 'NSS', msi: 'MSI' }[kind];
  if (!suffix) throw new Error('Unsupported bundle kind');
  const offset = original.indexOf(marker);
  if (offset < 0 || original.indexOf(marker, offset + 1) >= 0) {
    throw new Error('Expected exactly one unpatched Tauri bundle marker');
  }
  const expected = Buffer.from(original);
  Buffer.from('__TAURI_BUNDLE_TYPE_VAR_' + suffix).copy(expected, offset);
  const expectedSha256 = sha256(expected);
  const installedSha256 = sha256(installed);
  if (expectedSha256 !== installedSha256) {
    throw new Error('Installed EXE differs from exact expected Tauri bundle bytes');
  }
  return { builtSha256: sha256(original), expectedSha256, installedSha256,
    bundleMarkerOffset: offset, bundleMarker: suffix, result: 'pass' };
}

// OTA has the frozen source EXE digest rather than its original bytes. Recover
// exactly one matching original by undoing only this installer's marker patch.
export function verifyInstalledBundleHash(installed, expectedBuiltSha256, kind) {
  if (!/^[0-9a-f]{64}$/.test(expectedBuiltSha256)) throw new Error('Frozen original build SHA-256 required');
  const suffix = { nsis: 'NSS', msi: 'MSI' }[kind];
  if (!suffix) throw new Error('Unsupported bundle kind');
  const patched = Buffer.from('__TAURI_BUNDLE_TYPE_VAR_' + suffix);
  const matches = [];
  for (let offset = installed.indexOf(patched); offset >= 0; offset = installed.indexOf(patched, offset + 1)) {
    const original = Buffer.from(installed);
    marker.copy(original, offset);
    if (sha256(original) === expectedBuiltSha256) matches.push(original);
  }
  if (matches.length !== 1) throw new Error('One exact hash-matching frozen original EXE required');
  return verifyBundleIdentity(matches[0], installed, kind);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const args = process.argv.slice(2);
  if(args[0]==='--installed-hash'){
    const [,installed,kind,expected]=args;
    console.log(JSON.stringify(verifyInstalledBundleHash(readFileSync(installed),expected,kind)));
  }else{
    const [original,installed,kind]=args;
    console.log(JSON.stringify(verifyBundleIdentity(readFileSync(original),readFileSync(installed),kind)));
  }
}
