// 官方面板嵌入功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片十三）。
//
// 面板地址由运行状态快照推导（`loadUrl(snapshot)` 由根壳传入），因此本切片不反向依赖根 store。
import { defineStore } from 'pinia'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'

export const usePanelStore = defineStore('panel', {
  state: () => ({
    error: '',
    loading: false,
    // 本轮运行是否已成功加载过官方页面：原生子 WebView 在路由间被复用，
    // 回到面板路由不会再触发页面加载事件，需要靠它区分「复用」与「首次嵌入」。
    loaded: false,
    url: '',
    // 顶栏「刷新状态」在面板路由下的第二层动作：+1 表示请求重载内嵌官方面板。
    // 面板切片不反向依赖根 store，所以只发一个计数器，由 PanelRoute 监听后执行重载。
    reloadToken: 0,
  }),
  actions: {
    /** 依据最新状态快照推导官方面板地址；不满足运行条件时给出可读原因。 */
    loadUrl(snapshot: StatusSnapshot | null) {
      if (this.loading) return false
      this.loading = true
      this.error = ''
      if (!snapshot || snapshot.matrix.runtime !== 'running' || snapshot.port === null) {
        this.url = ''
        this.error = '官方面板需要 OpenCodex 处于运行且健康可用状态；请先启动服务。'
        this.loading = false
        return false
      }
      if (snapshot.facts.health === 'unhealthy') {
        this.url = ''
        this.error = `官方代理当前 health=${snapshot.facts.health}；暂不加载面板。`
        this.loading = false
        return false
      }
      this.url = `http://127.0.0.1:${snapshot.port}/web`
      this.loading = false
      return true
    },
    setLoaded(next: boolean) {
      this.loaded = next
    },
    requestReload() {
      this.reloadToken += 1
    },
  },
})
