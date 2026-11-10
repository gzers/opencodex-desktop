const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict'),util=require('node:util'),crypto=require('node:crypto');
const ts=require('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes/apps/desktop/ui/node_modules/typescript');
const root=__dirname,source=fs.readFileSync(path.join(root,'2.99.0/src/server/auth-cors.ts'),'utf8');
const names=new Set(['PROVIDER_CONFIG_FIELD_POLICY','FORBIDDEN_PROVIDER_RUNTIME_FIELDS','REDACTED_PROVIDER_FIELDS','RUNTIME_PROVIDER_FIELDS','PROVIDER_EDITOR_DERIVED_FIELDS','PROVIDER_EDITOR_DENIED_FIELDS','PROVIDER_EDITOR_DENIED_FIELD_SET','PROVIDER_CONFIG_FIELD_SET','isPlainDataRecord','providerEditorProviderDTO','providerEditorConfigDTO','parseProviderEditorConfigDTO']);
const ast=ts.createSourceFile('auth-cors.ts',source,ts.ScriptTarget.Latest,true);let pieces=[];
for(const node of ast.statements){const ids=ts.isFunctionDeclaration(node)?[node.name?.text]:ts.isVariableStatement(node)?node.declarationList.declarations.map(d=>d.name.text):[];if(ids.some(x=>names.has(x)))pieces.push(node.getText(ast).replace(/^export /,''));}
assert.equal(pieces.length,names.size);
const javascript=ts.transpileModule(pieces.join('\n'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
const backend=fs.readFileSync(path.join(root,'provider-routes-299.ts'),'utf8');
const start=backend.indexOf('    const observed = readConfigAdmissionSnapshot();',backend.indexOf('if (url.pathname === "/api/providers" && req.method === "PUT")'));
const end=backend.indexOf('    const preview = providerEditorCandidate(',start);assert(start>0&&end>start);
const block=backend.slice(start,end);
const factory=new Function('publicProviderBaseUrl','sanitizeModelCostsForDisplay','providerMatchesRegistryTransport','getProviderRegistryEntry','registryEntryForProviderDestination','providerCodexAccountMode','redactSecretString','isDeepStrictEqual',javascript+'\nfunction admission(snapshot,baselineResult){const readConfigAdmissionSnapshot=()=>snapshot;const jsonResponse=(body,status)=>({body,status});'+block+' return {status:200,body:{stage:"pre-candidate admission passed"}};}\nreturn {parseProviderEditorConfigDTO,providerEditorConfigDTO,admission};');
// Helpers intentionally model a custom provider with no registry enrichment.
// Exact extracted parser/admission, not full server/schema/runtime execution.
const api=factory(x=>x,()=>undefined,()=>false,()=>undefined,()=>undefined,()=>undefined,x=>x,util.isDeepStrictEqual);
const raw={defaultProvider:'fixture',providers:{fixture:{adapter:'openai-responses',baseUrl:'https://fixture.invalid/v1',models:['fixture-model'],apiKey:'synthetic-not-real'}}};
const publicDto=api.providerEditorConfigDTO(raw),baseline=api.parseProviderEditorConfigDTO(JSON.parse(JSON.stringify(publicDto)));assert(baseline.ok);
const good={kind:'read',diagnostics:{source:'file',error:null,config:raw}};const results=[];
function check(name,actual,predicate){assert(predicate(actual),name);results.push({name,result:'pass',observed:actual.code??actual.body?.code??actual.status});}
check('full-config-extra-root',api.parseProviderEditorConfigDTO({...raw,port:0}),x=>!x.ok&&x.error.includes('only defaultProvider and providers'));
for(const field of ['apiKey','apiKeyPool','initialModelSelection']){const candidate=JSON.parse(JSON.stringify(publicDto));candidate.providers.fixture[field]=field==='apiKey'?'synthetic-not-real':[];check('reject-'+field,api.parseProviderEditorConfigDTO(candidate),x=>!x.ok&&x.code==='invalid_provider_editor_field');}
check('unchanged-public-baseline-passes',api.admission(good,baseline),x=>x.status===200);
const changed=structuredClone(good);changed.diagnostics.config.providers.fixture.models.push('changed-after-editor-open');check('changed-disk-public-dto-stale',api.admission(changed,baseline),x=>x.body.code==='stale_provider_editor_baseline');
for(const [name,snapshot] of [['missing',{kind:'unreadable',diagnostics:{source:'default',error:null}}],['io-failure',{kind:'unreadable',diagnostics:{source:'fallback',error:'invalid_json'}}],['schema-failure',{kind:'read',diagnostics:{source:'fallback',error:'schema_invalid'}}]])check(name+'-unavailable',api.admission(snapshot,baseline),x=>x.body.code==='provider_config_unavailable');
const report={scope:'Extracted 2.99 provider DTO/parser and initial PUT admission only; synthetic config and supplied admission snapshots; not actual Mac reproduction, full schema or complete save/credentials acceptance',result:'pass',cases:results,sourceSha256:crypto.createHash('sha256').update(source).digest('hex'),admissionBlockSha256:crypto.createHash('sha256').update(block).digest('hex'),realCredentialsUsed:false,networkOrRealConfigWrites:false};fs.writeFileSync(path.join(root,'replay-result.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
