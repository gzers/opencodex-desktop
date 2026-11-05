import assert from 'node:assert/strict'
import {test} from 'bun:test'
import {join,resolve} from 'node:path'
import {pathToFileURL} from 'node:url'
const candidate=process.env.POST021_RUNTIME_CANDIDATE
assert(candidate,'Explicit local candidate path required')
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
