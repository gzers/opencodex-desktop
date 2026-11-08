import {test} from 'node:test'
import assert from 'node:assert/strict'
import {mkdtemp,writeFile,readFile,rm,readdir} from 'node:fs/promises'
import {spawn} from 'node:child_process'
import {tmpdir} from 'node:os'
import {join,resolve} from 'node:path'
import {retryable} from '../../.github/scripts/tauri_build_retry.mjs'
test('packaging download classification never retries compiler or signature failures',()=>{
 assert(retryable(1,'failed to bundle project: http status: 504'))
 for(const [code,text] of [[0,'failed to bundle project: http status: 504'],[1,'compiler error 504'],[1,'failed to bundle project: http status: 403'],[1,'failed to bundle project: signature verification failed']])assert.equal(retryable(code,text),false)
})
async function execute(cli,directory,args=['build']){
 const child=spawn(process.execPath,[resolve('.github/scripts/tauri_build_retry.mjs'),'--cli',cli,...args],{env:{...process.env,OCXD_TAURI_RETRY_EVIDENCE:directory},stdio:['ignore','pipe','pipe']})
 let output='';child.stdout.on('data',c=>output+=c);child.stderr.on('data',c=>output+=c)
 const code=await new Promise((accept,reject)=>{child.on('error',reject);child.on('close',accept)})
 return {code,output}
}
test('actual child retries transient tool download and preserves both attempts',async t=>{
 const root=await mkdtemp(join(tmpdir(),'ocxd-tauri-retry-'));t.after(()=>rm(root,{recursive:true,force:true}))
 const cli=join(root,'cli.mjs'),counter=join(root,'counter'),out=join(root,'evidence')
 await writeFile(cli,"import fs from 'node:fs';const p="+JSON.stringify(counter)+";const n=fs.existsSync(p)?Number(fs.readFileSync(p)):0;fs.writeFileSync(p,String(n+1));if(!n){console.error('failed to bundle project: http status: 504');process.exitCode=1}else console.log('original CLI built verified fixture');")
 const result=await execute(cli,out);assert.equal(result.code,0);assert(result.output.includes('verified fixture'))
 assert.equal(JSON.parse(await readFile(join(out,'attempt-1.json'))).exitCode,1)
 assert.equal(JSON.parse(await readFile(join(out,'attempt-2.json'))).exitCode,0)
 assert.equal(JSON.parse(await readFile(join(out,'attempt-2.json'))).accepted,false)
 assert((await readFile(join(out,'attempt-1.log'),'utf8')).includes('504'))
 const again=await execute(cli,out);assert.notEqual(again.code,0);assert.equal(await readFile(counter,'utf8'),'2')
})
test('actual compiler failure runs once and plain version retains its exact format',async t=>{
 const root=await mkdtemp(join(tmpdir(),'ocxd-tauri-noretry-'));t.after(()=>rm(root,{recursive:true,force:true}))
 const cli=join(root,'cli.mjs'),out=join(root,'evidence')
 await writeFile(cli,"if(process.argv.includes('--version'))console.log('tauri-cli 2.11.4');else{console.error('compiler error 504');process.exitCode=2}")
 assert.equal((await execute(cli,out)).code,2);assert.equal((await readdir(out)).length,2)
 const v=await execute(cli,join(root,'version-evidence'),['--version']);assert.equal(v.code,0);assert.equal(v.output,'tauri-cli 2.11.4\n')
})
