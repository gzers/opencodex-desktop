import { describe, expect, it } from 'vitest'
import { describeExtensionFailure, extensionFailureText } from '@/features/extensions/errors'

// 2026-09-24 真机：`remove_mcp` 把 TOML 写成 JSON 后，界面只报「同名冲突或配置不可写未覆盖」，
// 真因（解析失败）被吞掉。这里锁定「真实原因照抄 + 中文归一化」的行为。
describe('extensionFailureText', () => {
  it('把「配置不可用」翻成可执行的中文原因', () => {
    const failure = describeExtensionFailure({ code: 1, message: 'application is not configured yet' })
    expect(failure.reason).toBe('扩展配置不可用；数据目录无效或不可访问。')
    expect(extensionFailureText({ code: 1, message: 'application is not configured yet' })).toBe(
      '扩展配置不可用；数据目录无效或不可访问。（application is not configured yet）',
    )
  })

  it('解析失败（toast 真实抓到的报文）指向配置文件而非「同名冲突」', () => {
    const message =
      'file system operation failed: parse extension projection; projection is invalid or corrupted'
    const text = extensionFailureText({ code: 1, message })
    expect(text).toContain('目标配置或统一配置解析失败')
    expect(text).toContain(message)
    expect(text).not.toContain('目标冲突或文件不可写未覆盖')
  })

  it('区分目标未安装、锁超时、需确认、原子写失败', () => {
    const reason = (message: string) => describeExtensionFailure({ code: 1, message }).reason
    expect(
      reason(
        'file system operation failed: validate extension target; client target is not registered or not installed',
      ),
    ).toBe('目标客户端未安装或当前不可写。')
    expect(reason('target file lock timed out after 3000ms')).toBe('目标文件被占用或超时，请稍后重试。')
    expect(reason('atomic write failed: temp file')).toBe('写入未完成（原子写失败）。')
    expect(reason('another application instance is already running')).toBe('已有管理器实例在运行。')
  })

  it('未归类的失败保留原始报文，不退回旧猜测文案', () => {
    const text = extensionFailureText({ code: 99, message: 'brand new backend failure' })
    expect(text).toBe('未归类的失败；请把原始信息一并反馈。（brand new backend failure）')
  })

  it('拿不到 message 时只给中文原因，不虚构技术细节', () => {
    expect(extensionFailureText({ code: 1 })).toBe('未归类的失败；请把原始信息一并反馈。')
    expect(extensionFailureText(null)).toBe('未归类的失败；请把原始信息一并反馈。')
    expect(extensionFailureText('plain string error')).toBe('未归类的失败；请把原始信息一并反馈。')
  })
})
