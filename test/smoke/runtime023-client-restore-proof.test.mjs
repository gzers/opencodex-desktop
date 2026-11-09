import assert from 'node:assert/strict';
import {injectedClientRoute,restoredFreshNativeClient} from './runtime023-client-restore-proof.mjs';
assert(injectedClientRoute({openai_base_url:'http://127.0.0.1:55581/v1'},55581));
for(const value of [null,{}, {openai_base_url:'http://localhost:55581/v1'}, {openai_base_url:'http://127.0.0.1:55582/v1'}, {openai_base_url:'http://secret@127.0.0.1:55581/v1'}])assert(!injectedClientRoute(value,55581));
assert(restoredFreshNativeClient({model_provider:'openai'}));
for(const key of ['openai_base_url','experimental_realtime_ws_base_url','model_catalog_json']){
 assert(!restoredFreshNativeClient({model_provider:'openai',[key]:'stale'}));
 assert(!restoredFreshNativeClient({model_provider:'openai',[key]:null}));
}
for(const value of [null,{}, {model_provider:'opencodex'}])assert(!restoredFreshNativeClient(value));
console.log('Fresh client route exact port and restore stale-field/identity counterexamples: pass; no actual runtime execution');
