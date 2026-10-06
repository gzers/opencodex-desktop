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

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [original, installed, kind] = process.argv.slice(2);
  console.log(JSON.stringify(verifyBundleIdentity(readFileSync(original), readFileSync(installed), kind)));
}
