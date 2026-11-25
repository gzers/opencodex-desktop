import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import IntegratedPreparation from '@/features/environment/components/IntegratedPreparation.vue'
import { useEnvironmentStore } from '@/features/environment/store'
import { usePreparationStore, type PreparationPhase } from '@/features/environment/preparation'

const invoke = vi.hoisted(() => vi.fn())
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
function snapshot(phase: PreparationPhase, revision = 1, fields = {}) {
  return { attemptId: 'first', revision, phase, failedPhase: null, failure: null, retryable: false, ...fields }
}
let view: VueWrapper | undefined
beforeEach(() => {
  setActivePinia(createPinia())
  vi.useFakeTimers()
  invoke.mockReset()
})
afterEach(() => {
  view?.unmount()
  view = undefined
  usePreparationStore().stop()
  vi.useRealTimers()
})

it('the display can leave and return without starting a read or stopping the shared observation', async () => {
  invoke.mockResolvedValueOnce(snapshot('extracting')).mockResolvedValueOnce(snapshot('assembling', 2))
  view = mount(IntegratedPreparation)
  await flushPromises()
  expect(invoke).not.toHaveBeenCalled()
  usePreparationStore().start()
  await flushPromises()
  expect(view.get('h2').text()).toBe('展开运行包与依赖')
  expect(view.findAll('li b').map(row => row.text())).toEqual(['已完成', '已完成', '进行中', '待执行', '待执行'])
  view.unmount()
  await vi.advanceTimersByTimeAsync(600)
  view = mount(IntegratedPreparation)
  expect(view.get('h2').text()).toBe('核对程序与固定依赖')
  expect(invoke.mock.calls).toEqual([['runtime_preparation_status'], ['runtime_preparation_status']])
})

it('explicit retry binds the observed failure and prevents concurrent retries or old progress', async () => {
  let finish!: (value: unknown) => void
  invoke.mockResolvedValueOnce(snapshot('failed', 3, { failedPhase: 'verifying', failure: 'integrity', retryable: true }))
    .mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    .mockResolvedValueOnce(snapshot('failed', 3, { failure: 'integrity', retryable: true }))
    .mockResolvedValueOnce(snapshot('supplied', 6, { attemptId: 'retry' }))
  const preparation = usePreparationStore()
  preparation.start()
  view = mount(IntegratedPreparation)
  await flushPromises()
  expect(view.text()).toContain('校验未通过')
  expect(view.findAll('li b').map(row => row.text())).toEqual(['已完成', '未完成', '待执行', '待执行', '待执行'])
  const retry = view.findAll('button').find(button => button.text() === '重新准备内置环境')!
  await retry.trigger('click')
  await preparation.retry()
  expect(invoke.mock.calls[1]).toEqual(['runtime_preparation_retry', { request: { attemptId: 'first' } }])
  expect(retry.attributes('disabled')).toBeDefined()
  finish(snapshot('extracting', 4, { attemptId: 'retry' }))
  await flushPromises()
  await vi.advanceTimersByTimeAsync(600)
  expect(preparation.snapshot?.phase).toBe('extracting')
  await vi.advanceTimersByTimeAsync(1800)
  expect(preparation.snapshot?.phase).toBe('supplied')
  expect(invoke).toHaveBeenCalledTimes(4)
})

it.each(['failed', 'removed', 'review'] as const)('stops polling on %s without starting a retry', async phase => {
  invoke.mockResolvedValue(snapshot(phase))
  usePreparationStore().start()
  view = mount(IntegratedPreparation)
  await flushPromises()
  await vi.advanceTimersByTimeAsync(3000)
  expect(view.findAll('button').map(b => b.text())).toEqual(['查看安装配置'])
  expect(invoke.mock.calls).toEqual([['runtime_preparation_status']])
})

it('a failed observation offers a fresh read, even after an earlier known phase', async () => {
  invoke.mockResolvedValueOnce(snapshot('verifying')).mockRejectedValueOnce(new Error('unavailable'))
    .mockResolvedValueOnce(snapshot('review', 4, { failedPhase: 'checking', failure: 'recovery_required' }))
  usePreparationStore().start()
  view = mount(IntegratedPreparation)
  await flushPromises()
  await vi.advanceTimersByTimeAsync(600)
  expect(view.get('h2').text()).toBe('内置程序准备状态未知')
  expect(view.find('ol').exists()).toBe(false)
  await view.get('button').trigger('click')
  await flushPromises()
  expect(view.get('h2').text()).toBe('内置程序需要核对')
  expect(view.text()).toContain('继续原操作')
  await vi.advanceTimersByTimeAsync(3000)
  expect(invoke).toHaveBeenCalledTimes(3)
})

it('shares a slow read, discards a late result after root teardown and schedules no further work', async () => {
  let finish!: (value: unknown) => void
  invoke.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
  const preparation = usePreparationStore()
  const refresh = vi.spyOn(useEnvironmentStore(), 'refresh')
  preparation.start()
  preparation.start()
  const pending = preparation.observe()
  await vi.advanceTimersByTimeAsync(2400)
  preparation.stop()
  finish(snapshot('supplied'))
  await pending
  await vi.advanceTimersByTimeAsync(2400)
  expect(preparation.snapshot).toBeNull()
  expect(invoke.mock.calls).toEqual([['runtime_preparation_status']])
  expect(refresh).not.toHaveBeenCalled()
})
