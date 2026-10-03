import { describe, expect, it } from 'vitest'
import {
  installPhaseLabel,
  offlineSteps,
  phaseShowsCommandLine,
  proxyMaskLabel,
  sourceKindLabel,
  sourceLandingLabel,
} from '@/features/runtime/presentation'

describe('runtime presentation', () => {
  it('labels every frozen source kind without inventing values', () => {
    expect(sourceKindLabel('explicit')).toBe('用户指定')
    expect(sourceKindLabel('managed')).toBe('托管安装')
    expect(sourceKindLabel('discovered')).toBe('自动发现')
    expect(sourceKindLabel('unresolved')).toBe('未解析')
  })

  it('marks the landing side of the resolved path', () => {
    expect(sourceLandingLabel(true)).toBe('数据根内')
    expect(sourceLandingLabel(false)).toBe('数据根外')
  })

  it('labels every install phase', () => {
    expect(installPhaseLabel('preparing')).toBe('正在校验安装位置')
    expect(installPhaseLabel('downloading')).toBe('正在从 registry 取包')
    expect(installPhaseLabel('installing')).toBe('正在安装进私有前缀')
    expect(installPhaseLabel('validating')).toBe('正在校验离线包')
    expect(installPhaseLabel('extracting')).toBe('正在展开到私有前缀')
    expect(installPhaseLabel('verifying')).toBe('正在校验入口与版本')
    expect(installPhaseLabel('activating')).toBe('正在切换运行来源')
    expect(installPhaseLabel('done')).toBe('安装完成')
    expect(installPhaseLabel('failed')).toBe('安装失败')
  })

  it('only shows npm command lines for the networked phases', () => {
    expect(phaseShowsCommandLine('downloading')).toBe(true)
    expect(phaseShowsCommandLine('installing')).toBe(true)
    // 离线是纯包体展开：不套用联网的 npm 日志，避免出现假命令行。
    for (const phase of ['validating', 'extracting', 'verifying', 'activating', 'done', 'preparing'] as const) {
      expect(phaseShowsCommandLine(phase)).toBe(false)
    }
  })

  it('keeps offline substeps at three items and never leaves them all pending mid-run', () => {
    expect(offlineSteps('validating').length).toBe(3)
    expect(offlineSteps('extracting').length).toBe(3)
    const mid = offlineSteps('extracting')
    expect(mid.some(step => step.state !== 'pending')).toBe(true)
    const done = offlineSteps('done')
    expect(done.every(step => step.state === 'done')).toBe(true)
    // 失败不回填任何「已完成」。
    const failed = offlineSteps('failed')
    expect(failed.every(step => step.state === 'pending')).toBe(true)
  })

  it('masks the proxy and never renders credentials', () => {
    expect(proxyMaskLabel('http', '127.0.0.1:7890')).toBe('http://***@127.0.0.1:7890')
    expect(proxyMaskLabel('socks5h', '127.0.0.1:1080')).toBe('socks5h://***@127.0.0.1:1080')
    expect(proxyMaskLabel('http', '   ')).toBe('')
  })
})
