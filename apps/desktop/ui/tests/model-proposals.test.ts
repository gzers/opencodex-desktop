import { describe, expect, it } from 'vitest'
import { emptyContent, emptyRules, type TemplateDraft, type Proposal } from '@/features/models/api'
import { acceptProposal, proposalFields } from '@/features/models/proposals'
const draft = (): TemplateDraft => ({ templateId: null, baseRevision: null, label: '当前模版', content: emptyContent() })
const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v))
describe('three-way generation proposals', () => {
  it('compares optional advanced fields without a fake old-schema change and preserves explicit locks',()=>{
    const base=draft();delete base.content.rules.pinnedEffort;delete base.content.rules.effortMap;const same:Proposal=clone(base);same.content.rules.pinnedEffort={mode:'inherit'};same.content.rules.effortMap={mode:'inherit'};expect(proposalFields(base,base,same)).toEqual([]);
    const current=clone(base);current.content.lockedFields=['rules.pinnedEffort'];const generated:Proposal=clone(base);generated.content.rules.pinnedEffort={mode:'custom',value:'high'};generated.content.rules.effortMap={mode:'custom',value:{high:'__omit__'}};
    const fields=proposalFields(base,current,generated);expect(fields.find(f=>f.path.at(-1)==='pinnedEffort')?.locked).toBe(true);const result=acceptProposal(current,fields,fields.map(f=>f.path.join('.')));expect(result.content.rules.pinnedEffort).toBeUndefined();expect(result.content.rules.effortMap).toEqual({mode:'custom',value:{high:'__omit__'}})
  })
  it('marks concurrent human edits and preserves unselected fields', () => {
    const base = draft(); const current = clone(base); const generated: Proposal = clone(base)
    current.content.description = '人工刚修改'; generated.content.description = '助手描述'; generated.content.rules.fast = { mode:'custom',value:false }
    const fields = proposalFields(base,current,generated)
    expect(fields.find(f => f.path.join('.') === 'content.description')!.conflict).toBe(true)
    const selected = fields.filter(f => !f.conflict && !f.locked).map(f => f.path.join('.'))
    const result = acceptProposal(current,fields,selected)
    expect(result.content.description).toBe('人工刚修改');expect(result.content.rules.fast).toEqual({mode:'custom',value:false})
    expect(current.content.rules.fast).toEqual({mode:'inherit'})
  })
  it('cannot replace locked fields even if a caller checks their paths', () => {
    const base = draft(); base.content.lockedFields=['rules.fast'];base.content.rules.fast={mode:'custom',value:false}
    const generated: Proposal = clone(base);generated.content.rules.fast={mode:'custom',value:true}
    const fields=proposalFields(base,base,generated);expect(fields[0]!.locked).toBe(true)
    expect(acceptProposal(base,fields,['content.rules.fast']).content.rules.fast).toEqual({mode:'custom',value:false})
  })
  it('respects locks added after a generation began', () => {
    const base=draft();const current=clone(base);current.content.lockedFields=['description']
    const generated: Proposal=clone(base);generated.content.description='建议描述'
    expect(proposalFields(base,current,generated)[0]!.locked).toBe(true)
  })
  it('merges each stable variant field while preserving concurrent edits and unrelated locks', () => {
    const base=draft();base.content.variants=[{id:'a',name:'型号A',suffix:'-a',rules:emptyRules(),lockedFields:['fast']},{id:'b',name:'型号B',suffix:'-b',rules:emptyRules(),lockedFields:[]}]
    const current=clone(base);current.content.variants.reverse();current.content.variants[1]!.name='人工改名'
    const generated: Proposal=clone(base);generated.content.variants[0]!.name='助手改名';generated.content.variants[0]!.rules.fast={mode:'custom',value:true};generated.content.variants[0]!.rules.contextWindow={mode:'custom',value:131072};generated.content.variants[1]!.suffix='-new-b'
    const fields=proposalFields(base,current,generated)
    const selected=fields.filter(f=>!f.conflict&&!f.locked).map(f=>f.path.join('.'))
    const accepted=acceptProposal(current,fields,selected)
    expect(accepted.content.variants.map(v=>v.id)).toEqual(['b','a'])
    expect(accepted.content.variants[1]!.name).toBe('人工改名');expect(accepted.content.variants[1]!.rules.fast).toEqual({mode:'inherit'})
    expect(accepted.content.variants[1]!.rules.contextWindow).toEqual({mode:'custom',value:131072});expect(accepted.content.variants[0]!.suffix).toBe('-new-b')
  })
  it('requires an explicit choice for deletion after editing and protects a locked variant', () => {
    const base=draft();base.content.variants=[{id:'a',name:'型号A',suffix:'-a',rules:emptyRules(),lockedFields:[]},{id:'b',name:'型号B',suffix:'-b',rules:emptyRules(),lockedFields:['fast']}]
    const current=clone(base);current.content.variants[0]!.name='人工改名'
    const generated: Proposal=clone(base);generated.content.variants=[]
    const fields=proposalFields(base,current,generated)
    expect(fields.find(f=>f.path[2]==='a')!.conflict).toBe(true);expect(fields.find(f=>f.path[2]==='b')!.locked).toBe(true)
    const result=acceptProposal(current,fields,fields.map(f=>f.path.join('.')))
    expect(result.content.variants.map(v=>v.id)).toEqual(['b'])
  })
})
