// HTML file drop into real WebView2 -> bounded production IPC -> official archive inspector.
// Synthetic archive is only previewed, never installed or executed.
import assert from 'node:assert/strict'
import {gzipSync} from 'node:zlib'
import {createHash} from 'node:crypto'
import {mkdir,readFile,readdir,writeFile} from 'node:fs/promises'
import {join,win32} from 'node:path'
import {assertModelSandbox} from './windows-model-ui.mjs'
import {runtimeInstallHost,runtimeInstallDomExpression} from './windows-021-runtime-host.mjs'
import {assertNoNativeUiFault} from './windows-021-public-hosts.mjs'
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
 const about=await evaluate("window.__TAURI_INTERNALS__.invoke('app_about')"),dataRoot=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")
 assertModelSandbox({about,dataRoot},root)
 const host=runtimeInstallHost(about.version),faultChecks=[]
 const noFault=async()=>faultChecks.push(await assertNoNativeUiFault({evaluate},about.version))
 const inInstall=(fn,...args)=>runtimeInstallDomExpression(about.version,fn,...args)
 await noFault()
 const archive=gzipSync(Buffer.concat([
  tarFile('package/package.json',JSON.stringify({name:'@bitkyc08/opencodex',version:'2.91.0',bin:{ocx:'bin/ocx.mjs'}})),
  tarFile('package/bin/ocx.mjs',"throw new Error('preview fixture must never run')"),Buffer.alloc(1024),
 ]))
 const sourceDirectory=join(root,'native-preview-input');await mkdir(sourceDirectory,{recursive:true});const source=join(sourceDirectory,'opencodex-2.91.0.tgz');await writeFile(source,archive)
 const sha=createHash('sha256').update(archive).digest('hex')
 const originalSource=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")
 await click('导入离线包','.route-section.active');await wait(inInstall(root=>!!document.querySelector(root+' [data-testid=wiz-offline]')))
 await noFault()
 const box=await evaluate(inInstall(root=>{const b=document.querySelector(root+' [data-testid=wiz-offline]').getBoundingClientRect();return {x:b.x+b.width/2,y:b.y+b.height/2}}))
 await evaluate("(()=>{window.__offlineDropObservation=[];document.addEventListener('drop',e=>window.__offlineDropObservation.push({types:[...e.dataTransfer.types],files:[...e.dataTransfer.files].map(f=>({name:f.name,size:f.size,type:f.type}))}),{capture:true,once:true});return true})()")
 const slash=String.fromCharCode(92),extended=slash+slash+'?'+slash
 const readable=value=>value.startsWith(extended+'UNC'+slash)?slash+slash+value.slice(8):value.startsWith(extended)?value.slice(4):value
 const ordinarySource=readable(source)
 const browserSource=ordinarySource.length<260?ordinarySource:source
 assert.equal(win32.normalize(readable(browserSource)).toLowerCase(),win32.normalize(readable(source)).toLowerCase(),'CDP path must keep the exact file identity')
 const data={items:[],files:[browserSource],dragOperationsMask:1}
 for(const type of ['dragEnter','dragOver','drop'])await call('Input.dispatchDragEvent',{type,x:box.x,y:box.y,data})
 try {
 await wait(inInstall((root,sha)=>document.querySelector(root+' [data-testid=wiz-offline]')?.textContent.includes(sha),sha.slice(0,16)))
 } catch(error) {
  const detail=await evaluate(inInstall(root=>({drop:window.__offlineDropObservation,body:document.querySelector(root)?.textContent})))
  await writeFile(join(output,'offline-drop-failure.json'),JSON.stringify({error:error.message,...detail},null,2)+'\n')
  const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'offline-drop-failure.png'),Buffer.from(image.data,'base64'))
  throw error
 }

 const cache=join(root,'cache','runtime-offline-drop'),dirs=await readdir(cache);assert.equal(dirs.length,1)
 const copy=join(cache,dirs[0],'opencodex-2.91.0.tgz');assert.deepEqual(await readFile(copy),archive)
 await noFault()
 await evaluate(inInstall(root=>document.querySelector(root+' [data-testid=wiz-ack]').click()))
 await wait(inInstall(root=>![...document.querySelectorAll(root+' button')].find(b=>b.textContent.trim()==='开始安装').disabled))
 const shot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'offline-drop-preview.png'),Buffer.from(shot.data,'base64'))
 await click('清除',host.selector)
 await wait(inInstall(root=>[...document.querySelectorAll(root+' button')].find(b=>b.textContent.trim()==='开始安装').disabled))
 for(let n=0;n<100;n++){if((await readdir(cache)).length===0)break;await new Promise(r=>setTimeout(r,100))}
 assert.equal((await readdir(cache)).length,0,'clear must remove only its owned staged copy')
 assert.deepEqual(await readFile(source),archive,'clear must retain the original archive')
 const clearedBox=await evaluate(inInstall(root=>{const b=document.querySelector(root+' [data-testid=wiz-offline]').getBoundingClientRect();return {x:b.x+b.width/2,y:b.y+b.height/2}}))
 for(const type of ['dragEnter','dragOver','drop'])await call('Input.dispatchDragEvent',{type,x:clearedBox.x,y:clearedBox.y,data})
 await wait(inInstall((root,sha)=>document.querySelector(root+' [data-testid=wiz-offline]')?.textContent.includes(sha),sha.slice(0,16)))
 await noFault()
 await click('取消',host.selector);await wait('!document.querySelector('+JSON.stringify(host.selector)+')')
 for(let n=0;n<100;n++){if((await readdir(cache)).length===0)break;await new Promise(r=>setTimeout(r,100))}
 assert.equal((await readdir(cache)).length,0);assert.deepEqual(await readFile(source),archive)
 const after=await evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")
 await noFault()
 for(const key of ['kind','path','managedPrefix','version','history'])assert.deepEqual(after[key],originalSource[key])
 await writeFile(join(output,'offline-drop-host.json'),JSON.stringify({version:about.version,host:host.selector,faultChecks,rootVerified:true},null,2)+String.fromCharCode(10))
 await writeFile(join(output,'offline-drop.json'),JSON.stringify({result:'pass',method:'CDP native HTML File drop; real chunk IPC and archive inspector; preview only',archiveSha256:sha,temporaryCopyExact:true,clearRemovesOnlyCopy:true,clearDisablesInstall:true,redropAfterClear:true,cancelRemovesOnlyCopy:true,originalRetained:true,runtimeSourceUnchanged:true,installsExecuted:0},null,2)+'\n')
}
