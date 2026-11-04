import {spawn,execFileSync,type ChildProcess} from 'node:child_process'
import {existsSync,readFileSync,writeFileSync} from 'node:fs'
import {join,resolve} from 'node:path'
import {pathToFileURL} from 'node:url'

export async function ownedDesktop(root:string,official:string) {
 if(!resolve(root).includes('post021-restart-panel'))throw Error('isolated acceptance root required')
 const executable=join(root,'OwnedClient.exe')
 if(!existsSync(executable))throw Error('owned Windows executable missing')
 const {restartCodexDesktopApp}=await import(pathToFileURL(join(official,'src/codex/desktop-app-restart.ts')).href)
 let child:ChildProcess
 const launches:number[]=[],stops:number[]=[]
 const start=()=>{child=spawn(executable,[root],{cwd:root,env:process.env,windowsHide:false,stdio:'ignore'});if(!child.pid)throw Error('owned client spawn failed');launches.push(child.pid)}
 const alive=(pid:number)=>{if(!launches.includes(pid))return false;try{process.kill(pid,0);return true}catch{return false}}
 const observation=()=>{
  if(!child?.pid||!alive(child.pid))return []
  const powershell=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
  const command='Get-CimInstance Win32_Process -Filter "ProcessId='+child.pid+'" | Select-Object ProcessId,ParentProcessId,CreationDate,ExecutablePath | ConvertTo-Json -Compress'
  const text=execFileSync(powershell,['-NoProfile','-NonInteractive','-Command',command],{encoding:'utf8',windowsHide:true,timeout:5000})
  const p=JSON.parse(text);if(resolve(p.ExecutablePath).toLowerCase()!==resolve(executable).toLowerCase())throw Error('owned process identity changed')
  return [{pid:p.ProcessId,parentPid:p.ParentProcessId,createdAt:p.CreationDate,executable:p.ExecutablePath}]
 }
 const adapter={discover:()=>({id:'post021-owned-client',root,relaunch:executable}),listProcesses:observation,isShell:(p:any)=>resolve(p.executable).toLowerCase()===resolve(executable).toLowerCase(),ancestryPids:()=>[process.pid],captureRelaunchContext:()=>({}),requestQuit:(_e:any,_i:any,p:any)=>{if(!child?.pid||p.pid!==child.pid||!launches.includes(p.pid))throw Error('unowned stop');process.kill(p.pid);stops.push(p.pid)},forceStop:()=>{throw Error('force stop not used by owned fixture')},relaunch:start}
 start()
 return {restart:async()=>{
  const before=observation(),result=restartCodexDesktopApp({platform:'win32',adapter,execFile:()=>{throw Error('global OS exec forbidden')},allowHandoff:false,isAlive:alive,lock:{lockPath:join(root,'owned-restart.lock')}})
  const deadline=Date.now()+10000
  while(child?.pid&&!existsSync(join(root,'client-readback-'+child.pid+'.txt'))&&Date.now()<deadline)await new Promise(r=>setTimeout(r,50))
  const after=observation(),record={mode:'real Windows fixture shell through unmodified official restart ladder; not the daily Codex desktop app',before,after,result,launches,stops,readbackSha256:child?.pid&&existsSync(join(root,'client-readback-'+child.pid+'.txt'))?new Bun.CryptoHasher('sha256').update(readFileSync(join(root,'client-readback-'+child.pid+'.txt'))).digest('hex'):null}
  writeFileSync(join(root,'owned-restart-result.json'),JSON.stringify(record,null,2))
  if(result.relaunch==='started'&&(!after.length||after[0].pid===before[0]?.pid))throw Error('actual replacement process not proved')
  return result
 },stop:()=>{if(child?.pid&&alive(child.pid))child.kill()}}
}
