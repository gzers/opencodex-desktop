import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useOperationTasks } from '@/app/feedback/tasks'
beforeEach(() => setActivePinia(createPinia()))
describe('registered model task observations', () => {
  it('rejects unknown state and raw prompt/config payloads', () => {
    const tasks=useOperationTasks();const id=crypto.randomUUID()
    tasks.apply({id,kind:'unknown',phase:'running'});tasks.apply({id,kind:'generation',phase:'running',prompt:'private'})
    expect(tasks.tasks).toHaveLength(0)
  })
  it('keeps concurrent tasks separate and collapse does not cancel or lose terminal facts', () => {
    const tasks=useOperationTasks();const a=crypto.randomUUID();const b=crypto.randomUUID()
    tasks.apply({id:a,kind:'generation',phase:'running'});tasks.apply({id:b,kind:'external_submit',phase:'running'})
    tasks.collapse(a);tasks.apply({id:a,kind:'generation',phase:'review'})
    expect(tasks.tasks.find(t=>t.id===a)).toMatchObject({phase:'review',collapsed:true})
    expect(tasks.tasks.find(t=>t.id===b)!.phase).toBe('running')
    tasks.expandAll();expect(tasks.tasks.every(t=>!t.collapsed)).toBe(true)
    tasks.apply({id:a,kind:'generation',phase:'running'});expect(tasks.tasks.find(t=>t.id===a)!.phase).toBe('review')
  })
})
