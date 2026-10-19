import fs from 'node:fs'
import path from 'node:path'
import {describe,expect,it} from 'vitest'
import {componentRegistry} from '@/components/registry'
const src=path.resolve(__dirname,'../src')
const exists=(file:string)=>fs.existsSync(file.startsWith('../')?path.resolve(src,'../',file):path.resolve(src,file))
describe('component specification source map',()=>{
 it('has unique stable ids and real source/consumer paths, without claiming all platform matrices passed',()=>{
  expect(componentRegistry.length).toBe(43);expect(new Set(componentRegistry.map(c=>c.id)).size).toBe(componentRegistry.length)
  for(const item of componentRegistry){expect(item.verification).toBe('pending_full_matrix');for(const file of [...item.implementations,...item.consumers])expect(exists(file),item.id+': '+file).toBe(true)}
  expect(componentRegistry.find(c=>c.id==='overlay.annotation')).toMatchObject({kind:'prototype_only',implementations:[],consumers:[]})
 })
 it('keeps gallery and registry out of app startup and links only real gallery anchors',()=>{
  const gallery=fs.readFileSync(path.join(src,'dev/component-gallery/GalleryApp.vue'),'utf8')+fs.readFileSync(path.join(src,'dev/component-gallery/FoundationExamples.vue'),'utf8')
  for(const entry of componentRegistry.filter(c=>c.gallery))expect(gallery).toContain('id="'+entry.gallery+'"')
  for(const name of ['main.ts','App.vue'])expect(fs.readFileSync(path.join(src,name),'utf8')).not.toMatch(/components\/registry|component-gallery/)
 })
 it('requires existing switch consumers to use the shared named controlled component',()=>{
  const settings=fs.readFileSync(path.join(src,'routes/SettingsRoute.vue'),'utf8');expect(settings.match(/<UiSwitch\b/g)).toHaveLength(14);expect(settings).not.toMatch(/<button[^>]*role="switch"/)
  expect(componentRegistry.find(c=>c.id==='choice.switch')?.implementations).toContain('components/ui/UiSwitch.vue')
 })
})
