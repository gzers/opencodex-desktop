// Test admission only. No legacy search, path discovery or migration.
import assert from 'node:assert/strict'
import {win32} from 'node:path'
import {assertModelSandbox} from './windows-model-ui.mjs'
export function runtimeAcceptanceLayout(version,sandbox){
 assert(win32.isAbsolute(sandbox),'Explicit absolute physical sandbox required')
 const v2=version==='0.2.23'
 if(!v2)assert(['0.2.16','0.2.17','0.2.18','0.2.19','0.2.20','0.2.21','0.2.22'].includes(version),'Unadmitted runtime acceptance version')
 return {physicalRoot:sandbox,dataRoot:v2?win32.join(sandbox,'data','desktop'):sandbox,userHome:win32.join(sandbox,'home'),opencodexHome:v2?win32.join(sandbox,'data','opencodex','home'):win32.join(sandbox,'opencodex-home'),installationRoot:v2?win32.join(sandbox,'program'):null}
}
export function assert023RuntimeAdmission(facts,sandbox){
 assert.equal(facts.about.version,'0.2.23')
 const layout=runtimeAcceptanceLayout(facts.about.version,sandbox)
 assertModelSandbox(facts,layout.dataRoot)
 const slash=String.fromCharCode(92),extended=slash+slash+'?'+slash
 const normalize=p=>{const n=win32.normalize(p);return (n.startsWith(extended+'UNC'+slash)?slash+slash+n.slice(8):n.startsWith(extended)?n.slice(4):n).toLowerCase()}
 assert.equal(normalize(facts.dataRoot.aggregateRoot),normalize(win32.join(sandbox,'data')),'023 aggregate escaped the explicit owned layout')
 assert.equal(facts.dataRoot.opencodexHomeMode,'inside')
 for(const key of ['currentOpencodexHome','opencodexHome'])assert.equal(normalize(facts.dataRoot[key]),normalize(layout.opencodexHome),'023 HOME escaped the explicit owned layout')
 return layout
}
