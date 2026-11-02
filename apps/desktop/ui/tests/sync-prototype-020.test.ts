import {beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {defineComponent} from 'vue'
import {open} from '@tauri-apps/plugin-dialog'
import ObjectScopeTree from '@/features/sync/ObjectScopeTree.vue'
import ModelTransfer from '@/features/sync/ModelTransfer.vue'
import {modelTransferApi,transferChoiceApi,transferProtectionApi,importCredentialChoiceApi,type ModelPackage,type IncomingPlan} from '@/features/sync/modelTransfer'
import {useModelsStore} from '@/features/models/store'
vi.mock('@tauri-apps/plugin-dialog',()=>({open:vi.fn()}))
vi.mock('@/features/sync/modelTransfer',async original=>({...await original<typeof import('@/features/sync/modelTransfer')>(),transferChoiceApi:{load:vi.fn(),save:vi.fn()},transferProtectionApi:{load:vi.fn(),save:vi.fn()},importCredentialChoiceApi:{load:vi.fn(),save:vi.fn()},modelTransferApi:{prepareDraft:vi.fn(),prepare:vi.fn(),export:vi.fn(),receive:vi.fn(),accept:vi.fn(),select:vi.fn(),prepareUpload:vi.fn(),publish:vi.fn(),download:vi.fn(),latestIncoming:vi.fn(),recompare:vi.fn(),openReceived:vi.fn(),loadRemoteScope:vi.fn(),saveRemoteScope:vi.fn(),latestUpload:vi.fn(),checkSync:vi.fn(),accountCatalog:vi.fn(),channelCatalog:vi.fn(),extensionCatalog:vi.fn()}}))
const dialog=defineComponent({props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'})
const render=(remote=false)=>mount(ModelTransfer,{props:{remote},global:{stubs:{UiDialog:dialog,UiSelectField:true}}})
const pack=():ModelPackage=>({format:'OCXD_MODEL_CONFIG',formatVersion:1,schemaVersion:1,createdAt:'2026-11-02T00:00:00Z',digest:'fixture',payload:{templates:[],policy:null}})
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();localStorage.clear();vi.mocked(importCredentialChoiceApi.load).mockResolvedValue({keys:true,extensions:true});vi.mocked(importCredentialChoiceApi.save).mockResolvedValue(undefined);vi.mocked(transferProtectionApi.load).mockResolvedValue({enabled:false,hasPassword:false});vi.mocked(transferProtectionApi.save).mockResolvedValue({enabled:false,hasPassword:false});vi.mocked(transferChoiceApi.load).mockResolvedValue(null);vi.mocked(transferChoiceApi.save).mockImplementation(async(_remote,value)=>value);useModelsStore().snapshot={library:{schemaVersion:1,templates:[]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'policy',workspace:null,workspaceDigest:'workspace'};vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.extensionCatalog).mockResolvedValue({skills:['fixture-skill'],mcps:['fixture-mcp']});vi.mocked(modelTransferApi.loadRemoteScope).mockResolvedValue({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false,skillNames:['fixture-skill'],mcpNames:['fixture-mcp']});vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'export-plan',package:pack(),credentialCount:1});vi.mocked(modelTransferApi.export).mockResolvedValue({path:'/fixture/config.ocxdconf',digest:'fixture',channels:0,templates:0})})
describe('S20 adopted prototype regressions',()=>{
 it('read failure does not replace an unknown saved choice or allow export',async()=>{vi.mocked(transferChoiceApi.load).mockRejectedValue({message:'unreadable'});const w=render();await flushPromises();expect(w.text()).toContain('本机范围读取失败');expect(w.findAll('button').find(b=>b.text()==='导出配置')!.attributes('disabled')).toBeDefined();expect(transferChoiceApi.save).not.toHaveBeenCalled();w.unmount()})
 it('restores exact file scope and per-object credentials without applying defaults',async()=>{vi.mocked(transferChoiceApi.load).mockResolvedValue({selectedIds:[JSON.stringify(['skills','fixture-skill'])],credentialIds:['skill:fixture-skill'],versions:{},mcpSources:{},includeGlobal:false,webdavPasswords:false});const w=render();await flushPromises();await w.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises();expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.objectContaining({includeGlobal:false,channelIds:[]}),undefined,false,expect.objectContaining({preferences:false,extensions:false,skillNames:['fixture-skill'],mcpNames:[],credentialIds:['skill:fixture-skill'],extensionCredentials:true}));w.unmount()})
 it('selected resources permit preview without a human-review checkbox',async()=>{const w=render();await flushPromises();expect(w.text()).not.toContain('已复核所选资源');expect(w.findAll('button').find(b=>b.text()==='导出配置')!.attributes('disabled')).toBeUndefined();w.unmount()})
 it('toolbar contains search and tree selection instead of cancelled presets',async()=>{const w=render(true);await flushPromises();expect(w.find('input[aria-label="搜索配置对象"]').exists()).toBe(true);expect(w.text()).not.toContain('清空选择');expect(w.text()).not.toContain('后续新增对象');expect(w.text()).not.toContain('全局采用及渠道组顺序');w.unmount()})
 it('credential-bearing export defaults to plaintext and never adds a second confirmation',async()=>{const w=render();await flushPromises();await w.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises();expect(w.get<HTMLInputElement>('.export-encrypt input').element.checked).toBe(false);expect(w.find('input[aria-label="加密口令"]').exists()).toBe(false);await w.findAll('button').find(b=>b.text()==='生成配置包')!.trigger('click');await flushPromises();expect(modelTransferApi.export).toHaveBeenCalledExactlyOnceWith('export-plan',undefined);expect(w.text()).not.toContain('仍以明文导出');w.unmount()})
 it('keeps file selection and necessary decryption in the same public import window',async()=>{
  vi.mocked(open).mockResolvedValue('/fixture/protected.ocxdconf')
  vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required'})
  const incoming={id:'incoming',package:pack(),expectedPolicy:'policy',expectedLibrary:'library',expectedWorkspace:'workspace',mergedPolicy:null,notices:[],availableKeys:[],availableExtensionIds:[]} as IncomingPlan
  vi.mocked(modelTransferApi.receive).mockResolvedValueOnce(incoming)
  const w=render();await flushPromises();await w.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises()
  expect(w.find('.export-destination').exists()).toBe(false)
  await w.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises()
  expect(w.find('input[aria-label="导入解密口令"]').exists()).toBe(true)
  expect(w.findAll('section[title="导入配置"]')).toHaveLength(1)
  await w.get('input[aria-label="导入解密口令"]').setValue('fixture-password')
  await w.findAll('button').find(b=>b.text()==='读取配置')!.trigger('click');await flushPromises()
  expect(modelTransferApi.receive).toHaveBeenLastCalledWith('/fixture/protected.ocxdconf','fixture-password')
  expect(w.findAll('section[title="导入配置"]')).toHaveLength(1)
  expect(w.text()).not.toContain('比较所选范围')
  expect(modelTransferApi.accept).not.toHaveBeenCalled();w.unmount()
 })
 it('recompares a changed incoming scope automatically',async()=>{
  vi.mocked(open).mockResolvedValue('/fixture/config.ocxdconf')
  const incoming={id:'incoming',package:pack(),expectedPolicy:'policy',expectedLibrary:'library',expectedWorkspace:'workspace',mergedPolicy:null,notices:[],assets:{skills:[{name:'fixture-skill',digest:'fixture-skill-digest',files:1,bytes:5,warnings:[],protected:false}],mcps:[],conflicts:[]}} as IncomingPlan
  vi.mocked(modelTransferApi.receive).mockResolvedValue(incoming);vi.mocked(modelTransferApi.select).mockResolvedValue({...incoming,id:'selected'})
  const w=render();await flushPromises();await w.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await w.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises()
  w.findAllComponents(ObjectScopeTree).at(-1)!.vm.$emit('update:selectedIds',[]);await flushPromises()
  expect(modelTransferApi.select).toHaveBeenCalledWith('incoming',expect.objectContaining({skillNames:[]}))
  expect(w.findAll('section[title="导入配置"]')).toHaveLength(1);expect(modelTransferApi.accept).not.toHaveBeenCalled();w.unmount()
 })
 it('does not permit confirmation when a synchronization diff is unknown',async()=>{
  vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'local',hasRemote:true})
  vi.mocked(modelTransferApi.prepareUpload).mockResolvedValue({id:'upload',phase:'prepared',package:pack(),manager:{preferences:null,extensions:null,webdav:[]},credentialCount:0,protected:false,assets:{skills:[],mcps:[],conflicts:[]},sync:{source:'local',createdAt:'2026-11-02T00:00:00Z',changes:null,unchanged:0}})
  const w=render(true);await flushPromises();await w.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
  expect(w.findAll('button').find(b=>b.text()==='确认更新')!.attributes('disabled')).toBeDefined()
  expect(modelTransferApi.publish).not.toHaveBeenCalled();w.unmount()
 })

})
