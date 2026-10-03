import { describe, expect, it } from 'vitest'
import { applyInterfaceScale, clampScale } from '@/app/appearance/scale'

// 与冻结原型 applyInterfaceScale 的边界一致（原型/index.html）。
describe('interface scale normalization', () => {
  it('keeps in-range integers unchanged', () => {
    expect(clampScale(100)).toBe(100)
    expect(clampScale('150')).toBe(150)
  })

  it('rounds fractional values', () => {
    expect(clampScale(120.4)).toBe(120)
    expect(clampScale('120.6')).toBe(121)
  })

  it('clamps below 50 and above 200', () => {
    expect(clampScale(-30)).toBe(50)
    expect(clampScale(49)).toBe(50)
    expect(clampScale(999)).toBe(200)
    expect(clampScale('1e9')).toBe(200)
  })

  it('falls back to 100 for empty or unparsable input', () => {
    // 原型用 `Number(scale) || 100`，因此 0 与空值一样回退 100，而不是被夹到 50。
    expect(clampScale(0)).toBe(100)
    expect(clampScale('')).toBe(100)
    expect(clampScale('  ')).toBe(100)
    expect(clampScale('abc')).toBe(100)
    expect(clampScale(null)).toBe(100)
    expect(clampScale(undefined)).toBe(100)
    expect(clampScale(NaN)).toBe(100)
  })
})

// 回归：此前界面缩放只写入无人消费的 `--panel-zoom`，主界面看不到任何变化。
// 现在缩放的唯一落点是根元素上的 `--ui-zoom`，且写入的是归一化后的因子。
describe('interface scale application', () => {
  it('writes the normalized zoom factor to the root --ui-zoom', () => {
    const root = document.createElement('div')
    expect(applyInterfaceScale(150, root)).toBe(150)
    expect(root.style.getPropertyValue('--ui-zoom')).toBe('1.5')
    applyInterfaceScale(75, root)
    expect(root.style.getPropertyValue('--ui-zoom')).toBe('0.75')
    applyInterfaceScale(999, root)
    expect(root.style.getPropertyValue('--ui-zoom')).toBe('2')
    applyInterfaceScale('', root)
    expect(root.style.getPropertyValue('--ui-zoom')).toBe('1')
  })

  it('no longer writes the legacy unused variable', () => {
    const root = document.createElement('div')
    applyInterfaceScale(130, root)
    expect(root.style.getPropertyValue('--panel-zoom')).toBe('')
  })
})
