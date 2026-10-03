import { describe, expect, it } from 'vitest'
import {
  isRouteName,
  normalizeDiagnosticsTab,
  normalizeExtensionTab,
  parseHash,
  serializeHash,
} from '@/navigation'

// IMP-04 §13.2：导航为单一来源的纯解析，未知取值回到规定默认值，不依赖 store 或 window。
describe('navigation parse/serialize', () => {
  it('解析路由、设置分区与两个 Tab（同一 tab 参数按上下文归一到各自默认）', () => {
    expect(parseHash('#settings?section=sync')).toEqual({
      route: 'settings',
      section: 'sync',
      extensionTab: 'skills',
      diagnosticsTab: 'doctor',
    })
    expect(parseHash('#logs?tab=doctor').diagnosticsTab).toBe('doctor')
    expect(parseHash('#logs?tab=logs').diagnosticsTab).toBe('logs')
    expect(parseHash('#extensions?tab=mcp').extensionTab).toBe('mcp')
  })

  it('空哈希与未知取值回到默认：overview / general / skills / doctor', () => {
    expect(parseHash('')).toEqual({
      route: 'overview',
      section: 'general',
      extensionTab: 'skills',
      diagnosticsTab: 'doctor',
    })
    expect(parseHash('#nope?section=unknown&tab=zzz')).toEqual({
      route: 'overview',
      section: 'general',
      extensionTab: 'skills',
      diagnosticsTab: 'doctor',
    })
  })

  it('序列化保持参数顺序，且无参数时不带问号', () => {
    expect(serializeHash('settings', { section: 'sync' })).toBe('#settings?section=sync')
    expect(serializeHash('overview')).toBe('#overview')
  })

  it('归一函数对未知/空值给出确定默认', () => {
    expect(normalizeDiagnosticsTab('doctor')).toBe('doctor')
    expect(normalizeDiagnosticsTab('logs')).toBe('logs')
    expect(normalizeDiagnosticsTab(null)).toBe('doctor')
    expect(normalizeExtensionTab('mcp')).toBe('mcp')
    expect(normalizeExtensionTab('skills')).toBe('skills')
    expect(normalizeExtensionTab('other')).toBe('skills')
    expect(isRouteName('panel')).toBe(true)
    expect(isRouteName('nope')).toBe(false)
  })
})
