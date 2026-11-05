import assert from 'node:assert/strict'
import {test} from 'node:test'
import {runInNewContext} from 'node:vm'
import {observePublicRestart} from './formal-restart-observation.mjs'
function observe({dialog=false,error='',noticeOpen=true,toast=''}){
 const modal={querySelector:selector=>selector==='.ui-dialog-header'?{textContent:'重启 Codex'}:error?{textContent:error}:null}
 const document={querySelectorAll:selector=>selector==='dialog[open]'?(dialog?[modal]:[]):toast?[{textContent:toast}]:[],querySelector:()=>({__vue_app__:{config:{globalProperties:{$pinia:{_s:{get:()=>({open:noticeOpen})}}}}}})}
 return runInNewContext('('+observePublicRestart.toString()+')()',{document})
}
test('a real public error is terminal even when no invoke observer exists',()=>{
 const result=observe({dialog:true,error:'官方无法读取 Codex 桌面进程'})
 assert.equal(result.state,'failed');assert.equal(result.error,'官方无法读取 Codex 桌面进程')
})
test('closed notice alone cannot certify restart',()=>{
 assert.equal(observe({noticeOpen:false}).state,'pending')
 assert.equal(observe({noticeOpen:false,toast:'配置已保存'}).state,'pending')
 assert.equal(observe({noticeOpen:false,toast:'Codex 已重启'}).state,'ui-confirmed')
})
