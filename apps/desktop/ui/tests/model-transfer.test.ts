import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import ModelTransfer from '@/features/sync/ModelTransfer.vue'
import { modelTransferApi, scopeFor, remoteScope, type ModelPackage } from '@/features/sync/modelTransfer'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ObjectScopeTree from '@/features/sync/ObjectScopeTree.vue'
import { useModelsStore } from '@/features/models/store'
import { emptyContent, emptyRules, type ChannelPolicy, type ModelSnapshot } from '@/features/models/api'
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn().mockResolvedValue('/fixture/models.json') }))
vi.mock('@/features/sync/modelTransfer', async original => ({ ...await original<typeof import('@/features/sync/modelTransfer')>(), modelTransferApi: {prepareDraft:vi.fn(),prepare:vi.fn(),export:vi.fn(),receive:vi.fn(),accept:vi.fn(),select:vi.fn(),prepareUpload:vi.fn(),publish:vi.fn(),download:vi.fn(),latestIncoming:vi.fn(),recompare:vi.fn(),openReceived:vi.fn(),loadRemoteScope:vi.fn(),saveRemoteScope:vi.fn(),latestUpload:vi.fn(),accountCatalog:vi.fn(),channelCatalog:vi.fn(),extensionCatalog:vi.fn()} }))
function snapshot(): ModelSnapshot {
  return { library:{schemaVersion:1,templates:[{id:'family',label:'测试族',source:'user',defaultRevision:null,revisions:[{id:'revision-1',createdAt:'2026-10-18T00:00:00Z',savedLabel:'测试族',content:emptyContent(),digest:'digest',baseRevision:null,changes:[]}]}]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'policy',workspace:null,workspaceDigest:'workspace' }
}
const pack = (): ModelPackage => ({format:'OCXD_MODEL_CONFIG',formatVersion:1,schemaVersion:1,createdAt:'2026-10-18T00:00:00Z',digest:'frozen-digest',payload:{templates:snapshot().library.templates,policy:null}})
const dialog = defineComponent({ props:['open'], template:'<section v-if="open"><slot/><slot name="actions"/></section>' })
function render(remote = false) { return mount(ModelTransfer, {props:{remote},global:{stubs:{UiDialog:dialog,UiSelectField:true}}}) }
async function chooseExport(wrapper:ReturnType<typeof render>,name:string,selected=true){await flushPromises();const search=wrapper.get('input[aria-label="搜索配置对象"]');await search.setValue(name);await wrapper.get('input[aria-label="选择 '+name+'"]').setValue(selected);await search.setValue('')}

// 原型对象树接收侧默认折叠；操作子项前先展开对应分组。
async function expandGroup(wrapper: ReturnType<typeof render>, label: string) {
  const toggle = wrapper.findAll('button').find(b => b.attributes('aria-label') === '展开 ' + label)
  if (toggle) await toggle.trigger('click')
  await flushPromises()
}
async function customExportScope(wrapper:ReturnType<typeof render>){
 await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='清空选择')!.trigger('click')
 await chooseExport(wrapper,'桌面偏好');await chooseExport(wrapper,'扩展设置与名称清单')
 const global=wrapper.findAll('label').find(l=>l.text()==='全局采用及渠道组顺序');if(global)await global.get('input').setValue(false)
}


beforeEach(() => { setActivePinia(createPinia());vi.clearAllMocks();vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.extensionCatalog).mockResolvedValue({skills:['fixture-skill'],mcps:['fixture-mcp']});vi.mocked(modelTransferApi.loadRemoteScope).mockResolvedValue({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false});useModelsStore().snapshot=snapshot() })
describe('model transfer review', () => {
  it('shows scope inline without preparing or writing on mount, and blocks writes until the saved remote scope is known',async()=>{
    let resolve!:(value:Awaited<ReturnType<typeof modelTransferApi.loadRemoteScope>>)=>void
    vi.mocked(modelTransferApi.loadRemoteScope).mockReturnValue(new Promise(done=>{resolve=done}))
    const wrapper=render(true);await flushPromises()
    expect(wrapper.find('section[aria-label="同步范围"]').exists()).toBe(true)
    expect(wrapper.find('.sync-scope-credentials').exists()).toBe(true)
    const save=()=>wrapper.findAll('button').find(b=>b.text()==='保存本机范围')!
    expect(save().attributes('disabled')).toBeDefined()
    expect(wrapper.get('input[aria-label="导出保护口令"]').attributes('disabled')).toBeDefined()
    expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();expect(modelTransferApi.saveRemoteScope).not.toHaveBeenCalled()
    resolve({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:false,includeKeys:false});await flushPromises()
    expect(save().attributes('disabled')).toBeUndefined();expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled()
    wrapper.unmount()
  })
  it.skip('[0.2.5 原型无此入口] preserves explicit file selections after an initially failed catalogue is recovered, without applying all by surprise',async()=>{
    vi.mocked(modelTransferApi.channelCatalog).mockRejectedValueOnce({message:'目录读取失败'}).mockResolvedValue([])
    const wrapper=render();await flushPromises();await chooseExport(wrapper,'测试族')
    const before=[...wrapper.getComponent(ObjectScopeTree).props('selectedIds')]
    await wrapper.findAll('button').find(b=>b.text()==='重新读取范围目录')!.trigger('click');await flushPromises()
    expect(wrapper.getComponent(ObjectScopeTree).props('selectedIds')).toEqual(before)
    expect(wrapper.text()).not.toContain('目录读取失败');expect(modelTransferApi.prepare).not.toHaveBeenCalled();wrapper.unmount()
  })
  it.skip('[0.2.5 原型无此入口] does not overwrite an unreadable saved WebDAV scope with initial defaults; an explicit reload can recover',async()=>{
    vi.mocked(modelTransferApi.loadRemoteScope).mockRejectedValue({message:'范围读取失败'})
    const wrapper=render(true);await flushPromises()
    const save=()=>wrapper.findAll('button').find(b=>b.text()==='保存本机范围')!
    expect(save().attributes('disabled')).toBeDefined();expect(wrapper.text()).toContain('范围读取失败')
    expect(modelTransferApi.saveRemoteScope).not.toHaveBeenCalled()
    vi.mocked(modelTransferApi.loadRemoteScope).mockResolvedValue({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:false,includeKeys:false})
    await wrapper.findAll('button').find(b=>b.text()==='重新读取范围目录')!.trigger('click');await flushPromises()
    expect(save().attributes('disabled')).toBeUndefined();expect(modelTransferApi.saveRemoteScope).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('selects exact families instead of a model subset, carries shared fields only when checked, and serializes the same WebDAV scope',async()=>{
    const store=useModelsStore();const c:ChannelPolicy={id:'family-channel',label:'族范围渠道',templateRefs:[],modelRefs:{},connection:{providerId:'p',protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:false,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:['raw-a','raw-b'],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}
    store.snapshot!.policy.channels=[c];store.snapshot!.policy.templateRefs=[{templateId:'family',revisionId:'revision-1'}]
    vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([c]);vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'family-file',package:pack()})
    const wrapper=render();await flushPromises();await customExportScope(wrapper);await chooseExport(wrapper,'族范围渠道');await wrapper.get('input[aria-label="搜索配置对象"]').setValue('族范围渠道')
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
    await wrapper.findAll('button').find(b=>b.text()==='保存本机范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.saveRemoteScope).toHaveBeenCalledWith(expect.objectContaining({channelIds:['remote-channel'],channelFamilies:mask,includeGlobal:false,includeKeys:false}))
    expect(modelTransferApi.publish).not.toHaveBeenCalled();expect(wrapper.text()).toContain('尚未上传');wrapper.unmount()
  })
  it('does not offer a received family fragment as whole model or independent channel material',async()=>{
    const source=pack();source.formatVersion=5;source.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'fragment',label:'族片段',familyScope:{templateIds:['family'],includeChannelDefaults:false},modelSubset:[],selection:{mode:'none'}}]}
    vi.mocked(modelTransferApi.receive).mockResolvedValue({id:'received-family',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:source,notices:[]})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises();const expandChannel=wrapper.findAll('button').find(b=>b.attributes('aria-label')==='展开 渠道与模型');if(expandChannel)await expandChannel.trigger('click')
    expect(wrapper.text()).toContain('共用默认字段沿本机值解析');expect(wrapper.text()).toContain('仅合并所选族')
    expect(wrapper.find('input[aria-label="族片段型号范围搜索"]').exists()).toBe(false)
    const mapping=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='族片段接收方式')!
    expect(mapping.props('options')).not.toEqual(expect.arrayContaining([expect.objectContaining({value:'new'})]));wrapper.unmount()
  })

  it.skip('[0.2.5 待按新结构重写] share and device-transfer presets freeze separate protected selections and still require custom asset review',async()=>{
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'preset',package:pack()});const wrapper=render();await flushPromises();await flushPromises();await flushPromises()
    const preset=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='配置范围预设')!
    expect(preset.props('modelValue')).toBe('all');expect(wrapper.text()).toContain('已选 5 个对象')
    preset.vm.$emit('update:modelValue','move');await flushPromises()
    expect((wrapper.findAll('label').find(l=>l.text().includes('渠道 API Key'))!.get('input').element as HTMLInputElement).checked).toBe(true)
    expect(wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.attributes('disabled')).toBeDefined()
    preset.vm.$emit('update:modelValue','share');await flushPromises()
    expect((wrapper.findAll('label').find(l=>l.text().includes('渠道 API Key'))!.get('input').element as HTMLInputElement).checked).toBe(false)
    await wrapper.findAll('label').find(l=>l.text().startsWith('已检查所选Skill'))!.get('input').setValue(true)
    await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.objectContaining({includeGlobal:true}),undefined,false,expect.objectContaining({webdavPasswords:false,extensionCredentials:false,assetsReviewed:true}));wrapper.unmount()
  })
  it('resets a reselected WebDAV object to carrying its password while retaining other explicit choices',async()=>{
    const sync=await import('@/features/sync/api');vi.spyOn(sync,'getSyncConfig').mockResolvedValue({endpoint:{endpointId:'dav',url:'https://dav.example.test',remotePath:'config',username:'fixture',credentialRefId:'cred-fixture',tlsPolicy:'verify_required',payloadProtection:'integrity_only',conflictPolicy:'ask'}})
    const wrapper=render();await flushPromises();await flushPromises();await flushPromises()
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('WebDAV连接配置');await flushPromises();const webdavToggle=wrapper.findAll('button').find(b=>b.attributes('aria-label')==='展开 WebDAV 配置');if(webdavToggle)await webdavToggle.trigger('click');await flushPromises();await wrapper.findAll('label').find(l=>l.text()==='包含连接密码')!.get('input').setValue(false)
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
    await wrapper.get('input[aria-label="原有官方渠道范围内型号 original/other"]').setValue(false);
    await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises();
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.objectContaining({includePolicy:true,channelIds:['official-channel'],channelModels:{'official-channel':['original/model']}}),undefined,false,expect.any(Object));
    wrapper.unmount();
  })

  it.skip('[0.2.5 待按新结构重写] freezes selected extension target names only after comparing again',async()=>{
    const original={id:'asset-conflict',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],previewError:'asset_conflict',assets:{skills:[{name:'fixture-skill',digest:'skill-digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[{name:'fixture-mcp',digest:'mcp-digest',secretFields:[],warnings:[]}]}}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original);vi.mocked(modelTransferApi.select).mockResolvedValue({...original,previewError:null,assetMappings:{skills:{'fixture-skill':'received-skill'},mcps:{'fixture-mcp':'received-mcp'}}})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises()
    const expandSkills=wrapper.findAll('button').find(b=>b.attributes('aria-label')==='展开 Skills');if(expandSkills)await expandSkills.trigger('click');await wrapper.get('button[aria-label="展开 MCP"]').trigger('click')
    await expandGroup(wrapper,'Skills');await wrapper.get('input[aria-label="fixture-skill目标Skill名称"]').setValue('received-skill');await wrapper.get('input[aria-label="fixture-mcp目标MCP名称"]').setValue('received-mcp')
    expect(wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.attributes('disabled')).toBeDefined()
    await wrapper.findAll('button').find(b=>b.text()==='比较所选范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('asset-conflict',expect.objectContaining({assetMappings:{skills:{'fixture-skill':'received-skill'},mcps:{'fixture-mcp':'received-mcp'}}}))
    expect(wrapper.text()).toContain('received-skill（来源：fixture-skill）');expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it.skip('[0.2.5 待按新结构重写] requires a new comparison for channel mapping and sends the chosen independent name',async()=>{
    const value=pack();value.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel-source',label:'来源渠道',selection:{mode:'none'},connection:{nodes:[{id:'node-source',name:'主站',url:'https://source.example.test/v1'}]}}]}
    const original={id:'channel-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:value,notices:[],previewError:'channel_endpoint_conflict'}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original)
    vi.mocked(modelTransferApi.select).mockResolvedValue({...original,previewError:null,channelBindings:{'channel-source':{targetId:'new-channel',keepLocalConnection:false,label:'独立接收'}}})
    const select=defineComponent({props:['modelValue','options','label','disabled'],emits:['update:modelValue'],template:'<label>{{label}}<select :value="modelValue" :disabled="disabled" @change="$emit(\'update:modelValue\',$event.target.value)"><option v-for="item in options" :key="item.value" :value="item.value">{{item.label}}</option></select></label>'})
    const wrapper=mount(ModelTransfer,{global:{stubs:{UiDialog:dialog,UiSelectField:select}}})
    await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises()
    const expandChannel=wrapper.findAll('button').find(b=>b.attributes('aria-label')==='展开 渠道与模型');if(expandChannel)await expandChannel.trigger('click');await wrapper.findAll('label').find(l=>l.text().startsWith('来源渠道接收方式'))!.get('select').setValue('new');await wrapper.findAll('label').find(l=>l.text()==='新渠道名称')!.get('input').setValue('独立接收')
    expect(wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.attributes('disabled')).toBeDefined()
    await wrapper.findAll('button').find(b=>b.text()==='比较所选范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('channel-incoming',expect.objectContaining({channelMappings:{'channel-source':{mode:'new',label:'独立接收'}}}))
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(wrapper.text()).toContain('独立停用草稿');wrapper.unmount()
  })
  it('exports only explicitly selected account information and explains unsupported login migration',async()=>{
    vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([{account:{id:'fixture-account',provider:'chatgpt',subject:'fixture-subject',label:'测试账户',email:'fixture@example.test'},loginTransfer:'manualImportDisabled',localSource:true,informationSupported:true}])
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'account-file',package:pack()})
    const wrapper=render();await flushPromises();expect(modelTransferApi.accountCatalog).toHaveBeenCalledOnce();expect(modelTransferApi.prepare).not.toHaveBeenCalled()
    await flushPromises();await customExportScope(wrapper);await flushPromises()
    await wrapper.get('input[aria-label="搜索配置对象"]').setValue('测试账户');expect(wrapper.text()).toContain('官方不支持手工导入登录态');expect((wrapper.get('input[aria-label="选择 测试账户"]').element as HTMLInputElement).checked).toBe(false)
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
    await chooseExport(wrapper,'工作账户');const filter=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='账户平台筛选')!
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
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises()
    expect(wrapper.text()).toContain('不代表登录已恢复');await wrapper.get('button[aria-label="展开 账户"]').trigger('click');await wrapper.findAll('button').find(b=>b.attributes('aria-label')==='展开 账户')?.trigger('click');await flushPromises();await wrapper.get('input[aria-label="选择 测试账户"]').setValue(false)
    await wrapper.findAll('button').find(b=>b.text()==='比较所选范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('account-incoming',expect.objectContaining({accountIds:[]}));expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('exports the reviewed frozen plan instead of rebuilding from later editor content', async () => {
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'frozen-plan',package:pack()})
    vi.mocked(modelTransferApi.export).mockResolvedValue({path:'/fixture/export.json',digest:'frozen-digest',templates:1,channels:0})
    const wrapper=render()
    await flushPromises();await customExportScope(wrapper)
    await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith({includePolicy:false,channelIds:[],includeGlobal:false,templateRefs:[{templateId:'family',revisionId:'revision-1'}]},undefined,false,{preferences:true,extensions:true,webdav:false,webdavPasswords:true})
    expect(modelTransferApi.export).not.toHaveBeenCalled()
    useModelsStore().snapshot!.library.templates[0]!.label='later'
    await wrapper.findAll('button').find(b=>b.text()==='生成导出文件')!.trigger('click');await flushPromises()
    expect(modelTransferApi.export).toHaveBeenCalledExactlyOnceWith('frozen-plan', undefined)
    expect(wrapper.text()).toContain('/fixture/export.json');wrapper.unmount()
  })
  it('receives only a review and requires adoption while preserving the manual template buffer', async () => {
    const store=useModelsStore();store.edit();store.draft!.label='人工输入'
    vi.mocked(modelTransferApi.receive).mockResolvedValue({id:'incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]})
    vi.mocked(modelTransferApi.accept).mockResolvedValue({...snapshot(),libraryDigest:'received'})
    const wrapper=render()
    await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises()
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(store.snapshot!.libraryDigest).toBe('library')
    await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('incoming');expect(store.snapshot!.libraryDigest).toBe('received')
    expect(store.draft!.label).toBe('人工输入');expect(store.dirty).toBe(true);wrapper.unmount()
  })
  it.skip('[0.2.5 待按新结构重写] retains the original operation ID after uncertain WebDAV publication', async () => {
    vi.mocked(modelTransferApi.prepareUpload).mockResolvedValue({id:'upload-once',phase:'prepared',package:pack()})
    vi.mocked(modelTransferApi.publish).mockRejectedValueOnce({message:'结果待核对'}).mockResolvedValue({id:'upload-once',phase:'published',package:pack()})
    const wrapper=render(true)
    await wrapper.findAll('button').find(b=>b.text()==='重新读取范围目录')!.trigger('click')
    await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='确认上传')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('结果待核对')
    await wrapper.findAll('button').find(b=>b.text()==='确认上传')!.trigger('click');await flushPromises()
    expect(vi.mocked(modelTransferApi.publish).mock.calls).toEqual([['upload-once'],['upload-once']]);wrapper.unmount()
  })
  it('keeps global-only selection from expanding to every channel', () => {
    expect(scopeFor(snapshot(),[],[],true,{})).toEqual({includePolicy:true,channelIds:[],templateRefs:[],includeGlobal:true})
  })
  it.skip('[0.2.5 原型无此入口] requires another comparison after choosing an import subset and adopts only that reviewed plan',async()=>{
    const incoming={id:'original',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(incoming);vi.mocked(modelTransferApi.select).mockResolvedValue({...incoming,id:'selected-plan'});vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises()
    await wrapper.get('.model-transfer-scope input[type=checkbox]').setValue(false);expect(wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.attributes('disabled')).toBeDefined()
    await wrapper.get('.model-transfer-scope input[type=checkbox]').setValue(true);await wrapper.findAll('button').find(b=>b.text()==='比较所选范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('original',{templateRefs:[{templateId:'family',revisionId:'revision-1'}],channelIds:[],includeGlobal:false});await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('selected-plan');wrapper.unmount()
  })
  it('asks once for the protected file password and never adopts on decryption failure',async()=>{
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required',message:'请输入保护口令'}).mockRejectedValueOnce({code:'portable_authentication',message:'口令不正确或文件已损坏'}).mockResolvedValue({id:'opened',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises();await flushPromises()
    const input=wrapper.get('input[aria-label="导入解密口令"]');await input.setValue('fixture-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='解密并比较'||b.text()==='直接读取并比较')!.trigger('click');await flushPromises();expect(wrapper.text()).toContain('口令不正确');expect(modelTransferApi.accept).not.toHaveBeenCalled()
    await wrapper.get('input[aria-label="导入解密口令"]').setValue('correct-fixture-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='解密并比较'||b.text()==='直接读取并比较')!.trigger('click');await flushPromises();expect(modelTransferApi.receive).toHaveBeenLastCalledWith('/fixture/models.json','correct-fixture-password');expect(wrapper.find('input[aria-label="导入解密口令"]').exists()).toBe(false);expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('keeps channel Key adoption off until explicitly selected and passes the same file password',async()=>{
    const packageValue=pack();packageValue.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel',label:'测试渠道',selection:{mode:'none'}}]}
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue({id:'protected-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:packageValue,notices:[],availableKeys:['channel']})
    vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises();await wrapper.get('input[aria-label="导入解密口令"]').setValue('fixture-key-file-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='解密并比较'||b.text()==='直接读取并比较')!.trigger('click');await flushPromises()
    const toggle=wrapper.findAll('label').find(label=>label.text().startsWith('采用所选渠道的Key'))!.get<HTMLInputElement>('input');expect(toggle.element.checked).toBe(false);expect(modelTransferApi.accept).not.toHaveBeenCalled()
    await toggle.setValue(true);await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledWith('protected-incoming',{adoptKeys:true,passphrase:'fixture-key-file-password'});wrapper.unmount()
  })
  it.skip('[0.2.5 原型无此入口] restores the frozen import identity and exact credential intent without silently creating a new comparison',async()=>{
    const packageValue=pack();packageValue.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel',label:'测试渠道',selection:{mode:'none'}}]}
    const original={id:'original-adoption',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:packageValue,notices:[],availableKeys:['channel'],adoption:{keys:true,webdav:false,completed:false}}
    vi.mocked(modelTransferApi.latestIncoming).mockResolvedValue(original);vi.mocked(modelTransferApi.openReceived).mockResolvedValue(original);vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='恢复上次接收')!.trigger('click');await flushPromises();expect(modelTransferApi.recompare).not.toHaveBeenCalled();expect(wrapper.text()).toContain('原导入采用范围已冻结')
    await wrapper.findAll('button').find(b=>b.text()==='核对原导入')!.trigger('click');await wrapper.get('input[aria-label="导入解密口令"]').setValue('fixture-original-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='解密并比较'||b.text()==='直接读取并比较')!.trigger('click');await flushPromises()
    expect(modelTransferApi.openReceived).toHaveBeenCalledExactlyOnceWith('original-adoption','fixture-original-password');await wrapper.findAll('button').find(b=>b.text()==='核对原导入')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledWith('original-adoption',{adoptKeys:true,passphrase:'fixture-original-password'});expect(modelTransferApi.recompare).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('saves a device scope without starting transport or applying remote objects',async()=>{
    vi.mocked(modelTransferApi.saveRemoteScope).mockResolvedValue({templateIds:['family'],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false})
    const wrapper=render(true);await flushPromises();await flushPromises();await chooseExport(wrapper,'测试族');await wrapper.findAll('button').find(b=>b.text()==='保存本机范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.saveRemoteScope).toHaveBeenCalledExactlyOnceWith({templateIds:['family'],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false});expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();expect(modelTransferApi.publish).not.toHaveBeenCalled();expect(modelTransferApi.download).not.toHaveBeenCalled();expect(wrapper.text()).toContain('尚未上传');wrapper.unmount()
  })
  it.skip('[0.2.5 待按新结构重写] uses the separate content password for a protected remote download and leaves adoption pending',async()=>{
    vi.mocked(modelTransferApi.download).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue({id:'remote-received',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]})
    const wrapper=render(true);await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='下载并比较')!.trigger('click');await flushPromises();await wrapper.get('input[aria-label="导入解密口令"]').setValue('fixture-sync-content-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='解密并比较'||b.text()==='直接读取并比较')!.trigger('click');await flushPromises()
    expect(modelTransferApi.download).toHaveBeenLastCalledWith('fixture-sync-content-password');expect(modelTransferApi.receive).not.toHaveBeenCalled();expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(wrapper.findAll('button').some(b=>b.text()==='采用到本机')).toBe(true);wrapper.unmount()
  })
  it.skip('[0.2.5 原型无此入口] restores the persisted upload and queries its original publication instead of preparing another plan',async()=>{
    vi.mocked(modelTransferApi.latestUpload).mockResolvedValue({id:'original-remote-plan',phase:'publishing',package:pack()});vi.mocked(modelTransferApi.publish).mockResolvedValue({id:'original-remote-plan',phase:'published',package:pack()})
    const wrapper=render(true);await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='查看原上传计划')!.trigger('click');await flushPromises();expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();await wrapper.findAll('button').find(b=>b.text()==='核对原上传')!.trigger('click');await flushPromises();expect(modelTransferApi.publish).toHaveBeenCalledExactlyOnceWith('original-remote-plan');expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();wrapper.unmount()
  })
  it.skip('[0.2.5 待按新结构重写] requires review of chosen custom assets and freezes their separate credential choice into the file plan',async()=>{
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'asset-file',package:pack(),protected:false,credentialCount:0,assets:{skills:[{name:'fixture-skill',digest:'skill-digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[]}})
    const wrapper=render();await flushPromises();await flushPromises();await customExportScope(wrapper);const labels=()=>wrapper.findAll('label');await chooseExport(wrapper,'Skill · fixture-skill')
    expect(wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.attributes('disabled')).toBeDefined();await labels().find(label=>label.text().startsWith('已检查所选Skill'))!.get('input').setValue(true);await wrapper.findAll('button').find(b=>b.text()===(wrapper.props('remote')?'同步配置':'导出配置'))!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.anything(),undefined,false,expect.objectContaining({skillNames:['fixture-skill'],mcpNames:[],extensionCredentials:false,assetsReviewed:true}));expect(wrapper.text()).toContain('2 文件');expect(modelTransferApi.export).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('keeps the temporary file password for an encrypted Skill even when no credential was included',async()=>{
    const original={id:'asset-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],protected:true,assets:{skills:[{name:'fixture-skill',digest:'digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[]}}
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue(original);vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await flushPromises();await wrapper.get('input[aria-label="导入解密口令"]').setValue('fixture-assets-password');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='解密并比较'||b.text()==='直接读取并比较')!.trigger('click');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('asset-incoming',{adoptKeys:false,passphrase:'fixture-assets-password'});wrapper.unmount()
  })
})
