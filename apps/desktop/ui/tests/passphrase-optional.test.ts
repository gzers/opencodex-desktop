import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {describe,expect,it} from 'vitest'

const source = readFileSync(resolve(__dirname, '../src/features/sync/ModelTransfer.vue'), 'utf8')

describe('文件保护口令改为可选', () => {
  it('导出按钮不再因为未填口令而禁用', () => {
    expect(source).not.toContain('requiresPassword&&!exportPassword')
  })

  it('标签与占位符说明留空即明文写入', () => {
    expect(source).toContain('文件保护口令（可选）')
    expect(source).toContain('留空则凭据以明文写入文件')
    expect(source).not.toContain('含凭据，必需')
  })

  it('导入不再要求先填口令', () => {
    // 旧的禁用条件形如 :disabled="!importPassword"，现在必须消失。
    expect(source.includes(':disabled="!importPassword"')).toBe(false)
    expect(source).toContain('直接读取并比较')
  })

  it('勾选项不再声称必须独立口令', () => {
    expect(source).not.toContain('携带所选扩展凭据（独立口令保护）')
    expect(source).not.toContain('（须设置独立口令）')
  })
})
