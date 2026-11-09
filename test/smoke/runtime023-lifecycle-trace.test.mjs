// Hermetic trace behavior only; never imports/starts the actual runtime.
import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,rmSync,existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {traceSync,traceAsync} from './runtime023-lifecycle-trace.ts';
const root=mkdtempSync(join(tmpdir(),'ocxd023-trace-behavior-'));
const saved=Object.fromEntries(['RUNNER_TEMP','GITHUB_ACTIONS','OCXD_CANDIDATE_TRACE_FILE'].map(k=>[k,process.env[k]]));
try{
 process.env.RUNNER_TEMP=root;process.env.GITHUB_ACTIONS='true';process.env.OCXD_CANDIDATE_TRACE_FILE=join(root,'trace.jsonl');
 const object={value:42},failure=new Error('must-never-serialize-secret');let calls=0;
 assert.equal(traceSync('manager-risk',()=>{calls++;return object}),object);
 assert.equal(await traceAsync('official-teardown',async()=>{calls++;return object}),object);
 assert.throws(()=>traceSync('manager-stop',()=>{calls++;throw failure}),e=>e===failure);
 await assert.rejects(traceAsync('native-client-restore',async()=>{calls++;throw failure}),e=>e===failure);
 assert.equal(calls,4);
 const raw=readFileSync(process.env.OCXD_CANDIDATE_TRACE_FILE,'utf8'),rows=raw.trim().split('\n').map(JSON.parse);
 assert.equal(rows.length,8);assert(!raw.includes('must-never-serialize-secret')&&!raw.includes('value'));
 assert(rows.every(row=>Object.keys(row).sort().join(',')==='elapsedMilliseconds,label,state'&&row.elapsedMilliseconds>=0));
 assert.equal(rows.filter(r=>r.state==='throw').length,2);
 traceSync('unreviewed-label-secret',()=>object);assert.equal(readFileSync(process.env.OCXD_CANDIDATE_TRACE_FILE,'utf8'),raw);
 process.env.OCXD_CANDIDATE_TRACE_FILE=join(root,'..','ocxd023-must-not-create-trace.jsonl');
 const denied=process.env.OCXD_CANDIDATE_TRACE_FILE;assert(!existsSync(denied));assert.equal(traceSync('manager-risk',()=>object),object);assert(!existsSync(denied));
 process.env.OCXD_CANDIDATE_TRACE_FILE=join(root,'trace.jsonl');process.env.GITHUB_ACTIONS='false';
 assert.equal(traceSync('manager-risk',()=>object),object);assert.equal(readFileSync(process.env.OCXD_CANDIDATE_TRACE_FILE,'utf8'),raw);
 console.log('Trace return/rejection identity, exact call count, fixed fields, no raw error, denied label/path/environment: pass; native lifecycle not executed');
}finally{
 for(const [key,value] of Object.entries(saved)){if(value===undefined)delete process.env[key];else process.env[key]=value;}
 rmSync(root,{recursive:true,force:true});assert(!existsSync(root));
}
