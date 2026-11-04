import {writeFile} from 'node:fs/promises'
const [port,output]=process.argv.slice(2)
const targets=await(await fetch('http://127.0.0.1:'+port+'/json/list')).json(),target=targets.find(t=>t.type==='page'&&t.url.startsWith('http://tauri.localhost/'))
if(!target)throw Error('native target missing')
const socket=new WebSocket(target.webSocketDebuggerUrl)
await new Promise((r,j)=>{socket.addEventListener('open',r,{once:true});socket.addEventListener('error',j,{once:true})})
try{
 const message=await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error('native capture timeout')),15000);socket.addEventListener('message',event=>{const value=JSON.parse(event.data);if(value.id===1){clearTimeout(timer);value.error?reject(Error(value.error.message)):resolve(value.result)}});socket.send(JSON.stringify({id:1,method:'Page.captureScreenshot',params:{format:'png'}}))})
 await writeFile(output,Buffer.from(message.data,'base64'))
 console.log(JSON.stringify({target:target.id,url:target.url,output}))
}finally{socket.close()}
