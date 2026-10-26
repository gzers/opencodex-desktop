import { defineStore } from 'pinia'
import { events, jobs } from '../../../../tauri/config/event-registry.json'
export type ModelOperationKind = 'generation' | 'research' | 'external_submit' | 'policy_save' | 'transfer' | 'node_measurement'
export type ModelOperationPhase = 'running' | 'review' | 'succeeded' | 'failed' | 'stopped'
export interface OperationTask { id: string; kind: ModelOperationKind; phase: ModelOperationPhase; collapsed: boolean; count?: number }

/** 终止态停留时长：到点自动收起。收起不是删除——铃铛与通知历史里仍在。 */
export const TERMINAL_TTL_MS = 8000
/** 同屏最多展开几张任务卡；超出的自动收起，避免把页面内容整片压住。 */
export const MAX_VISIBLE_TASKS = 3
/** 历史上限：只淘汰终止态，运行中与待核对永不淘汰。 */
const MAX_TASKS = 24

/**
 * 这些操作反复触发时复用同一张卡。
 *
 * 只复用**已终止**的那张：运行中的另起一张，避免旧 run 的晚回事件覆盖新 run 的结果。
 */
const MERGE_KINDS: ModelOperationKind[] = ['node_measurement']

const TERMINAL: ModelOperationPhase[] = ['succeeded', 'failed', 'stopped']
export function isTerminalPhase(phase: ModelOperationPhase): boolean { return TERMINAL.includes(phase) }

const owner = jobs.find(job => job.id === 'model_config')
const registered = owner?.availability === 'active' && events.some(event => event.id === 'model-operation-progress' && event.job === owner.id && event.fact === 'signal' && event.availability === 'active' && event.policy === null)
export const operationTitles: Record<ModelOperationKind,string> = { generation:'模版生成',research:'在线资料检索',external_submit:'外部模版提交',policy_save:'渠道配置保存',transfer:'模型配置传输',node_measurement:'节点连接检测' }
export const operationPhases: Record<ModelOperationPhase,string> = { running:'正在执行',review:'结果待比较，尚未保存',succeeded:'已完成',failed:'未完成，请查看对应操作',stopped:'已停止等待' }
function phaseDetail(task: OperationTask): string {
  if(task.phase==='review'&&task.kind==='generation')return '建议已生成，可在模版编辑中查看'
  if(task.phase==='review'&&task.kind==='policy_save')return '已写入；客户端结果请在核对入口查看'
  if(task.phase==='review'&&task.kind==='transfer')return '已接收，待比较采用'
  return operationPhases[task.phase]
}
export function operationPhaseText(task: OperationTask): string {
  const repeat=(task.count??1)>1?` · 第 ${task.count} 次`:''
  return phaseDetail(task)+repeat
}

/** 收起计时器按任务 id 持有；不进 Pinia state，避免不可序列化的值混进快照。 */
const collapseTimers = new Map<string, ReturnType<typeof setTimeout>>()
/** 已被合并/淘汰的旧 run 身份：它们的晚回事件必须丢弃，不能复活一张卡。 */
const retired = new Set<string>()
const RETIRED_LIMIT = 512
function retire(id: string) {
  retired.add(id)
  while (retired.size > RETIRED_LIMIT) { const first = retired.values().next().value; if (first === undefined) break; retired.delete(first) }
}
function clearCollapseTimer(id: string) {
  const timer = collapseTimers.get(id)
  if (timer !== undefined) { clearTimeout(timer); collapseTimers.delete(id) }
}

export const useOperationTasks = defineStore('operation-tasks', {
  state: () => ({ tasks: [] as OperationTask[] }),
  actions: {
    apply(payload: unknown) {
      if (!registered || typeof payload !== 'object' || payload === null) return
      const value = payload as Record<string,unknown>
      if (Object.keys(value).some(key => !['id','kind','phase','previousId'].includes(key)) || typeof value.id !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value.id) || typeof value.kind !== 'string' || !Object.hasOwn(operationTitles,value.kind) || typeof value.phase !== 'string' || !Object.hasOwn(operationPhases,value.phase)) return
      // 被合并或淘汰的旧 run 再晚回也不复活：否则一次重试就能把已收起的卡重新刷出来。
      if (retired.has(value.id)) return
      if(value.previousId!==undefined){
        if(value.kind!=='transfer'||value.phase!=='review'||typeof value.previousId!=='string'||!/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value.previousId)||value.previousId===value.id)return
        const pending=this.tasks.find(t=>t.id===value.previousId)
        const receipt=this.tasks.find(t=>t.id===value.id)
        if(pending?.kind!=='transfer'||pending.phase!=='running')return
        if(receipt){if(receipt.kind!=='transfer')return;clearCollapseTimer(pending.id);this.tasks.splice(this.tasks.indexOf(pending),1);return}
        clearCollapseTimer(pending.id);pending.id=value.id;pending.phase='review';return
      }
      const previous = this.tasks.find(t => t.id === value.id)
      if (previous) {
        if(previous.kind!==value.kind)return
        if((previous.kind==='policy_save'||previous.kind==='transfer')&&value.phase==='running'){previous.phase='running';previous.collapsed=false;clearCollapseTimer(previous.id);return}
        if(previous.phase!=='running')return
        previous.phase=value.phase as ModelOperationPhase
        if(isTerminalPhase(previous.phase))this.scheduleCollapse(previous)
        this.settle()
        return
      }
      const kind=value.kind as ModelOperationKind,phase=value.phase as ModelOperationPhase
      const reusable = MERGE_KINDS.includes(kind) ? [...this.tasks].reverse().find(t => t.kind===kind && isTerminalPhase(t.phase)) : undefined
      if (reusable) {
        clearCollapseTimer(reusable.id)
        retire(reusable.id)
        reusable.id=value.id
        reusable.phase=phase
        reusable.collapsed=false
        reusable.count=(reusable.count??1)+1
        for (const other of [...this.tasks]) {
          if (other!==reusable && other.kind===kind && isTerminalPhase(other.phase)) { clearCollapseTimer(other.id); retire(other.id); this.tasks.splice(this.tasks.indexOf(other),1) }
        }
        if(isTerminalPhase(phase))this.scheduleCollapse(reusable)
        this.settle()
        return
      }
      this.tasks.push({ id:value.id,kind,phase,collapsed:false,count:1 })
      const created=this.tasks[this.tasks.length-1]!
      if(isTerminalPhase(phase))this.scheduleCollapse(created)
      this.settle()
    },
    /** 终止态到点收起（不是删除）；运行中不设 TTL。 */
    scheduleCollapse(task: OperationTask) {
      if (typeof setTimeout !== 'function') return
      clearCollapseTimer(task.id)
      const timer = setTimeout(() => {
        collapseTimers.delete(task.id)
        const current = this.tasks.find(t => t.id === task.id)
        if (current && isTerminalPhase(current.phase)) current.collapsed = true
      }, TERMINAL_TTL_MS)
      collapseTimers.set(task.id, timer)
    },
    /** 两条约束：历史只淘汰终止态；同屏展开数不超过上限。 */
    settle() {
      while (this.tasks.length > MAX_TASKS) {
        // 淘汰优先级：先丢「已完成 / 已停止」，再丢「失败」。
        // 失败卡承载用户还要处理的信息，比成功卡更该多留一会儿；但终究可以丢，
        // 因为通知实体（铃铛）才是留痕的地方，任务卡只是观察窗口。
        const done = this.tasks.findIndex(t => t.phase === 'succeeded' || t.phase === 'stopped')
        const victimIndex = done >= 0 ? done : this.tasks.findIndex(t => t.phase === 'failed')
        if (victimIndex < 0) break
        const removed = this.tasks[victimIndex]!
        clearCollapseTimer(removed.id)
        retire(removed.id)
        this.tasks.splice(victimIndex, 1)
      }
      for (;;) {
        const visible = this.tasks.filter(t => !t.collapsed)
        if (visible.length <= MAX_VISIBLE_TASKS) break
        // 先收起最老的已终止卡；全都还在跑时收起最老的一张（运行不受影响，只是不再占屏）。
        const victim = visible.find(t => isTerminalPhase(t.phase)) ?? visible[0]!
        victim.collapsed = true
      }
    },
    collapse(id: string) { const task=this.tasks.find(t => t.id===id);if(task)task.collapsed=true },
    expandAll() { for(const task of this.tasks)task.collapsed=false },
    /** 供测试与退出时清理；运行期不需要调用。 */
    dispose() { for (const id of [...collapseTimers.keys()]) clearCollapseTimer(id) },
  },
})

