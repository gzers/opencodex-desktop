import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {describe,expect,it} from 'vitest'

const view=readFileSync(resolve(__dirname,'../src/features/sync/ModelTransfer.vue'),'utf8')
const api=readFileSync(resolve(__dirname,'../src/features/sync/modelTransfer.ts'),'utf8')
const caps=readFileSync(resolve(__dirname,'../../tauri/capabilities/default.json'),'utf8')

describe('导出可选保存路径 (WP-32)', () => {
  it('确认对话框提供导出目录与选择目录', () => {
    expect(view).toContain('export-destination')
    expect(view).toContain('选择目录…')
    // WP-56：按“日期 / 单次备份包”组织，不再散落成单文件。
    expect(view).toContain('每次导出生成一个独立备份包')
    expect(view).toContain('日期 / 单次包')
  })

  it('结果区显示备份包路径与打开目录', () => {
    expect(view).toContain('备份包')
    expect(view).toContain('打开备份包目录')
  })

  it('导出调用带目标目录，默认留空', () => {
    expect(api).toContain('targetDir')
    expect(view).toContain("modelTransferApi.export(filePlan.value.id, exportTargetDir.value || undefined)")
  })

  it('能力清单允许系统保存对话框', () => {
    expect(caps).toContain('dialog:allow-save')
  })
})
