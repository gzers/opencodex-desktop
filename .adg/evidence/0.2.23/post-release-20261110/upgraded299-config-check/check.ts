import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
let warnings=0;console.warn=()=>{warnings++};
const {readConfigAdmissionSnapshot,validateConfigCandidate}=await import('./package/src/config/diagnostics');
const {loadConfig}=await import('./package/src/config');
const {providerEditorConfigDTO}=await import('./package/src/server/auth-cors');
const path=join(process.env.OPENCODEX_HOME!,'config.json'),bytes=readFileSync(path),raw=JSON.parse(bytes.toString()),before=readFileSync(join(import.meta.dir,'before.json'));
const hash=(x:any)=>createHash('sha256').update(x).digest('hex');
function status(result:any){return {ok:result.ok,errorClass:result.ok?null:typeof result.error==='string'?result.error.split(':')[0]:'unknown',errorDigest:result.ok?null:hash(String(result.error))};}
function diffPaths(a:any,b:any,p=''):string[]{if(JSON.stringify(a)===JSON.stringify(b))return [];if(!a||!b||typeof a!=='object'||typeof b!=='object')return [p];return [...new Set([...Object.keys(a),...Object.keys(b)])].flatMap(k=>diffPaths(a[k],b[k],p+'/'+k.replaceAll('~','~0').replaceAll('/','~1')));}
const diagnostic=readConfigAdmissionSnapshot(),loaded=loadConfig();
const providerNames=Object.keys(raw.providers),candidates=providerNames.filter(k=>!['openai','anthropic','anthropic2','google-antigravity','combos'].includes(k)&&Array.isArray(raw.providers[k].models)&&raw.providers[k].models.length>0&&String(raw.providers[k].adapter).startsWith('openai'));
assert(candidates.length>0,'need existing custom provider for schema-only model edit');const provider=candidates[0],model=raw.providers[provider].models[0];
const cases=[];
function check(name:string,change:(x:any)=>void){const next=structuredClone(raw);change(next);const result=validateConfigCandidate(next);const secretFields=['apiKey','apiKeyPool','headers','proxy'];assert(Object.keys(raw.providers).every(k=>secretFields.every(f=>JSON.stringify(raw.providers[k][f])===JSON.stringify(next.providers[k][f]))));cases.push({name,...status(result),credentialsPreservedInCandidate:true});}
check('unchanged-config',()=>{});
check('channel-note-edit',d=>d.providers[provider].note='isolated-schema-check');
check('existing-model-selection',d=>{d.providers[provider].selectedModels=[model];d.providers[provider].retainModels=[...d.providers[provider].models]});
check('existing-model-context',d=>{d.providers[provider].modelContextWindows={...d.providers[provider].modelContextWindows,[model]:131072}});
check('add-custom-channel-and-model',d=>{d.providers['desktop-analysis-fixture']={adapter:'openai-responses',baseUrl:'https://fixture.invalid/v1',apiKey:'synthetic-never-sent',models:['gpt-analysis-fixture'],selectedModels:['gpt-analysis-fixture'],retainModels:['gpt-analysis-fixture']}});
const rawDto=providerEditorConfigDTO(raw),loadDto=providerEditorConfigDTO(loaded),diagDto=providerEditorConfigDTO(diagnostic.diagnostics.config);
assert(readFileSync(path).equals(bytes),'fixture changed by readonly modules');
const report={scope:'Actual installed299 source validates private copy of actual current config and isolated candidate edits. No real file write, server/API persistence or model request. Does not prove Desktop entire read-compile-save or reload compatibility.',packageVersion:JSON.parse(readFileSync(join(import.meta.dir,'package/package.json'),'utf8')).version,currentInput:{bytes:bytes.length,sha256:hash(bytes),equalDownloadedBackup:bytes.equals(before),providerCount:providerNames.length,apiKeyProviders:providerNames.filter(k=>!!raw.providers[k].apiKey).length,keyPoolProviders:providerNames.filter(k=>!!raw.providers[k].apiKeyPool?.length).length},admission:{kind:diagnostic.kind,source:diagnostic.diagnostics.source,errorClass:diagnostic.diagnostics.error?.split(':')[0]??null,errorDigest:diagnostic.diagnostics.error?hash(diagnostic.diagnostics.error):null},readonlyLoad:{providerCount:Object.keys(loaded.providers).length,publicProjectionSameAsDiagnostic:JSON.stringify(loadDto)===JSON.stringify(diagDto),publicDifferentPaths:diffPaths(loadDto,diagDto),rawVersusLoadedPublicDifferentPaths:diffPaths(rawDto,loadDto),warningCount:warnings},cases,fixtureBytesUnchanged:true,realConfigWritten:false,credentialsPrinted:false};writeFileSync(join(import.meta.dir,'result.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
