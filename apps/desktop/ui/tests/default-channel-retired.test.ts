import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {describe,expect,it} from 'vitest'
import {policyChanges} from '@/features/models/policyChanges'
import {emptyRules,type ChannelPolicy,type ModelPolicy} from '@/features/models/api'

function channel(id:string):ChannelPolicy{
  return {id,label:'渠道 '+id,templateRefs:[],modelRefs:{},connection:{providerId:id,protocol:'openai-chat',nodes:[{id:'n',name:'节点',url:'https://example.test'}],activeNodeId:'n',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['m']},modelOrder:['m'],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}
}
const policy=(channels:ChannelPolicy[]):ModelPolicy=>({schemaVersion:1,templateRefs:[],channels})

describe('默认渠道已退役', () => {
  it('草稿里残留 defaultChannelId 也不进入差异清单', () => {
    const saved=policy([channel('a')])
    const draft={...structuredClone(saved),defaultChannelId:'a'}
    const fields=policyChanges(saved,draft).global.map(change=>change.field)
    expect(fields).not.toContain('默认渠道')
  })

  it('保存范围说明不再宣称包含默认渠道', () => {
    const source=readFileSync(resolve(__dirname,'../src/features/models/ChannelsView.vue'),'utf8')
    expect(source).not.toContain('默认渠道与渠道顺序')
    // 共享项比较只看全局模版采用与渠道顺序
    expect(source).toContain('[saved.templateRefs,saved.channels.map(c=>c.id)]')
  })
})
