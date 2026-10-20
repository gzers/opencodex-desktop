import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import ModelTransfer from '@/features/sync/ModelTransfer.vue'
import { modelTransferApi, scopeFor, type ModelPackage } from '@/features/sync/modelTransfer'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import { useModelsStore } from '@/features/models/store'
import { emptyContent, type ModelSnapshot } from '@/features/models/api'
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn().mockResolvedValue('/fixture/models.json') }))
vi.mock('@/features/sync/modelTransfer', async original => ({ ...await original<typeof import('@/features/sync/modelTransfer')>(), modelTransferApi: {prepare:vi.fn(),export:vi.fn(),receive:vi.fn(),accept:vi.fn(),select:vi.fn(),prepareUpload:vi.fn(),publish:vi.fn(),download:vi.fn(),latestIncoming:vi.fn(),recompare:vi.fn(),openReceived:vi.fn(),loadRemoteScope:vi.fn(),saveRemoteScope:vi.fn(),latestUpload:vi.fn(),accountCatalog:vi.fn(),channelCatalog:vi.fn(),extensionCatalog:vi.fn()} }))
function snapshot(): ModelSnapshot {
  return { library:{schemaVersion:1,templates:[{id:'family',label:'测试族',source:'user',defaultRevision:null,revisions:[{id:'revision-1',createdAt:'2026-10-18T00:00:00Z',savedLabel:'测试族',content:emptyContent(),digest:'digest',baseRevision:null,changes:[]}]}]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'policy',workspace:null,workspaceDigest:'workspace' }
}
const pack = (): ModelPackage => ({format:'OCXD_MODEL_CONFIG',formatVersion:1,schemaVersion:1,createdAt:'2026-10-18T00:00:00Z',digest:'frozen-digest',payload:{templates:snapshot().library.templates,policy:null}})
const dialog = defineComponent({ props:['open'], template:'<section v-if="open"><slot/><slot name="actions"/></section>' })
function render(remote = false) { return mount(ModelTransfer, {props:{remote},global:{stubs:{UiDialog:dialog,UiSelectField:true}}}) }
async function chooseExport(wrapper:ReturnType<typeof render>,name:string,selected=true){await flushPromises();const search=wrapper.get('input[aria-label="导出配置对象搜索"]');await search.setValue(name);await wrapper.get('input[aria-label="选择 '+name+'"]').setValue(selected);await search.setValue('')}
async function customExportScope(wrapper:ReturnType<typeof render>){
 await flushPromises();await wrapper.get('.scope-tree-toolbar input[type=checkbox]').setValue(false)
 await chooseExport(wrapper,'桌面偏好');await chooseExport(wrapper,'扩展设置与名称清单')
 const global=wrapper.findAll('label').find(l=>l.text()==='全局采用及渠道组顺序');if(global)await global.get('input').setValue(false)
}


beforeEach(() => { setActivePinia(createPinia());vi.clearAllMocks();vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([]);vi.mocked(modelTransferApi.extensionCatalog).mockResolvedValue({skills:['fixture-skill'],mcps:['fixture-mcp']});vi.mocked(modelTransferApi.loadRemoteScope).mockResolvedValue({templateIds:[],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false});useModelsStore().snapshot=snapshot() })
describe('model transfer review', () => {
  it('share and device-transfer presets freeze separate protected selections and still require custom asset review',async()=>{
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'preset',package:pack()});const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises()
    const preset=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='配置范围预设')!
    expect(preset.props('modelValue')).toBe('all');expect(wrapper.text()).toContain('已选 5 个对象')
    preset.vm.$emit('update:modelValue','move');await flushPromises()
    expect((wrapper.findAll('label').find(l=>l.text().includes('携带所选渠道的Key池'))!.get('input').element as HTMLInputElement).checked).toBe(true)
    expect(wrapper.findAll('button').find(b=>b.text()==='预览清单')!.attributes('disabled')).toBeDefined()
    preset.vm.$emit('update:modelValue','share');await flushPromises()
    expect((wrapper.findAll('label').find(l=>l.text().includes('携带所选渠道的Key池'))!.get('input').element as HTMLInputElement).checked).toBe(false)
    await wrapper.findAll('label').find(l=>l.text().startsWith('已检查所选Skill'))!.get('input').setValue(true)
    await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.objectContaining({includeGlobal:true}),undefined,false,expect.objectContaining({webdavPasswords:false,extensionCredentials:false,assetsReviewed:true}));wrapper.unmount()
  })
  it('resets a reselected WebDAV object to carrying its password while retaining other explicit choices',async()=>{
    const sync=await import('@/features/sync/api');vi.spyOn(sync,'getSyncConfig').mockResolvedValue({endpoint:{endpointId:'dav',url:'https://dav.example.test',remotePath:'config',username:'fixture',credentialRefId:'cred-fixture',tlsPolicy:'verify_required',payloadProtection:'integrity_only',conflictPolicy:'ask'}})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await flushPromises()
    await wrapper.get('input[aria-label="导出配置对象搜索"]').setValue('WebDAV连接配置');await wrapper.findAll('label').find(l=>l.text()==='包含连接密码')!.get('input').setValue(false)
    await chooseExport(wrapper,'WebDAV连接配置',false);await chooseExport(wrapper,'WebDAV连接配置',true);await wrapper.get('input[aria-label="导出配置对象搜索"]').setValue('WebDAV连接配置')
    expect((wrapper.findAll('label').find(l=>l.text()==='包含连接密码')!.get('input').element as HTMLInputElement).checked).toBe(true);wrapper.unmount();vi.mocked(sync.getSyncConfig).mockRestore()
  })
  it('lists saved original channels independently of the Desktop policy and exports their exact selected identity',async()=>{
    const original={id:'official-channel',label:'原有官方渠道',templateRefs:[],modelRefs:{},connection:{providerId:'official-source',protocol:'openai-chat',nodes:[{id:'service',name:'主站',url:'https://official.example.test'}],activeNodeId:'service',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist' as const,models:['original/model']},modelOrder:['original/model','original/other'],discovered:[{id:'original/model',label:'模型A',rules:{}},{id:'original/other',label:'模型B',rules:{}}],defaults:{},overrides:{},familyOverrides:{}};
    vi.mocked(modelTransferApi.channelCatalog).mockResolvedValue([original as never]);
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'original-export',package:pack()});
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await customExportScope(wrapper);await flushPromises();
    expect(useModelsStore().snapshot!.policy.channels).toHaveLength(0);
    await chooseExport(wrapper,'原有官方渠道');
    await wrapper.get('input[aria-label="导出配置对象搜索"]').setValue('原有官方渠道');
    await wrapper.get('input[aria-label="原有官方渠道范围内型号 original/other"]').setValue(false);
    await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises();
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.objectContaining({includePolicy:true,channelIds:['official-channel'],channelModels:{'official-channel':['original/model']}}),undefined,false,expect.any(Object));
    wrapper.unmount();
  })

  it('freezes selected extension target names only after comparing again',async()=>{
    const original={id:'asset-conflict',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],previewError:'asset_conflict',assets:{skills:[{name:'fixture-skill',digest:'skill-digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[{name:'fixture-mcp',digest:'mcp-digest',secretFields:[],warnings:[]}]}}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original);vi.mocked(modelTransferApi.select).mockResolvedValue({...original,previewError:null,assetMappings:{skills:{'fixture-skill':'received-skill'},mcps:{'fixture-mcp':'received-mcp'}}})
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises()
    await wrapper.get('button[aria-label="展开 Skills"]').trigger('click');await wrapper.get('button[aria-label="展开 MCP"]').trigger('click')
    await wrapper.get('input[aria-label="fixture-skill目标Skill名称"]').setValue('received-skill');await wrapper.get('input[aria-label="fixture-mcp目标MCP名称"]').setValue('received-mcp')
    expect(wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.attributes('disabled')).toBeDefined()
    await wrapper.findAll('button').find(b=>b.text()==='比较所选范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('asset-conflict',expect.objectContaining({assetMappings:{skills:{'fixture-skill':'received-skill'},mcps:{'fixture-mcp':'received-mcp'}}}))
    expect(wrapper.text()).toContain('received-skill（来源：fixture-skill）');expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('requires a new comparison for channel mapping and sends the chosen independent name',async()=>{
    const value=pack();value.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel-source',label:'来源渠道',selection:{mode:'none'},connection:{nodes:[{id:'node-source',name:'主站',url:'https://source.example.test/v1'}]}}]}
    const original={id:'channel-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:value,notices:[],previewError:'channel_endpoint_conflict'}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original)
    vi.mocked(modelTransferApi.select).mockResolvedValue({...original,previewError:null,channelBindings:{'channel-source':{targetId:'new-channel',keepLocalConnection:false,label:'独立接收'}}})
    const select=defineComponent({props:['modelValue','options','label','disabled'],emits:['update:modelValue'],template:'<label>{{label}}<select :value="modelValue" :disabled="disabled" @change="$emit(\'update:modelValue\',$event.target.value)"><option v-for="item in options" :key="item.value" :value="item.value">{{item.label}}</option></select></label>'})
    const wrapper=mount(ModelTransfer,{global:{stubs:{UiDialog:dialog,UiSelectField:select}}})
    await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises()
    await wrapper.get('button[aria-label="展开 渠道与模型"]').trigger('click');await wrapper.findAll('label').find(l=>l.text().startsWith('来源渠道接收方式'))!.get('select').setValue('new');await wrapper.findAll('label').find(l=>l.text()==='新渠道名称')!.get('input').setValue('独立接收')
    expect(wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.attributes('disabled')).toBeDefined()
    await wrapper.findAll('button').find(b=>b.text()==='比较所选范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('channel-incoming',expect.objectContaining({channelMappings:{'channel-source':{mode:'new',label:'独立接收'}}}))
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(wrapper.text()).toContain('独立停用草稿');wrapper.unmount()
  })
  it('exports only explicitly selected account information and explains unsupported login migration',async()=>{
    vi.mocked(modelTransferApi.accountCatalog).mockResolvedValue([{account:{id:'fixture-account',provider:'chatgpt',subject:'fixture-subject',label:'测试账户',email:'fixture@example.test'},loginTransfer:'manualImportDisabled',localSource:true,informationSupported:true}])
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'account-file',package:pack()})
    const wrapper=render();await flushPromises();expect(modelTransferApi.accountCatalog).not.toHaveBeenCalled()
    await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await customExportScope(wrapper);await flushPromises()
    await wrapper.get('input[aria-label="导出配置对象搜索"]').setValue('测试账户');expect(wrapper.text()).toContain('官方不支持手工导入登录态');expect((wrapper.get('input[aria-label="选择 测试账户"]').element as HTMLInputElement).checked).toBe(false)
    await chooseExport(wrapper,'测试账户');await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises()
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
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await customExportScope(wrapper)
    await chooseExport(wrapper,'工作账户');const filter=wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')==='账户平台筛选')!
    filter.vm.$emit('update:modelValue','antigravity');await flushPromises();expect(wrapper.text()).toContain('筛选外仍有')
    filter.vm.$emit('update:modelValue','chatgpt');await flushPromises();await wrapper.get('input[aria-label="导出配置对象搜索"]').setValue('工作账户')
    expect((wrapper.get('input[aria-label="选择 工作账户"]').element as HTMLInputElement).checked).toBe(true)
    await wrapper.get('input[aria-label="导出配置对象搜索"]').setValue('未支持来源');expect((wrapper.get('input[aria-label="选择 未支持来源"]').element as HTMLInputElement).disabled).toBe(true)
    await wrapper.get('input[aria-label="导出配置对象搜索"]').setValue('');await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.anything(),undefined,false,expect.objectContaining({accountIds:['work']}));wrapper.unmount()
  })
  it('lets incoming account information be excluded without touching credential adoption',async()=>{
    const original={id:'account-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],manager:{preferences:null,extensions:null,webdav:[],accounts:[{id:'a',provider:'chatgpt',subject:'subject',label:'测试账户',email:null}]}}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(original)
    vi.mocked(modelTransferApi.select).mockResolvedValue({...original,manager:{...original.manager,accounts:[]}})
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('不代表登录已恢复');await wrapper.get('button[aria-label="展开 账户信息"]').trigger('click');await wrapper.get('input[aria-label="选择 测试账户"]').setValue(false)
    await wrapper.findAll('button').find(b=>b.text()==='比较所选范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('account-incoming',expect.objectContaining({accountIds:[]}));expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('exports the reviewed frozen plan instead of rebuilding from later editor content', async () => {
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'frozen-plan',package:pack()})
    vi.mocked(modelTransferApi.export).mockResolvedValue({path:'/fixture/export.json',digest:'frozen-digest',templates:1,channels:0})
    const wrapper=render()
    await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await customExportScope(wrapper)
    await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith({includePolicy:false,channelIds:[],includeGlobal:false,templateRefs:[{templateId:'family',revisionId:'revision-1'}]},undefined,false,{preferences:true,extensions:true,webdav:false,webdavPasswords:true})
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
    await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(store.snapshot!.libraryDigest).toBe('library')
    await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('incoming');expect(store.snapshot!.libraryDigest).toBe('received')
    expect(store.draft!.label).toBe('人工输入');expect(store.dirty).toBe(true);wrapper.unmount()
  })
  it('retains the original operation ID after uncertain WebDAV publication', async () => {
    vi.mocked(modelTransferApi.prepareUpload).mockResolvedValue({id:'upload-once',phase:'prepared',package:pack()})
    vi.mocked(modelTransferApi.publish).mockRejectedValueOnce({message:'结果待核对'}).mockResolvedValue({id:'upload-once',phase:'published',package:pack()})
    const wrapper=render(true)
    await wrapper.findAll('button').find(b=>b.text()==='选择本设备范围')!.trigger('click')
    await chooseExport(wrapper,'测试族')
    await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises()
    await wrapper.findAll('button').find(b=>b.text()==='确认上传')!.trigger('click');await flushPromises()
    expect(wrapper.text()).toContain('结果待核对')
    await wrapper.findAll('button').find(b=>b.text()==='确认上传')!.trigger('click');await flushPromises()
    expect(vi.mocked(modelTransferApi.publish).mock.calls).toEqual([['upload-once'],['upload-once']]);wrapper.unmount()
  })
  it('keeps global-only selection from expanding to every channel', () => {
    expect(scopeFor(snapshot(),[],[],true,{})).toEqual({includePolicy:true,channelIds:[],templateRefs:[],includeGlobal:true})
  })
  it('requires another comparison after choosing an import subset and adopts only that reviewed plan',async()=>{
    const incoming={id:'original',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]}
    vi.mocked(modelTransferApi.receive).mockResolvedValue(incoming);vi.mocked(modelTransferApi.select).mockResolvedValue({...incoming,id:'selected-plan'});vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises()
    await wrapper.get('.model-transfer-scope input[type=checkbox]').setValue(false);expect(wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.attributes('disabled')).toBeDefined()
    await wrapper.get('.model-transfer-scope input[type=checkbox]').setValue(true);await wrapper.findAll('button').find(b=>b.text()==='比较所选范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.select).toHaveBeenCalledWith('original',{templateRefs:[{templateId:'family',revisionId:'revision-1'}],channelIds:[],includeGlobal:false});await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('selected-plan');wrapper.unmount()
  })
  it('asks once for the protected file password and never adopts on decryption failure',async()=>{
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required',message:'请输入保护口令'}).mockRejectedValueOnce({code:'portable_authentication',message:'口令不正确或文件已损坏'}).mockResolvedValue({id:'opened',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]})
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises()
    const input=wrapper.get('input[type=password]');await input.setValue('fixture-password');await wrapper.findAll('button').find(b=>b.text()==='解密并比较')!.trigger('click');await flushPromises();expect(wrapper.text()).toContain('口令不正确');expect(modelTransferApi.accept).not.toHaveBeenCalled()
    await wrapper.get('input[type=password]').setValue('correct-fixture-password');await wrapper.findAll('button').find(b=>b.text()==='解密并比较')!.trigger('click');await flushPromises();expect(modelTransferApi.receive).toHaveBeenLastCalledWith('/fixture/models.json','correct-fixture-password');expect(wrapper.find('input[type=password]').exists()).toBe(false);expect(modelTransferApi.accept).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('keeps channel Key adoption off until explicitly selected and passes the same file password',async()=>{
    const packageValue=pack();packageValue.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel',label:'测试渠道',selection:{mode:'none'}}]}
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue({id:'protected-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:packageValue,notices:[],availableKeys:['channel']})
    vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await wrapper.get('input[type=password]').setValue('fixture-key-file-password');await wrapper.findAll('button').find(b=>b.text()==='解密并比较')!.trigger('click');await flushPromises()
    const toggle=wrapper.findAll('label').find(label=>label.text().startsWith('采用所选渠道的Key'))!.get<HTMLInputElement>('input');expect(toggle.element.checked).toBe(false);expect(modelTransferApi.accept).not.toHaveBeenCalled()
    await toggle.setValue(true);await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledWith('protected-incoming',{adoptKeys:true,passphrase:'fixture-key-file-password'});wrapper.unmount()
  })
  it('restores the frozen import identity and exact credential intent without silently creating a new comparison',async()=>{
    const packageValue=pack();packageValue.payload.policy={templateRefs:null,channelOrder:null,channels:[{id:'channel',label:'测试渠道',selection:{mode:'none'}}]}
    const original={id:'original-adoption',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:packageValue,notices:[],availableKeys:['channel'],adoption:{keys:true,webdav:false,completed:false}}
    vi.mocked(modelTransferApi.latestIncoming).mockResolvedValue(original);vi.mocked(modelTransferApi.openReceived).mockResolvedValue(original);vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='恢复上次接收')!.trigger('click');await flushPromises();expect(modelTransferApi.recompare).not.toHaveBeenCalled();expect(wrapper.text()).toContain('原导入采用范围已冻结')
    await wrapper.findAll('button').find(b=>b.text()==='核对原导入')!.trigger('click');await wrapper.get('input[type=password]').setValue('fixture-original-password');await wrapper.findAll('button').find(b=>b.text()==='解密并比较')!.trigger('click');await flushPromises()
    expect(modelTransferApi.openReceived).toHaveBeenCalledExactlyOnceWith('original-adoption','fixture-original-password');await wrapper.findAll('button').find(b=>b.text()==='核对原导入')!.trigger('click');await flushPromises();expect(modelTransferApi.accept).toHaveBeenCalledWith('original-adoption',{adoptKeys:true,passphrase:'fixture-original-password'});expect(modelTransferApi.recompare).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('saves a device scope without starting transport or applying remote objects',async()=>{
    vi.mocked(modelTransferApi.saveRemoteScope).mockResolvedValue({templateIds:['family'],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false})
    const wrapper=render(true);await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='选择本设备范围')!.trigger('click');await chooseExport(wrapper,'测试族');await wrapper.findAll('button').find(b=>b.text()==='仅保存范围')!.trigger('click');await flushPromises()
    expect(modelTransferApi.saveRemoteScope).toHaveBeenCalledExactlyOnceWith({templateIds:['family'],channelIds:[],includeGlobal:false,preferences:true,extensions:true,includeKeys:false});expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();expect(modelTransferApi.publish).not.toHaveBeenCalled();expect(modelTransferApi.download).not.toHaveBeenCalled();expect(wrapper.text()).toContain('尚未上传');wrapper.unmount()
  })
  it('uses the separate content password for a protected remote download and leaves adoption pending',async()=>{
    vi.mocked(modelTransferApi.download).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue({id:'remote-received',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[]})
    const wrapper=render(true);await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='下载并比较')!.trigger('click');await flushPromises();await wrapper.get('input[type=password]').setValue('fixture-sync-content-password');await wrapper.findAll('button').find(b=>b.text()==='解密并比较')!.trigger('click');await flushPromises()
    expect(modelTransferApi.download).toHaveBeenLastCalledWith('fixture-sync-content-password');expect(modelTransferApi.receive).not.toHaveBeenCalled();expect(modelTransferApi.accept).not.toHaveBeenCalled();expect(wrapper.findAll('button').some(b=>b.text()==='采用到本机')).toBe(true);wrapper.unmount()
  })
  it('restores the persisted upload and queries its original publication instead of preparing another plan',async()=>{
    vi.mocked(modelTransferApi.latestUpload).mockResolvedValue({id:'original-remote-plan',phase:'publishing',package:pack()});vi.mocked(modelTransferApi.publish).mockResolvedValue({id:'original-remote-plan',phase:'published',package:pack()})
    const wrapper=render(true);await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='查看原上传计划')!.trigger('click');await flushPromises();expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();await wrapper.findAll('button').find(b=>b.text()==='核对原上传')!.trigger('click');await flushPromises();expect(modelTransferApi.publish).toHaveBeenCalledExactlyOnceWith('original-remote-plan');expect(modelTransferApi.prepareUpload).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('requires review of chosen custom assets and freezes their separate credential choice into the file plan',async()=>{
    vi.mocked(modelTransferApi.prepare).mockResolvedValue({id:'asset-file',package:pack(),protected:false,credentialCount:0,assets:{skills:[{name:'fixture-skill',digest:'skill-digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[]}})
    const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='导出配置')!.trigger('click');await customExportScope(wrapper);const labels=()=>wrapper.findAll('label');await chooseExport(wrapper,'Skill · fixture-skill')
    expect(wrapper.findAll('button').find(b=>b.text()==='预览清单')!.attributes('disabled')).toBeDefined();await labels().find(label=>label.text().startsWith('已检查所选Skill'))!.get('input').setValue(true);await wrapper.findAll('button').find(b=>b.text()==='预览清单')!.trigger('click');await flushPromises()
    expect(modelTransferApi.prepare).toHaveBeenCalledWith(expect.anything(),undefined,false,expect.objectContaining({skillNames:['fixture-skill'],mcpNames:[],extensionCredentials:false,assetsReviewed:true}));expect(wrapper.text()).toContain('2 文件');expect(modelTransferApi.export).not.toHaveBeenCalled();wrapper.unmount()
  })
  it('keeps the temporary file password for an encrypted Skill even when no credential was included',async()=>{
    const original={id:'asset-incoming',expectedLibrary:'library',expectedPolicy:'policy',expectedWorkspace:'workspace',package:pack(),notices:[],protected:true,assets:{skills:[{name:'fixture-skill',digest:'digest',files:2,bytes:100,warnings:[],protected:false}],mcps:[]}}
    vi.mocked(modelTransferApi.receive).mockRejectedValueOnce({code:'passphrase_required'}).mockResolvedValue(original);vi.mocked(modelTransferApi.accept).mockResolvedValue(snapshot())
    const wrapper=render();await wrapper.findAll('button').find(b=>b.text()==='导入配置')!.trigger('click');await flushPromises();await wrapper.get('input[type=password]').setValue('fixture-assets-password');await wrapper.findAll('button').find(b=>b.text()==='解密并比较')!.trigger('click');await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='采用到本机')!.trigger('click');await flushPromises()
    expect(modelTransferApi.accept).toHaveBeenCalledExactlyOnceWith('asset-incoming',{adoptKeys:false,passphrase:'fixture-assets-password'});wrapper.unmount()
  })
})
