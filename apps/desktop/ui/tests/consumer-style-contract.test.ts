import { describe, expect, it } from 'vitest'
import fs from 'node:fs'
import path from 'node:path'
// @ts-expect-error QA scanner is intentionally usable by node without a TS build.
import { consumerStyleViolations } from '../qa/consumer-style-contract.mjs'
const src = path.resolve(__dirname, '../src')
function files(dir:string):string[] { return fs.readdirSync(dir,{withFileTypes:true}).flatMap(e=>e.isDirectory()?files(path.join(dir,e.name)):e.name.endsWith('.vue')?[path.join(dir,e.name)]:[]) }
describe('public control consumer style contract',()=>{
 it('rejects real consumer inset/type overrides including content slots',()=>{
  expect(consumerStyleViolations('<template><UiButton class="action" /></template><style>.action{padding:0;font-size:10px}.box :deep(.btn-content){line-height:1}</style>')).toHaveLength(3)
 })
 it('allows placement and separate icon geometry without a directory exemption',()=>{
  expect(consumerStyleViolations('<template><UiButton class="action" /></template><style>.action{width:100%;justify-content:center}.action svg{height:16px}.tab{padding:8px}</style>')).toEqual([])
 })
 it('checks actual route/feature consumers, not only public component internals',()=>{
  const issues=['routes','features'].flatMap(d=>files(path.join(src,d))).flatMap(file=>consumerStyleViolations(fs.readFileSync(file,'utf8'),path.relative(src,file)))
  expect(issues).toEqual([])
 })
})
