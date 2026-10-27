import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { MAX_VISIBLE_TASKS, TERMINAL_TTL_MS, useOperationTasks } from '@/app/feedback/tasks'
beforeEach(() => { setActivePinia(createPinia()); vi.useRealTimers() })
afterEach(() => { useOperationTasks().dispose(); vi.useRealTimers() })
describe('registered model task observations', () => {
  it('keeps failed observations beyond the display budget but still evicts them before running ones',()=>{
    const tasks=useOperationTasks(),failed=crypto.randomUUID(),running=crypto.randomUUID()
    tasks.apply({id:failed,kind:'research',phase:'running'});tasks.apply({id:failed,kind:'research',phase:'failed'})
    tasks.apply({id:running,kind:'transfer',phase:'running'})
    // 30 张已完成的卡把历史上限顶满；failed 与 running 不在淘汰候选里。
    const done=Array.from({length:30},()=>crypto.randomUUID())
    for(const id of done){tasks.apply({id,kind:'research',phase:'running'});tasks.apply({id,kind:'research',phase:'succeeded'})}
    expect(tasks.tasks.find(t=>t.id===failed)?.phase).toBe('failed')
    expect(tasks.tasks.find(t=>t.id===running)?.phase).toBe('running')
    expect(tasks.tasks.length).toBeLessThanOrEqual(24)
  })

  it('rejects unknown state and raw prompt/config payloads', () => {
    const tasks=useOperationTasks();const id=crypto.randomUUID()
    tasks.apply({id,kind:'unknown',phase:'running'});tasks.apply({id,kind:'generation',phase:'running',prompt:'private'})
    expect(tasks.tasks).toHaveLength(0)
  })
  it('ignores legacy save progress so the one-time saved feedback has no duplicate task',()=>{
    const tasks=useOperationTasks();const id=crypto.randomUUID()
    for (const phase of ['running', 'failed', 'review', 'succeeded']) tasks.apply({id,kind:'policy_save',phase})
    expect(tasks.tasks).toHaveLength(0)
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
    expect(tasks.tasks.find(t=>t.id===a)!.collapsed).toBe(true)
    tasks.apply({id:a,kind:'generation',phase:'running'});expect(tasks.tasks.find(t=>t.id===a)!.phase).toBe('review')
  })
})

describe('0.2.12 timing: repeat measurements fold into one card', () => {
  it('removes ordinary success at the TTL, including collapsed success, and rejects late events',()=>{
    vi.useFakeTimers()
    const tasks=useOperationTasks(),id=crypto.randomUUID()
    tasks.apply({id,kind:'research',phase:'running'})
    tasks.apply({id,kind:'research',phase:'succeeded'})
    tasks.collapse(id)
    vi.advanceTimersByTime(TERMINAL_TTL_MS - 1)
    expect(tasks.tasks).toHaveLength(1)
    vi.advanceTimersByTime(1)
    expect(tasks.tasks).toHaveLength(0)
    tasks.apply({id,kind:'research',phase:'succeeded'})
    expect(tasks.tasks).toHaveLength(0)
  })
  it('folds a repeated node measurement into the same card and counts attempts',()=>{
    const tasks=useOperationTasks()
    for(let i=0;i<10;i++){
      const id=crypto.randomUUID()
      tasks.apply({id,kind:'node_measurement',phase:'running'})
      tasks.apply({id,kind:'node_measurement',phase:'failed'})
    }
    // 十次检测只留一张（同 kind 合并），并把次数累加出来。
    expect(tasks.tasks).toHaveLength(1)
    expect(tasks.tasks[0]).toMatchObject({kind:'node_measurement',phase:'failed',count:10})
  })

  it('does not fold a still-running measurement into the finished one',()=>{
    const tasks=useOperationTasks()
    const first=crypto.randomUUID(),second=crypto.randomUUID()
    tasks.apply({id:first,kind:'node_measurement',phase:'running'})
    tasks.apply({id:second,kind:'node_measurement',phase:'running'})
    // 旧的还在跑时另起一张，避免旧 run 的晚回覆盖新 run。
    expect(tasks.tasks.map(t=>t.id)).toEqual([first,second])
  })

  it('ignores late events from a retired (folded) run so a collapsed card never pops back',()=>{
    const tasks=useOperationTasks()
    const first=crypto.randomUUID(),second=crypto.randomUUID()
    tasks.apply({id:first,kind:'node_measurement',phase:'running'})
    tasks.apply({id:first,kind:'node_measurement',phase:'failed'})
    tasks.apply({id:second,kind:'node_measurement',phase:'running'})
    tasks.apply({id:second,kind:'node_measurement',phase:'failed'})
    tasks.collapse(tasks.tasks[0]!.id)
    // 旧 run 的晚回不得复活任何卡。
    tasks.apply({id:first,kind:'node_measurement',phase:'running'})
    expect(tasks.tasks).toHaveLength(1)
    expect(tasks.tasks[0]!.collapsed).toBe(true)
  })

  it('collapses a terminal card after the TTL without deleting it',()=>{
    vi.useFakeTimers()
    const tasks=useOperationTasks(),id=crypto.randomUUID()
    tasks.apply({id,kind:'node_measurement',phase:'running'})
    tasks.apply({id,kind:'node_measurement',phase:'failed'})
    expect(tasks.tasks[0]!.collapsed).toBe(false)
    vi.advanceTimersByTime(TERMINAL_TTL_MS-1)
    expect(tasks.tasks[0]!.collapsed).toBe(false)
    vi.advanceTimersByTime(1)
    expect(tasks.tasks).toHaveLength(1)
    expect(tasks.tasks[0]!.collapsed).toBe(true)
    tasks.dispose()
  })

  it('never applies the TTL to a running card',()=>{
    vi.useFakeTimers()
    const tasks=useOperationTasks(),id=crypto.randomUUID()
    tasks.apply({id,kind:'generation',phase:'running'})
    vi.advanceTimersByTime(TERMINAL_TTL_MS*10)
    expect(tasks.tasks[0]!.collapsed).toBe(false)
    tasks.dispose()
  })

  it('caps visible cards and collapses the oldest terminal one first',()=>{
    const tasks=useOperationTasks()
    const ids=Array.from({length:MAX_VISIBLE_TASKS+2},()=>crypto.randomUUID())
    for(const [index,id] of ids.entries()){
      tasks.apply({id,kind:'research',phase:'running'})
      if(index<MAX_VISIBLE_TASKS-1)tasks.apply({id,kind:'research',phase:'failed'})
    }
    expect(tasks.tasks.filter(t=>!t.collapsed).length).toBeLessThanOrEqual(MAX_VISIBLE_TASKS)
  })
})

