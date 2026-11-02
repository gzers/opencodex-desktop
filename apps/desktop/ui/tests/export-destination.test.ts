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
    expect(view).toContain('复制整个备份包即可带到另一台设备')
    expect(view).toContain('同日多次导出各自建包')
  })

  it('does not retain the retired human-review contract',()=>{
    // Actual preview/button behavior is covered by sync-prototype-020.test.ts.
    expect(view).not.toContain('v-model="assetsReviewed"')
    expect(view).not.toContain('已复核所选资源')
  })

  it('确认页展示生成位置目录树（与原型一致，WP-58）', () => {
    expect(view).toContain('export-tree')
    expect(view).toContain('生成位置')
    expect(view).toContain('manifest / 校验信息')
    expect(view).toContain('同日多次导出各自建包')
  })

  it('结果区显示备份包路径与打开目录', () => {
    expect(view).not.toContain('v-if="exportedPath"')
    // Actual result reading/directory actions are exercised in sync-result.test.ts.
  })

  it('导出调用带目标目录，默认留空', () => {
    expect(api).toContain('targetDir')
    expect(view).toContain("modelTransferApi.export(id, exportTargetDir.value || undefined)")
  })

  it('能力清单允许系统保存对话框', () => {
    expect(caps).toContain('dialog:allow-save')
  })
})
