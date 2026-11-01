import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import ModelTransfer from '@/features/sync/ModelTransfer.vue'
import { modelTransferApi, type FilePlan, type McpConflict, type ModelPackage } from '@/features/sync/modelTransfer'
import { useModelsStore } from '@/features/models/store'
import type { ModelSnapshot } from '@/features/models/api'
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))
vi.mock('@/features/sync/modelTransfer', async original => ({ ...await original<typeof import('@/features/sync/modelTransfer')>(), modelTransferApi: {prepareDraft:vi.fn(),prepare:vi.fn(),export:vi.fn(),receive:vi.fn(),accept:vi.fn(),select:vi.fn(),prepareUpload:vi.fn(),publish:vi.fn(),download:vi.fn(),latestIncoming:vi.fn(),recompare:vi.fn(),openReceived:vi.fn(),loadRemoteScope:vi.fn(),saveRemoteScope:vi.fn(),latestUpload:vi.fn(),accountCatalog:vi.fn(),channelCatalog:vi.fn(),extensionCatalog:vi.fn()} }))

const snapshot = (): ModelSnapshot => ({ library:{schemaVersion:1,templates:[]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'policy',workspace:null,workspaceDigest:'workspace' })
const pack = (): ModelPackage => ({format:'OCXD_MODEL_CONFIG',formatVersion:1,schemaVersion:1,createdAt:'2026-10-18T00:00:00Z',digest:'frozen-digest',payload:{templates:[],policy:null}})
const plan = (conflicts:McpConflict[]): FilePlan => ({id:'mcp-plan',package:pack(),credentialCount:0,assets:{skills:[],mcps:[],conflicts}})
const dialog = defineComponent({ props:['open'], template:'<section v-if="open"><slot/><slot name="actions"/></section>' })
const select = defineComponent({props:['modelValue','options','label','disabled'],emits:['update:modelValue'],template:'<label>{{label}}<select :aria-label="label" :value="modelValue" @change="$emit(\'update:modelValue\',$event.target.value)"><option v-for="item in options" :key="item.value" :value="item.value">{{item.label}}</option></select></label>'})
const render = () => mount(ModelTransfer, {props:{remote:false},global:{stubs:{UiDialog:dialog,UiSelectField:select}}})
const button = (wrapper:ReturnType<typeof render>,label:string) => wrapper.findAll('button').find(b=>b.text()===label)!
// 对象树分组默认折叠；按名称搜索后子项才可见。
async function checkbox(wrapper:ReturnType<typeof render>,label:string){await wrapper.get('input[aria-label="搜索配置对象"]').setValue(label);return wrapper.get('input[aria-label="选择 '+label+'"]')}
const conflict = {name:'fixture-mcp',clients:['codex','claude'],fields:['command','args']}

beforeEach(() => {
  setActivePinia(createPinia());vi.clearAllMocks()
  vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([])
  vi.mocked(modelTransferApi.extensionCatalog).mockResolvedValue({skills:[],mcps:['fixture-mcp','node_repl']})
  useModelsStore().snapshot=snapshot()
})

describe('S18-03 同名 MCP 选来源与 node_repl 默认不同步', () => {
  it('默认范围不勾选 node_repl，可手动勾选且标明由客户端应用维护', async () => {
    const wrapper=render();await flushPromises();await flushPromises()
    expect(((await checkbox(wrapper,'MCP · fixture-mcp')).element as HTMLInputElement).checked).toBe(true)
    const nodeRepl=(await checkbox(wrapper,'MCP · node_repl')).element as HTMLInputElement
    expect(nodeRepl.checked).toBe(false)
    expect(nodeRepl.disabled).toBe(false)
    expect(wrapper.text()).toContain('由客户端应用维护 · 含本机路径')
    wrapper.unmount()
  })

  it('导出确认按原型顺序列出冲突，选定来源后按同一范围重算并带上来源', async () => {
    vi.mocked(modelTransferApi.prepare).mockResolvedValueOnce(plan([conflict])).mockResolvedValue(plan([]))
    const wrapper=render();await flushPromises();await flushPromises()
    await wrapper.findAll('label').find(l=>l.text().startsWith('已复核所选资源'))!.get('input').setValue(true)
    await button(wrapper,'导出配置').trigger('click');await flushPromises()
    const text=wrapper.text()
    expect(text).toContain('同名定义在 Codex、Claude 中不同（差异字段：command、args），请选择来源')
    // 原型顺序：生成位置 → 同名 MCP 选来源 → 范围与凭据说明（.sx-info）。
    const follows=(a:string,b:string)=>Boolean(wrapper.get(a).element.compareDocumentPosition(wrapper.get(b).element)&Node.DOCUMENT_POSITION_FOLLOWING)
    expect(follows('.export-tree','.transfer-mcp-conflicts')).toBe(true)
    expect(follows('.transfer-mcp-conflicts','.export-info')).toBe(true)
    const picker=wrapper.get('select[aria-label="fixture-mcp 采用来源"]')
    expect(picker.findAll('option').map(o=>o.text())).toEqual(['暂不导出','Codex','Claude'])
    await picker.setValue('claude');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledTimes(2)
    expect(vi.mocked(modelTransferApi.prepare).mock.calls[1]![3]).toEqual(expect.objectContaining({mcpSources:{'fixture-mcp':'claude'}}))
    // 已选来源的冲突仍保留在列表中，便于改选。
    expect(wrapper.find('select[aria-label="fixture-mcp 采用来源"]').exists()).toBe(true)
    wrapper.unmount()
  })

  it('范围只剩未选来源的同名 MCP 时不能生成，提示先选来源或调整范围', async () => {
    vi.mocked(modelTransferApi.prepare).mockResolvedValue(plan([conflict]))
    const wrapper=render();await flushPromises();await flushPromises()
    await button(wrapper,'清空选择').trigger('click')
    await (await checkbox(wrapper,'MCP · fixture-mcp')).setValue(true)
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('')
    await wrapper.findAll('label').find(l=>l.text().startsWith('已复核所选资源'))!.get('input').setValue(true)
    await button(wrapper,'导出配置').trigger('click');await flushPromises()
    await button(wrapper,'生成配置包').trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('请为同名 MCP 选择来源，或调整导出范围。')
    expect(modelTransferApi.export).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})

