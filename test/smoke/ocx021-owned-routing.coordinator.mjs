// Probe only new per-home SQLite locks before starting an official service.
// No catalog rows are written here, and no OS service verdict is replaced.
import { pathToFileURL } from 'node:url';
import { join, resolve } from 'node:path';
import { chmodSync } from 'node:fs';
import { Database } from 'bun:sqlite';

const [official, expectedRoot, oldProxyPort] = process.argv.slice(2);
const home = resolve(process.env.CODEX_HOME);
if (!home.startsWith(resolve(expectedRoot) + '/')) throw new Error('non-owned Codex home');
if (oldProxyPort === '--runtime-selection-only') {
  // Read the actual unmasked runtime selected by the published resolver under
  // the same owned env/PATH. No dependency seams, config edits, persistence or
  // native override. Preserve server settings separately; never replace them.
  if (process.env.CODEX_CLI_PATH) throw new Error('forced runtime variable in default npm probe');
  const { resolveCodexRuntime, displayCodexRuntimePath, loadPersistedCodexRuntime } = await import(
    pathToFileURL(join(official, 'src/codex/runtime.ts')).href);
  const persistedRuntimeAtRead = loadPersistedCodexRuntime();
  const selected = resolveCodexRuntime();
  console.log(JSON.stringify({ actualPublishedResolver: true, seamsInjected: false,
    persistedByThisProbe: false, resolver: selected,
    displayPath: displayCodexRuntimePath(selected.runtime.command),
    rawRuntime: selected.runtime, persistedRuntimeAtRead, controlledPath: process.env.PATH,
    forcedRuntimeEnvironmentPresent: false }));
} else {
const identity = await import(pathToFileURL(join(official, 'src/codex/user-identity.ts')).href);
const effective = identity.resolveEffectiveUserIdentity();
const paths = [
  identity.resolveCodexCoordinatorDatabasePath(effective, home),
  identity.resolveCodexCatalogSerializationDatabasePath(effective, home),
];
for (const path of paths) {
  const database = new Database(path, { create: true });
  chmodSync(path, 0o600);
  database.exec('BEGIN IMMEDIATE; ROLLBACK;');
  database.close();
}
// Initialize the OLD clean native state using the published owner. A raw
// zero-byte SQLite file is not an authoritative installation transition row.
// This happens before any routed catalog exists, never after channel rename.
const { openCodexCoordinatorTransaction } = await import(
  pathToFileURL(join(official, 'src/codex/transition-state.ts')).href);
const controller = openCodexCoordinatorTransaction(paths[0]);
try { controller.commit(); } finally { controller.close(); }
const { withCatalogWriteSerialization } = await import(
  pathToFileURL(join(official, 'src/codex/catalog-write-serialization.ts')).href);
const outcome = withCatalogWriteSerialization(home, () => 'owned-lock-probe', {
  intent: 'cache', writer: 'sync-cache',
});
if (outcome.kind !== 'completed' || outcome.value !== 'owned-lock-probe') {
  throw new Error('official catalog serialization refused: ' + JSON.stringify(outcome));
}
// Seed the old integration through the exact official producer so all its
// catalog/config/journal/authority records form one valid pre-rename fixture.
const { syncModelsToCodex } = await import(
  pathToFileURL(join(official, 'src/codex/sync.ts')).href);
const oldIntegration = await syncModelsToCodex(Number(oldProxyPort), undefined, null);
console.log(JSON.stringify({ paths, officialSerialization: outcome,
  oldFixtureSeededByPublishedProducer: true, oldIntegration }));
if (!oldIntegration.ok || oldIntegration.status !== 'applied' || !oldIntegration.catalogWritten) {
  throw new Error('old integration producer did not confirm catalog/config initialization');
}
}
