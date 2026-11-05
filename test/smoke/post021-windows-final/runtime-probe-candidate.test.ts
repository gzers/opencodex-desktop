import assert from 'node:assert/strict'
import {afterAll,test} from 'bun:test'
import {join,resolve} from 'node:path'
import {pathToFileURL} from 'node:url'
const candidate=process.env.POST021_RUNTIME_CANDIDATE
assert(candidate,'Explicit local candidate path required')
// These tests inject every exec callback; no PowerShell process is launched.
// Unix needs the upstream test seam before it can reach that callback. Windows
// retains real trusted-system-directory resolution. This is protocol coverage,
// not a substitute for the separate native Windows owner/restart evidence.
if(process.platform!=='win32'){
 const {setTrustedWindowsElevationExecutablesForTests}=await import(pathToFileURL(join(resolve(candidate),'src/lib/windows-elevation.ts')).href)
 setTrustedWindowsElevationExecutablesForTests({powershell:'/test-only/no-execution/powershell.exe'})
 afterAll(()=>setTrustedWindowsElevationExecutablesForTests(null))
}
const {windowsDesktopAppAdapter:adapter}=await import(pathToFileURL(join(resolve(candidate),'src/codex/desktop-app/windows.ts')).href)
const install={id:'fixture',root:"C:\\Program Files\\Fixture' Unicode 包",relaunch:'fixture!App'}
const good='42 10 2026-11-05T08:00:00.1234567Z '+install.root+'\\app\\ChatGPT.exe'
test('candidate sends hex root and native owner probe with original bounded timeout',()=>{
 let reads=0
 const actual=adapter.listProcesses((file,args,options)=>{
  reads++;assert.equal(options.timeout,10000);assert.equal(options.windowsHide,true)
  const script=args.at(-1)
  assert(script.includes(Buffer.from(install.root,'utf16le').toString('hex')))
  assert(!script.includes(install.root),'Root must not become executable PowerShell source')
  assert(script.includes('GetTokenInformation'));assert(script.includes('CreatedUtcTicks'))
  assert(!script.includes('Invoke-CimMethod'))
  return good
 },install)
 assert.equal(reads,1);assert.equal(actual.length,1);assert.equal(actual[0].createdAt,'2026-11-05T08:00:00.1234567Z')
})
test('unreadable native identity stays unknown, never an empty process list',()=>{
 assert.equal(adapter.listProcesses(()=>{throw Error('native identity unavailable')},install),null)
})
test('same-prefix neighboring package is still rejected after query',()=>{
 assert.equal(adapter.listProcesses(()=>good.replace(install.root,install.root+'-other'),install).length,0)
 assert.equal(adapter.listProcesses(()=>'',install).length,0)
})
const {listWindowsSnapshots,parseWindowsSnapshotOutput}=await import(pathToFileURL(join(resolve(candidate),'src/codex/app-server-processes.ts')).href)
test('app-server candidate keeps narrow source regexes and rejects incomplete output',()=>{
 let calls=0
 const rows=listWindowsSnapshots((script:string)=>{
  calls++;assert(script.includes('GetTokenInformation'));assert(!script.includes('Invoke-CimMethod'))
  assert(script.includes('$taskBasenameMatch='));assert(script.includes('$taskCodeModeMatch='))
  assert(script.includes('Still-live candidate owner identity unavailable'))
  return '42\tD:\\Fixture\\codex.exe app-server\tFixture\\Test'
 })
 assert.equal(calls,1);assert.equal(rows[0].pid,42)
 assert.throws(()=>parseWindowsSnapshotOutput('42\tD:\\Fixture\\codex.exe app-server\tFixture\\Test\n__OCX_ENUM_INCOMPLETE__'),/windows_enum_incomplete/)
 assert.throws(()=>listWindowsSnapshots(()=>{throw Error('native query timeout')}),/native query timeout/)
})
