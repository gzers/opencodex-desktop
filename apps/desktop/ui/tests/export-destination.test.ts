import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {describe,expect,it} from 'vitest'

const view=readFileSync(resolve(__dirname,'../src/features/sync/ConfigFileTransferV2.vue'),'utf8')
const api=readFileSync(resolve(__dirname,'../src/features/sync/file-api-v2.ts'),'utf8')
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

  it('结果区显示后端确认的备份包路径与打开目录', () => {
    expect(view).not.toContain('v-if="exportedPath"');expect(view).toContain('displayPath(files.output.path)');expect(view).toContain('打开备份包目录');expect(api).toContain("'open_config_file_export_directory_v2',{operationId}")
    // Actual result reading/directory actions are exercised in sync-result.test.ts.
  })

  it('导出仅传默认或自定义目录意图，由后端选择并冻结实际路径', () => {
    expect(api).toContain("destination:ExportDestinationV2='default'");expect(api).toContain("'save_config_file_export_v2',{operationId,destination}")
    expect(view).toContain('v-model="files.destination"');expect(api).not.toContain('targetDir');expect(api).not.toContain('path:string)=>invoke')
  })

  it('能力清单允许系统原生目录选择', () => {
    expect(caps).toContain('dialog:allow-open')
  })
})
