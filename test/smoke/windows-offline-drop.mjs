// HTML file drop into real WebView2 -> bounded production IPC -> official archive inspector.
// Synthetic archive is only previewed, never installed or executed.
import assert from 'node:assert/strict'
import {gzipSync} from 'node:zlib'
import {createHash} from 'node:crypto'
import {mkdir,readFile,readdir,writeFile} from 'node:fs/promises'
import {join} from 'node:path'
function tarFile(name,body){
 const header=Buffer.alloc(512),bytes=Buffer.from(body)
 header.write(name,0,100);header.write('0000644\0',100);header.write('0000000\0',108);header.write('0000000\0',116)
 header.write(bytes.length.toString(8).padStart(11,'0')+'\0',124);header.write('00000000000\0',136);header.fill(32,148,156);header.write('0',156);header.write('ustar\0',257);header.write('00',263)
 header.write([...header].reduce((a,b)=>a+b,0).toString(8).padStart(6,'0')+'\0 ',148)
 return Buffer.concat([header,bytes,Buffer.alloc((512-bytes.length%512)%512)])
}
export async function verifyWindowsOfflineDrop(call,output,root){
 async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
 async function wait(expression){for(let n=0;n<100;n++){const v=await evaluate(expression);if(v)return v;await new Promise(r=>setTimeout(r,100))}throw Error('offline HTML drop timeout: '+expression)}
 async function click(text,selector){assert(await evaluate('(()=>{const b=[...document.querySelector('+JSON.stringify(selector)+')?.querySelectorAll("button")??[]].find(b=>b.textContent.trim()==='+JSON.stringify(text)+'&&!b.disabled);if(!b)return false;b.click();return true})()'),'offline action missing: '+text)}
 const archive=gzipSync(Buffer.concat([
  tarFile('package/package.json',JSON.stringify({name:'@bitkyc08/opencodex',version:'2.91.0',bin:{ocx:'bin/ocx.mjs'}})),
  tarFile('package/bin/ocx.mjs',"throw new Error('preview fixture must never run')"),Buffer.alloc(1024),
 ]))
 const sourceDirectory=join(root,'native-preview-input');await mkdir(sourceDirectory,{recursive:true});const source=join(sourceDirectory,'opencodex-2.91.0.tgz');await writeFile(source,archive)
 const sha=createHash('sha256').update(archive).digest('hex')
 const originalSource=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")
 await click('导入离线包','.route-section.active');await wait("!!document.querySelector('[data-testid=wiz-offline]')")
 const box=await evaluate("(()=>{const b=document.querySelector('[data-testid=wiz-offline]').getBoundingClientRect();return {x:b.x+b.width/2,y:b.y+b.height/2}})()")
 await evaluate("(()=>{window.__offlineDropObservation=[];document.addEventListener('drop',e=>window.__offlineDropObservation.push({types:[...e.dataTransfer.types],files:[...e.dataTransfer.files].map(f=>({name:f.name,size:f.size,type:f.type}))}),{capture:true,once:true});return true})()")
 const data={items:[],files:[source],dragOperationsMask:1}
 for(const type of ['dragEnter','dragOver','drop'])await call('Input.dispatchDragEvent',{type,x:box.x,y:box.y,data})
 try {
 await wait("document.querySelector('[data-testid=wiz-offline]')?.textContent.includes("+JSON.stringify(sha.slice(0,16))+")")
 } catch(error) {
  const detail=await evaluate("({drop:window.__offlineDropObservation,body:document.querySelector('[data-testid=runtime-install-modal]')?.textContent})")
  await writeFile(join(output,'offline-drop-failure.json'),JSON.stringify({error:error.message,...detail},null,2)+'\n')
  const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'offline-drop-failure.png'),Buffer.from(image.data,'base64'))
  throw error
 }

 const cache=join(root,'cache','runtime-offline-drop'),dirs=await readdir(cache);assert.equal(dirs.length,1)
 const copy=join(cache,dirs[0],'opencodex-2.91.0.tgz');assert.deepEqual(await readFile(copy),archive)
 await evaluate("document.querySelector('[data-testid=wiz-ack]').click()")
 await wait("![...document.querySelectorAll('[data-testid=runtime-install-modal] button')].find(b=>b.textContent.trim()==='开始安装').disabled")
 const shot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'offline-drop-preview.png'),Buffer.from(shot.data,'base64'))
 await click('取消','[data-testid=runtime-install-modal]');await wait("!document.querySelector('[data-testid=runtime-install-modal]')")
 for(let n=0;n<100;n++){if((await readdir(cache)).length===0)break;await new Promise(r=>setTimeout(r,100))}
 assert.equal((await readdir(cache)).length,0);assert.deepEqual(await readFile(source),archive)
 const after=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")
 for(const key of ['kind','path','managedPrefix','version','history'])assert.deepEqual(after[key],originalSource[key])
 await writeFile(join(output,'offline-drop.json'),JSON.stringify({result:'pass',method:'CDP native HTML File drop; real chunk IPC and archive inspector; preview only',archiveSha256:sha,temporaryCopyExact:true,cancelRemovesOnlyCopy:true,originalRetained:true,runtimeSourceUnchanged:true,installsExecuted:0},null,2)+'\n')
}
