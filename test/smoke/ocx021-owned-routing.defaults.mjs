// Real producer only: never construct catalog rows or rewrite client model.
import { pathToFileURL } from 'node:url';
import { join } from 'node:path';
const [official, port] = process.argv.slice(2);
const { syncModelsToCodex } = await import(pathToFileURL(join(official, 'src/codex/sync.ts')).href);
const result = await syncModelsToCodex(Number(port), undefined, null);
console.log(JSON.stringify(result));
if (!result.ok || result.status !== 'applied' || result.refreshOutcome !== 'committed') {
  throw new Error('official producer did not confirm the current-config probe');
}
