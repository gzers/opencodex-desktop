import assert from 'node:assert/strict'
import {readFile,writeFile,cp,access,readdir,mkdir} from 'node:fs/promises'
import {resolve,join,relative} from 'node:path'
import {createHash} from 'node:crypto'
const [sourceArgument,candidateArgument]=process.argv.slice(2)
assert(sourceArgument&&candidateArgument,'usage: node build-runtime-probe-candidate.mjs <published package> <fresh sibling candidate>')
const source=resolve(sourceArgument),candidate=resolve(candidateArgument)
assert.equal(resolve(candidate,'..'),resolve(source,'..'),'Candidate must be a fresh sibling to preserve unchanged dependency resolution')
assert(candidate!==source)
await assert.rejects(access(candidate),{code:'ENOENT'},'Do not overwrite a candidate')
assert.equal(JSON.parse(await readFile(join(source,'package.json'),'utf8')).version,'2.84.0')
const path='src/codex/desktop-app/windows.ts',original=await readFile(join(source,path),'utf8')
const sha=bytes=>createHash('sha256').update(bytes).digest('hex')
assert.equal(sha(Buffer.from(original)),'0c794535ff163d5da262d1d7e33e1aff1e072952889e9421cda9438b0380e207','Exact reviewed upstream adapter required')
const begin=original.indexOf('  const literal = install.root.replace'),end=original.indexOf('  let stdout: string;',begin)
assert(begin>0&&end>begin)
const replacement=[
 '  const rootHex = Buffer.from(install.root, "utf16le").toString("hex");',
 '  const script = [',
 '    "$taskRootBytes=New-Object byte[] (" + rootHex.length / 2 + ")",',
 '    "$taskRootHex=\'" + rootHex + "\'",',
 '    "for($taskIndex=0;$taskIndex -lt $taskRootBytes.Length;$taskIndex++){$taskRootBytes[$taskIndex]=[Convert]::ToByte($taskRootHex.Substring($taskIndex*2,2),16)}",',
 '    "$taskInstallRoot=[Text.Encoding]::Unicode.GetString($taskRootBytes)",',
 '    readFileSync(new URL("./windows-desktop-owner-probe.ps1", import.meta.url), "utf8"),',
 '  ].join("\\n");',
 ''
].join('\n')
const modified='import { readFileSync } from "node:fs";\n'+original.slice(0,begin)+replacement+original.slice(end)
assert(modified.includes('const PROBE_TIMEOUT_MS = 10_000;'),'Original bound must remain')
assert.equal(modified.slice(modified.indexOf('function windowsAncestryPids')),original.slice(original.indexOf('function windowsAncestryPids')),'Ancestry/quit/force/relaunch must remain byte-identical')
await cp(source,candidate,{recursive:true,errorOnExist:true,force:false})
await writeFile(join(candidate,path),modified)
const helper=await readFile(new URL('./windows-desktop-owner-probe.ps1',import.meta.url))
await writeFile(join(candidate,'src/codex/desktop-app/windows-desktop-owner-probe.ps1'),helper)
const manifest=[]
async function verify(folder){for(const item of await readdir(folder,{withFileTypes:true})){
 const file=join(folder,item.name),rel=relative(source,file).replaceAll('\\','/')
 assert(!item.isSymbolicLink(),'Unexpected source symlink')
 if(item.isDirectory())await verify(file)
 else{const before=await readFile(file),after=await readFile(join(candidate,rel));assert(rel===path||before.equals(after),'Unrelated package file changed: '+rel);manifest.push({path:rel,sourceSha256:sha(before),candidateSha256:sha(after),bytes:after.length})}
}}
await verify(source)
manifest.push({path:'src/codex/desktop-app/windows-desktop-owner-probe.ps1',sourceSha256:null,candidateSha256:sha(helper),bytes:helper.length})
await writeFile(candidate+'.manifest.json',JSON.stringify({kind:'local-runtime-source-candidate-not-published2.84.0',source,candidate,baseVersion:'2.84.0',originalAdapterSha256:sha(Buffer.from(original)),candidateAdapterSha256:sha(Buffer.from(modified)),timeoutMs:10000,changes:['native handle ownerSID/path/creation query; no GetOwner per-processRPC'],files:manifest},null,2)+'\n')
console.log(JSON.stringify({candidate,files:manifest.length,adapterSha256:sha(Buffer.from(modified)),published:false}))
