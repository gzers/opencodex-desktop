import {describe,it,expect} from 'vitest'
import {policyChanges} from '@/features/models/policyChanges'
import {emptyRules,type ModelPolicy,type ChannelPolicy} from '@/features/models/api'
function channel(id:string):ChannelPolicy{return{id,label:'渠道 '+id,templateRefs:[],modelRefs:{},connection:{providerId:id,protocol:'openai-chat',nodes:[{id:'node',name:'主节点',url:'https://example.test'}],activeNodeId:'node',credentialRef:'SECRET-local-ref',enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['m']},modelOrder:['m','absent'],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}}
const policy=():ModelPolicy=>({schemaVersion:1,templateRefs:[],channels:[channel('one'),channel('two')]})
describe('complete read-only policy draft comparison',()=>{
 it('reports global, order, deleted and unfiltered channel edits while keeping protected local identities out of displayed data',()=>{
  const saved=policy(),draft=structuredClone(saved);draft.defaultChannelId='one';draft.templateRefs=[{templateId:'f',revisionId:'v'}];draft.channels.reverse();draft.channels[0]!.connection.credentialRef='SECRET-new-ref';draft.channels[0]!.connection.nodes[0]!.url='https://changed.example.test';draft.channels[0]!.familyOverrides.f={...emptyRules(),fast:{mode:'custom',value:true}};draft.channels[0]!.naming={m:{alias:'route-a'}}
  draft.channels.splice(1,1);draft.channels.push(channel('three'));draft.pendingRemovals=[{readId:'SECRET-read-ref',channelId:'old',providerId:'legacy',credentialRef:'SECRET-old'}]
  const before=JSON.stringify([saved,draft]),report=policyChanges(saved,draft),data=JSON.stringify(report)
  // “默认渠道”不是本产品的概念：即使草稿里残留该字段，也不进入差异清单。
  expect(report.global.map(c=>c.field)).toEqual(expect.arrayContaining(['全局族精确采用','渠道顺序','已读取官方渠道的待移除项']))
  expect(report.global.map(c=>c.field)).not.toContain('默认渠道')
  expect(report.channels.map(c=>[c.id,c.kind])).toEqual([['one','removed'],['two','changed'],['three','added']]);expect(report.channels[1]!.changes.map(c=>c.field)).toEqual(expect.arrayContaining(['连接与节点','族覆盖','别名与显示名','本机认证引用']))
  expect(data).not.toContain('SECRET');expect(JSON.stringify([saved,draft])).toBe(before)
 })
})
