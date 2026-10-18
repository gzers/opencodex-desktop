import type { Proposal, TemplateDraft } from './api'
export interface ProposalField { path: string[]; label: string; before: unknown; current: unknown; after: unknown; conflict: boolean; locked: boolean }
function at(value: unknown, path: string[]): unknown { return path.reduce<unknown>((v,k) => Array.isArray(v) ? v.find(item => item && typeof item === 'object' && item.id === k) : typeof v === 'object' && v !== null ? (v as Record<string,unknown>)[k] : undefined, value) }
const rules = ['protocol','contextWindow','inputModalities','reasoningEfforts','defaultEffort','fast']
export function proposalFields(base: TemplateDraft, current: TemplateDraft, proposal: Proposal): ProposalField[] {
  const fields = [['label','名称'],['content.description','说明'],['content.matching','匹配规则'],...rules.map(key => ['content.rules.'+key,key]),['content.sources','资料来源']]
  const ids = new Set([...base.content.variants,...proposal.content.variants].map(v => v.id))
  for (const id of ids) {
    const before = base.content.variants.find(v => v.id === id)
    const now = current.content.variants.find(v => v.id === id)
    const after = proposal.content.variants.find(v => v.id === id)
    const name = after?.name || before?.name || id
    if (!before || !after || !now) fields.push(['content.variants.'+id, name+' · '+(!after ? '移除' : !before ? '新增' : '恢复')])
    else {
      for (const key of ['name','suffix']) fields.push(['content.variants.'+id+'.'+key,name+' · '+(key === 'name' ? '名称' : '后缀')])
      for (const key of rules) fields.push(['content.variants.'+id+'.rules.'+key,name+' · '+key])
    }
  }
  return fields.map(([key,label]) => {
    const path = key!.split('.'); const before = at(base,path); const now = at(current,path); const after = at(proposal,path)
    const lockPath = path.slice(1).join('.')
    const variant = path[1] === 'variants' ? [...base.content.variants,...current.content.variants].filter(v => v.id === path[2]) : []
    const locked = path[0] === 'content' && [...base.content.lockedFields, ...current.content.lockedFields].some(lock => lockPath === lock || lockPath.startsWith(lock+'.')) || variant.some(v => path.length === 3 ? v.lockedFields.length > 0 : path[3] === 'rules' && v.lockedFields.includes(path[4]!))
    return { path,label:label!,before,current:now,after,conflict:JSON.stringify(before)!==JSON.stringify(now),locked }
  }).filter(field => JSON.stringify(field.before)!==JSON.stringify(field.after))
}
export function acceptProposal(current: TemplateDraft, fields: ProposalField[], selected: string[]): TemplateDraft {
  const result: TemplateDraft = JSON.parse(JSON.stringify(current))
  for (const field of fields) {
    if (field.locked || !selected.includes(field.path.join('.'))) continue
    let target = result as unknown as Record<string,unknown>
    let valid = true
    for (const key of field.path.slice(0,-1)) {
      const next = Array.isArray(target) ? target.find(item => item.id === key) : target[key]
      if (!next || typeof next !== 'object') { valid = false; break }
      target = next as Record<string,unknown>
    }
    if (!valid) continue
    const key = field.path[field.path.length-1]!
    if (Array.isArray(target)) {
      const index = target.findIndex(item => item.id === key)
      if (field.after === undefined) { if (index >= 0) target.splice(index,1) }
      else {
        const next = JSON.parse(JSON.stringify(field.after))
        // Only human controls can add or remove locks.
        next.lockedFields = index >= 0 ? target[index].lockedFields : []
        if (index >= 0) target[index] = next; else target.push(next)
      }
    } else target[key] = field.after === undefined ? undefined : JSON.parse(JSON.stringify(field.after))
  }
  return result
}
