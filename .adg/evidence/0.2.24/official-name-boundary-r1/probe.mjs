import assert from 'node:assert/strict';
const root=process.argv[2];
const names=await import(root+'/src/config/provider-name.ts');
const validation=await import(root+'/src/config/provider-validation.ts');
const hints=await import(root+'/src/codex/catalog/model-hints.ts');
const effort=await import(root+'/src/codex/catalog/effort.ts');
const keyCases=['Krill','Krill.v2','中文渠道','渠道/组','ocxd-0123456789abcdef'];
const result={providerKeys:keyCases.map(value=>({value,accepted:names.isValidProviderName(value)})),labels:[]};
for(const label of ['中文渠道/gpt-6-sol','中文渠道 · gpt-6-sol','中文渠道 · vendor/a','人工名称']){
 const error=validation.modelDisplayNamesConfigError({'gpt-6-sol':label});
 result.labels.push({label,error});
}
assert.equal(result.providerKeys.find(x=>x.value==='中文渠道').accepted,false);
assert.equal(result.labels[0].error?.includes('must not contain /'),true);
assert.equal(result.labels[1].error,null);
const label='中文渠道 · gpt-6-sol';
const before={slug:'ocxd-0123456789abcdef/gpt-6-sol',display_name:'old',priority:4,capabilitySentinel:{keep:true}};
const model={provider:'ocxd-0123456789abcdef',id:'gpt-6-sol',displayName:hints.configuredModelDisplayName({modelDisplayNames:{'gpt-6-sol':label}},'gpt-6-sol')};
const entry=structuredClone(before);effort.applyCatalogModelMetadata(entry,model);
assert.equal(entry.slug,before.slug);assert.equal(entry.display_name,label);assert.deepEqual(entry.capabilitySentinel,before.capabilitySentinel);
result.projection={input:model,output:entry,routeUnchanged:true};
result.scope='official exported validators/display projection only; no save/service/catalog-refresh/real client';
console.log(JSON.stringify(result,null,2));
