import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import {describe,it,expect} from 'vitest'
import {componentInventory} from '../qa/component-inventory.mjs'
import {componentRegistry} from '../src/components/registry.ts'
describe('complete component source ownership',()=>{
 it('registers every public visual/helper source and traces actual production and development imports',()=>{
  const result=componentInventory(path.resolve(__dirname,'../src'),componentRegistry)
  expect(result.unregistered).toEqual([]);expect(result.missingImplementations).toEqual([])
  const buttons=result.components.find(c=>c.id==='button.text');expect(buttons.applicationReach).toContain('App.vue');expect(buttons.applicationReach).toContain('features/models/ChannelBatchDialog.vue');expect(buttons.developmentReach).toContain('dev/component-gallery/GalleryApp.vue')
  const nodes=result.components.find(c=>c.id==='node.selection');expect(nodes.directConsumers).toEqual(expect.arrayContaining(['features/models/ChannelsView.vue','features/models/ChannelEditor.vue','dev/component-gallery/NodeExamples.vue']))
 })
 it('finds a newly unregistered source and resolves relative, alias and dynamic import chains without inventing runtime coverage',()=>{
  const root=fs.mkdtempSync(path.join(os.tmpdir(),'ocx-component-inventory-'))
  try{
   fs.mkdirSync(path.join(root,'components/ui'),{recursive:true});fs.mkdirSync(path.join(root,'features'),{recursive:true});fs.mkdirSync(path.join(root,'dev'))
   fs.writeFileSync(path.join(root,'components/ui/Control.vue'),'<template>Control</template>');fs.writeFileSync(path.join(root,'components/ui/Unregistered.vue'),'<template>new</template>')
   fs.writeFileSync(path.join(root,'features/Consumer.vue'),"import Control from '@/components/ui/Control.vue'")
   fs.writeFileSync(path.join(root,'App.vue'),"import Consumer from './features/Consumer.vue'")
   fs.writeFileSync(path.join(root,'dev/gallery.ts'),"void import('../features/Consumer.vue')")
   const result=componentInventory(root,[{id:'control',kind:'component',gallery:'control',implementations:['components/ui/Control.vue'],consumers:[]}])
   expect(result.unregistered).toEqual(['components/ui/Unregistered.vue']);expect(result.components[0].applicationReach).toEqual(['App.vue','features/Consumer.vue']);expect(result.components[0].developmentReach).toEqual(['dev/gallery.ts'])
   expect(result.scope).toContain('visual/functional/platform acceptance separate')
  }finally{fs.rmSync(root,{recursive:true,force:true})}
 })
})
