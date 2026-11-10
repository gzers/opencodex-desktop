import {writeFileSync,readFileSync,existsSync,unlinkSync} from 'node:fs';
import {join} from 'node:path';
import {isDeepStrictEqual} from 'node:util';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const {loadConfig}=await import('./official299/src/config');
const {readConfigAdmissionSnapshot}=await import('./official299/src/config/diagnostics');
const {safeConfigDTO,providerEditorConfigDTO,parseProviderEditorConfigDTO}=await import('./official299/src/server/auth-cors');
const {handleProviderRoutes}=await import('./official299/src/server/management/provider-routes');
const {withProviderCatalogCapabilityDTO}=await import('./official299/src/server/management/provider-capability-config');
const configPath=join(process.env.OPENCODEX_HOME!,'config.json');
const base={port:0,defaultProvider:'fixture',providers:{fixture:{adapter:'openai-responses',baseUrl:'https://fixture.invalid/v1',apiKey:'synthetic-fixture-not-real',models:['fixture-model']}}};
const cases=[
 {name:'valid-control'},
 {name:'invalid-provider-alias',change:(d:any)=>d.providers.fixture.alias='invalid alias with spaces'},
 {name:'colliding-provider-alias',change:(d:any)=>d.providers.fixture.alias='fixture'},
 {name:'invalid-model-alias',change:(d:any)=>d.providers.fixture.modelAliases={'fixture-model':'bad alias with spaces'}},
 {name:'reserved-model-alias',change:(d:any)=>d.providers.fixture.modelAliases={'fixture-model':'gpt-shadow'}},
 {name:'invalid-sidecar-instance',change:(d:any)=>d.webSearchSidecar={enabled:true,backend:'openai',anthropicInstance:'wrong'}},
 {name:'missing-after-load',after:()=>unlinkSync(configPath)},
 {name:'invalid-json-after-load',after:()=>writeFileSync(configPath,'{malformed-fixture')},
 {name:'models-changed-after-load',after:(d:any)=>{d.providers.fixture.models.push('external-fixture');writeFileSync(configPath,JSON.stringify(d));}},
];
const deny=new Set(['hasApiKey','hasHeaders','xaiResponsesOptInState','initialModelSelection']);
// Same DTO-to-editor projection observed in the actual 299 GUI hook.
function editorProjection(c:any){return {defaultProvider:c.defaultProvider,providers:Object.fromEntries(Object.entries(c.providers).map(([name,row]:any)=>[name,Object.fromEntries(Object.entries(row).filter(([field])=>!deny.has(field)))]))};}
function differingPaths(a:any,b:any,p=''):string[]{if(JSON.stringify(a)===JSON.stringify(b))return [];if(!a||!b||typeof a!=='object'||typeof b!=='object')return [p];return [...new Set([...Object.keys(a),...Object.keys(b)])].flatMap(k=>differingPaths(a[k],b[k],p+'/'+k.replaceAll('~','~0').replaceAll('/','~1')));}
const results=[];
for(const c of cases){const raw:any=structuredClone(base);c.change?.(raw);const before=JSON.stringify(raw);writeFileSync(configPath,before);const loaded=loadConfig();const fileUnchangedByLoad=readFileSync(configPath,'utf8')===before;const gui=withProviderCatalogCapabilityDTO(safeConfigDTO(loaded),loaded);const baseline=editorProjection(gui);const parsed=parseProviderEditorConfigDTO(baseline);assert(parsed.ok,c.name);c.after?.(raw);const snapshot=readConfigAdmissionSnapshot();const inputBytes=existsSync(configPath)?readFileSync(configPath):null;const admitted=snapshot.kind==='read'&&snapshot.diagnostics.source==='file'&&snapshot.diagnostics.error===null;const same=admitted&&isDeepStrictEqual(providerEditorConfigDTO(snapshot.diagnostics.config),parsed.value);let route=null;
 if(!same){const url=new URL('http://127.0.0.1/api/providers');const req=new Request(url,{method:'PUT',headers:{'Content-Type':'application/json'},body:JSON.stringify({baseline,next:baseline})});const response=await handleProviderRoutes({req,url,config:loaded,deps:{}} as any);assert(response);const body:any=await response.json();route={status:response.status,code:body.code,error:body.error};assert.equal(route.status,409);assert.equal(route.code,admitted?'stale_provider_editor_baseline':'provider_config_unavailable');}
 const afterBytes=existsSync(configPath)?readFileSync(configPath):null;assert((inputBytes===null&&afterBytes===null)||(inputBytes!==null&&afterBytes!==null&&inputBytes.equals(afterBytes)),c.name+' rejected write modified file');results.push({name:c.name,fileUnchangedByLoad,publicDifferingPaths:admitted?differingPaths(baseline,providerEditorConfigDTO(snapshot.diagnostics.config)):[],loadedProviderCount:Object.keys(loaded.providers).length,loadedKeyPreserved:loaded.providers.fixture?.apiKey==='synthetic-fixture-not-real',diskKind:snapshot.kind,diskSource:snapshot.diagnostics.source,diagnosticError:snapshot.diagnostics.error,baselineMatchesDisk:same,route,fileUnchangedByAdmission:true,fixtureSha256:inputBytes?createHash('sha256').update(inputBytes).digest('hex'):null});
}
assert(results.find(x=>x.name==='valid-control')?.baselineMatchesDisk);
const report={scope:'Actual official299 loadConfig, disk diagnostics, dashboard DTO and provider PUT rejection route; synthetic fixtures; no listening server, remote network or real credentials. Dependency packages reused read-only: Bun1.4.4/zod4.6.6 match declared exact versions; other existing transitive versions not a clean pinned299 install.',result:'pass',cases:results,productionConfigModified:false};writeFileSync(join(import.meta.dir,'result.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
