import {test} from 'node:test'
import assert from 'node:assert/strict'
import {win32} from 'node:path'
import {assert023Paths} from './windows023-native-model.mjs'
const sandbox='C:/fixture/sandbox'
const root=win32.join(sandbox,'data/desktop'),home=win32.join(sandbox,'data/opencodex/home')
function facts(){return {about:{platform:'Windows',version:'0.2.23'},dataRoot:{aggregateRoot:win32.join(sandbox,'data'),currentDataRoot:root,activeDataRoot:root,currentOpencodexHome:home,opencodexHome:home,runtimeActive:true}}}
test('023 native checks require new desktop and opencodex partitions',()=>assert.equal(assert023Paths(facts(),sandbox).config,win32.join(home,'config.json')))
test('023 native checks reject old paths, wrong user roots and inactive saved bindings',()=>{
 for(const [field,value] of [['currentDataRoot',sandbox],['activeDataRoot','C:/other/data/desktop'],['currentOpencodexHome',win32.join(sandbox,'opencodex-home')],['aggregateRoot','C:/other/data'],['runtimeActive',false]]){
  const actual=facts();actual.dataRoot[field]=value;assert.throws(()=>assert023Paths(actual,sandbox),field)
 }
})
test('actual Windows extended drive canonical paths compare to admitted native roots',()=>{
 const actual=facts();for(const field of ['aggregateRoot','currentDataRoot','activeDataRoot','currentOpencodexHome','opencodexHome'])actual.dataRoot[field]=String.fromCharCode(92,92,63,92)+actual.dataRoot[field]
 assert.doesNotThrow(()=>assert023Paths(actual,sandbox))
})
