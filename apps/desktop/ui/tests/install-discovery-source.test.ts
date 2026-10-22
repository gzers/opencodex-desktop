import {readFileSync} from 'node:fs'
import {resolve} from 'node:path'
import {describe,expect,it} from 'vitest'

const source=readFileSync(resolve(__dirname,'../src/routes/SettingsRoute.vue'),'utf8')

describe('安装发现不再把托管安装报成未发现', () => {
  it('可执行文件与安装形态同时参考运行来源', () => {
    // OpenCodex 可以由托管安装 / 指定路径 / 自动发现提供，npm 全局安装只是其中一种。
    expect(source).toContain("source.kind !== 'unresolved'")
    expect(source).toContain('report.ocx.found || fromSource')
    expect(source).toContain('运行来源 · ${sourceLabel}')
  })

  it('版本优先取运行来源解析出的版本', () => {
    expect(source).toContain('(fromSource ? source!.version : report.ocx.version)')
  })
})
