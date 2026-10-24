import {describe,expect,it} from 'vitest'
import {inlineChanges,isModelActive,setModelActive} from '@/features/models/saveReview'
import {emptyRules,type ChannelPolicy,type ModelPolicy} from '@/features/models/api'

function channel(id:string, models:string[], mode:'whitelist'|'all'|'none'='whitelist'):ChannelPolicy{
  return {id,label:'渠道 '+id,templateRefs:[],modelRefs:{},connection:{providerId:id,protocol:'openai-chat',nodes:[{id:'n',name:'主节点',url:'https://example.test'}],activeNodeId:'n',credentialRef:'SECRET-ref',enabled:true,allowPrivateNetwork:false},selection:mode==='whitelist'?{mode:'whitelist',models}:{mode},modelOrder:[...models],discovered:models.map(model=>({id:model,label:model,rules:emptyRules()})),defaults:emptyRules(),overrides:{},familyOverrides:{}}
}
const policy=():ModelPolicy=>({schemaVersion:1,templateRefs:[],channels:[channel('a',['m1','m2'])]})

describe('保存预览的启停口径',()=>{
  it('停用一个型号得到明确白名单，不会把整个渠道变成全部隐藏',()=>{
    const draft=policy().channels[0]!
    setModelActive(draft,['m1'],false)
    expect(draft.selection).toEqual({mode:'whitelist',models:['m2']})
    expect(isModelActive(draft,'m1')).toBe(false);expect(isModelActive(draft,'m2')).toBe(true)
  })

  it('全部模型范围下停用一个型号，保留其余已知型号',()=>{
    const draft=channel('a',['m1','m2','m3'],'all')
    setModelActive(draft,['m2'],false)
    expect(draft.selection).toEqual({mode:'whitelist',models:['m1','m3']})
  })

  it('停用最后一个型号与恢复启停都收敛到稳定状态',()=>{
    const draft=channel('a',['m1'])
    setModelActive(draft,['m1'],false);expect(draft.selection).toEqual({mode:'none'})
    setModelActive(draft,['m1'],true);expect(draft.selection).toEqual({mode:'whitelist',models:['m1']})
  })
})

describe('不再单列启停差异，只保留标签表达不了的变化',()=>{
  it('启停与顺序变化不进入字段差异',()=>{
    const saved=policy(),draft=structuredClone(saved)
    draft.channels[0]!.selection={mode:'whitelist',models:['m1']}
    draft.channels[0]!.modelOrder=['m2','m1']
    expect(inlineChanges(saved,draft)).toEqual([])
  })

  it('名称、协议、节点与认证变化仍可查，且不带本机认证引用',()=>{
    const saved=policy(),draft=structuredClone(saved)
    draft.channels[0]!.label='改过的渠道'
    draft.channels[0]!.connection.protocol='openai-responses'
    draft.channels[0]!.connection.nodes[0]!.url='https://moved.example.test'
    draft.channels[0]!.connection.credentialRef='SECRET-other'
    const changes=inlineChanges(saved,draft)
    const labels=changes.map(change=>change.label)
    // 组标题取草稿名：改名后这一组仍用同一个名称归类，条目分别列出各字段。
    expect(labels).toEqual(expect.arrayContaining(['改过的渠道 · 名称','改过的渠道 · 协议','改过的渠道 · 服务节点','改过的渠道 · 认证']))
    expect(JSON.stringify(changes)).not.toContain('SECRET')
  })

  it('渠道新增与删除只表达存在性，不展开其余字段',()=>{
    const saved=policy(),draft=structuredClone(saved)
    draft.channels.push(channel('b',['x']))
    const changes=inlineChanges(saved,draft)
    expect(changes.some(change=>change.label.startsWith('渠道 b · 渠道')||change.label.includes('渠道 b'))).toBe(true)
  })
})

