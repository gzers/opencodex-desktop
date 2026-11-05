import assert from 'node:assert/strict'
import {test} from 'node:test'
import {waitForNativeReady} from './formal-native-ready.mjs'
test('native commands wait beyond an initial blank WebView',async()=>{
 let at=0,reads=0
 await waitForNativeReady(async()=>++reads>=3,{now:()=>at,sleep:async ms=>{at+=ms}})
 assert.equal(reads,3);assert.equal(at,400)
})
test('a WebView that never mounts fails before any native command',async()=>{
 let at=0
 await assert.rejects(waitForNativeReady(async()=>false,{now:()=>at,sleep:async ms=>{at+=ms},timeoutMs:500}),/no native command performed/)
})
