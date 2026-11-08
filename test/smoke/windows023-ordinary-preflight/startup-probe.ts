// Diagnostic only: actual Node -> original official CLI in a new real SID.
import {spawn} from 'node:child_process';
import {mkdirSync,writeFileSync,readFileSync,existsSync} from 'node:fs';
import {join} from 'node:path';
import {request} from 'node:http';
const program=process.env.OCXD_STARTUP_DIAG_PROGRAM!;
const profile=process.env.USERPROFILE!;
const home=join(profile,'.codex-desktop/opencodex/home');
const node=join(program,'runtime/node/node.exe');
const pkg=join(program,'runtime/opencodex/node_modules/@bitkyc08/opencodex');
const version=JSON.parse(readFileSync(join(pkg,'package.json'),'utf8')).version;
if(version!=='2.97.0-desktop023.names.2')throw Error('exact runtime required');
mkdirSync(home,{recursive:true});
const initializeClient=process.env.OCXD_STARTUP_DIAG_INITIALIZE_CODEX==='1';
if(initializeClient){
 const client=join(profile,'.codex');
 if(existsSync(client))throw Error('fresh client fixture required');
 mkdirSync(client,{mode:0o700});
 console.log(JSON.stringify({probe:'explicit-empty-codex-home-fixture',configSeeded:false,catalogSeeded:false,accepted:false}));
}
if(existsSync(join(home,'config.json')))throw Error('fresh diagnostic H required');
const config={hostname:'127.0.0.1',openaiProviderTierVersion:2,defaultProvider:'sweep-fixture',defaultModelAliases:false,providers:{'sweep-fixture':{adapter:'openai-chat',baseUrl:'http://127.0.0.1:1/v1',allowPrivateNetwork:true,apiKey:'synthetic-unused',models:['sweep-fixture-model'],selectedModels:['sweep-fixture-model'],liveModels:false,defaultAliases:false}}};
writeFileSync(join(home,'config.json'),JSON.stringify(config),{mode:0o600,flag:'wx'});
const env:any={HOME:profile,USERPROFILE:profile,OPENCODEX_HOME:home,PATH:join(program,'runtime/node')+';'+join(process.env.SystemRoot!,'System32')+';'+process.env.SystemRoot};
for(const key of ['SystemRoot','WINDIR','COMSPEC','TEMP','TMP','APPDATA','LOCALAPPDATA'])if(process.env[key])env[key]=process.env[key];
const child=spawn(node,[join(pkg,'bin/ocx.mjs'),'start'],{cwd:profile,env,stdio:['ignore','inherit','inherit'],windowsHide:true});
console.log(JSON.stringify({probe:'actual-official-cli-start',parentPid:process.pid,nodePid:child.pid,version,portRequested:'omitted',accepted:false}));
const started=Date.now(),samples:any[]=[];
let exit:any=null;child.on('exit',(code,signal)=>{exit={code,signal}});
const sleep=(ms:number)=>new Promise(r=>setTimeout(r,ms));
const readReady=(port:number)=>new Promise<any>((accept,reject)=>{
 const req=request({hostname:'127.0.0.1',port,path:'/readyz',method:'GET'},res=>{let bytes=0;const chunks:Buffer[]=[];res.on('data',chunk=>{bytes+=chunk.length;if(bytes>65536){res.destroy();reject(Error('bounded ready'))}else chunks.push(chunk)});res.on('error',reject);res.on('end',()=>accept({code:res.statusCode,body:JSON.parse(Buffer.concat(chunks).toString('utf8'))}))});
 req.on('error',reject);req.setTimeout(1000,()=>req.destroy(Error('readonly timeout')));req.end();
});
while(Date.now()-started<60000&&exit===null){
 try{
  const record=JSON.parse(readFileSync(join(home,'runtime-port.json'),'utf8'));
  const ready=await readReady(record.port),body=ready.body;
  const row={elapsedMilliseconds:Date.now()-started,pid:body.pid,port:body.port,identity:body.service==='opencodex'&&body.pid===record.pid&&body.port===record.port,status:body.status,httpStatus:ready.code};
  samples.push(row);
  if(row.status==='ready'||row.status==='failed')break;
 }catch{}
 await sleep(250);
}
console.log(JSON.stringify({probe:'actual-official-cli-readiness-diagnostic',version,initializeClient,elapsedMilliseconds:Date.now()-started,samples,exit,accepted:false,fullSoftwarePassed:false,cleanupAuthority:'original ordinary executor retained SID/image/creation/process handles; no PID-only kill here'}));
// The unchanged executor owns this process family and must prove exit of all
// retained handles before removing the disposable account/Profile.
if(process.env.OCXD_STARTUP_DIAG_LIFECYCLE==='1'&&samples.at(-1)?.status==='ready'){
 const original=JSON.parse(readFileSync(join(home,'runtime-port.json'),'utf8'));
 const conflictHome=join(profile,'.codex-desktop/opencodex/conflict-probe');mkdirSync(conflictHome,{recursive:false});
 writeFileSync(join(conflictHome,'config.json'),JSON.stringify({...config,port:original.port}),{mode:0o600,flag:'wx'});
 const execute=async(action:string,h:string,budget:number)=>{
  const started=Date.now();const p=spawn(node,[join(pkg,'bin/ocx.mjs'),action],{cwd:profile,env:{...env,OPENCODEX_HOME:h},stdio:['ignore','inherit','inherit'],windowsHide:true});
  let result:any=null;p.on('exit',(code,signal)=>{result={code,signal}});
  while(result===null&&Date.now()-started<budget)await sleep(100);
  const report={probe:'actual-official-'+action+'-diagnostic',pid:p.pid,elapsedMilliseconds:Date.now()-started,budgetMilliseconds:budget,exited:result!==null,exit:result,accepted:false};
  console.log(JSON.stringify(report));p.unref();return report;
 };
 await execute('start',conflictHome,30000);
 const same=JSON.parse(readFileSync(join(home,'runtime-port.json'),'utf8'));
 console.log(JSON.stringify({probe:'after-explicit-conflict',originalUnchanged:same.pid===original.pid&&same.port===original.port,conflictRecordPresent:existsSync(join(conflictHome,'runtime-port.json')),accepted:false}));
 await execute('stop',home,10000);
 console.log(JSON.stringify({probe:'after-original-stop-budget',originalRecordPresent:existsSync(join(home,'runtime-port.json')),accepted:false}));
}
child.unref();
