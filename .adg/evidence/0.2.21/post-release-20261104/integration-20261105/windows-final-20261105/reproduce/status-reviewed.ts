import {readFileSync,appendFileSync} from 'node:fs'
import {join} from 'node:path'
const home=process.env.OPENCODEX_HOME??''
if(!home.includes('post021-windows-final'))throw Error('diagnostic status driver requires its isolated home')
if(process.argv.includes('--version')){console.log('2.84.0');process.exit(0)}
if(!process.argv.includes('status')||!process.argv.includes('--json'))throw Error('status-only diagnostic driver')
const begin=performance.now();let record:any=null,healthy=false
try{record=JSON.parse(readFileSync(join(home,'runtime-port.json'),'utf8'));const health=await(await fetch('http://127.0.0.1:'+record.port+'/healthz',{signal:AbortSignal.timeout(1000)})).json();healthy=health.service==='opencodex'&&health.pid===record.pid}catch{}
const value={status:healthy?'running':'stopped',ready:healthy,health:healthy?'healthy':'unknown',port:healthy?record.port:null,pid:healthy?String(record.pid):null,dataRoot:home,startup:{protection:'none',rebootSafe:false,servicePresent:false,shimPresent:false}}
appendFileSync(join(home,'status-probe.jsonl'),JSON.stringify({at:new Date().toISOString(),elapsedMs:performance.now()-begin,healthy,port:record?.port})+'\n')
console.log(JSON.stringify(value))
