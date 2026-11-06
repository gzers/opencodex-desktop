import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import ModelTransfer from '@/features/sync/ModelTransfer.vue'
import { modelTransferApi,cloudCatalogApi,transferChoiceApi,transferProtectionApi,importCredentialChoiceApi, scopeFor, remoteScope, type ModelPackage } from '@/features/sync/modelTransfer'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ObjectScopeTree from '@/features/sync/ObjectScopeTree.vue'
import { useModelsStore } from '@/features/models/store'
import { emptyContent, emptyRules, type ChannelPolicy, type ModelSnapshot } from '@/features/models/api'
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn().mockResolvedValue('/fixture/models.json') }))
vi.mock('@/features/sync/api', async original => ({...await original<typeof import('@/features/sync/api')>(),getSyncConfig:vi.fn().mockResolvedValue({endpoint:{endpointId:'default',revision:'a'.repeat(64),url:'https://dav.example.test',remotePath:'scope',username:'fixture',credentialRefId:'opaque',tlsPolicy:'verify_required',payloadProtection:'integrity_only',conflictPolicy:'ask'}})}))
vi.mock('@/features/backup/api', async original => ({ ...await original<typeof import('@/features/backup/api')>(), createBackup: vi.fn().mockResolvedValue({ id: 'backup-1' }) }))
import { createBackup } from '@/features/backup/api'
import { useSyncStore } from '@/features/sync/store'
import {getSyncConfig} from '@/features/sync/api'
vi.mock('@/features/sync/modelTransfer', async original => ({ ...await original<typeof import('@/features/sync/modelTransfer')>(), transferChoiceApi:{load:vi.fn(),save:vi.fn()},transferProtectionApi:{load:vi.fn(),save:vi.fn()},importCredentialChoiceApi:{load:vi.fn(),save:vi.fn()},modelTransferApi: {protect:vi.fn(),discardUnattempted:vi.fn(),prepareDraft:vi.fn(),prepare:vi.fn(),export:vi.fn(),receive:vi.fn(),accept:vi.fn(),select:vi.fn(),prepareUpload:vi.fn(),publish:vi.fn(),download:vi.fn(),latestIncoming:vi.fn(),recompare:vi.fn(),openReceived:vi.fn(),loadRemoteScope:vi.fn(),saveRemoteScope:vi.fn(),latestUpload:vi.fn(),checkSync:vi.fn(),accountCatalog:vi.fn(),channelCatalog:vi.fn(),extensionCatalog:vi.fn()} }))
function snapshot(): ModelSnapshot {
  return { library:{schemaVersion:1,templates:[{id:'family',label:'测试族',source:'user',defaultRevision:null,revisions:[{id:'revision-1',createdAt:'2026-10-18T00:00:00Z',savedLabel:'测试族',content:emptyContent(),digest:'digest',baseRevision:null,changes:[]}]}]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'policy',workspace:null,workspaceDigest:'workspace' }
}
const pack = (): ModelPackage => ({format:'OCXD_MODEL_CONFIG',formatVersion:1,schemaVersion:1,createdAt:'2026-10-18T00:00:00Z',digest:'frozen-digest',payload:{templates:snapshot().library.templates,policy:null}})
const dialog = defineComponent({ props:['open','busy'], emits:['close'], template:'<section v-if="open"><button aria-label="关闭" :disabled="busy" @click="$emit(\'close\')">×</button><slot/><slot name="actions" :close="()=> $emit(\'close\')"/></section>' })
function render(remote = false,source:'saved'|'draft'='saved') { return mount(ModelTransfer, {props:{remote,source},global:{stubs:{UiDialog:dialog,UiSelectField:true}}}) }
async function chooseExport(wrapper:ReturnType<typeof render>,name:string,selected=true){await flushPromises();const search=wrapper.get('input[aria-label="搜索配置对象"]');await search.setValue(name);await wrapper.get('input[aria-label="选择 '+name+'"]').setValue(selected);await search.setValue('')}

async function openReceivedDetails(wrapper:ReturnType<typeof render>,group:string,label:string){const host=wrapper.get('.model-transfer-scope');const toggle=host.findAll('button').find(b=>b.attributes('aria-label')==='展开 '+group);if(toggle)await toggle.trigger('click');await host.get('button[aria-label="'+label+'"]').trigger('click');await flushPromises()}
async function openDetails(wrapper:ReturnType<typeof render>,label:string){const search=wrapper.find('input[aria-label="搜索配置对象"]');if(search.exists())await search.setValue(label);await flushPromises();await wrapper.findAll('button').find(b=>b.attributes('aria-label')===label)!.trigger('click');await flushPromises()}
// 原型对象树接收侧默认折叠；操作子项前先展开对应分组。
async function customExportScope(wrapper:ReturnType<typeof render>){
 await flushPromises();await wrapper.get('input[aria-label="选择当前筛选全部配置对象"]').setValue(true);await wrapper.get('input[aria-label="选择当前筛选全部配置对象"]').setValue(false)
 await chooseExport(wrapper,'通用偏好')
 const global=wrapper.findAll('label').find(l=>l.text()==='全局采用及渠道组顺序');if(global)await global.get('input').setValue(false)
}


beforeEach(() => { setActivePinia(createPinia());vi.clearAllMocks();localStorage.clear();vi.mocked(getSyncConfig).mockResolvedValue({endpoint:{endpointId:'default',revision:'a'.repeat(64),url:'https://dav.example.test',remotePath:'scope',username:'fixture',credentialRefId:'opaque',tlsPolicy:'verify_required',payloadProtection:'integrity_only',conflictPolicy:'ask'}});vi.mocked(transferChoiceApi.load).mockResolvedValue(null);vi.mocked(transferChoiceApi.save).mockImplementation(async(_remote,choice)=>choice);vi.mocked(transferProtectionApi.load).mockResolvedValue({enabled:false,hasPassword:false});vi.mocked(transferProtectionApi.save).mockResolvedValue({enabled:false,hasPassword:false});vi.mocked(importCredentialChoiceApi.load).mockResolvedValue({keys:true,extensions:true});vi.mocked(modelTransferApi.discardUnattempted).mockResolvedValue(true);vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.extensionCatalog).mockResolvedValue({skills:['fixture-skill'],mcps:['fixture-mcp']});vi.mocked(modelTransferApi.loadRemoteScope).mockResolvedValue({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false});useModelsStore().snapshot=snapshot() })
describe('model transfer review', () => {
  it('keeps a cloud-only template selected through the actual first range save/check flow', async () => {
    useModelsStore().snapshot!.library.templates=[]
    vi.mocked(modelTransferApi.loadRemoteScope).mockResolvedValue({templateIds:['cloud-template'],channelIds:[],includeGlobal:false,preferences:true,extensions:false,includeKeys:false})
    const catalogue=vi.spyOn(cloudCatalogApi,'load').mockResolvedValue({binding:'fixture',createdAt:'',credentialIds:[],objects:[{id:JSON.stringify(['templates','cloud-template']),category:'templates',categoryLabel:'模型模版',label:'仅云端模版',detail:'云端已保存内容 · 本机尚无',mark:'云端配置'}]})
    vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'unchanged',hasRemote:true})
    const wrapper=render(true);await flushPromises()
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('仅云端模版')
    expect((wrapper.get('input[aria-label="选择 仅云端模版"]').element as HTMLInputElement).checked).toBe(true)
    await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    expect(modelTransferApi.saveRemoteScope).toHaveBeenCalledWith(expect.objectContaining({templateIds:['cloud-template']}))
    expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();expect(modelTransferApi.download).not.toHaveBeenCalled()
    wrapper.unmount();catalogue.mockRestore()
  })

  it('distinguishes same-name channel, Skill and MCP credential choices without changing their source identities',async()=>{
    const packageValue=pack();packageValue.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel',label:'same-name',selection:{mode:'none'}}]}
    const plan={id:'same-name-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:packageValue,notices:[],availableKeys:['channel'],availableExtensionIds:['skill:same-name','mcp:same-name'],availableExtensionCredentials:true,assets:{skills:[{name:'same-name',digest:'fixture-skill',files:1,bytes:10,warnings:[],protected:true}],mcps:[{name:'same-name',digest:'fixture-mcp',secretFields:['/env/API_KEY'],warnings:[]}]}}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(plan);vi.mocked(modelTransferApi.select).mockResolvedValue(plan);vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises()
    const choice=(text:string)=>wrapper.findAll('label').find(label=>label.text()===text)!
    expect(choice('渠道 · same-name · 凭据')).toBeDefined();expect(choice('Skill · same-name · 凭据')).toBeDefined();expect(choice('MCP · same-name · 凭据')).toBeDefined()
    await choice('MCP · same-name · 凭据').get('input').setValue(false);await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).toHaveBeenCalledWith('same-name-incoming',expect.objectContaining({credentialIds:['channel:channel','skill:same-name'],adoptKeys:true,adoptExtensionCredentials:true}));wrapper.unmount()
  })
  it('exports display preferences without the hidden whole-extension configuration even for remembered legacy choices',async()=>{
    vi.mocked(transferChoiceApi.load).mockResolvedValue({selectedIds:[JSON.stringify(['manager','preferences'])],preferences:true,extensions:true,credentialIds:[],versions:{},mcpSources:{},includeGlobal:false,webdavPasswords:false})
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'display-only',package:pack()})
    const wrapper=render();await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.anything(),undefined,false,expect.objectContaining({preferences:true,extensions:false}))
    wrapper.unmount()
  })
  it('reselects a legacy file before confirmation so display preferences do not adopt extension configuration',async()=>{
    const original={id:'legacy-settings',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],manager:{preferences:{theme:'system'},extensions:{skills:['not-selected-skill'],servers:['not-selected-mcp'],sync_method:'copy'},webdav:[],accounts:[]},comparison:{source:'file' as const,createdAt:'2026-10-18T00:00:00Z',changes:[{kind:'preferences',id:'preferences',label:'通用偏好',mark:'update' as const}],unchanged:0}}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original)
    vi.mocked(modelTransferApi.select).mockResolvedValue({...original,manager:{...original.manager,extensions:null}})
    const wrapper=render();await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click')
    await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('legacy-settings',expect.objectContaining({preferences:true,extensions:false}))
    expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })

  it('blocks legacy-file confirmation if projecting the visible portable scope fails',async()=>{
    const original={id:'legacy-settings-failed',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],manager:{preferences:{theme:'system'},extensions:{skills:['not-selected'],servers:[],sync_method:'copy'},webdav:[]},comparison:{source:'file' as const,createdAt:'2026-10-18T00:00:00Z',changes:[],unchanged:0}}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original)
    vi.mocked(modelTransferApi.select).mockRejectedValue({message:'fixture projection failed'})
    const wrapper=render();await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click')
    await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises()
    const confirm=wrapper.findAll('button').find(b=>b.text()==='确认更新')!
    expect(confirm.attributes('disabled')).toBeDefined();expect(wrapper.text()).toContain('fixture projection failed')
    expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('shows scope inline without preparing or writing on mount, and blocks writes until the saved remote scope is known',async()=>{
    let resolve!:(value:Awaited<ReturnType<typeof modelTransferApi.loadRemoteScope>>)=>void
    vi.mocked(modelTransferApi.loadRemoteScope).mockReturnValue(new Promise(done=>{resolve=done}))
    const wrapper=render(true);await flushPromises()
    expect(wrapper.find('section[aria-label="同步范围"]').exists()).toBe(true)
    expect(wrapper.find('.sync-scope-credentials').exists()).toBe(true)
    const save=()=>wrapper.findAll('button').find(b=>b.text()==='同步')!
    expect(save().attributes('disabled')).toBeDefined()
    // SY-1：口令只在确认弹窗内，不占范围侧栏。
    expect(wrapper.find('input[aria-label="导出保护口令"]').exists()).toBe(false)
    expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();expect(modelTransferApi.saveRemoteScope).not.toHaveBeenCalled()
    resolve({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:false,includeKeys:false});await flushPromises()
    expect(save().attributes('disabled')).toBeUndefined();expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled()
    wrapper.unmount()
  })
  it('preserves explicit file selections after an initially failed catalogue is recovered, without applying all by surprise',async()=>{
    vi.mocked(modelTransferApi.channelCatalog).mockRejectedValueOnce({message:'目录读取失败'}).mockResolvedValue([])
    const wrapper=render();await flushPromises();await chooseExport(wrapper,'测试族')
    const before=[...wrapper.getComponent(ObjectScopeTree).props('selectedIds')]
    await wrapper.findAll('button').find(b=>b.text()==='重新读取范围目录')!.trigger('click');await flushPromises()
    expect(wrapper.getComponent(ObjectScopeTree).props('selectedIds')).toEqual(before)
    expect(wrapper.text()).not.toContain('目录读取失败');expect(modelTransferApi.prepare).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('does not overwrite an unreadable saved WebDAV scope with initial defaults; an explicit reload can recover',async()=>{
    vi.mocked(modelTransferApi.loadRemoteScope).mockRejectedValue({message:'范围读取失败'})
    const wrapper=render(true);await flushPromises()
    const save=()=>wrapper.findAll('button').find(b=>b.text()==='同步')!
    expect(save().attributes('disabled')).toBeDefined();expect(wrapper.text()).toContain('范围读取失败')
    expect(modelTransferApi.saveRemoteScope).not.toHaveBeenCalled()
    vi.mocked(modelTransferApi.loadRemoteScope).mockResolvedValue({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:false,includeKeys:false})
    await wrapper.findAll('button').find(b=>b.text()==='重新读取范围目录')!.trigger('click');await flushPromises()
    expect(save().attributes('disabled')).toBeUndefined();expect(modelTransferApi.saveRemoteScope).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('preserves explicit legacy partial selections and selects exact families instead of a model subset, carries shared fields only when checked, and serializes the same WebDAV scope',async()=>{
    const store=useModelsStore();const c:ChannelPolicy={id:'family-channel',label:'族范围渠道',templateRefs:[],modelRefs:{},connection:{providerId:'p',protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:false,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:['raw-a','raw-b'],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}
    store.snapshot!.policy.channels=[c];store.snapshot!.policy.templateRefs=[{templateId:'family',revisionId:'revision-1'}]
    vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([c]);vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'family-file',package:pack()})
    vi.mocked(transferChoiceApi.load).mockResolvedValue({selectedIds:[JSON.stringify(['channels','family-channel'])],credentialIds:[],versions:{},mcpSources:{},includeGlobal:false,webdavPasswords:false,channelModels:{'family-channel':['raw-a','raw-b']}})
    const wrapper=render();await flushPromises();await openDetails(wrapper,'族范围渠道')
    await wrapper.get('input[aria-label="族范围渠道范围内型号 raw-b"]').setValue(false)
    const granularity=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='族范围渠道策略范围')!
    granularity.vm.$emit('update:modelValue','families');await flushPromises()
    expect(wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.attributes('disabled')).toBeDefined()
    await wrapper.get('input[aria-label="族范围渠道范围内模型族 测试族"]').setValue(true)
    expect((wrapper.get('input[aria-label="族范围渠道包含渠道共用默认字段"]').element as HTMLInputElement).checked).toBe(false)
    expect(wrapper.find('input[aria-label="族范围渠道范围内型号 raw-a"]').exists()).toBe(false)
    await wrapper.get('input[aria-label="族范围渠道包含渠道共用默认字段"]').setValue(true)
    await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises()
    const scope=vi.mocked(modelTransferApi.prepare).mock.calls.at(-1)![0]
    expect(scope.channelFamilies).toEqual({'family-channel':{templateIds:['family'],includeChannelDefaults:true}})
    expect(scope.channelModels).toBeUndefined();expect(scope.includeGlobal).toBe(false)
    expect(remoteScope(scope).channelFamilies).toEqual(scope.channelFamilies)
    wrapper.unmount()
  })
  it('restores and saves the same explicit family scope for this WebDAV device',async()=>{
    const c:ChannelPolicy={id:'remote-channel',label:'本设备渠道',templateRefs:[{templateId:'family',revisionId:'revision-1'}],modelRefs:{},connection:{providerId:'p',protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:false,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:[],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}
    useModelsStore().snapshot!.policy.channels=[c]
    const mask={'remote-channel':{templateIds:['family'],includeChannelDefaults:false}}
    vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([c]);vi.mocked(modelTransferApi.loadRemoteScope).mockResolvedValue({templateIds:[],channelIds:['remote-channel'],channelFamilies:mask,includeGlobal:false,preferences:false,extensions:false,includeKeys:false})
    vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    const wrapper=render(true);await flushPromises();await flushPromises();await flushPromises()
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'local',hasRemote:false})
    vi.mocked(modelTransferApi.prepareUpload).mockResolvedValue({id:'up',phase:'prepared',package:pack(),sync:{source:'local',changes:[],unchanged:1,createdAt:'2026-11-01T10:00:00Z'}})
    await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    expect(modelTransferApi.saveRemoteScope).toHaveBeenCalledWith(expect.objectContaining({channelIds:['remote-channel'],channelFamilies:mask,includeGlobal:false,includeKeys:false}))
    expect(modelTransferApi.publish).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('does not offer a received family fragment as whole model or independent channel material',async()=>{
    const source=pack();source.formatVersion=5;source.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'fragment',label:'族片段',familyScope:{templateIds:['family'],includeChannelDefaults:false},modelSubset:[],selection:{mode:'none'}}]}
    vi.mocked(modelTransferApi.receive).mockResolvedValue({id:'received-family',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:source,notices:[]})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises();const expandChannel=wrapper.findAll('button').find(b=>b.attributes('aria-label')==='展开 渠道与模型');if(expandChannel)await expandChannel.trigger('click')
    await openDetails(wrapper,'族片段');expect(wrapper.text()).toContain('共用默认字段沿本机值解析');expect(wrapper.text()).toContain('仅合并所选族')
    expect(wrapper.find('input[aria-label="族片段型号范围搜索"]').exists()).toBe(false)
    expect(wrapper.findAllComponents(UiSelectField).some(c=>c.props('label')==='族片段接收方式')).toBe(false);wrapper.unmount()
  })

  it('does not reintroduce cancelled scope presets or a separate clear action',async()=>{const wrapper=render();await flushPromises();expect(wrapper.findAllComponents(UiSelectField).some(c=>c.props('label')==='配置范围预设')).toBe(false);expect(wrapper.text()).not.toContain('清空选择');wrapper.unmount()})
  it('resets a reselected WebDAV object to carrying its password while retaining other explicit choices',async()=>{
    const sync=await import('@/features/sync/api');vi.spyOn(sync,'getSyncConfig').mockResolvedValue({endpoint:{endpointId:'dav',url:'https://dav.example.test',remotePath:'config',username:'fixture',credentialRefId:'cred-fixture',tlsPolicy:'verify_required',payloadProtection:'integrity_only',conflictPolicy:'ask'}})
    const wrapper=render();await flushPromises();await flushPromises();await flushPromises()
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('WebDAV连接配置');await flushPromises();const webdavToggle=wrapper.findAll('button').find(b=>b.attributes('aria-label')==='展开 WebDAV 配置');if(webdavToggle)await webdavToggle.trigger('click');await flushPromises();await openDetails(wrapper,'WebDAV连接配置');await wrapper.findAll('label').find(l=>l.text()==='包含连接密码')!.get('input').setValue(false)
    await chooseExport(wrapper,'WebDAV连接配置',false);await chooseExport(wrapper,'WebDAV连接配置',true);await wrapper.get('input[aria-label="搜索配置对象"]').setValue('WebDAV连接配置')
    expect((wrapper.findAll('label').find(l=>l.text()==='包含连接密码')!.get('input').element as HTMLInputElement).checked).toBe(true);wrapper.unmount();vi.mocked(sync.getSyncConfig).mockRestore()
  })
  it('lists saved original channels independently of the Desktop policy and exports their exact selected identity',async()=>{
    const original={id:'official-channel',label:'原有官方渠道',templateRefs:[],modelRefs:{},connection:{providerId:'official-source',protocol:'openai-chat',nodes:[{id:'service',name:'主站',url:'https://official.example.test'}],activeNodeId:'service',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist' as const,models:['original/model']},modelOrder:['original/model','original/other'],discovered:[{id:'original/model',label:'模型A',rules:{}},{id:'original/other',label:'模型B',rules:{}}],defaults:{},overrides:{},familyOverrides:{}};
    vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([original as never]);
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'original-export',package:pack()});
    const wrapper=render();await flushPromises();await customExportScope(wrapper);await flushPromises();
    expect(useModelsStore().snapshot!.policy.channels).toHaveLength(0);
    await chooseExport(wrapper,'原有官方渠道');
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('原有官方渠道');
    await openDetails(wrapper,'原有官方渠道');expect(wrapper.find('input[aria-label="原有官方渠道范围内型号 original/other"]').exists()).toBe(false);
    await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises();
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.objectContaining({includePolicy:true,channelIds:['official-channel']}),undefined,false,expect.any(Object));
    wrapper.unmount();
  })

  it('automatically compares explicit draft compatibility asset mappings',async()=>{
    const original={id:'asset-conflict',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],previewError:'asset_conflict',assets:{skills:[{name:'fixture-skill',digest:'skill-digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[{name:'fixture-mcp',digest:'mcp-digest',secretFields:[],warnings:[]}]}}
    original.package.payload.candidate={expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace'}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original);vi.mocked(modelTransferApi.select).mockResolvedValue({...original,previewError:null,assetMappings:{skills:{'fixture-skill':'received-skill'},mcps:{'fixture-mcp':'received-mcp'}}})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises()
    await openReceivedDetails(wrapper,'Skills','Skill · fixture-skill');await wrapper.get('input[aria-label="fixture-skill目标Skill名称"]').setValue('received-skill');await openReceivedDetails(wrapper,'MCP','MCP · fixture-mcp');await wrapper.get('input[aria-label="fixture-mcp目标MCP名称"]').setValue('received-mcp')
    expect(wrapper.findAll('button').find(b=>b.text()==='确认更新')!.attributes('disabled')).toBeDefined()
    await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('asset-conflict',expect.objectContaining({assetMappings:{skills:{'fixture-skill':'received-skill'},mcps:{'fixture-mcp':'received-mcp'}}}))
    expect(wrapper.text()).toContain('received-skill（来源：fixture-skill）');expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('automatically compares an explicit endpoint-conflict channel mapping',async()=>{
    const value=pack();value.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel-source',label:'来源渠道',selection:{mode:'none'},connection:{nodes:[{id:'node-source',name:'主站',url:'https://source.example.test/v1'}]}}]}
    const original={id:'channel-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:value,notices:[],previewError:'channel_endpoint_conflict'}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original)
    vi.mocked(modelTransferApi.select).mockResolvedValue({...original,previewError:null,channelBindings:{'channel-source':{targetId:'new-channel',keepLocalConnection:false,label:'独立接收'}}})
    const select=defineComponent({props:['modelValue','options','label','disabled'],emits:['update:modelValue'],template:'<label>{{label}}<select :value="modelValue" :disabled="disabled" @change="$emit(\'update:modelValue\',$event.target.value)"><option v-for="item in options" :key="item.value" :value="item.value">{{item.label}}</option></select></label>'})
    const wrapper=mount(ModelTransfer,{global:{stubs:{UiDialog:dialog,UiSelectField:select}}});await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises()
    await openReceivedDetails(wrapper,'渠道与模型','来源渠道');await wrapper.findAll('label').find(l=>l.text().startsWith('来源渠道接收方式'))!.get('select').setValue('new');await flushPromises();await wrapper.findAll('label').find(l=>l.text()==='新渠道名称')!.get('input').setValue('独立接收')
    expect(modelTransferApi.accept).not.toHaveBeenCalled()
    await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('channel-incoming',expect.objectContaining({channelMappings:{'channel-source':{mode:'new',label:'独立接收'}}}))
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(wrapper.text()).toContain('独立停用草稿');wrapper.unmount()
  })
  it('exports only explicitly selected account information and explains unsupported login migration',async()=>{
    vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([{account:{id:'fixture-account',provider:'chatgpt',subject:'fixture-subject',label:'测试账户',email:'fixture@example.test'},loginTransfer:'manualImportDisabled',localSource:true,informationSupported:true}])
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'account-file',package:pack()})
    const wrapper=render();await flushPromises();expect(modelTransferApi.accountCatalog).toHaveBeenCalledOnce();expect(modelTransferApi.prepare).not.toHaveBeenCalled()
    await flushPromises();await customExportScope(wrapper);await flushPromises()
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('测试账户');await openDetails(wrapper,'测试账户');expect(wrapper.text()).toContain('官方不支持手工导入登录态');expect((wrapper.get('input[aria-label="选择 测试账户"]').element as HTMLInputElement).checked).toBe(false)
    await chooseExport(wrapper,'测试账户');await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.anything(),undefined,false,expect.objectContaining({accountIds:['fixture-account']}));wrapper.unmount()
  })
  it('keeps two accounts independently selected across platform filtering and excludes an unsupported source',async()=>{
    const origin={version:1,accountPlatform:'chatgpt',owner:'opencodex',authKind:'oauth',sourceNamespace:'codex-account-pool',sourceAccountId:'slot',tenant:'shared'}
    vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([
      {account:{id:'personal',provider:'chatgpt',subject:'personal-sub',label:'个人账户',email:'same@example.test',origin:{...origin,sourceAccountId:'personal'}},loginTransfer:'manualImportDisabled',localSource:true,informationSupported:true},
      {account:{id:'work',provider:'chatgpt',subject:'work-sub',label:'工作账户',email:'same@example.test',origin:{...origin,sourceAccountId:'work'}},loginTransfer:'manualImportDisabled',localSource:true,informationSupported:true},
      {account:{id:'unsupported',provider:'chatgpt',subject:'unavailable',label:'未支持来源',email:null,origin:{...origin,owner:'platform-native'}},loginTransfer:'unsupportedProvider',localSource:false,informationSupported:false},
    ])
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'accounts-selected',package:pack()})
    const wrapper=render();await flushPromises();await flushPromises();await customExportScope(wrapper)
    await chooseExport(wrapper,'工作账户');const accountToggle=wrapper.find('button[aria-label="展开 账户"]');if(accountToggle.exists())await accountToggle.trigger('click');const filter=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='账户平台筛选')!
    filter.vm.$emit('update:modelValue','antigravity');await flushPromises();expect(wrapper.text()).toContain('筛选外仍有')
    filter.vm.$emit('update:modelValue','chatgpt');await flushPromises();await wrapper.get('input[aria-label="搜索配置对象"]').setValue('工作账户')
    expect((wrapper.get('input[aria-label="选择 工作账户"]').element as HTMLInputElement).checked).toBe(true)
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('未支持来源');expect((wrapper.get('input[aria-label="选择 未支持来源"]').element as HTMLInputElement).disabled).toBe(true)
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('');await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.anything(),undefined,false,expect.objectContaining({accountIds:['work']}));wrapper.unmount()
  })
  it('lets incoming account information be excluded without touching credential adoption',async()=>{
    const original={id:'account-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],manager:{preferences:null,extensions:null,webdav:[],accounts:[{id:'a',provider:'chatgpt',subject:'subject',label:'测试账户',email:null}]}}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original)
    vi.mocked(modelTransferApi.select).mockResolvedValue({...original,manager:{...original.manager,accounts:[]}})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises()
    expect(wrapper.text()).toContain('不代表登录已恢复');await wrapper.get('button[aria-label="展开 账户"]').trigger('click');await wrapper.findAll('button').find(b=>b.attributes('aria-label')==='展开 账户')?.trigger('click');await flushPromises();await wrapper.get('input[aria-label="选择 测试账户"]').setValue(false)
    await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('account-incoming',expect.objectContaining({accountIds:[]}));expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('exports the reviewed frozen plan instead of rebuilding from later editor content', async () => {
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'frozen-plan',package:pack()})
    vi.mocked(modelTransferApi.export).mockResolvedValue({path:'/fixture/export.json',digest:'frozen-digest',templates:1,channels:0})
    const wrapper=render()
    await flushPromises();await customExportScope(wrapper)
    await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith({includePolicy:false,channelIds:[],includeGlobal:false,templateRefs:[{templateId:'family',revisionId:'revision-1'}]},undefined,false,{preferences:true,extensions:false,webdav:false,webdavPasswords:true,credentialIds:[]})
    expect(modelTransferApi.export).not.toHaveBeenCalled()
    useModelsStore().snapshot!.library.templates[0]!.label='later'
    await wrapper.findAll('button').find(b=>b.text()==='生成配置包')!.trigger('click');await flushPromises()
    expect(modelTransferApi.export).toHaveBeenCalledExactlyOnceWith('frozen-plan', undefined)
    expect(useSyncStore().lastResultId).toBe('frozen-plan');expect(wrapper.find('.model-transfer-path').exists()).toBe(false);wrapper.unmount()
  })
  it('removes the unattempted frozen package after an export preview is canceled', async () => {
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'canceled-file-plan',package:pack()})
    const wrapper=render();await flushPromises();await customExportScope(wrapper);await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledOnce();expect(modelTransferApi.export).not.toHaveBeenCalled()
    await wrapper.findAll('button').find(b=>b.text()==='取消')!.trigger('click');await flushPromises()
    expect(modelTransferApi.discardUnattempted).toHaveBeenCalledExactlyOnceWith('canceled-file-plan')
    expect(modelTransferApi.export).not.toHaveBeenCalled();expect(wrapper.find('.transfer-review').exists()).toBe(false);wrapper.unmount()
  })
  it('discards both plaintext source and protected child when the protected preview is canceled after a failed attempt', async () => {
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'plain-preview-plan',package:pack()})
    vi.mocked(modelTransferApi.protect).mockResolvedValue({id:'protected-preview-plan',package:pack(),protected:true})
    vi.mocked(modelTransferApi.export).mockRejectedValue({message:'fixture destination unavailable'})
    vi.mocked(modelTransferApi.discardUnattempted).mockImplementation(async id=>id!=='protected-preview-plan')
    const wrapper=render();await flushPromises();await customExportScope(wrapper);await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises()
    await wrapper.get('.export-encrypt input').setValue(true)
    await wrapper.get('input[aria-label="加密口令"]').setValue('fixture-passphrase')
    await wrapper.get('input[aria-label="再次输入加密口令"]').setValue('fixture-passphrase')
    await wrapper.findAll('button').find(b=>b.text()==='生成配置包')!.trigger('click');await flushPromises()
    expect(modelTransferApi.protect).toHaveBeenCalledExactlyOnceWith('plain-preview-plan','fixture-passphrase')
    expect(modelTransferApi.export).toHaveBeenCalledExactlyOnceWith('protected-preview-plan',undefined)
    await wrapper.findAll('button').find(b=>b.text()==='取消')!.trigger('click');await flushPromises()
    expect(modelTransferApi.discardUnattempted).toHaveBeenCalledWith('plain-preview-plan')
    expect(modelTransferApi.discardUnattempted).toHaveBeenCalledWith('protected-preview-plan')
    expect(wrapper.text()).not.toContain('临时导出快照清理失败');wrapper.unmount()
  })
  it('SY-1/SY-2/N-SY-2：导出确认弹窗按原型顺序承载保存位置、树、说明、凭据与可选口令，范围首项保持 all',async()=>{
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'prototype-shape',package:pack(),credentialCount:1})
    vi.mocked(modelTransferApi.export).mockResolvedValue({path:'/fixture/export.json',digest:'frozen-digest',templates:1,channels:0})
    const wrapper=render();await flushPromises();await flushPromises()
    expect(wrapper.findAllComponents(UiSelectField).some(c=>c.props('label')==='配置范围预设')).toBe(false)
    await customExportScope(wrapper);await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises()
    const text=wrapper.text()
    expect(text.indexOf('保存位置')).toBeLessThan(text.indexOf('生成位置'))
    expect(text.indexOf('生成位置')).toBeLessThan(text.indexOf('项配置 · 同一备份包'))
    expect(text.indexOf('项配置 · 同一备份包')).toBeLessThan(text.indexOf('加密配置包'))
    expect(wrapper.findAll('input[type="radio"]')).toHaveLength(2)
    expect(wrapper.find('input[aria-label="自定义导出目录"]').exists()).toBe(false)
    await wrapper.findAll('input[type="radio"]')[1]!.setValue(true)
    expect(wrapper.find('input[aria-label="自定义导出目录"]').exists()).toBe(true)
    expect(wrapper.findAll('button').some(b=>b.text()==='选择目录…')).toBe(true)
    // 口令只在弹窗内；按原型固定呈现两次输入。含凭据时默认建议加密且可取消。
    expect(wrapper.find('input[aria-label="加密口令"]').exists()).toBe(false)
    await wrapper.findAll('label').find(l=>l.text().startsWith('加密配置包'))!.get('input').setValue(true)
    expect(wrapper.get('input[aria-label="加密口令"]').attributes('disabled')).toBeUndefined()
    expect(wrapper.find('input[aria-label="再次输入加密口令"]').exists()).toBe(true)
    wrapper.unmount()
  })
  it('credential-bearing plaintext export has one confirmation',async()=>{
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'plaintext-plan',package:pack(),credentialCount:2})
    vi.mocked(modelTransferApi.export).mockResolvedValue({path:'/fixture/export.json',digest:'frozen-digest',templates:1,channels:0})
    const wrapper=render();await flushPromises();await customExportScope(wrapper);await chooseExport(wrapper,'测试族');await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises()
    expect(wrapper.get<HTMLInputElement>('.export-encrypt input').element.checked).toBe(false)
    expect(wrapper.text()).toContain('以明文写入');await wrapper.findAll('button').find(b=>b.text()==='生成配置包')!.trigger('click');await flushPromises()
    expect(modelTransferApi.export).toHaveBeenCalledExactlyOnceWith('plaintext-plan',undefined);expect(wrapper.text()).not.toContain('仍以明文导出');wrapper.unmount()
  })
  it('N-SY-1：树表列名与原型一致',async()=>{
    const wrapper=mount(ObjectScopeTree,{props:{objects:[{id:'x',category:'channels',categoryLabel:'渠道与模型',label:'测试渠道',detail:'原设置'}],selectedIds:['x'],label:'导出配置对象'}})
    expect(wrapper.findAll('thead th').map(th=>th.text())).toEqual(['配置对象','范围与凭据']);wrapper.unmount()
  })
  it('SY-2：目录读取失败时仍保持默认范围并明确报错',async()=>{
    vi.mocked(modelTransferApi.channelCatalog).mockRejectedValueOnce({message:'目录读取失败'})
    const wrapper=render();await flushPromises();await flushPromises()
    expect(wrapper.text()).toContain('目录读取失败')
    expect(wrapper.text()).not.toContain('自定义范围');wrapper.unmount()
  })
  it('receives only a review and requires adoption while preserving the manual template buffer', async () => {
    const store=useModelsStore();store.edit();store.draft!.label='人工输入'
    vi.mocked(modelTransferApi.receive).mockResolvedValue({id:'incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]})
    vi.mocked(modelTransferApi.accept).mockResolvedValue({...snapshot(),libraryDigest:'received'})
    const wrapper=render()
    await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises()
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(store.snapshot!.libraryDigest).toBe('library')
    expect((wrapper.findAll('label').find(l=>l.text()==='更新前备份')!.get('input').element as HTMLInputElement).checked).toBe(true)
    await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises()
    expect(createBackup).not.toHaveBeenCalled()
    expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('incoming',expect.objectContaining({backupBeforeUpdate:true,adoptKeys:false}));expect(store.snapshot!.libraryDigest).toBe('received')
    expect(store.draft!.label).toBe('人工输入');expect(store.dirty).toBe(true);wrapper.unmount()
  })
  it('blocks a preview overlapping live channel edits before calling accept',async()=>{
    const store=useModelsStore(),base=snapshot()
    const channel:ChannelPolicy={id:'channel',label:'saved label',templateRefs:[],modelRefs:{},connection:{providerId:'provider',protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:false,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:[],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}
    base.policy.channels=[channel];store.acceptTransfer(base);store.channelDraft!.channels[0]!.label='unsaved label';store.clearCredential('channel')
    const incoming=structuredClone(base.policy);incoming.channels[0]!.label='incoming label'
    vi.mocked(modelTransferApi.receive).mockResolvedValue({id:'overlap',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',mergedPolicy:incoming,package:pack(),notices:[]})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(createBackup).not.toHaveBeenCalled();expect(wrapper.text()).toContain('当前输入已保留')
    expect(store.channelDraft!.channels[0]!.label).toBe('unsaved label');expect(store.credentialEdits.channel?.action).toBe('clear');wrapper.unmount()
  })
  it('exports the frozen in-memory draft policy and reuses it for optional passphrase preparation',async()=>{
    const store=useModelsStore(),base=snapshot()
    base.policy.channels=[{id:'channel',label:'saved label',templateRefs:[],modelRefs:{},connection:{providerId:'provider',protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:false,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:[],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}]
    store.acceptTransfer(base);store.channelDraft!.channels[0]!.label='in-memory label';store.editCredential('channel','local-only-secret','saved')
    const frozen=JSON.parse(JSON.stringify(store.channelDraft))
    vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue(base.policy.channels)
    vi.mocked(modelTransferApi.prepareDraft).mockResolvedValueOnce({id:'plain-draft',package:pack(),credentialCount:0});vi.mocked(modelTransferApi.protect).mockResolvedValue({id:'encrypted-draft',package:pack(),credentialCount:0,protected:true})
    vi.mocked(modelTransferApi.export).mockResolvedValue({path:'/fixture/draft.ocxdconf',digest:'digest',templates:0,channels:1})
    const wrapper=render(false,'draft');await flushPromises();await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises()
    const candidate={expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace'}
    expect(modelTransferApi.prepareDraft).toHaveBeenCalledExactlyOnceWith({includePolicy:true,channelIds:['channel'],includeGlobal:false,templateRefs:[]},candidate,undefined,frozen)
    expect(modelTransferApi.prepare).not.toHaveBeenCalled();expect(JSON.stringify(vi.mocked(modelTransferApi.prepareDraft).mock.calls[0]![3])).not.toContain('local-only-secret')
    store.channelDraft!.channels[0]!.label='typed after export preview'
    expect(wrapper.get<HTMLInputElement>('.export-encrypt input[type="checkbox"]').element.checked).toBe(false)
    await wrapper.get('.export-encrypt input[type="checkbox"]').setValue(true)
    await wrapper.get('input[aria-label="加密口令"]').setValue('fixture-export-passphrase');await wrapper.get('input[aria-label="再次输入加密口令"]').setValue('fixture-export-passphrase')
    await wrapper.findAll('button').find(b=>b.text()==='生成配置包')!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepareDraft).toHaveBeenCalledOnce();expect(modelTransferApi.protect).toHaveBeenCalledExactlyOnceWith('plain-draft','fixture-export-passphrase')
    expect(modelTransferApi.export).toHaveBeenCalledExactlyOnceWith('encrypted-draft',undefined)
    expect(store.channelDraft!.channels[0]!.label).toBe('typed after export preview');expect(store.credentialEdits.channel?.value).toBe('local-only-secret');wrapper.unmount()
  })
  it('retains the original operation ID after uncertain WebDAV publication',async()=>{
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'local',hasRemote:false});vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    const plan={id:'upload-once',phase:'prepared',package:pack(),sync:{source:'local' as const,changes:[{kind:'templates',id:'family',label:'测试族',mark:'update' as const}],unchanged:0,createdAt:''}}
    vi.mocked(modelTransferApi.prepareUpload).mockResolvedValue(plan);vi.mocked(modelTransferApi.publish).mockRejectedValueOnce({message:'结果待核对'}).mockResolvedValue({...plan,phase:'published'})
    const wrapper=render(true);await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises();expect(wrapper.text()).toContain('结果待核对')
    await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises();expect(vi.mocked(modelTransferApi.publish).mock.calls).toEqual([['upload-once'],['upload-once']]);expect(modelTransferApi.prepareUpload).toHaveBeenCalledOnce();wrapper.unmount()
  })
  it('keeps global-only selection from expanding to every channel', () => {
    expect(scopeFor(snapshot(),[],[],true,{})).toEqual({includePolicy:true,channelIds:[],templateRefs:[],includeGlobal:true})
  })
  it('automatically compares an import subset and adopts only its resulting plan',async()=>{
    const incoming={id:'original',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(incoming);vi.mocked(modelTransferApi.select).mockResolvedValue({...incoming,id:'selected-plan'});vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises()
    await wrapper.get('.model-transfer-scope input[type=checkbox]').setValue(false);await flushPromises()
    await wrapper.get('.model-transfer-scope input[type=checkbox]').setValue(true);await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('original',{templateRefs:[{templateId:'family',revisionId:'revision-1'}],channelIds:[],includeGlobal:false});await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('selected-plan',expect.objectContaining({backupBeforeUpdate:true}));wrapper.unmount()
  })
  it('asks once for the protected file password and never adopts on decryption failure',async()=>{
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required',message:'请输入保护口令'}).mockRejectedValueOnce({code:'portable_authentication',message:'口令不正确或文件已损坏'}).mockResolvedValue({id:'opened',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises();await flushPromises()
    const input=wrapper.get('input[aria-label="导入解密口令"]');await input.setValue('fixture-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='读取配置')!.trigger('click');await flushPromises();expect(wrapper.text()).toContain('口令不正确');expect(modelTransferApi.accept).not.toHaveBeenCalled()
    await wrapper.get('input[aria-label="导入解密口令"]').setValue('correct-fixture-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='读取配置')!.trigger('click');await flushPromises();expect(modelTransferApi.receive).toHaveBeenLastCalledWith('/fixture/models.json','correct-fixture-password');expect(wrapper.find('input[aria-label="导入解密口令"]').exists()).toBe(false);expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('restores the default import Key preference and passes the same file password',async()=>{
    const packageValue=pack();packageValue.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel',label:'测试渠道',selection:{mode:'none'}}]}
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue({id:'protected-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:packageValue,notices:[],availableKeys:['channel']})
    vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises();await wrapper.get('input[aria-label="导入解密口令"]').setValue('fixture-key-file-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='读取配置')!.trigger('click');await flushPromises()
    const toggle=wrapper.findAll('label').find(label=>label.text().startsWith('采用所选渠道的Key'))!.get<HTMLInputElement>('input');expect(toggle.element.checked).toBe(true);expect(modelTransferApi.accept).not.toHaveBeenCalled()
    await toggle.setValue(true);await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledWith('protected-incoming',expect.objectContaining({adoptKeys:true,passphrase:'fixture-key-file-password',credentialIds:['channel:channel']}));wrapper.unmount()
  })
  it('restores the original import receipt and credential intent through the shared retry dialog',async()=>{
    const packageValue=pack();packageValue.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel',label:'测试渠道',selection:{mode:'none'}}]}
    const original={id:'original-adoption',modelsAccepted:true,expectedLibrary:'library',expectedPolicy:'old-file',expectedWorkspace:'workspace',package:packageValue,notices:[],availableKeys:['channel'],protected:true,adoption:{keys:true,webdav:false,completed:false}}
    const receipt={id:'original-adoption',kind:'import' as const,phase:'partial' as const,startedAt:'',finishedAt:null,items:[],errorCode:'fixture_partial',exportPath:null,sourceDigest:'frozen',retry:{adoptKeys:true,adoptWebdavPasswords:false,adoptExtensionCredentials:false,backupBeforeUpdate:false,credentialIds:['channel:channel']}}
    vi.mocked(modelTransferApi.openReceived).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue(original);vi.mocked(modelTransferApi.accept).mockResolvedValue({...snapshot(),libraryDigest:'received-library'})
    const store=useModelsStore();store.edit();store.draft!.label='retained manual input'
    const wrapper=mount(ModelTransfer,{props:{actionOnly:true,retryReceipt:receipt},global:{stubs:{UiDialog:dialog,UiSelectField:true}}});await flushPromises();expect(modelTransferApi.openReceived).toHaveBeenCalledExactlyOnceWith('original-adoption');expect(modelTransferApi.receive).not.toHaveBeenCalled();expect(modelTransferApi.recompare).not.toHaveBeenCalled()
    await wrapper.get('input[aria-label="导入解密口令"]').setValue('fixture-original-password');await wrapper.findAll('button').find(b=>b.text()==='读取配置')!.trigger('click');await flushPromises();expect(modelTransferApi.openReceived).toHaveBeenLastCalledWith('original-adoption','fixture-original-password');expect(wrapper.text()).not.toContain('选择文件')
    await wrapper.findAll('button').find(b=>b.text()==='核对原导入')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('original-adoption',expect.objectContaining({backupBeforeUpdate:false,adoptKeys:true,credentialIds:['channel:channel'],passphrase:'fixture-original-password'}));expect(store.draft!.label).toBe('retained manual input');expect(wrapper.emitted('close')).toHaveLength(1);wrapper.unmount()
  })
  // S18-01：0.2.18 原型 sync-config.js syncPreview：一个「同步」按钮，按整份快照新旧定方向，确认前只预览。
  it('saves the device scope on 同步, previews a newer local snapshot and publishes only after 确认更新',async()=>{
    vi.mocked(modelTransferApi.saveRemoteScope).mockResolvedValue({templateIds:['family'],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false})
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'local',hasRemote:true})
    vi.mocked(modelTransferApi.prepareUpload).mockResolvedValue({id:'up',phase:'prepared',package:pack(),sync:{source:'local',changes:[{kind:'template',id:'family',label:'测试族',mark:'update',before:1,after:2}],unchanged:3,createdAt:'2026-11-01T10:00:00Z'}})
    vi.mocked(modelTransferApi.publish).mockResolvedValue({id:'up',phase:'published',package:pack()})
    const wrapper=render(true);await flushPromises();await flushPromises();await chooseExport(wrapper,'测试族');await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    expect(modelTransferApi.saveRemoteScope).toHaveBeenCalledExactlyOnceWith(expect.objectContaining({templateIds:['family'],channelIds:[],includeGlobal:false,preferences:true,extensions:false,includeKeys:false}))
    expect(modelTransferApi.prepareUpload).toHaveBeenCalledOnce();expect(modelTransferApi.download).not.toHaveBeenCalled();expect(modelTransferApi.publish).not.toHaveBeenCalled()
    expect(wrapper.text()).toContain('本机配置 → 云端');expect(wrapper.text()).toContain('1 项变化');expect(wrapper.text()).toContain('3 项保持不变');expect(wrapper.text()).toContain('测试族')
    expect(wrapper.findAll('label').some(l=>l.text()==='更新前备份')).toBe(false)
    await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises()
    expect(modelTransferApi.publish).toHaveBeenCalledExactlyOnceWith('up');expect(createBackup).not.toHaveBeenCalled();expect(useSyncStore().webdavState).toBe('synced');expect(useSyncStore().observation?.remoteRevision).toBe('up');wrapper.unmount()
  })
  it('downloads a newer remote snapshot, backs up first when ticked, and never writes on cancel',async()=>{
    vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'remote',hasRemote:true})
    const plan={id:'remote-in',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],sync:{source:'remote' as const,changes:[{kind:'template',id:'family',label:'测试族',mark:'update' as const,before:1,after:2}],unchanged:0,createdAt:'2026-11-01T11:02:00Z'}}
    vi.mocked(modelTransferApi.download).mockResolvedValue(structuredClone(plan));vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render(true);await flushPromises();await flushPromises();await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('云端配置 → 本机')
    const backup=wrapper.findAll('label').find(l=>l.text()==='更新前备份')!.get<HTMLInputElement>('input');expect(backup.element.checked).toBe(true)
    await wrapper.findAll('button').find(b=>b.text()==='取消')!.trigger('click');await flushPromises()
    expect(createBackup).not.toHaveBeenCalled();expect(modelTransferApi.accept).not.toHaveBeenCalled()
    await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises()
    expect(createBackup).not.toHaveBeenCalled();expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('remote-in',expect.objectContaining({backupBeforeUpdate:true}))
    wrapper.unmount()
  })
  it('uses only the shared header X when the selected range is already the same',async()=>{
    vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'remote',hasRemote:true})
    vi.mocked(modelTransferApi.download).mockResolvedValue({id:'same',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],sync:{source:'remote',changes:[],unchanged:4,createdAt:'2026-11-01T11:02:00Z'}})
    const wrapper=render(true);await flushPromises();await flushPromises();await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('所选内容已一致，无需更新。');expect(wrapper.findAll('label').some(l=>l.text()==='更新前备份')).toBe(false)
    expect(wrapper.findAll('button').find(b=>b.text()==='确认更新')).toBeUndefined()
    await wrapper.find('button[aria-label="关闭"]').trigger('click');await flushPromises()
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(useSyncStore().webdavState).toBe('connected');expect(useSyncStore().observation?.comparison).toBe('same');wrapper.unmount()
  })
  it.each(['unreliable_save_time','same_instant_different_content'])('blocks every write when direction is undetermined: %s',async reason=>{
    vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'undetermined',reason,hasRemote:true,localSavedAt:'2026-11-04T04:00:00Z',remoteSavedAt:reason==='same_instant_different_content'?'2026-11-04T12:00:00+08:00':null})
    const wrapper=render(true);await flushPromises();await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('无法判定新版');expect(wrapper.text()).toContain('本机保存时间')
    expect(wrapper.findAll('button').find(b=>b.text()==='确认更新')!.attributes('disabled')).toBeDefined()
    expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();expect(modelTransferApi.download).not.toHaveBeenCalled();expect(modelTransferApi.publish).not.toHaveBeenCalled();expect(modelTransferApi.accept).not.toHaveBeenCalled()
    expect(useSyncStore().webdavState).toBe('connected')
    await wrapper.find('button[aria-label="关闭"]').trigger('click');await flushPromises()
    expect(wrapper.text()).not.toContain('本机保存时间');wrapper.unmount()
  })
  it('uses the unchanged check directly without staging upload or download',async()=>{
    vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'unchanged',reason:'same_content',hasRemote:true,remoteSavedAt:'2026-11-04T04:00:00Z'})
    const wrapper=render(true);await flushPromises();await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('所选内容已一致，无需更新。');expect(wrapper.findAll('button').find(b=>b.text()==='确认更新')).toBeUndefined()
    expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();expect(modelTransferApi.download).not.toHaveBeenCalled();expect(useSyncStore().webdavState).toBe('connected');expect(useSyncStore().observation?.comparison).toBe('same')
    expect(useSyncStore().observation?.remoteTime).toBe('2026-11-04T04:00:00Z');wrapper.unmount()
  })
  it('asks for the MCP source inside the sync preview and skips an unpicked conflict',async()=>{
    vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'local',hasRemote:true})
    vi.mocked(modelTransferApi.prepareUpload).mockResolvedValue({id:'up',phase:'prepared',package:pack(),sync:{source:'local',changes:[{kind:'mcp',id:'docs',label:'docs',mark:'conflict',clients:['codex','claude'],fields:['env','type']}],unchanged:2,createdAt:'2026-11-01T10:00:00Z'}})
    const wrapper=render(true);await flushPromises();await flushPromises();await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('差异字段：env、type');expect(wrapper.text()).toContain('选择来源')
    const picker=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='docs 采用来源')!
    expect(picker.props('modelValue')).toBe('');expect(picker.props('options')[0]).toEqual({value:'',label:'暂不同步'});wrapper.unmount()
  })
  it('uses the separate content password inside the remote preview without adopting',async()=>{
    vi.mocked(modelTransferApi.checkSync).mockResolvedValue({source:'remote',hasRemote:true});vi.mocked(modelTransferApi.saveRemoteScope).mockImplementation(async scope=>scope)
    vi.mocked(modelTransferApi.download).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue({id:'remote-received',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],sync:{source:'remote',changes:[],unchanged:1,createdAt:''}})
    const wrapper=render(true);await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='同步')!.trigger('click');await flushPromises();await wrapper.get('input[aria-label="导入解密口令"]').setValue('fixture-sync-content-password');await wrapper.findAll('button').find(b=>b.text()==='读取配置')!.trigger('click');await flushPromises()
    expect(modelTransferApi.download).toHaveBeenLastCalledWith('fixture-sync-content-password');expect(modelTransferApi.receive).not.toHaveBeenCalled();expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  // Original upload ID recovery is exercised by sync-result.test.ts through the real result entry.
  it('previews selected assets with no human review gate',async()=>{
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'asset-file',package:pack(),assets:{skills:[{name:'fixture-skill',digest:'skill-digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[]}})
    const wrapper=render();await flushPromises();await customExportScope(wrapper);await chooseExport(wrapper,'Skill · fixture-skill')
    const action=wrapper.findAll('button').find(b=>b.text()==='导出配置')!;expect(action.attributes('disabled')).toBeUndefined();expect(wrapper.text()).not.toContain('已复核所选资源');await action.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.anything(),undefined,false,expect.objectContaining({skillNames:['fixture-skill'],mcpNames:[],extensionCredentials:false}));expect(wrapper.text()).toContain('2 文件');expect(modelTransferApi.export).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('keeps the temporary file password for an encrypted Skill even when no credential was included',async()=>{
    const original={id:'asset-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],protected:true,assets:{skills:[{name:'fixture-skill',digest:'digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[]}}
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue(original);vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='选择文件')!.trigger('click');await flushPromises();await flushPromises();await flushPromises();await wrapper.get('input[aria-label="导入解密口令"]').setValue('fixture-assets-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='读取配置')!.trigger('click');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='确认更新')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('asset-incoming',expect.objectContaining({adoptKeys:false,passphrase:'fixture-assets-password',backupBeforeUpdate:true}));expect(wrapper.findAll('button').filter(b=>b.text()==='读取配置'||b.text()==='选择文件')).toHaveLength(0);wrapper.unmount()
  })
})
