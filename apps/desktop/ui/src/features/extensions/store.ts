// 扩展功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片二）。
// 根 store 通过 getter/委托保留 `app.extensions*` 读取与动作调用；直接写入点改用本 store。
import { defineStore } from 'pinia'
import {
  getExtensionConfig,
  executeExtensionWrite as writeExtension,
  listExtensions,
  type ExtensionClientId,
  type ExtensionConfigDto,
  type ExtensionSyncMethod,
  type ExtensionWriteCommand,
} from './api'
import { projectExtensions } from './projection'
import { extensionFailureText } from './errors'

export const useExtensionsStore = defineStore('extensions', {
  state: () => ({
    // 扩展发现结果（Skills + MCP 投影）。
    discovered: null as ReturnType<typeof projectExtensions>,
    loading: false,
    error: false,
    // 扩展配置（客户端启用矩阵、源目录、分发方式）。
    config: null as ExtensionConfigDto | null,
    configLoading: false,
    configError: false,
    // 最近一次扩展写入失败的可读原因（供 toast 直接展示）。
    writeError: '',
    toggleBusy: false,
  }),
  actions: {
    async loadConfig() {
      if (this.configLoading) return
      this.configLoading = true
      try {
        this.config = await getExtensionConfig()
        this.configError = false
      } catch {
        this.configError = true
      } finally {
        this.configLoading = false
      }
    },
    async write(command: ExtensionWriteCommand) {
      if (this.toggleBusy) return null
      this.toggleBusy = true
      try {
        const result = await writeExtension(command)
        this.config = result.config
        this.configError = false
        this.writeError = ''
        await this.load()
        return result
      } catch (error) {
        this.configError = true
        // 保留真实原因：此前一律吞掉，界面只能给出「同名冲突或配置不可写」这类猜测。
        this.writeError = extensionFailureText(error)
        return null
      } finally {
        this.toggleBusy = false
      }
    },
    async toggleClient(client: ExtensionClientId, enabled: boolean) {
      const result = await this.write({ kind: 'toggle_client', client, enabled })
      return result !== null
    },
    /** FZ-23：设置源目录；`null` 回到默认目录。失败保留原值。 */
    async setSourceDir(path: string | null) {
      const result = await this.write({ kind: 'set_source_dir', path })
      return result !== null
    },
    /** FZ-24：设置分发方式。失败保留原值。 */
    async setSyncMethod(method: ExtensionSyncMethod) {
      const result = await this.write({ kind: 'set_sync_method', method })
      return result !== null
    },
    /** 按当前源目录 + 分发方式对齐已落地的客户端目标（不改配置）。 */
    async resyncSkills() {
      const result = await this.write({ kind: 'resync_skills' })
      return result !== null
    },
    async load() {
      if (this.loading) return
      this.loading = true
      try {
        const payload = await listExtensions()
        this.discovered = projectExtensions(payload)
        this.error = false
      } catch {
        // IPC 失败时保留当前扩展发现结果，不回退 mock。
        this.error = true
      } finally {
        this.loading = false
      }
    },
  },
})
