import {describe,expect,it} from 'vitest'
import {copyChannel,protectedChannel} from '@/features/models/channelActions'
import {emptyRules,type ChannelPolicy} from '@/features/models/api'
function channel():ChannelPolicy{return {id:'source',label:'来源',templateRefs:[{templateId:'template',revisionId:'revision'}],modelRefs:{},connection:{providerId:'custom',protocol:'openai-chat',authMode:'key',nodes:[{id:'node-a',name:'a',url:'https://a.example.test'},{id:'node-b',name:'b',url:'https://b.example.test'}],activeNodeId:'node-b',credentialRef:'local-key-ref',enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['native/id']},modelOrder:['native/id'],discovered:[{id:'native/id',label:'显示名称',rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{}}}
describe('independent channel copies',()=>{
 it('creates new product, provider and node identities and keeps a secret only after explicit reuse',()=>{
  const source=channel();const copied=copyChannel(source,'副本',false)
  expect(copied.id).not.toBe(source.id);expect(copied.connection.providerId).not.toBe(source.connection.providerId);expect(copied.connection.nodes[0]!.id).not.toBe('node-a');expect(copied.connection.activeNodeId).toBe(copied.connection.nodes[1]!.id)
  expect(copied.connection.enabled).toBe(false);expect(copied.connection.credentialRef).toBeNull();expect(copied.discovered[0]!.id).toBe('native/id');expect(copied.templateRefs).toEqual(source.templateRefs)
  copied.defaults.fast={mode:'custom',value:true};expect(source.defaults.fast.mode).toBe('inherit');expect(copyChannel(source,'复用',true).connection.credentialRef).toBe('local-key-ref')
 })
 it('does not convert protected native or subscription authorization into a key copy',()=>{
  const source=channel();source.connection.authMode='oauth';expect(protectedChannel(source)).toBe(true);expect(()=>copyChannel(source,'不能复制',true)).toThrow()
  source.connection.authMode='key';source.connection.providerId='openai';expect(()=>copyChannel(source,'不能复制',false)).toThrow()
 })
})
