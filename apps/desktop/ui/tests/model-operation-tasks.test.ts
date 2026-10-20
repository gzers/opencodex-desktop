import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useOperationTasks } from '@/app/feedback/tasks'
beforeEach(() => setActivePinia(createPinia()))
describe('registered model task observations', () => {
  it('does not evict running, failed or pending-review observations at the display history budget, even when collapsed',()=>{
    const tasks=useOperationTasks(),pending=Array.from({length:26},()=>crypto.randomUUID()),failed=crypto.randomUUID(),running=crypto.randomUUID()
    for(const id of pending){tasks.apply({id,kind:'generation',phase:'running'});tasks.apply({id,kind:'generation',phase:'review'});tasks.collapse(id)}
    tasks.apply({id:failed,kind:'policy_save',phase:'running'});tasks.apply({id:failed,kind:'policy_save',phase:'failed'});tasks.apply({id:running,kind:'transfer',phase:'running'})
    const completed=crypto.randomUUID();tasks.apply({id:completed,kind:'research',phase:'running'});tasks.apply({id:completed,kind:'research',phase:'succeeded'})
    expect(tasks.tasks.map(t=>t.id)).toEqual([...pending,failed,running]);expect(tasks.tasks.find(t=>t.id===failed)?.phase).toBe('failed');expect(tasks.tasks.find(t=>t.id===running)?.phase).toBe('running')
    tasks.expandAll();expect(tasks.tasks.filter(t=>t.phase==='review')).toHaveLength(26);expect(tasks.tasks.every(t=>!t.collapsed)).toBe(true)
  })

  it('rejects unknown state and raw prompt/config payloads', () => {
    const tasks=useOperationTasks();const id=crypto.randomUUID()
    tasks.apply({id,kind:'unknown',phase:'running'});tasks.apply({id,kind:'generation',phase:'running',prompt:'private'})
    expect(tasks.tasks).toHaveLength(0)
  })
  it('reuses the same policy plan task when readback resolves a prior failure',()=>{
    const tasks=useOperationTasks();const id=crypto.randomUUID()
    tasks.apply({id,kind:'policy_save',phase:'running'});tasks.apply({id,kind:'policy_save',phase:'failed'});tasks.collapse(id)
    tasks.apply({id,kind:'policy_save',phase:'running'});expect(tasks.tasks).toHaveLength(1);expect(tasks.tasks[0]).toMatchObject({phase:'running',collapsed:false})
    tasks.apply({id,kind:'policy_save',phase:'review'});expect(tasks.tasks).toHaveLength(1);expect(tasks.tasks[0]!.phase).toBe('review')
  })
  it('rebinds receive to its durable receipt and adopts or retries the same transfer card',()=>{
    const tasks=useOperationTasks(),pending=crypto.randomUUID(),receipt=crypto.randomUUID()
    tasks.apply({id:pending,kind:'transfer',phase:'running'});tasks.apply({id:receipt,kind:'transfer',phase:'review',previousId:pending})
    expect(tasks.tasks).toHaveLength(1);expect(tasks.tasks[0]).toMatchObject({id:receipt,phase:'review'})
    tasks.apply({id:receipt,kind:'transfer',phase:'running'});tasks.apply({id:receipt,kind:'transfer',phase:'failed'});tasks.collapse(receipt)
    tasks.apply({id:receipt,kind:'transfer',phase:'running'});tasks.apply({id:receipt,kind:'transfer',phase:'succeeded'})
    expect(tasks.tasks).toHaveLength(1);expect(tasks.tasks[0]).toMatchObject({id:receipt,phase:'succeeded',collapsed:false})
  })
  it('does not let receipt rebind steal another operation or overwrite an existing terminal receipt',()=>{
    const tasks=useOperationTasks(),pending=crypto.randomUUID(),receipt=crypto.randomUUID(),other=crypto.randomUUID()
    tasks.apply({id:other,kind:'generation',phase:'running'});tasks.apply({id:receipt,kind:'transfer',phase:'review',previousId:other});expect(tasks.tasks).toHaveLength(1)
    tasks.apply({id:receipt,kind:'transfer',phase:'running'});tasks.apply({id:receipt,kind:'transfer',phase:'succeeded'})
    tasks.apply({id:pending,kind:'transfer',phase:'running'});tasks.apply({id:receipt,kind:'transfer',phase:'review',previousId:pending})
    expect(tasks.tasks).toHaveLength(2);expect(tasks.tasks.find(t=>t.id===receipt)!.phase).toBe('succeeded');expect(tasks.tasks.find(t=>t.id===other)!.phase).toBe('running')
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
