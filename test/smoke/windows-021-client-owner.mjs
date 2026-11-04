// Run by the pinned Bun executable inside the launcher's owned Windows Job. Import
// the published GET /api/settings handler directly; never start a provider.
import assert from 'node:assert/strict'
import {readFile,writeFile,realpath,stat} from 'node:fs/promises'
import {createReadStream} from 'node:fs'
import {execFile} from 'node:child_process'
import {createRequire} from 'node:module'
import {promisify} from 'node:util'
import {createHash} from 'node:crypto'
import {win32,join,resolve,dirname} from 'node:path'
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
export function assertRuntimeCorrespondence(settingsRuntime,machine){
 assert(machine.displayPath===settingsRuntime.path,'Official display of raw command differs from actual settings path')
 assert.equal(machine.runtime.source,settingsRuntime.source,'Resolver/settings binding sources differ')
 assert.equal(machine.runtime.version,settingsRuntime.version,'Resolver/settings runtime versions differ')
}
export function nativeLaunchObserver(output){
 return 'import {channel} from "node:diagnostics_channel";import {writeFileSync} from "node:fs";channel("child_process").subscribe(({process:child})=>child.once("spawn",()=>writeFileSync('+JSON.stringify(output)+',JSON.stringify({path:child.spawnfile,args:child.spawnargs,pid:child.pid,method:"actual published launcher child_process spawn event"}))));'
}
async function publicFile(path,root){
 const actual=await realpath(path);assertOwnedPath(actual,root)
 assert((await stat(actual)).size<=131072,'Published text artifact exceeds capture bound')
 const bytes=await readFile(actual)
 return {path:actual,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),content:bytes.toString('utf8')}
}
async function observePackagedNative(root){
 const codexManifest=join(root,'installation','node_modules','@openai','codex','package.json'),launcher=join(dirname(codexManifest),'bin','codex.js')
 const trace=join(root,'native-launch-observed.json'),node=process.env.OCX_WINDOWS_OWNER_NODE
 assert(node&&win32.isAbsolute(node)&&win32.extname(node).toLowerCase()==='.exe','Explicit native test Node required')
 const preload='data:text/javascript;base64,'+Buffer.from(nativeLaunchObserver(trace)).toString('base64')
 // Observe the published launcher selecting its own optional native package.
 // No hand-written shim/native resolver and no change to the default binding.
 const version=await promisify(execFile)(node,['--import',preload,launcher,'--version'],{cwd:process.cwd(),env:process.env,windowsHide:true,timeout:15000,maxBuffer:131072})
 assert(/codex-cli\s+0[.]166[.]0\b/.test(version.stdout),'Published native version probe did not return 0.166.0')
 const spawned=JSON.parse(await readFile(trace,'utf8')),canonicalRoot=await realpath(root),native=await realpath(spawned.path)
 assertOwnedPath(native,canonicalRoot);assert.equal(win32.extname(native).toLowerCase(),'.exe')
 assert.deepEqual(spawned.args,[spawned.path,'--version'],'Native observation must be a version-only launcher child')
 const require=createRequire(codexManifest),optionalManifest=require.resolve('@openai/codex-win32-x64/package.json')
 const optional=JSON.parse(await readFile(optionalManifest,'utf8'))
 assert.equal(optional.version,'0.166.0-win32-x64');assert(optional.os.includes('win32')&&optional.cpu.includes('x64'))
 assertOwnedPath(native,await realpath(dirname(optionalManifest)))
 const hash=createHash('sha256');for await(const chunk of createReadStream(native))hash.update(chunk)
 return {bindingReplacement:false,actualLauncherSpawn:spawned,path:native,sha256:hash.digest('hex'),bytes:(await stat(native)).size,versionOutput:version.stdout.trim(),
  codexManifest:await publicFile(codexManifest,canonicalRoot),publishedLauncher:await publicFile(launcher,canonicalRoot),optionalManifest:await publicFile(optionalManifest,canonicalRoot)}
}
export async function preserveBindingObservation(out,result,root){
 // Persist the actual returned settings before checking ownership. A failed
 // boundary is still a failure, but must not erase the path needed to explain it.
 await writeFile(out,JSON.stringify(result,null,2))
 const runtime=result.machineObservation.runtime
 assertRuntimeCorrespondence(result.codexRuntime,result.machineObservation)
 const canonicalRoot=await realpath(root),resolved=await realpath(runtime.command)
 assertOwnedPath(resolved,canonicalRoot)
 result.machineObservation.canonicalRoot=canonicalRoot;result.machineObservation.canonicalCommand=resolved
 result.admission=bindingAdmission({path:resolved,source:runtime.source,version:runtime.version},canonicalRoot)
 if(['.cmd','.bat'].includes(win32.extname(resolved).toLowerCase())){
  assert((await stat(resolved)).size<=65536,'Public npm shim exceeds the capture bound')
  const bytes=await readFile(resolved)
  result.publicShim={path:runtime.command,resolvedPath:resolved,bytes:bytes.length,sha256:createHash('sha256').update(bytes).digest('hex'),content:bytes.toString('utf8'),origin:'actual raw official resolver command; display correspondence to unchanged settings proven'}
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
 const {resolveCodexRuntimeAsync,displayCodexRuntimePath}=await imported('src/codex/runtime.ts')
 const {handleConfigRoutes}=await imported('src/server/management/config-routes.ts')
 // Warm the same official resolver used by its stale-while-revalidate settings
 // route, with no dependency injection or replacement discovery result.
 const resolvedRuntime=await resolveCodexRuntimeAsync()
 const url=new URL('http://127.0.0.1/api/settings')
 const forbidden=()=>{throw Error('Provider/catalog mutation is forbidden in the settings-only probe')}
 const response=await handleConfigRoutes({req:new Request(url),url,config:loadConfig(),deps:{},version:manifest.version,principal:undefined,trustedLoopbackIngress:true,guiSessionIssuance:null,convergeCodexCatalog:forbidden,syncClaudeAgentDefsBestEffort:forbidden})
 assert(response&&response.status===200,'Actual official settings handler failed')
 const settings=await response.json(),runtime=settings.codexRuntime
 const hashes={}
 for(const file of ['package.json','src/server/management/config-routes.ts','src/codex/runtime.ts','src/codex/exec-invocation.ts'])hashes[file]=createHash('sha256').update(await readFile(join(official,file))).digest('hex')
 const result={schemaVersion:1,mode:'default-npm-PATH-discovery',platform:process.platform,officialPackage:manifest.version,codexPackage:codexManifest.version,
  settingsMethod:'published handleConfigRoutes GET /api/settings; no HTTP server/provider',codexRuntime:runtime,admission:{admitted:false,status:'not-yet-validated'},
  machineObservation:{source:'same published resolveCodexRuntimeAsync return value',runtime:resolvedRuntime.runtime,displayPath:displayCodexRuntimePath(resolvedRuntime.runtime.command)},
  forcedRuntimeVariablesAbsent:OVERRIDES.every(key=>!process.env[key]),controlledPath:process.env.PATH,home:process.env.HOME,codexHome:process.env.CODEX_HOME,opencodexHome:process.env.OPENCODEX_HOME,
  officialRoot:await realpath(official),sourceHashes:hashes,providerStarted:false,turnSent:false,toolsExecuted:false,settings}
 await preserveBindingObservation(out,result,root)
 result.publicArtifacts={officialPackage:await publicFile(join(official,'package.json'),await realpath(root)),codex:await observePackagedNative(root)}
 await writeFile(out,JSON.stringify(result,null,2))
 return result
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const [root,official,out]=process.argv.slice(2)
 // Assign native Bun to the launcher's owned Job before upstream imports.
 const gate=new Promise((accept,reject)=>{let line='';process.stdin.setEncoding('utf8');process.stdin.on('data',chunk=>{line+=chunk;if(line.includes(String.fromCharCode(10)))line.trim()==='owned-job-ready'?accept():reject(Error('Owned process gate rejected'))});process.stdin.on('end',()=>reject(Error('Owned process gate missing')))})
 gate.then(()=>probeDefaultSettings(root,official,out)).then(()=>process.exit(0),error=>{process.stderr.write(error.stack+String.fromCharCode(10));process.exit(1)})
}
