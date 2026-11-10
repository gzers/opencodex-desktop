import {join} from 'node:path';import {readFileSync,writeFileSync} from 'node:fs';import assert from 'node:assert/strict';
const root=process.env.PROBE_ROOT!,imp=(p:string)=>import(join(root,'package/src',p));
const {loadConfig}=await imp('config.ts');const {validateConfigCandidate}=await imp('config/diagnostics.ts');const {gatherRoutedModelsForCatalogGather}=await imp('codex/catalog/routed-gather.ts');const {buildCatalogEntriesFromObservedState}=await imp('codex/catalog/build-entries.ts');const {routeConcreteModel}=await imp('router.ts');
const names=['Krill-Renamed','中文渠道','Team One','团队/海外'],id='gpt-6.1-sol',nested='vendor/gpt-6.1-sol',manual='gpt-6-sol',config:any={port:0,defaultProvider:'fixture0',providers:{}};
for(let i=0;i<names.length;i++)config.providers['fixture'+i]={adapter:'openai-responses',baseUrl:'https://fixture.invalid/v1',apiKey:'synthetic-never-sent',liveModels:false,models:[id,nested,manual],selectedModels:[id,nested,manual],modelDisplayNames:{[manual]:'人工指定名称'}};
const v=validateConfigCandidate(config);assert(v.ok,v.ok?'':v.error);writeFileSync(join(process.env.OPENCODEX_HOME!,'config.json'),JSON.stringify(config));
const loaded=loadConfig(),go=await gatherRoutedModelsForCatalogGather(loaded,{authStoreBuffer:null});
const models=buildCatalogEntriesFromObservedState({template:null,gptSlugs:['gpt-5.6'],goModels:go,wsEnabled:false,multiAgentMode:'default',exactComboSlugs:new Set(),accountSelectors:[],suppressedBareNativeSlugs:new Set(),disabledNativeAccountSlugs:new Set(),multiAgentV2Enabled:false});
const official={models},source=JSON.stringify(official,null,2),projected=JSON.parse(source),cases:any[]=[];
const map=Object.fromEntries(names.map((name,i)=>['fixture'+i,name]));
for(const row of projected.models){
 const info=row.opencodex_capability_provenance;
 if(!info||!map[info.provider])continue;
 const before=models.find((x:any)=>x.slug===row.slug)!;assert(before);
 const route=routeConcreteModel(loaded,row.slug);assert.equal(route.providerName,info.provider);assert.equal(route.modelId,info.model_id);
 if(!loaded.providers[info.provider].modelDisplayNames?.[info.model_id])row.display_name=map[info.provider]+'/'+info.model_id;
 const original={...before},actual={...row};delete original.display_name;delete actual.display_name;assert.deepEqual(actual,original);
 if(info.model_id===manual)assert.equal(row.display_name,'人工指定名称');
 const after=routeConcreteModel(loaded,row.slug);assert.equal(after.providerName,route.providerName);assert.equal(after.modelId,route.modelId);
 cases.push({slug:row.slug,label:row.display_name,routeProvider:after.providerName,routeModel:after.modelId,explicitDisplayPreserved:info.model_id===manual});
}
assert.equal(cases.length,12);assert.equal(projected.models.find((x:any)=>x.slug==='gpt-5.6').display_name,models.find((x:any)=>x.slug==='gpt-5.6').display_name);
writeFileSync(join(root,'out/official-catalog.json'),source);writeFileSync(join(root,'out/desktop-catalog.json'),JSON.stringify(projected,null,2));writeFileSync(join(root,'out/projection-source-result.json'),JSON.stringify({scope:'Real config load + official static gather/build + Desktop-only display label projection + actual route resolution',cases,nativeEntryUnchanged:true,nonDisplayFieldsUnchanged:true,upstreamSourceModified:false},null,2));
console.log(JSON.stringify({result:'pass',routedCases:cases.length,nativeEntryUnchanged:true}));
