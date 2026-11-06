/** Artifact acceptance only. Run by the Rust test with bundled Node, which
 * delegates Bun-specific delivered source modules to that same bundle's Bun.
 * No runtime source functions, compiler, matcher or catalog results are mocked.
 */
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, realpathSync } from 'node:fs';
import { dirname, isAbsolute, join, relative, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { spawnSync } from 'node:child_process';

const [mode, inputFile] = process.argv.slice(2);
const input = JSON.parse(readFileSync(inputFile, 'utf8'));
const root = realpathSync(input.root);
function owned(path) {
  assert(isAbsolute(path));
  const part = relative(root, realpathSync(path));
  assert(part && part !== '..' && !part.startsWith('..' + sep) && !isAbsolute(part));
}
for (const path of [input.install, process.env.OPENCODEX_HOME, process.env.CODEX_HOME]) owned(path);
const program = join(input.install, 'runtime/opencodex/node_modules/@bitkyc08/opencodex');
const npm = join(input.install, process.platform === 'win32'
  ? 'runtime/node/node_modules/npm/bin/npm-cli.js'
  : 'runtime/node/lib/node_modules/npm/bin/npm-cli.js');
const bun = join(input.install, process.platform === 'win32' ? 'runtime/bun/bun.exe' : 'runtime/bun/bun');
const script = fileURLToPath(import.meta.url);
if (mode === 'node') {
  owned(process.execPath); owned(npm); owned(bun);
  assert.equal(process.versions.node, input.nodeVersion);
  const npmRun = spawnSync(process.execPath, [npm, '--version'], {
    env: process.env, encoding: 'utf8', timeout: 15000, maxBuffer: 1024 * 1024,
  });
  assert.ifError(npmRun.error);
  assert.equal(npmRun.status, 0, npmRun.stderr);
  assert.equal(npmRun.stdout.trim(), input.npmVersion);
  console.log('BUNDLED_NODE ' + JSON.stringify({ node: process.versions.node, npm: npmRun.stdout.trim() }));
  // The delivered TS modules use Bun APIs. Use the bundle's fixed Bun path,
  // not PATH, a compatibility loader, or source copies/transpilation.
  const run = spawnSync(bun, ['--no-env-file', script, 'runtime', inputFile], {
    cwd: program, env: process.env, encoding: 'utf8', timeout: 45000,
    maxBuffer: 4 * 1024 * 1024,
  });
  if (run.stdout) process.stdout.write(run.stdout);
  if (run.stderr) process.stderr.write(run.stderr);
  assert.ifError(run.error);
  assert.equal(run.status, 0, 'delivered runtime source stage failed');
} else {
  assert.equal(mode, 'runtime');
  assert.equal(Bun.version, input.bunVersion);
  assert.equal(JSON.parse(readFileSync(join(program, 'package.json'))).version, input.runtimeVersion);
  owned(process.execPath);
  const protectedFiles = ['package.json','src/config.ts','src/codex/catalog/routed-gather.ts',
    'src/codex/catalog/build-entries.ts','src/codex/catalog/retained-sync.ts','src/providers/slug-codec.ts'];
  const fileHash = (name) => createHash('sha256').update(readFileSync(join(program, name))).digest('hex');
  const sourceHashes = Object.fromEntries(protectedFiles.map((name) => [name, fileHash(name)]));
  const source = (path) => import(pathToFileURL(join(program, path)).href);
  const { loadConfig } = await source('src/config.ts');
  const { gatherRoutedModelsForCatalogGather } = await source('src/codex/catalog/routed-gather.ts');
  const { buildCatalogEntriesFromObservedState } = await source('src/codex/catalog/build-entries.ts');
  const { syncCatalogModels } = await source('src/codex/catalog/retained-sync.ts');
  const { routedSlug, decodeRoutedModelId } = await source('src/providers/slug-codec.ts');
  const configPath = join(process.env.OPENCODEX_HOME, 'config.json');
  const catalogPath = join(process.env.CODEX_HOME, 'catalog.json');
  const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
  const originalBytes = readFileSync(configPath);
  const actual = JSON.parse(originalBytes);
  assert.equal(actual.providers[input.providerId].name, input.label);
  assert.deepEqual(actual.providers[input.providerId].models, input.rawIds);
  assert.equal(actual.providers[input.providerId].liveModels, false);
  assert.equal(actual.providers[input.providerId].authMode, 'none');
  assert.equal(actual.providers[input.providerId].apiKey, undefined);
  assert.deepEqual(actual.futureRoot, input.unknown);
  assert.deepEqual(actual.providers[input.providerId].futureProvider, input.unknown);
  assert.deepEqual(actual.providers.untouched, input.untouched);
  const config = loadConfig();
  assert.equal(config.providers[input.providerId].name, input.label);
  assert.equal(config.providers[input.providerId].liveModels, false);
  const gathered = await gatherRoutedModelsForCatalogGather(config, { authStoreBuffer: null });
  const material = gathered.filter((model) => model.provider === input.providerId);
  assert.deepEqual(material.map((m) => m.id).sort(), [...input.rawIds].sort());
  const entries = buildCatalogEntriesFromObservedState({
    template: null, gptSlugs: [], goModels: gathered, wsEnabled: false,
    multiAgentMode: 'default', exactComboSlugs: new Set(), accountSelectors: [],
    suppressedBareNativeSlugs: new Set(), disabledNativeAccountSlugs: new Set(),
    multiAgentV2Enabled: false,
  });
  for (const id of input.rawIds) {
    const slug = routedSlug(input.providerId, id);
    assert.equal(slug.split('/')[0], input.providerId);
    assert.equal(decodeRoutedModelId(slug.slice(slug.indexOf('/') + 1), input.rawIds), id);
    assert.equal(entries.find((m) => m.slug === slug)?.display_name, input.label.trim() + '/' + id);
  }
  const { resolveCodexCatalogSerializationDatabasePath, resolveEffectiveUserIdentity } = await source('src/codex/user-identity.ts');
  assert.equal(resolveCodexCatalogSerializationDatabasePath(resolveEffectiveUserIdentity(), realpathSync(process.env.CODEX_HOME)), input.catalogLock);
  const result = await syncCatalogModels(config);
  assert.equal(result.refreshOutcome, 'committed', 'actual retained-sync must commit');
  assert.equal(realpathSync(result.path), realpathSync(catalogPath));
  const catalog = JSON.parse(readFileSync(catalogPath, 'utf8'));
  assert.deepEqual(catalog.futureCatalog, input.unknown);
  const rows = input.rawIds.map((id) => {
    const slug = routedSlug(input.providerId, id);
    const row = catalog.models.find((m) => m.slug === slug);
    assert.equal(row?.display_name, input.label.trim() + '/' + id);
    return { slug, display_name: row.display_name, rawId: id };
  });
  assert.equal(hash(readFileSync(configPath)), hash(originalBytes), 'catalog consumer must not rewrite manager config');
  assert.deepEqual(Object.fromEntries(protectedFiles.map((name) => [name, fileHash(name)])), sourceHashes,
    'actual installed module/package bytes remain read-only');
  console.log('MANAGER_RUNTIME_CATALOG ' + JSON.stringify({
    label: input.label, channelId: input.channelId, providerId: input.providerId,
    runtimeVersion: JSON.parse(readFileSync(join(program, 'package.json'))).version,
    node: input.nodeVersion, bun: Bun.version, rows, configSha256: hash(originalBytes),
    catalogSha256: hash(readFileSync(catalogPath)), catalogLock: 'exact own-home hash; cleaned by Rust RAII', catalogHome: 'own TempDir CODEX_HOME',
    network: 'denied by OS sandbox', installedProgram: 'OS sandbox read-only; module hashes unchanged', accountSelectors: [], liveModels: false,
  }));
}
