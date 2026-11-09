// Retry only a transient packaging-tool download after the real CLI failed.
// The official CLI still owns compilation, signatures, tool verification and
// artifact identity. Each failed attempt is retained, never accepted/promoted.
import {spawn} from 'node:child_process'
import {mkdirSync,createWriteStream,writeFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {fileURLToPath} from 'node:url'
export function retryable(code, output) {
 const text=output.replace(/\x1b\[[0-?]*[ -/]*[@-~]/g,'')
 return code!==0 && /failed to bundle project:/i.test(text) && /(?:http status:\s*|status code\s*[:=]?\s*)(?:408|429|500|502|503|504)\b/i.test(text)
}
export async function run(cli,args,directory) {
 if(!cli)throw Error('Explicit pinned Tauri JavaScript CLI is required')
 const build=args.includes('build')
 const maximum=build?3:1
 if(build&&!directory)throw Error('Explicit new packaging evidence directory required')
 if(build)mkdirSync(directory,{recursive:false})
 for(let attempt=1;attempt<=maximum;attempt++){
  const log=build?createWriteStream(resolve(directory,'attempt-'+attempt+'.log'),{flags:'wx'}):null
  let output='',bytes=0,truncated=false
  const child=spawn(process.execPath,[cli,...args],{stdio:['inherit','pipe','pipe'],env:process.env})
  const observe=(chunk,stream)=>{
   stream.write(chunk)
   if(log){const remaining=16*1024*1024-bytes;if(remaining>0){const kept=chunk.subarray(0,remaining);log.write(kept);bytes+=kept.length}if(chunk.length>remaining)truncated=true}
   output=(output+chunk.toString('utf8')).slice(-1024*1024)
  }
  child.stdout.on('data',chunk=>observe(chunk,process.stdout));child.stderr.on('data',chunk=>observe(chunk,process.stderr))
  const code=await new Promise((accept,reject)=>{child.on('error',reject);child.on('close',code=>accept(code??1))})
  if(log)await new Promise(accept=>log.end(accept))
  const transient=retryable(code,output)
  if(build)writeFileSync(resolve(directory,'attempt-'+attempt+'.json'),JSON.stringify({attempt,exitCode:code,transientToolDownloadFailure:transient,logBytes:bytes,logTruncated:truncated,accepted:false},null,2)+'\n',{flag:'wx'})
  if(code===0)return 0
  if(!transient||attempt===maximum)return code
  await new Promise(accept=>setTimeout(accept,attempt*1000))
 }
 return 1
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const args=process.argv.slice(2);let cli=process.env.OCXD_TAURI_CLI
 if(args[0]==='--cli'){cli=args[1];args.splice(0,2)}
 run(cli,args,process.env.OCXD_TAURI_RETRY_EVIDENCE).then(code=>{process.exitCode=code}).catch(error=>{console.error(error.message);process.exitCode=1})
}
