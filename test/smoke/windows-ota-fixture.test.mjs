import {test} from 'node:test'
import assert from 'node:assert/strict'
import {seedOtaBaseline} from './windows-ota-fixture.mjs'
test('OTA baseline writes reject an unowned data root before calling any application command',async()=>{
 let calls=0;const root='C:\\runner\\owned';const facts={about:{platform:'Windows'},dataRoot:{currentDataRoot:root,activeDataRoot:root,runtimeActive:true,currentOpencodexHome:root+'\\opencodex-home',opencodexHome:root+'\\opencodex-home',opencodexHomeMode:'inside'}}
 await assert.rejects(seedOtaBaseline(async()=>{calls++},'.',facts,'C:\\Users\\daily'))
 assert.equal(calls,0)
})
