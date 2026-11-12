import {pathToFileURL} from 'node:url';
import {join} from 'node:path';
const pkg=process.argv.at(-1);const imp=p=>import(pathToFileURL(join(pkg,'src',p)).href);
const [{effectiveModelAliases,defaultAliasesEnabled},{catalogModelSlug},{deriveEntry},{expandFastRow}]=await Promise.all([imp('providers/default-aliases.ts'),imp('codex/catalog/parsing.ts'),imp('codex/catalog/derive-entry.ts'),imp('server/fast-row.ts')]);
const ids=['claude-opus-5-5','gpt-6-sol','vendor/gemini-3.5-pro'];
const cases=[['omitted',{},{}],['global-on',{defaultModelAliases:true},{}],['provider-off',{defaultModelAliases:true},{defaultAliases:false}],['user-alias',{defaultModelAliases:true},{modelAliases:{'gpt-6-sol':'custom-sol'}}]];
const results=[];
for(const [name,global,provider]of cases){
 const config={...global,providers:{'Team_One':provider}};const aliases=effectiveModelAliases(config,provider,ids);
 const entries=ids.map(id=>{const alias=aliases.get(id);const model={provider:'Team_One',id,...(alias?{displayName:'Team_One/'+alias.alias}:{})};const slug=catalogModelSlug(model);return deriveEntry(null,slug,'Routed via opencodex → Team_One (system).',5,model);});
 const slugs=entries.map(e=>e.slug);if(JSON.stringify(slugs)!==JSON.stringify(ids.map(id=>'Team_One/'+id.replaceAll('/','-'))))throw Error('display settings changed routing');
 results.push({name,defaultEnabled:defaultAliasesEnabled(config,provider),effectiveAliases:[...aliases],slugs,displayNames:entries.map(e=>e.display_name)});
}
const external=expandFastRow({id:'Team_One/gpt-6-sol'},true,{fastRows:true});if(external.length!==2||external[1].id!=='Team_One/gpt-6-sol--fast')throw Error('Fast external projection');
console.log(JSON.stringify({accepted:false,scope:'pure unchanged official alias/derive/Fast functions; no discovery or network',results,externalFastList:external,allDiskSlugsUnchanged:true}));
