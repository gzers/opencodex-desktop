import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import ModelTransfer from '@/features/sync/ModelTransfer.vue'
import { modelTransferApi, scopeFor, type ModelPackage } from '@/features/sync/modelTransfer'
import { useModelsStore } from '@/features/models/store'
import { emptyContent, type ModelSnapshot } from '@/features/models/api'
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn().mockResolvedValue('/fixture/models.json') }))
vi.mock('@/features/sync/modelTransfer', async original => ({ ...await original<typeof import('@/features/sync/modelTransfer')>(), modelTransferApi: {prepare:vi.fn(),export:vi.fn(),receive:vi.fn(),accept:vi.fn(),prepareUpload:vi.fn(),publish:vi.fn(),download:vi.fn()} }))
function snapshot(): ModelSnapshot {
  return { library:{schemaVersion:1,templates:[{id:'family',label:'测试族',source:'user',defaultRevision:null,revisions:[{id:'revision-1',createdAt:'2026-10-18T00:00:00Z',savedLabel:'测试族',content:emptyContent(),digest:'digest',baseRevision:null,changes:[]}]}]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'policy',workspace:null,workspaceDigest:'workspace' }
}
const pack = (): ModelPackage => ({format:'OCXD_MODEL_CONFIG',formatVersion:1,schemaVersion:1,createdAt:'2026-10-18T00:00:00Z',digest:'frozen-digest',payload:{templates:snapshot().library.templates,policy:null}})
const dialog = defineComponent({ props:['open'], template:'<section v-if="open"><slot/><slot name="actions"/></section>' })
function render(remote = false) { return mount(ModelTransfer, {props:{remote},global:{stubs:{UiDialog:dialog,UiSelectField:true}}}) }
beforeEach(() => { setActivePinia(createPinia());vi.clearAllMocks();useModelsStore().snapshot=snapshot() })
describe('model transfer review', () => {
  it('exports the reviewed frozen plan instead of rebuilding from later editor content', async () => {
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'frozen-plan',package:pack()})
    vi.mocked(modelTransferApi.export).mockResolvedValue({path:'/fixture/export.json',digest:'frozen-digest',templates:1,channels:0})
    const wrapper=render()
    await wrapper.findAll('button').find(b=>b.text()==='导出模型配置')!.trigger('click')
    await wrapper.find('input[type=checkbox]').setValue(true)
    await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith({includePolicy:false,channelIds:[],includeGlobal:false,templateRefs:[{templateId:'family',revisionId:'revision-1'}]})
    expect(modelTransferApi.export).not.toHaveBeenCalled()
    useModelsStore().snapshot!.library.templates[0]!.label='later'
    await wrapper.findAll('button').find(b=>b.text()==='生成导出文件')!.trigger('click');await flushPromises()
    expect(modelTransferApi.export).toHaveBeenCalledExactlyOnceWith('frozen-plan')
    expect(wrapper.text()).toContain('/fixture/export.json');wrapper.unmount()
  })
  it('receives only a review and requires adoption while preserving the manual template buffer', async () => {
    const store=useModelsStore();store.edit();store.draft!.label='人工输入'
    vi.mocked(modelTransferApi.receive).mockResolvedValue({id:'incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]})
    vi.mocked(modelTransferApi.accept).mockResolvedValue({...snapshot(),libraryDigest:'received'})
    const wrapper=render()
    await wrapper.findAll('button').find(b=>b.text()==='导入模型配置')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(store.snapshot!.libraryDigest).toBe('library')
    await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('incoming');expect(store.snapshot!.libraryDigest).toBe('received')
    expect(store.draft!.label).toBe('人工输入');expect(store.dirty).toBe(true);wrapper.unmount()
  })
  it('retains the original operation ID after uncertain WebDAV publication', async () => {
    vi.mocked(modelTransferApi.prepareUpload).mockResolvedValue({id:'upload-once',phase:'prepared',package:pack()})
    vi.mocked(modelTransferApi.publish).mockRejectedValueOnce({message:'结果待核对'}).mockResolvedValue({id:'upload-once',phase:'published',package:pack()})
    const wrapper=render(true)
    await wrapper.findAll('button').find(b=>b.text()==='选择上传范围')!.trigger('click')
    await wrapper.find('input[type=checkbox]').setValue(true)
    await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='确认上传')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('结果待核对')
    await wrapper.findAll('button').find(b=>b.text()==='确认上传')!.trigger('click');await flushPromises()
    expect(vi.mocked(modelTransferApi.publish).mock.calls).toEqual([['upload-once'],['upload-once']]);wrapper.unmount()
  })
  it('keeps global-only selection from expanding to every channel', () => {
    expect(scopeFor(snapshot(),[],[],true,{})).toEqual({includePolicy:true,channelIds:[],templateRefs:[],includeGlobal:true})
  })
})
