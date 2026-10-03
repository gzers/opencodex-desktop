// 受管路径投影（IMP-04 §19.4 E：从总 store 拆出，切片九）。
// 管理数据目录分区与各客户端 Agent 目录的只读路径映射；打开动作只是转发平台命令。
import { defineStore } from 'pinia'
import {
  getManagedPathTargets,
  openExternalLink as openExternalLinkCommand,
  openLocalDocument as openLocalDocumentCommand,
  openManagedPath as openManagedPathCommand,
  type ManagedPathTargetsDto,
} from '@/platform/workspace'

export const usePathsStore = defineStore('paths', {
  state: () => ({
    managed: null as ManagedPathTargetsDto['managed'] | null,
    agent: null as ManagedPathTargetsDto['agent'] | null,
  }),
  actions: {
    async load() {
      try {
        const result = await getManagedPathTargets()
        this.managed = result.managed
        this.agent = result.agent
        return true
      } catch {
        // IPC 失败时保留上一次路径投影；不臆造本机目录。
        return false
      }
    },
    async open(path?: string) {
      if (!path) return false
      try {
        const result = await openManagedPathCommand(path)
        return result.opened
      } catch {
        return false
      }
    },
    async openExternal(url: string) {
      try {
        const result = await openExternalLinkCommand(url)
        return result.opened
      } catch {
        return false
      }
    },
    async openDocument(documentId: 'license' | 'third-party') {
      try {
        const result = await openLocalDocumentCommand(documentId)
        return result.opened
      } catch {
        return false
      }
    },
  },
})
