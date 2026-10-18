import { defineStore } from 'pinia'
import { events, jobs } from '../../../../tauri/config/event-registry.json'
export type ModelOperationKind = 'generation' | 'research' | 'external_submit' | 'policy_save' | 'transfer'
export type ModelOperationPhase = 'running' | 'review' | 'succeeded' | 'failed' | 'stopped'
export interface OperationTask { id: string; kind: ModelOperationKind; phase: ModelOperationPhase; collapsed: boolean }
const owner = jobs.find(job => job.id === 'model_config')
const registered = owner?.availability === 'active' && events.some(event => event.id === 'model-operation-progress' && event.job === owner.id && event.fact === 'signal' && event.availability === 'active' && event.policy === null)
export const operationTitles: Record<ModelOperationKind,string> = { generation:'模版生成',research:'在线资料检索',external_submit:'外部模版提交',policy_save:'渠道配置保存',transfer:'模型配置传输' }
export const operationPhases: Record<ModelOperationPhase,string> = { running:'正在执行',review:'结果待比较，尚未保存',succeeded:'已完成',failed:'未完成，请查看对应操作',stopped:'已停止等待' }
export const useOperationTasks = defineStore('operation-tasks', {
  state: () => ({ tasks: [] as OperationTask[] }),
  actions: {
    apply(payload: unknown) {
      if (!registered || typeof payload !== 'object' || payload === null) return
      const value = payload as Record<string,unknown>
      if (Object.keys(value).some(key => !['id','kind','phase'].includes(key)) || typeof value.id !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value.id) || typeof value.kind !== 'string' || !Object.hasOwn(operationTitles,value.kind) || typeof value.phase !== 'string' || !Object.hasOwn(operationPhases,value.phase)) return
      const previous = this.tasks.find(t => t.id === value.id)
      if (previous) { if (previous.kind !== value.kind || previous.phase !== 'running') return; previous.phase = value.phase as ModelOperationPhase }
      else this.tasks.push({ id:value.id,kind:value.kind as ModelOperationKind,phase:value.phase as ModelOperationPhase,collapsed:false })
      while (this.tasks.length > 24) { const index=this.tasks.findIndex(t => t.phase !== 'running'); if(index < 0) break;this.tasks.splice(index,1) }
    },
    collapse(id: string) { const task=this.tasks.find(t => t.id===id);if(task)task.collapsed=true },
    expandAll() { for(const task of this.tasks)task.collapsed=false },
  },
})
