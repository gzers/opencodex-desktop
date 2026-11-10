import {join} from 'node:path';import {writeFileSync,readFileSync,mkdirSync} from 'node:fs';import assert from 'node:assert/strict';
const root=process.env.PROBE_ROOT!,{injectGrokConfig,stripGrokConfig}=await import(join(process.env.OFFICIAL_SRC!,'grok/inject.ts'));
const cases:any[]=[];
for(const [name,text] of [
 ['empty',''],
 ['native-table','# synthetic fixture\nmodel = "native"\n\n[model.native]\nmodel = "grok-fixture"\nname = "Synthetic native"\napi_backend = "xai"\n'],
 ['original-invalid-shape','# synthetic original\nmodel = "synthetic-grok"\n'],
] as const){const home=join(root,'grok',name);mkdirSync(home);const path=join(home,'config.toml');writeFileSync(path,text);const injected=injectGrokConfig(55581,[{id:'fixture/gpt-6.1-sol',name:'Synthetic routed model'}],{grokHome:home,hostname:'127.0.0.1'});const changed=readFileSync(path,'utf8')!==text;
 let restored:any=null;if(injected.ok&&changed){restored=stripGrokConfig({grokHome:home});assert(restored.ok);assert.equal(readFileSync(path,'utf8'),text)}
 cases.push({name,injectionOk:injected.ok,actualChanged:changed,restorationOk:restored?.ok??false,originalBytesEqual:readFileSync(path,'utf8')===text,message:injected.message});
}
assert(cases.some(x=>x.injectionOk&&x.actualChanged&&x.restorationOk));writeFileSync(join(root,'out/grok-result.json'),JSON.stringify({result:'pass',cases,scope:'real official inject and strip with synthetic selected model; no real server or outbound request'},null,2));console.log(JSON.stringify(cases));
