import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir} from 'node:fs/promises'
import {join,resolve} from 'node:path'
import {pathToFileURL} from 'node:url'
import {execFileSync} from 'node:child_process'
import {createHash} from 'node:crypto'
const [candidateArgument,outputArgument]=process.argv.slice(2)
assert(candidateArgument&&outputArgument)
const candidate=resolve(candidateArgument),output=resolve(outputArgument)
await mkdir(output,{recursive:false})
const {windowsDesktopProcessQuery,windowsAppServerProcessQuery}=await import(pathToFileURL(join(candidate,'src/codex/windows-process-identity.ts')).href)
const slash=String.fromCharCode(92)
const basename="'(?i)(?:^|["+slash+slash+"/])codex"+slash+".exe(?:"+slash+"s|$)'"
const queries={desktop:windowsDesktopProcessQuery('D:\\Fixture\\OpenAI.Codex'),appserver:windowsAppServerProcessQuery(basename,"'(?i)codex-code-mode-host'")}
const ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe'),results=[]
for(const [name,query]of Object.entries(queries)){
 const suffix=name==='desktop'?'desktop':'appserver'
 const queryPath=join(output,name+'-generated-query.ps1')
 await writeFile(queryPath,query)
 let harness=await readFile(new URL('./windows-'+suffix+'-owner-probe.test.ps1',import.meta.url),'utf8')
 const variable=name==='desktop'?'taskScript':'caseScript'
 const original='$'+variable+"=Join-Path $PSScriptRoot 'windows-"+suffix+"-owner-probe.ps1'"
 assert(harness.includes(original))
 harness=harness.replace(original,'$'+variable+"=Join-Path $PSScriptRoot '"+name+"-generated-query.ps1'")
 if(name==='appserver')harness=harness.replace("'@","'@\nfunction Add-Type {param($TypeDefinition)}")
 const harnessPath=join(output,name+'-policy-harness.ps1')
 await writeFile(harnessPath,harness)
 let stdout
 try{stdout=execFileSync(ps,['-NoProfile','-NonInteractive','-File',harnessPath],{encoding:'utf8',windowsHide:true,timeout:15000})}
 catch(error){await writeFile(join(output,name+'-failure.stdout.log'),error.stdout??'');await writeFile(join(output,name+'-failure.stderr.log'),error.stderr??'');throw error}
 await writeFile(join(output,name+'-policy.stdout.log'),stdout)
 results.push({query:name,result:'passed',querySha256:createHash('sha256').update(query).digest('hex'),scope:'Installed embedded query; synthetic native identity and CIM rows; no real process control',stdout:stdout.trim()})
}
await writeFile(join(output,'review.json'),JSON.stringify({candidate,results},null,2)+'\n')
console.log(JSON.stringify(results))
