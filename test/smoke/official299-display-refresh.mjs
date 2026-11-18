// Explicit temporary-profile probe. Uses unchanged official functions; no service
// startup, provider requests, catalog postprocessing or live user configuration.
import assert from 'node:assert/strict'
import fs from 'node:fs'
import path from 'node:path'
const [packageRoot,input,reportFile]=process.argv.slice(2)
const profile=fs.realpathSync(process.env.HOME??'')
assert(profile.startsWith('/private/tmp/ocxd024-name-refresh-')||profile.startsWith('/private/tmp/ocxd024-name-rollback-'))
assert.equal(process.env.OPENCODEX_HOME,path.join(profile,'opencodex'))
assert.equal(process.env.CODEX_HOME,path.join(profile,'.codex'))
assert(path.isAbsolute(packageRoot)&&path.isAbsolute(input)&&path.isAbsolute(reportFile))
const source=JSON.parse(fs.readFileSync(input,'utf8'))
assert(Object.keys(source.providers).length>0)
for(const provider of Object.values(source.providers)){
 assert.equal(provider.apiKey,'synthetic-only-never-return')
 assert.equal(provider.liveModels,false)
}
const codex=process.env.CODEX_HOME,home=process.env.OPENCODEX_HOME
fs.mkdirSync(codex,{recursive:true,mode:0o700});fs.mkdirSync(home,{recursive:true,mode:0o700})
const catalog=path.join(codex,'opencodex-catalog.json')
if(!fs.existsSync(path.join(codex,'config.toml'))){
 fs.writeFileSync(path.join(codex,'config.toml'),'model_provider = "openai"\nmodel_catalog_json = '+JSON.stringify(catalog)+'\n',{mode:0o600,flag:'wx'})
 fs.writeFileSync(catalog,JSON.stringify({models:[{slug:'gpt-6-sol',display_name:'Native synthetic template',visibility:'list',supported_in_api:true}]}),{mode:0o600,flag:'wx'})
}
const config=await import(packageRoot+'/src/config.ts')
const validation=await import(packageRoot+'/src/config/provider-validation.ts')
const sync=await import(packageRoot+'/src/codex/catalog/retained-sync.ts')
const parsing=await import(packageRoot+'/src/codex/catalog/parsing.ts')
const before=structuredClone(source)
for(const provider of Object.values(source.providers))assert.equal(validation.modelDisplayNamesConfigError(provider.modelDisplayNames),null)
config.saveConfig(source)
const saved=JSON.parse(fs.readFileSync(path.join(home,'config.json'),'utf8'))
for(const [id,provider] of Object.entries(before.providers)){
 assert.deepEqual(saved.providers[id].modelDisplayNames,provider.modelDisplayNames)
 assert.deepEqual(saved.providers[id].models,provider.models)
 assert.equal(saved.providers[id].apiKey,provider.apiKey)
}
const results=[]
for(let pass=0;pass<2;pass++){
 const observed=config.loadConfig()
 const outcome=await sync.syncCatalogModels(observed,{allowWhenDesiredDisabled:true})
 assert.notEqual(outcome.refreshOutcome,'refused')
 const actualPath=parsing.readCodexCatalogPath()
 assert.equal(actualPath,catalog)
 const written=JSON.parse(fs.readFileSync(actualPath,'utf8'))
 const routed=[]
 for(const [provider,row] of Object.entries(before.providers)){
  for(const model of row.models){
   const slug=provider+'/'+model.replaceAll('/','-')
   const entries=written.models.filter(entry=>entry.slug===slug)
   assert.equal(entries.length,1,'unique routed catalog model '+slug)
   assert.equal(entries[0].display_name,row.modelDisplayNames[model])
   routed.push({slug,displayName:entries[0].display_name,contextWindow:entries[0].context_window,reasoning:entries[0].supported_reasoning_levels})
  }
 }
 const readback=JSON.parse(fs.readFileSync(path.join(home,'config.json'),'utf8'))
 for(const [provider,row] of Object.entries(before.providers)){
  assert.deepEqual(readback.providers[provider].modelDisplayNames,row.modelDisplayNames)
  assert.equal(readback.providers[provider].apiKey,row.apiKey)
 }
 results.push({pass,outcome,routed})
}
fs.writeFileSync(reportFile,JSON.stringify({results,officialSaveAndReadback:true,actualCatalogRefresh:true,
 serviceStarted:false,businessSync:false,releaseAcceptance:false},null,2)+'\n',{flag:'wx'})
console.log(JSON.stringify({status:'passed',refreshPasses:results.length,models:results[0].routed.length}))
