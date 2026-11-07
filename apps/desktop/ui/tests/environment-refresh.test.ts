import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import { useEnvironmentStore } from '@/features/environment/store'

const discoverEnvironment = vi.fn()

vi.mock('@/features/environment/api', () => ({
  discoverEnvironment: (...args: unknown[]) => discoverEnvironment(...args),
}))

describe('store.refreshEnvironment', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    discoverEnvironment.mockReset()
  })

  it('重新发现经正式 store 服务执行，成功后写入环境并清除 loading', async () => {
    const report = { ocx: { found: true } } as never
    discoverEnvironment.mockResolvedValue(report)
    const app = useAppStore()

    await app.refreshEnvironment()

    expect(discoverEnvironment).toHaveBeenCalledTimes(1)
    expect(app.environment).toEqual(report)
    expect(app.environmentLoading).toBe(false)
  })

  it('最新发现失败使旧报告失效，不再保留旧系统Node缺失结论', async () => {
    discoverEnvironment.mockRejectedValue(new Error('boom'))
    const app = useAppStore()
    app.setEnvironment({ gate: 'missing_node' } as never)

    await app.refreshEnvironment()

    expect(app.environmentLoading).toBe(false)
    expect(app.environment).toBeNull()
    expect(useEnvironmentStore().error).toBe('集成运行程序检查未就绪，可稍后重新检查或查看安装状态。')
  })

  it('newer discovery wins over an initial slow MissingNode response', async () => {
    let finish!:(value:unknown)=>void
    discoverEnvironment.mockReturnValueOnce(new Promise(resolve=>{finish=resolve})).mockResolvedValueOnce({gate:'ready'})
    const store=useEnvironmentStore(),old=store.refresh()
    await store.refresh();finish({gate:'missing_node'});await old
    expect(store.report).toEqual({gate:'ready'});expect(store.loading).toBe(false);expect(store.error).toBe('')
  })

  it('a stale rejection cannot clear a newer report or its loading state', async () => {
    let fail!:(cause:Error)=>void,finish!:(value:unknown)=>void
    discoverEnvironment.mockReturnValueOnce(new Promise((_,reject)=>{fail=reject})).mockReturnValueOnce(new Promise(resolve=>{finish=resolve}))
    const store=useEnvironmentStore(),old=store.refresh(),current=store.refresh()
    fail(new Error('old'));expect(await old).toBe(true);expect(store.loading).toBe(true);expect(store.error).toBe('')
    finish({gate:'ready'});await current;expect(store.report).toEqual({gate:'ready'})
  })

  it('an explicit report invalidates an older in-flight discovery', async () => {
    let finish!:(value:unknown)=>void
    discoverEnvironment.mockReturnValueOnce(new Promise(resolve=>{finish=resolve}))
    const store=useEnvironmentStore(),old=store.refresh();store.set({gate:'ready'} as never)
    finish({gate:'missing_node'});await old;expect(store.report).toEqual({gate:'ready'});expect(store.loading).toBe(false)
  })
})
