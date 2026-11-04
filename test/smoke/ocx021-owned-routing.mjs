// Manual isolated handoff only. This is not the official CLI startup/injection.
// Uses the unmodified published catalog builder; no injected ownership verdict.
import { pathToFileURL } from 'node:url';
import { join } from 'node:path';
import { readFileSync, writeFileSync } from 'node:fs';

const [official, destination, bundledCatalog] = process.argv.slice(2);
if (!official || !destination || !bundledCatalog) throw new Error('explicit package, owned destination and bundled catalog required');
const { loadConfig } = await import(pathToFileURL(join(official, 'src/config.ts')).href);
const { gatherRoutedModels, filterCatalogVisibleModels, buildCatalogEntries } =
  await import(pathToFileURL(join(official, 'src/codex/catalog.ts')).href);
const config = loadConfig();
const models = filterCatalogVisibleModels(await gatherRoutedModels(config), config);
const template = JSON.parse(readFileSync(bundledCatalog, 'utf8')).models
  .find(row => !row.slug.includes('/') && row.base_instructions);
if (!template?.base_instructions) throw new Error('real bundled native template required');
const entries = buildCatalogEntries(template, [], models);
writeFileSync(destination, JSON.stringify({ models: entries }, null, 2), { mode: 0o600 });
console.log(JSON.stringify({ manualHandoff: true, automaticInjection: false,
  builder: 'published src/codex/catalog.ts::buildCatalogEntries',
  selectors: entries.map(row => row.slug), destination }));
