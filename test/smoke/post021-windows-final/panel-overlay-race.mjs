import assert from 'node:assert/strict'
import { connect } from './cdp-session.mjs'
import { assertModelSandbox, modelConfigPath } from '../windows-model-ui.mjs'
import { mkdir } from 'node:fs/promises'
import { join } from 'node:path'
import {writeFile,readFile}from 'node:fs/promises'
const [endpoint, sandbox, output] = process.argv.slice(2)
assert(endpoint && sandbox && output, 'usage: panel-overlay-race.mjs ENDPOINT SANDBOX OUTPUT')
await mkdir(output, { recursive: true })
const c = await connect(endpoint)
const report = { result: 'fail', fixture: 'Five real native Hide/public-close/Show races; original payload and callback values preserved; no delayed response or mock', cases: [] }
let before, config

try{
const facts={about:await c.invoke('app_about'),dataRoot:await c.invoke('get_data_root_config')}
assertModelSandbox(facts,sandbox);config=modelConfigPath(facts,sandbox);before=await readFile(config)
assert(await c.invoke('app_foreground'),'native foreground required')
const r=await c.evaluate('('+ (async()=>{const pinia=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;document.querySelector('.notification-panel.show button[aria-label="收起通知中心到铃铛"]')?.click();await new Promise(r=>setTimeout(r,300));const original=window.chrome.webview.postMessage,trace=window.__panelTransportTrace=[],cases=[];let serial=0,closeNext=false;window.chrome.webview.postMessage=function(raw){const message=JSON.parse(raw);if(message.cmd!=='sync_embedded_panel')return original.call(this,raw);const id=++serial,payload=message.payload;const begin={id,event:'native-submit',at:performance.now(),action:payload.request?.action,reload:payload.request?.reload};trace.push(begin);for(const [kind,key]of[['native-result','callback'],['native-error','error']]){const callback=window.__TAURI_INTERNALS__.callbacks.get(message[key]);if(callback)window.__TAURI_INTERNALS__.callbacks.set(message[key],value=>{trace.push({id,event:kind,at:performance.now(),value});return callback(value)})}const result=original.call(this,raw);if(begin.action==='hide'&&closeNext){closeNext=false;queueMicrotask(()=>{const button=document.querySelector('.notification-panel.show button[aria-label="收起通知中心到铃铛"]');if(!button){trace.push({event:'missing-public-close',at:performance.now(),id});return}button.click();trace.push({id,event:'public-close',at:performance.now()})})}return result};try{for(let i=0;i<5;i++){const start=trace.length;closeNext=true;pinia._s.get('notifications').panelOpen=true;await new Promise(r=>setTimeout(r,500));const s=trace.slice(start),hide=s.find(e=>e.event==='native-submit'&&e.action==='hide'),close=s.find(e=>e.event==='public-close'),end=s.find(e=>e.event==='native-result'&&e.id===hide?.id),show=s.find(e=>e.event==='native-submit'&&e.action==='show'),shown=s.find(e=>e.event==='native-result'&&e.id===show?.id);cases.push({i,hide,close,end,show,shown,notificationOpen:pinia._s.get('notifications').panelOpen,loaded:pinia._s.get('panel').loaded,trace:JSON.parse(JSON.stringify(pinia._s.get('panel').loadTrace))})}return{cases,transport:trace}}finally{window.chrome.webview.postMessage=original;closeNext=false}}).toString()+')()')
report.cases=r.cases;report.transport=r.transport
for(const row of r.cases){assert(row.hide&&row.close&&row.end&&row.show&&row.shown,'incomplete real IPC observations');assert(row.hide.at<=row.close.at&&row.close.at<row.end.at,'close must precede actual Hide response');assert(row.shown.value.visible&&row.shown.value.ready&&!row.notificationOpen&&row.loaded,'restored native Show must be visible-ready')}
assert((await readFile(config)).equals(before),'config bytes changed');assert(await c.invoke('app_foreground'),'native foreground lost');report.result='pass'
}catch(e){report.error={message:e.message,stack:e.stack};process.exitCode=1;report.transport=await c.evaluate('window.__panelTransportTrace').catch(()=>null)}finally{await writeFile(join(output,'panel-hide-race.json'),JSON.stringify(report,null,2));c.close()}
