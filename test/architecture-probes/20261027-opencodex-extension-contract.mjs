// Isolated source-contract probe. No config, credentials, server, or network access.
// Optional first argument: absolute OpenCodex package root.
import test from 'node:test';
import assert from 'node:assert/strict';
import {stripTypeScriptTypes} from 'node:module';
import {resolve} from 'node:path';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const root=process.argv[2]||'/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/@bitkyc08/opencodex';
const hookFile=resolve(root,'src/plugins/upstream-hooks.ts');
async function importPureSource(file){const code=stripTypeScriptTypes(await readFile(file,'utf8'),{mode:'strip'});return import('data:text/javascript;base64,'+Buffer.from(code).toString('base64'));}
const h=await importPureSource(hookFile);
const forwarding=await importPureSource(resolve(root,'src/lib/provider-client-headers.ts'));
const originalUrl='https://example.invalid/v1/responses';
const reset=()=>h.resetUpstreamRewritersForTests();
const source=await readFile(hookFile);
console.log(JSON.stringify({probe:'source-contract-only',source:hookFile,sha256:createHash('sha256').update(source).digest('hex'),network:false,configRead:false}));
test('No plugin keeps the exact target and headers object',()=>{
 reset();const headers=new Headers({'x-fixture':'unchanged'}),out=h.rewriteUpstream(originalUrl,headers,'http');assert.equal(out.url,originalUrl);assert.equal(out.headers,headers);
});
test('Documented hook exposes only destination, headers and chosen transport; source headers stay unchanged',()=>{
 reset();const headers=new Headers({'thread-id':'fixture-thread-a'});let keys;
 const remove=h.registerUpstreamRewriter('probe',target=>{keys=Object.keys(target).sort();target.url='http://127.0.0.1:19991/outbound/demo';target.headers.set('x-fixture-route','demo');});
 const out=h.rewriteUpstream(originalUrl,headers,'http');assert.deepEqual(keys,['headers','transport','url']);assert.match(out.url,/127/);assert.equal(out.headers.get('x-fixture-route'),'demo');assert.equal(headers.has('x-fixture-route'),false);remove();assert.equal(h.rewriteUpstream(originalUrl,headers,'http').url,originalUrl);
});
test('Concurrent logical sends keep request headers separate; no shared global mutation needed',async()=>{
 reset();h.registerUpstreamRewriter('probe',t=>t.headers.set('x-fixture-copy',t.headers.get('thread-id')));
 const out=await Promise.all(['a','b'].map(async id=>h.rewriteUpstream(originalUrl,new Headers({'thread-id':id}),'http')));
 assert.equal(out[0].headers.get('x-fixture-copy'),'a');assert.equal(out[1].headers.get('x-fixture-copy'),'b');
});
test('Thrown rewrite rolls back partial edits and is disabled for following sends',()=>{
 reset();let calls=0;h.registerUpstreamRewriter('probe',t=>{calls++;t.url='http://127.0.0.1:1';t.headers.set('x-fixture','partial');throw Error('fixture');});
 const out=h.rewriteUpstream(originalUrl,new Headers(),'http');assert.equal(out.url,originalUrl);assert.equal(out.headers.has('x-fixture'),false);h.rewriteUpstream(originalUrl,new Headers(),'http');assert.equal(calls,1);
});
test('Loopback WS rewrite retains WS transport and removes remote proxy for the local hop',()=>{
 reset();let transport;h.registerUpstreamRewriter('probe',t=>{transport=t.transport;t.url='ws://127.0.0.1:19991/outbound/demo';});
 const out=h.rewriteWebSocketDial('wss://example.invalid/v1/responses',{},'http://proxy.invalid:8080');assert.equal(transport,'websocket');assert.match(out.url,/^ws:/);assert.equal(out.proxy,undefined);
});
test('API-key caller-metadata allowlist does not prove stable thread or spawn identity at egress',()=>{
 for(const name of ['thread-id','session-id','x-codex-parent-thread-id','x-openai-subagent','x-codex-turn-metadata'])assert.equal(forwarding.normalizeForwardedClientHeaderName(name),null);
 assert.equal(forwarding.normalizeForwardedClientHeaderName('x-client-request-id'),'x-client-request-id');
});
