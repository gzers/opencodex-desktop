import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { App } from 'vue'
import { describeFault, installErrorHandlers, readableFaultInfo, sanitizeFaultText } from '@/app/errors'
import { useAppStore } from '@/stores/app'

// IMP-04 §14.3：运行期异常兜底必须有责任位置与恢复方式，且捕获不能只 console.log、不能吞异常变绿。
describe('runtime fault fallback', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('sanitizes credentials and long tokens in fault text', () => {
    expect(sanitizeFaultText('failed https://user:pass@dav.example/x')).toContain('https://***@dav.example/x')
    expect(sanitizeFaultText('token=' + 'a'.repeat(40))).toContain('***')
    expect(sanitizeFaultText('x'.repeat(500)).length).toBeLessThanOrEqual(300)
  })

  it('describes Error and non-Error values without throwing', () => {
    expect(describeFault('界面异常', new TypeError('boom'))).toContain('TypeError: boom')
    expect(describeFault('异步', { code: 13 })).toContain('{"code":13}')
    const cyclic: Record<string, unknown> = {}
    cyclic.self = cyclic
    expect(() => describeFault('环', cyclic)).not.toThrow()
  })

  // 生产构建里 Vue 的第三参数是错误文档 URL；拼进用户可见文案会泄漏英文技术串（UI规范 §10）。
  it('keeps user-facing fault text free of English technical strings', () => {
    expect(readableFaultInfo('https://vuejs.org/error-reference/#runtime-5')).toBeUndefined()
    expect(readableFaultInfo('  https://example.com/x  ')).toBeUndefined()
    // 开发构建的 Vue 标识映射成中文（不直接把英文拼进用户可见文案）。
    expect(readableFaultInfo('render function')).toBe('渲染')
    expect(readableFaultInfo('native event handler')).toBe('事件处理')
    expect(readableFaultInfo('v-on handler')).toBe('事件处理')
    // 未知且含拉丁字母：一律丢弃。
    expect(readableFaultInfo('render')).toBeUndefined()
    expect(readableFaultInfo('some unknown thing')).toBeUndefined()
    // 非拉丁标识可原样保留（截断到 60）。
    expect(readableFaultInfo('渲染阶段')).toBe('渲染阶段')
    expect(readableFaultInfo('')).toBeUndefined()
    expect(readableFaultInfo(null)).toBeUndefined()
    expect(readableFaultInfo(undefined)).toBeUndefined()
    expect(readableFaultInfo('中'.repeat(80))!.length).toBe(60)
  })

  it('records the fault and exposes recovery state instead of swallowing it', () => {
    const app = useAppStore()
    app.reportUiFault('界面异常：TypeError: boom', 'render')
    expect(app.uiFault).toContain('boom')
    expect(app.uiFault).toContain('render')
    expect(app.recentEvents[0]?.message).toContain('界面异常：TypeError: boom')
    app.clearUiFault()
    expect(app.uiFault).toBe('')
  })

  it('routes Vue errors and unhandled rejections to the store (idempotent handlers)', () => {
    const store = useAppStore()
    const appInstance = { config: {} } as unknown as App
    installErrorHandlers(appInstance, store)

    appInstance.config.errorHandler!(new Error('render broke'), null, 'render function')
    expect(store.uiFault).toContain('render broke')
    expect(store.uiFault).toContain('渲染')

    // 生产构建的真实入参：第三参数是文档 URL —— 不得出现在用户可见文案里。
    store.clearUiFault()
    appInstance.config.errorHandler!(new Error('prod broke'), null, 'https://vuejs.org/error-reference/#runtime-5')
    expect(store.uiFault).toContain('prod broke')
    expect(store.uiFault).not.toContain('vuejs.org')
    expect(store.uiFault).not.toMatch(/[（(]\s*[)）]/)

    store.clearUiFault()
    window.dispatchEvent(
      Object.assign(new Event('unhandledrejection'), { reason: new Error('async broke') }) as PromiseRejectionEvent,
    )
    expect(store.uiFault).toContain('async broke')

    // 幂等：重复安装不叠加旧监听（旧处理器先移除）。
    const removeSpy = vi.spyOn(window, 'removeEventListener')
    installErrorHandlers(appInstance, store)
    expect(removeSpy).toHaveBeenCalledWith('unhandledrejection', expect.any(Function))
    removeSpy.mockRestore()
  })
})
