import {test} from 'node:test'
import assert from 'node:assert/strict'
import {assertConnectionDialogLayout,overviewScene} from './windows-ui-alignment.mjs'
test('021 connection edit host requires native title, one X and cancel/save actions in order',()=>{
 const layout={native:true,title:'WebDAV 连接',closeCount:1,actions:['取消','保存连接']}
 assert.doesNotThrow(()=>assertConnectionDialogLayout(layout))
 for(const change of [{native:false},{title:'旧连接设置'},{closeCount:0},{closeCount:2},{actions:[]},{actions:['保存连接','取消']},{actions:['取消','保存连接','关闭']}])assert.throws(()=>assertConnectionDialogLayout({...layout,...change}))
})
test('real overview setup gating never admits a partial or missing page',()=>{
 assert.equal(overviewScene([],{present:true,settled:true,checks:3}),'environment-gated')
 assert.equal(overviewScene(Array(6).fill({}),{}),'ready')
 assert.throws(()=>overviewScene([],{present:false,settled:true,checks:3}))
 assert.throws(()=>overviewScene([{}],{present:true,settled:true,checks:3}))
})
