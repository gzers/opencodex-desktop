import {readFile,writeFile} from 'node:fs/promises';
const [port,expressionFile,outputFile]=process.argv.slice(2);
const targets=await(await fetch('http://127.0.0.1:'+port+'/json/list')).json();
const target=targets.find(t=>t.url.startsWith('http://tauri.localhost/'));
if(!target)throw Error('main target absent');
const ws=new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r,j)=>{ws.addEventListener('open',r,{once:true});ws.addEventListener('error',j,{once:true})});
let id=0;const pending=new Map();
ws.addEventListener('message',e=>{const m=JSON.parse(e.data),p=pending.get(m.id);if(p){pending.delete(m.id);m.error?p.reject(Error(m.error.message)):p.resolve(m.result)}});
const call=(method,params={})=>new Promise((resolve,reject)=>{const n=++id,t=setTimeout(()=>{pending.delete(n);reject(Error('timeout '+method))},45000);pending.set(n,{resolve:v=>{clearTimeout(t);resolve(v)},reject});ws.send(JSON.stringify({id:n,method,params}))});
try {
 const r=await call('Runtime.evaluate',{expression:await readFile(expressionFile,'utf8'),awaitPromise:true,returnByValue:true});
 if(r.exceptionDetails)throw Error(JSON.stringify(r.exceptionDetails));
 await writeFile(outputFile,JSON.stringify(r.result.value,null,2));
 console.log(JSON.stringify(r.result.value,null,2));
}finally{ws.close()}
