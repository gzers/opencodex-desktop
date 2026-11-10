// Test bridge only. Caller MUST own/seal these input files through retained native handles.
import {readFileSync,writeFileSync} from 'node:fs';import {createHash} from 'node:crypto';
import {validate} from './cache-candidate.ts';
const [beforeFile,readyFile,preFile,afterFile,contextFile,scopeFile,out]=process.argv.slice(2);if(!out)throw Error('seven input/output paths required');
const sha=(b:Buffer)=>createHash('sha256').update(b).digest('hex'),stages=['before','ready','pre-stop','official-stop'];
const raw=[beforeFile,readyFile,preFile,afterFile].map(p=>{const b=readFileSync(p);if(b.length>8*1024*1024)throw Error('stage too large');return b});
const rows=raw.map(b=>JSON.parse(b.toString('utf8'))),context=JSON.parse(readFileSync(contextFile,'utf8')),scope=JSON.parse(readFileSync(scopeFile,'utf8'));
context.stageBytesSha256={};context.stageCanonicalSha256={};for(let i=0;i<stages.length;i++){context.stageBytesSha256[stages[i]]=sha(raw[i]);context.stageCanonicalSha256[stages[i]]=sha(Buffer.from(JSON.stringify(rows[i])))}
const result=validate(rows[0],rows[1],rows[2],rows[3],context.runtime.pid,{context,scope});writeFileSync(out,JSON.stringify(result,null,2),{flag:'wx'});
