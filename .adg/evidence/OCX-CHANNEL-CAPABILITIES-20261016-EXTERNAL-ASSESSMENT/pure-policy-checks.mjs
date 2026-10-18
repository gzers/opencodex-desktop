import assert from 'node:assert/strict';
const root="/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/@bitkyc08/opencodex";
const {resolveFastPolicy,decideTier,tierValueAfterDecision}=await import(root+'/src/providers/fastwire.ts');
const {preserveOperatorUltraFastTiers}=await import(root+'/src/codex/catalog/operator-tiers.ts');
const authority=(extra={})=>({providerAdapter:'openai-responses',fastWireDeclaration:undefined,modelWireOverrideAllowed:true,authTransport:'authorization_bearer',capability:{provider:true,models:{}},modelAdapters:{},hardPins:{},registryWireDefaults:{},...extra});
const cases=[
 ['responses inherit preserves ultrafast',authority(),undefined,'ultrafast'],
 ['forced Fast becomes priority',authority(),true,'priority'],
 ['forced default removes tier',authority(),false,undefined],
 ['unsupported removes tier',authority({capability:{provider:false,models:{}}}),undefined,undefined],
 ['wire disallows foreign tiers',authority({fastWireDeclaration:{kind:'service-tier',canonicalToWire:{priority:'priority'},foreignCallerTiers:'drop'}}),undefined,undefined],
 ['unclassified keeps caller tier',authority({capability:{models:{}}}),true,'ultrafast'],
 ['chat without opt-in removes tier',authority({providerAdapter:'openai-chat'}),undefined,undefined],
 ['chat explicit opt-in preserves tier',authority({providerAdapter:'openai-chat',capability:{provider:true,models:{},chatServiceTier:true}}),undefined,'ultrafast'],
];
for(const [name,a,mode,expected] of cases){const p=resolveFastPolicy(a,'gpt-6.1-sol');const d=decideTier(p,mode,'ultrafast');assert.equal(tierValueAfterDecision(d,'ultrafast'),expected,name);console.log('PASS '+name);}
const old={service_tiers:[{id:'ultrafast'}],additional_speed_tiers:['ultrafast']};
let row={slug:'Krill-1/gpt-6.1-sol'};preserveOperatorUltraFastTiers(row,old,true);assert.equal(row.service_tiers[0].id,'ultrafast');console.log('PASS opt-in retains supplied catalog tier');
row={slug:'Krill-1/gpt-6.1-sol',...structuredClone(old)};preserveOperatorUltraFastTiers(row,old,false);assert.equal(row.service_tiers,undefined);assert.equal(row.additional_speed_tiers,undefined);console.log('PASS opt-out removes catalog tier');
row={slug:'Krill-1/gpt-6.1-sol'};preserveOperatorUltraFastTiers(row,undefined,true);assert.equal(row.service_tiers,undefined);console.log('PASS opt-in does not create catalog tier');
row={slug:'Krill-1/gpt-6.1-sol',...structuredClone(old)};preserveOperatorUltraFastTiers(row,undefined,true);assert.equal(row.service_tiers,undefined);console.log('PASS fresh template is not operator declaration');
console.log('12 isolated pure-function checks passed; no server, requests or credentials used.');
