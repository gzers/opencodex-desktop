import fs from 'node:fs';
import {spawn} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createServer} from '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes/apps/desktop/ui/node_modules/vite/dist/node/index.js';
const out=fileURLToPath(new URL('.',import.meta.url));
const root='/Users/ezio/.codex/worktrees/ocxd-023-ui/ocx-0219-fixes/apps/desktop/ui';
const entry=fs.readFileSync(new URL('./component-entry.mjs',import.meta.url),'utf8');
const binary=fileURLToPath(new URL('../ac-match-05-trace/driver/target/debug/l3-condition-trace-driver',import.meta.url));
if(!fs.existsSync(binary)||!fs.existsSync(out+'../ac-match-05-trace/driver-source-manifest.json'))throw new Error('Frozen driver must be built before browser tests.');
let sequence=0;
const server=await createServer({root,cacheDir:out+'vite-cache',configFile:root+'/vite.config.ts',plugins:[{
name:'l3-native-condition-trace',
resolveId(id){if(id==='virtual:l3-trace')return '\0l3-trace';},
load(id){if(id==='\0l3-trace')return entry;},
configureServer(server){
 server.middlewares.use('/__frozen-cases',(req,res)=>{res.setHeader('Content-Type','application/json');res.end(fs.readFileSync('/Users/ezio/.codex/visualizations/2026/11/06/01a1b0c0-c6d5-74d0-9348-44a8af294a97/condition-preview.real-cases.json','utf8'));});
 server.middlewares.use('/__rust-preview',async(req,res)=>{
  if(req.method!=='POST'){res.statusCode=405;res.end();return;}
  try{
   let text='';for await(const chunk of req){text+=chunk;if(text.length>1024*1024)throw new Error('fixture input limit');}
   const input=JSON.parse(text);if(!['model_v2_validate_content','model_v2_preview_conditions'].includes(input.command))throw new Error('forbidden fixture command');
   const id=++sequence;
   const result=await new Promise((resolve,reject)=>{
    const process=spawn(binary,[],{cwd:out,stdio:['pipe','pipe','pipe']});let stdout='',stderr='';
    const timer=setTimeout(()=>{process.kill('SIGTERM');reject(new Error('fixture Rust driver timeout'));},10000);
    process.stdout.on('data',c=>stdout+=c);process.stderr.on('data',c=>stderr+=c);process.on('error',reject);
    process.on('close',code=>{clearTimeout(timer);if(code!==0)reject(new Error('driver exit '+code+': '+stderr));else {try{resolve(JSON.parse(stdout));}catch(e){reject(e);}}});process.stdin.end(text);
   });
   fs.appendFileSync(out+'native-receipts.ndjson',JSON.stringify({id,input,result})+'\n');
   res.setHeader('Content-Type','application/json');res.end(JSON.stringify(result));
  }catch(cause){res.statusCode=500;res.setHeader('Content-Type','application/json');res.end(JSON.stringify({ok:false,error:{code:'isolated_transport_failure',message:String(cause)}}));}
 });
 server.middlewares.use('/__l3-component',async(req,res,next)=>{if(req.url?.includes('html-proxy'))return next();res.setHeader('Content-Type','text/html');res.end(await server.transformIndexHtml('/__l3-component','<!doctype html><html lang="zh-CN"><meta charset="UTF-8"><title>L3 actual condition trace</title><div id="app"></div><script type="module">import "virtual:l3-trace"</script></html>'));});
}
}],server:{port:43131,host:'127.0.0.1',strictPort:true,hmr:false}});
await server.listen();console.log('Actual frozen Rust condition trace fixture: http://127.0.0.1:43131/__l3-component');
for(const signal of ['SIGTERM','SIGINT'])process.on(signal,async()=>{await server.close();process.exit(0)});
