import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const root = process.env.OCX_PROBE_PACKAGE;
const resultPath = process.env.OCX_PROBE_RESULT;
assert.ok(root && resultPath && process.env.HOME.includes('ocx-channel-validation-'));
const checks = [], observations = {};
const check = (name, fn) => { fn(); checks.push({ name, result: 'passed' }); };
const nativeFetch = globalThis.fetch;
let forbiddenFetches = 0, server;
globalThis.fetch = async (input, init) => {
  const url = new URL(typeof input === 'string' || input instanceof URL ? input : input.url);
  if (url.hostname !== '127.0.0.1' || url.protocol !== 'http:') {
    forbiddenFetches++;
    throw new Error('Probe disallows external fetch');
  }
  return nativeFetch(input, init);
};
try {
  const { parseRequest } = await import(root + '/src/responses/parser.ts');
  const { createResponsesPassthroughAdapter } = await import(root + '/src/adapters/openai-responses/passthrough.ts');
  const { createTranslatorBudget } = await import(root + '/src/lib/translator-budget.ts');
  const { normalizeRoutedCatalogEntry } = await import(root + '/src/codex/catalog/parsing.ts');
  const { supportsNativeResponsesCompactEndpoint } = await import(root + '/src/providers/openai-tiers-destination.ts');
  const { contextExperimentalEnabled, contextEndpoint } = await import(root + '/src/codex/context-compat.ts');
  const { handleContextHistory } = await import(root + '/src/server/context-history.ts');
  const { resolveDataPlaneAdmissionSecret } = await import(root + '/src/server/auth-cors.ts');
  const { COMPACT_PROMPT } = await import(root + '/src/responses/compaction.ts');
  const received = [];
  const mockSse = 'event: response.completed\ndata: {"type":"response.completed","response":{"id":"fixture-response","status":"completed","output":[],"service_tier":"ultrafast"}}\n\n';
  server = Bun.serve({ hostname: '127.0.0.1', port: 0, async fetch(req) {
    received.push({ path: new URL(req.url).pathname, body: await req.json() });
    return new Response(mockSse, { headers: { 'content-type': 'text/event-stream' } });
  }});
  const provider = { adapter: 'openai-responses', authMode: 'key', baseUrl: server.url.origin,
    responsesPath: '/backend-api/codex/responses', apiKey: 'fixture-upstream-only', allowPrivateNetwork: true };
  function build(body, changes = {}, providerChanges = {}) {
    const parsed = parseRequest(structuredClone(body));
    Object.assign(parsed, changes);
    const budget = createTranslatorBudget();
    try {
      const out = createResponsesPassthroughAdapter({ ...provider, ...providerChanges }).buildRequest(parsed, {
        headers: new Headers({ 'session-id': 'fixture-session' }), providerName: 'krill-fixture', translatorBudget: budget,
      });
      assert.ok(!(out instanceof Promise));
      const result = { ...out, json: JSON.parse(out.body), parsedCompaction: parsed._compactionRequest === true };
      out.releaseBodyObservation?.();
      return result;
    } finally { budget.dispose(); }
  }
  const message = { type: 'message', role: 'user', content: [{ type: 'input_text', text: 'Synthetic validation input only.' }] };
  const base = { model: 'gpt-6.1-sol', stream: true, input: [message], service_tier: 'ultrafast' };
  const plain = build(base);
  check('normal Responses serialization preserves inherited ultrafast', () => assert.equal(plain.json.service_tier, 'ultrafast'));
  check('custom Responses outlet path is used', () => assert.equal(new URL(plain.url).pathname, '/backend-api/codex/responses'));
  const forced = build(base, { options: { tierDecision: { kind: 'set', value: 'priority' } } });
  check('adapter serializes forced priority', () => assert.equal(forced.json.service_tier, 'priority'));
  const dropped = build(base, { options: { tierDecision: { kind: 'drop' } } });
  check('adapter removes dropped caller tier', () => assert.equal(dropped.json.service_tier, undefined));
  const compact = build({ ...base, input: [message, { type: 'compaction_trigger' }],
    tools: [{ type: 'function', name: 'fixture_tool', parameters: { type: 'object', properties: {} } }] });
  check('parser identifies remote-v2 compaction trigger', () => assert.equal(compact.parsedCompaction, true));
  check('routed outlet removes native compaction trigger', () => assert.ok(!compact.json.input.some(x => x.type === 'compaction_trigger')));
  check('routed outlet asks for plaintext checkpoint summary', () => assert.ok(compact.json.input.some(item =>
    Array.isArray(item.content) && item.content.some(part => part.text === COMPACT_PROMPT))));
  check('routed summary outlet removes callable tools', () => assert.equal(compact.json.tools, undefined));
  const decodes = build({ ...base, input: [message, { type: 'compaction_trigger' }] }, {}, { decodesNativeCompactionBlobs: true });
  check('blob replay opt-in does not preserve native trigger', () => assert.ok(!decodes.json.input.some(x => x.type === 'compaction_trigger')));
  check('custom relay has no native standalone compact eligibility', () => assert.equal(supportsNativeResponsesCompactEndpoint('krill-fixture', provider), false));
  check('blob replay flag does not enable standalone compact', () => assert.equal(supportsNativeResponsesCompactEndpoint('krill-fixture', { ...provider, decodesNativeCompactionBlobs: true }), false));
  const row = normalizeRoutedCatalogEntry({ slug: 'krill-fixture/gpt-6.1-sol', display_name: 'Fixture', supports_experimental_context: true, context_window: 128000 }, false, undefined, { ultraFastTier: false });
  check('routed catalog removes experimental context assertion', () => assert.equal(row.supports_experimental_context, undefined));
  const featureConfig = '[features]\ncontext_management = true\n';
  writeFileSync(join(process.env.CODEX_HOME, 'config.toml'), featureConfig);
  check('context gate recognizes boolean form', () => assert.equal(contextExperimentalEnabled(featureConfig), true));
  check('context gate recognizes table form', () => assert.equal(contextExperimentalEnabled('[features.context_management]\nexperimental_mode = true\n'), true));
  check('history endpoint is recognized', () => assert.equal(contextEndpoint('/v1/alpha/history/v2/list_windows'), 'alpha/history/v2/list_windows'));
  const config = { defaultProvider: 'krill-fixture', providers: { 'krill-fixture': provider },
    apiKeys: [{ id: 'fixture-admission', name: 'Fixture', key: 'fixture-proxy-only', createdAt: '2026-10-16T00:00:00Z' }] };
  const admission = resolveDataPlaneAdmissionSecret('fixture-proxy-only', config, 'bearer');
  assert.ok(admission);
  const before = received.length;
  const contextResponse = await handleContextHistory(new Request(server.url.origin + '/v1/alpha/history/v2/list_windows', {
    method: 'POST', headers: { authorization: 'Bearer fixture-proxy-only', 'content-type': 'application/json' },
    body: JSON.stringify({ context: { session_id: 'fixture-session' } }),
  }), config, {}, 'alpha/history/v2/list_windows', undefined, admission);
  const contextJson = await contextResponse.json();
  observations.historyRefusal = { status: contextResponse.status, body: contextJson };
  check('Krill-only config cannot select native history destination', () => {
    assert.equal(contextResponse.status, 400);
    assert.ok(JSON.stringify(contextJson).includes('native ChatGPT forward provider'));
  });
  check('rejected history never reaches channel outlet', () => assert.equal(received.length, before));
  const wireResponse = await fetch(plain.url, { method: plain.method, headers: plain.headers, body: plain.body });
  const wireBytes = await wireResponse.text();
  check('loopback mock receives final ultrafast request', () => assert.equal(received.at(-1).body.service_tier, 'ultrafast'));
  check('unparsed local SSE payload remains unchanged in raw fetch', () => assert.equal(wireBytes, mockSse));
  check('no external fetch attempted', () => assert.equal(forbiddenFetches, 0));
  observations.sseScope = 'Adapter-built request and raw fetch only; no production egress component or whole-proxy stream acceptance tested.';
  observations.mockRequests = received.map(x => ({ path: x.path, serviceTier: x.body.service_tier }));
  const result = { kind: 'isolated-original-component-probe', packageVersion: JSON.parse(readFileSync(root + '/package.json')).version,
    passed: checks.length, failed: 0, checks, observations, forbiddenFetches,
    notVerified: ['Krill acceptance', 'Native background concurrency', 'History/notes success', 'Desktop picker', 'Billing', 'All combinations', 'Whole running proxy'] };
  writeFileSync(resultPath, JSON.stringify(result, null, 2) + '\n');
  console.log(JSON.stringify({ passed: result.passed, failed: result.failed, observations, forbiddenFetches }));
} catch (err) {
  writeFileSync(resultPath, JSON.stringify({ kind: 'probe-failure', passedBeforeFailure: checks.length, checks, error: String(err?.stack ?? err), forbiddenFetches }, null, 2) + '\n');
  console.error(String(err?.stack ?? err)); process.exitCode = 1;
} finally { server?.stop(true); globalThis.fetch = nativeFetch; }
