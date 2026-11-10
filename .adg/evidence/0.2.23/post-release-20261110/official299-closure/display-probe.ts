import {writeFileSync,readFileSync,mkdirSync} from 'node:fs';
import {join} from 'node:path';
import assert from 'node:assert/strict';
const root=process.env.PROBE_ROOT!;
const imp=(p:string)=>import(join(root,'package/src',p));
const {validateConfigCandidate}=await imp('config/diagnostics.ts');
const {loadConfig}=await imp('config.ts');
const {gatherRoutedModelsForCatalogGather}=await imp('codex/catalog/routed-gather.ts');
const {buildCatalogEntriesFromObservedState}=await imp('codex/catalog/build-entries.ts');
const {routeConcreteModel}=await imp('router.ts');
const home=process.env.OPENCODEX_HOME!;mkdirSync(home,{recursive:true});
const id='gpt-6.1-sol';
const base=(name='fixture')=>({port:0,defaultProvider:name,providers:{[name]:{adapter:'openai-responses',baseUrl:'https://fixture.invalid/v1',apiKey:'synthetic-never-sent',liveModels:false,models:[id],selectedModels:[id]}}});
const results:any[]=[];
for(const name of ['Krill','中文渠道','Team One','团队/海外']){
 const c=base();c.providers.fixture.modelDisplayNames={[id]:name+'/'+id};const v=validateConfigCandidate(c);assert.equal(v.ok,false);assert(v.error.includes('must not contain /'));results.push({case:'direct-display-map',name,result:'rejected',reason:'slash forbidden'});
}
for(const name of ['Krill','中文渠道','Team One','团队/海外']){
 const c=base();Object.assign(c.providers.fixture,{alias:name,modelAliases:{[id]:id}});const v=validateConfigCandidate(c);writeFileSync(join(home,'config.json'),JSON.stringify(c));const loaded=loadConfig();const p=loaded.providers.fixture;
 results.push({case:'identity-alias-through-real-load',name,schemaAccepted:v.ok,providerAliasAfterLoad:p.alias??null,modelAliasAfterLoad:p.modelAliases?.[id]??null});
 assert.equal(p.modelAliases?.[id],undefined);
}
for(const name of ['Krill','Krill-Renamed','Team_One','中文渠道','Team One','团队/海外']){
 const c=base(name),v=validateConfigCandidate(c);
 if(!v.ok){results.push({case:'provider-key-real-flow',name,result:'schema-rejected'});continue}
 writeFileSync(join(home,'config.json'),JSON.stringify(c));const loaded=loadConfig();
 const go=await gatherRoutedModelsForCatalogGather(loaded,{authStoreBuffer:null});
 const entries=buildCatalogEntriesFromObservedState({template:null,gptSlugs:[],goModels:go,wsEnabled:false,multiAgentMode:'default',exactComboSlugs:new Set(),accountSelectors:[],suppressedBareNativeSlugs:new Set(),disabledNativeAccountSlugs:new Set(),multiAgentV2Enabled:false});
 const row=entries.find((x:any)=>x.slug===name+'/'+id);assert(row);assert.equal(row.display_name,name+'/'+id);
 const route=routeConcreteModel(loaded,row.slug);assert.equal(route.providerName,name);assert.equal(route.modelId,id);
 results.push({case:'provider-key-real-flow',name,result:'pass',catalogSlug:row.slug,catalogLabel:row.display_name,routeProvider:route.providerName,routeModel:route.modelId});
 if(name==='Krill')writeFileSync(join(root,'out/official-catalog.json'),JSON.stringify({models:entries},null,2));
}
writeFileSync(join(root,'out/display-result.json'),JSON.stringify({scope:'official299 full config validator+real file load+static gather+catalog build+route; no upstream HTTP',results},null,2));
console.log(JSON.stringify({cases:results.length,result:'pass'}));
