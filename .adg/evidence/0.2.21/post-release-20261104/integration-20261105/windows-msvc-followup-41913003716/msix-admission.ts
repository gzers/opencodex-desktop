import {writeFileSync} from 'node:fs'
import {createHash} from 'node:crypto'
import {windowsDesktopAppAdapter as adapter,windowsDefaultExec} from '../post021-restart-panel/tools/node_modules/@bitkyc08/opencodex/src/codex/desktop-app/windows.ts'
const probes:any[]=[]
const exec=(file:string,args:readonly string[],options:any)=>{const at=performance.now();try{const v=windowsDefaultExec(file,args,options);probes.push({executable:file,argsDigest:createHash('sha256').update(JSON.stringify(args)).digest('hex'),timeout:options?.timeout,elapsedMs:performance.now()-at,result:'returned',bytes:v.length});return v}catch(e:any){probes.push({executable:file,timeout:options?.timeout,elapsedMs:performance.now()-at,result:'error',code:e.code,status:e.status,signal:e.signal,killed:e.killed});throw e}}
const install=adapter.discover(exec),processes=install?adapter.listProcesses(exec,install):null
writeFileSync(import.meta.dir+'/msix-admission.json',JSON.stringify({scope:'unmodified official2.84 Windows default adapter discovery/enumeration only; exact default timeouts; no stop/start/restart/config',install,processes,probes,admittedForRestart:false,reason:processes===null?'Enumeration unknown; no disposable user/session proved':'Current-user package discovery has no per-CODEX_HOME ownership scope; no disposable session proved'},null,2))
console.log('read-only default adapter checks finished')
