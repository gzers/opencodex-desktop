// Run by the pinned Bun executable inside a production OwnedQueryJob. Import
// the published GET /api/settings handler directly; never start a provider.
import assert from 'node:assert/strict'
import {readFile,writeFile,realpath,stat} from 'node:fs/promises'
import {createHash} from 'node:crypto'
import {win32,join,resolve} from 'node:path'
import {fileURLToPath,pathToFileURL} from 'node:url'

export const OVERRIDES=['CODEX_CLI_PATH','CODEX_BINARY_PATH','CODEX_EXECUTABLE','CODEX_RUNTIME_PATH','OCX_CODEX_PATH']
export function assertOwnedPath(value,root){
 assert(typeof value==='string'&&win32.isAbsolute(value),'Explicit Windows absolute path required')
 const relative=win32.relative(root,value)
 assert(relative&&!relative.startsWith('..')&&!win32.isAbsolute(relative),'Path escaped this runner-owned root')
}
export function assertDefaultDiscoveryEnvironment(env,root,platform){
 assert.equal(platform,'win32','Actual Windows runtime required')
 assert.equal(env.OCX_WINDOWS_ACTUAL_OWNER,'1','Explicit isolated Windows acceptance required')
 for(const key of OVERRIDES)assert(!env[key],'Default discovery must not force a native entry: '+key)
 for(const key of ['HOME','USERPROFILE','CODEX_HOME','OPENCODEX_HOME','TEMP','TMP','APPDATA','LOCALAPPDATA'])assertOwnedPath(env[key],root)
 assert(env.PATH.split(';').some(part=>win32.normalize(part).toLowerCase()===win32.join(root,'installation','node_modules','.bin').toLowerCase()),'Controlled npm installation PATH missing')
}
export function bindingAdmission(runtime,root){
 assert(runtime&&runtime.version==='0.166.0'&&runtime.source==='path','Default published Codex 0.166 PATH binding was not observed')
 assertOwnedPath(runtime.path,root)
 const extension=win32.extname(runtime.path).toLowerCase()
 return {extension,admitted:extension==='.exe',reason:extension==='.exe'?'native-executable':'client_selection_executable: default settings binding is not a native EXE'}
}
export async function preserveBindingObservation(out,result,root){
 // Persist the actual returned settings before checking ownership. A failed
 // boundary is still a failure, but must not erase the path needed to explain it.
 await writeFile(out,JSON.stringify(result,null,2))
 const runtime=result.codexRuntime
 result.admission=bindingAdmission(runtime,root)
 if(['.cmd','.bat'].includes(win32.extname(runtime.path).toLowerCase())){
  const resolved=await realpath(runtime.path);assertOwnedPath(resolved,root)
  assert((await stat(resolved)).size<=65536,'Public npm shim exceeds the capture bound')
  const bytes=await readFile(resolved)
  result.publicShim={path:runtime.path,resolvedPath:resolved,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),content:bytes.toString('utf8'),origin:'actual default settings path; owned public npm install'}
 }
 await writeFile(out,JSON.stringify(result,null,2))
 return result
}
export async function probeDefaultSettings(root,official,out){
 assertDefaultDiscoveryEnvironment(process.env,root,process.platform)
 assertOwnedPath(official,root);assertOwnedPath(out,root)
 const manifest=JSON.parse(await readFile(join(official,'package.json'),'utf8'))
 assert.equal(manifest.name,'@bitkyc08/opencodex');assert.equal(manifest.version,'2.84.0')
 const codexManifest=JSON.parse(await readFile(join(root,'installation','node_modules','@openai','codex','package.json'),'utf8'))
 assert.equal(codexManifest.version,'0.166.0')
 const imported=relative=>import(pathToFileURL(join(official,relative)).href)
 const {loadConfig,getDefaultConfig}=await imported('src/config.ts')
 const configPath=join(process.env.OPENCODEX_HOME,'config.json')
 const config=getDefaultConfig();config.providers={};config.codexAutoStart=false;config.oauthOpenBrowser=false
 await writeFile(configPath,JSON.stringify(config))
 const {resolveCodexRuntimeAsync}=await imported('src/codex/runtime.ts')
 const {handleConfigRoutes}=await imported('src/server/management/config-routes.ts')
 // Warm the same official resolver used by its stale-while-revalidate settings
 // route, with no dependency injection or replacement discovery result.
 await resolveCodexRuntimeAsync()
 const url=new URL('http://127.0.0.1/api/settings')
 const forbidden=()=>{throw Error('Provider/catalog mutation is forbidden in the settings-only probe')}
 const response=await handleConfigRoutes({req:new Request(url),url,config:loadConfig(),deps:{},version:manifest.version,principal:undefined,trustedLoopbackIngress:true,guiSessionIssuance:null,convergeCodexCatalog:forbidden,syncClaudeAgentDefsBestEffort:forbidden})
 assert(response&&response.status===200,'Actual official settings handler failed')
 const settings=await response.json(),runtime=settings.codexRuntime
 const hashes={}
 for(const file of ['package.json','src/server/management/config-routes.ts','src/codex/runtime.ts','src/codex/exec-invocation.ts'])hashes[file]=createHash('sha256').update(await readFile(join(official,file))).digest('hex')
 const result={schemaVersion:1,mode:'default-npm-PATH-discovery',platform:process.platform,officialPackage:manifest.version,codexPackage:codexManifest.version,
  settingsMethod:'published handleConfigRoutes GET /api/settings; no HTTP server/provider',codexRuntime:runtime,admission:{admitted:false,status:'not-yet-validated'},
  forcedRuntimeVariablesAbsent:OVERRIDES.every(key=>!process.env[key]),controlledPath:process.env.PATH,home:process.env.HOME,codexHome:process.env.CODEX_HOME,opencodexHome:process.env.OPENCODEX_HOME,
  officialRoot:await realpath(official),sourceHashes:hashes,providerStarted:false,turnSent:false,toolsExecuted:false,settings}
 return preserveBindingObservation(out,result,root)
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const [root,official,out]=process.argv.slice(2)
 // Assign native Bun to the launcher's owned Job before upstream imports.
 const gate=new Promise((accept,reject)=>{let line='';process.stdin.setEncoding('utf8');process.stdin.on('data',chunk=>{line+=chunk;if(line.includes(String.fromCharCode(10)))line.trim()==='owned-job-ready'?accept():reject(Error('Owned process gate rejected'))});process.stdin.on('end',()=>reject(Error('Owned process gate missing')))})
 gate.then(()=>probeDefaultSettings(root,official,out)).then(()=>process.exit(0),error=>{process.stderr.write(error.stack+String.fromCharCode(10));process.exit(1)})
}
