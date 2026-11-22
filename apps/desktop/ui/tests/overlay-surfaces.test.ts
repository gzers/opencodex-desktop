import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { useOverlaySurfaces } from '@/app/surfaces'
import { useNotificationsStore } from '@/features/notifications/store'
import { useModalStore } from '@/app/modal/store'

// IMP-04 §13.5：多个表面重叠时，只有最后一个相关表面关闭才恢复子 WebView。
describe('overlay surfaces coordinator', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('任一必需表面可见即需避让；全部关闭才恢复', () => {
    const { requiredOverlays } = useOverlaySurfaces()
    expect(requiredOverlays.value).toBe(false)

    useNotificationsStore().panelOpen = true
    expect(requiredOverlays.value).toBe(true)

    useModalStore().current = { kind: 'confirm' } as never
    useNotificationsStore().panelOpen = false
    expect(requiredOverlays.value).toBe(true) // 另一个表面仍开着

    useModalStore().current = null
    expect(requiredOverlays.value).toBe(false)
  })
})
