<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref,watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ObjectScopeTree,{type ScopeObject} from './ObjectScopeTree.vue'
import ChannelModelScope from './ChannelModelScope.vue'
import { useModelsStore } from '@/features/models/store'
import {useAppStore} from '@/stores/app'
import { errorText, versionLabel, type ChannelPolicy, type ModelPolicy } from '@/features/models/api'
import { modelTransferApi, scopeFor,remoteScope, accountInformationLabel, accountInformationSupported, accountTransferLabel, accountPlatforms, type AccountCatalogEntry, type ChannelImportMapping, type DraftCandidate, type FamilyScope, type FilePlan, type IncomingPlan, type ManagerScope, type RemoteOptions, type TransferScope, type UploadPlan } from './modelTransfer'
import {getSyncConfig} from './api'
import { displayPath } from '@/lib/paths'
const props = defineProps<{ remote?: boolean; importOnly?: boolean; importOpen?: boolean }>()
const emit = defineEmits<{ 'update:importOpen': [value: boolean] }>()
const pickOpen = ref(false)
const models = useModelsStore()
const app=useAppStore()
const busy = ref(false)
const error = ref('')
const templateIds = ref<string[]>([])
const channelIds = ref<string[]>([])
const channelFamilies=ref<Record<string,FamilyScope>>({});
const channelModels=ref<Record<string,string[]>>({});const receiveChannelModels=ref<Record<string,string[]>>({})
const channelCatalog=ref<ChannelPolicy[]|null>(null);const channelError=ref('')
const savedChannels=computed(()=>exportSource.value==='draft'?(models.channelDraft?.channels??[]):channelCatalog.value??models.snapshot?.policy.channels??[])
const versions = ref<Record<string,string>>({})
const includeGlobal = ref(false)
const selectionOpen = ref(true)
const filePlan = ref<FilePlan | null>(null)
const upload = ref<UploadPlan | null>(null)
const incoming = ref<IncomingPlan | null>(null)
const incomingSource=ref<IncomingPlan|null>(null)
const receiveRefs=ref<string[]>([]);const receiveChannels=ref<string[]>([]);const receiveGlobal=ref(false);const receiveDirty=ref(false)
/** 生成位置预览：与后端 exports/<日期>/<单次包>/ 结构一致，仅供确认，不参与写入。 */
const exportTreeText = computed(() => {
  const root = destinationMode.value === 'custom' ? (displayPath(exportTargetDir.value) || '自定义目录') : '默认导出目录'
  const today = new Date().toLocaleDateString('sv-SE')
  return [
    root + '/',
    '└── ' + today + '/',
    '    └── 备份包-时刻-短码/',
    '        ├── 本次所选配置',
    '        ├── 模版与所选版本',
    '        └── manifest / 校验信息',
  ].join('\n')
})
const exportedPath = ref('')
const exportSource=ref('saved')
const importPassword=ref('');const passwordOpen=ref(false);const importPath=ref('')
const recompareId=ref('');const openingOriginal=ref(false)
const remoteUnlock=ref(false);const scopeSaved=ref(false);const remoteScopeLoaded=ref(!props.remote);const scopeReadError=ref('')
const exportKeys=ref(false);const adoptKeys=ref(false);let unlockedPassword=''
const managerPreferences=ref(true);const managerExtensions=ref(true);const managerWebdav=ref(true);const webdavPasswords=ref(true);const activeWebdav=ref(false)
const receivePreferences=ref(false);const receiveExtensions=ref(false);const receiveEndpoints=ref<string[]>([]);const adoptWebdavPasswords=ref(true)
const skillNames=ref<string[]>([]);const mcpNames=ref<string[]>([]);const extensionCredentials=ref(false);const assetsReviewed=ref(false);const assetCatalog=ref<{skills:string[];mcps:string[]}>({skills:[],mcps:[]});const assetError=ref('')
const receiveMappings=ref<Record<string,string>>({});const receiveNewLabels=ref<Record<string,string>>({})
const localChannels=computed(()=>models.channelDraft?.channels??models.snapshot?.policy.channels??[])
const receiveSkillTargets=ref<Record<string,string>>({});const receiveMcpTargets=ref<Record<string,string>>({})
const receiveSkills=ref<string[]>([]);const receiveMcps=ref<string[]>([]);const adoptExtensionCredentials=ref(false)
const accountPlatform=ref('all');const accountIds=ref<string[]>([]);const accountCatalog=ref<AccountCatalogEntry[]>([]);const accountError=ref('');const receiveAccounts=ref<string[]>([])
function scopeAccounts(id:string){const value=accountCatalog.value.find(a=>a.account.id===JSON.parse(id)[1])?.account;return value?[value]:[]}
const allTemplates=ref(false);const allChannels=ref(false);const allSkills=ref(false);const allMcps=ref(false)
const hasAssets=computed(()=>skillNames.value.length>0||mcpNames.value.length>0||props.remote&&(allSkills.value||allMcps.value))
const keyChannelIds=computed(()=>(allChannels.value?savedChannels.value.map(c=>c.id):channelIds.value).filter(id=>!channelFamilies.value[id]));
watch(()=>keyChannelIds.value.length,length=>{if(!length)exportKeys.value=false})
onBeforeUnmount(()=>{exportPackagePassword.value='';exportPackagePasswordAgain.value='';importPassword.value='';unlockedPassword=''})
const hasScope = computed(() => accountIds.value.length>0 || templateIds.value.length > 0 || channelIds.value.length > 0 || includeGlobal.value||managerPreferences.value||managerExtensions.value||!props.remote&&managerWebdav.value&&activeWebdav.value||hasAssets.value||props.remote&&(allTemplates.value||allChannels.value))
const familyScopesValid=computed(()=>Object.values(channelFamilies.value).every(scope=>scope.templateIds.length>0));
const preview = computed(() => incoming.value?.package ?? upload.value?.package ?? filePlan.value?.package)
const scopeKey=(category:string,id:string)=>JSON.stringify([category,id])
const scopeObjects=computed<ScopeObject[]>(()=>[
 {id:scopeKey('manager','preferences'),category:'manager',categoryLabel:'桌面偏好',label:'桌面偏好',detail:'可移植偏好；排除本机执行身份与路径',mark:'配置'},
 {id:scopeKey('manager','extensions'),category:'manager',categoryLabel:'桌面偏好',label:'扩展设置与名称清单',detail:'启用及分发设置；实际资源分别选择',mark:'配置'},
 ...(!props.remote&&activeWebdav.value?[{id:scopeKey('manager','webdav'),category:'manager',categoryLabel:'WebDAV 配置',label:'WebDAV连接配置',detail:'文件接收后为未启用连接',mark:webdavPasswords.value?'含密码':'仅配置'}]:[]),
 ...(models.snapshot?.library.templates??[]).map(t=>({id:scopeKey('templates',t.id),category:'templates',categoryLabel:'模型模版',label:t.label,detail:props.remote?'全部已保存版本':'选择要导出的保存版本',mark:'配置',disabled:props.remote&&allTemplates.value})),
 ...savedChannels.value.map(c=>({id:scopeKey('channels',c.id),category:'channels',categoryLabel:exportSource.value==='draft'?'渠道草稿':'渠道与模型',label:c.label,detail:exportSource.value==='draft'?'未提交草稿；精确模版依赖一并导出':'原设置、服务节点、模型规则及精确模版依赖',mark:exportKeys.value&&!channelFamilies.value[c.id]?'含凭据':'不带 Key',disabled:props.remote&&allChannels.value})),
 ...accountCatalog.value.filter(a=>accountPlatform.value==='all'||(a.account.origin?.accountPlatform??'legacy')===accountPlatform.value).map(a=>({id:scopeKey('accounts',a.account.id),category:'accounts',categoryLabel:'账户',label:a.account.label,detail:accountInformationLabel(a.account),mark:a.informationSupported?'仅账户信息':'暂不支持',disabled:!a.informationSupported})),
 ...assetCatalog.value.skills.map(name=>({id:scopeKey('skills',name),category:'skills',categoryLabel:'Skills',label:'Skill · '+name,detail:'实际SKILL.md与所需资源',mark:hasAssets.value&&extensionCredentials.value?'含凭据':'配置',disabled:props.remote&&allSkills.value})),
 ...assetCatalog.value.mcps.map(name=>({id:scopeKey('mcps',name),category:'mcps',categoryLabel:'MCP',label:'MCP · '+name,detail:'完整源定义；不自动执行',mark:hasAssets.value&&extensionCredentials.value?'含凭据':'配置',disabled:props.remote&&allMcps.value})),
].filter(item=>exportSource.value!=='draft'||['templates','channels'].includes(item.category)))
const scopeKeys=computed(()=>[
 ...(managerPreferences.value?[scopeKey('manager','preferences')]:[]),...(managerExtensions.value?[scopeKey('manager','extensions')]:[]),...(!props.remote&&managerWebdav.value&&activeWebdav.value?[scopeKey('manager','webdav')]:[]),
 ...(allTemplates.value?(models.snapshot?.library.templates??[]).map(t=>t.id):templateIds.value).map(id=>scopeKey('templates',id)),
 ...(allChannels.value?savedChannels.value.map(c=>c.id):channelIds.value).map(id=>scopeKey('channels',id)),...accountIds.value.map(id=>scopeKey('accounts',id)),
 ...(allSkills.value?assetCatalog.value.skills:skillNames.value).map(id=>scopeKey('skills',id)),...(allMcps.value?assetCatalog.value.mcps:mcpNames.value).map(id=>scopeKey('mcps',id)),
])
const receiveObjects=computed<ScopeObject[]>(()=>{
 const source=incomingSource.value;if(!source)return []
 return [
  ...(source.manager?.preferences?[{id:scopeKey('manager','preferences'),category:'manager',categoryLabel:'桌面偏好',label:'桌面偏好'}]:[]),
  ...(source.manager?.extensions?[{id:scopeKey('manager','extensions'),category:'manager',categoryLabel:'桌面偏好',label:'扩展设置与名称清单'}]:[]),
  ...(source.manager?.webdav??[]).map(e=>({id:scopeKey('endpoints',e.id),category:'endpoints',categoryLabel:'WebDAV连接',label:e.username+' · '+e.url,detail:e.remotePath+' · 采用后未启用'})),
  ...(source.manager?.accounts??[]).map(a=>({id:scopeKey('accounts',a.id),category:'accounts',categoryLabel:'账户',label:a.label,detail:accountInformationLabel(a)+'；不代表登录已恢复',disabled:!accountInformationSupported(a)})),
  ...source.package.payload.templates.flatMap(t=>t.revisions.map(r=>({id:scopeKey('templates',JSON.stringify([t.id,r.id])),category:'templates',categoryLabel:'模型模版',label:t.label+' · '+versionLabel(r),detail:'精确保存版本；不会自动采用或切换本机默认'}))),
  ...(source.package.payload.policy?.channels??[]).map(c=>({id:scopeKey('channels',c.id),category:'channels',categoryLabel:'渠道与模型',label:c.label,detail:'所需精确模版版本一并接收'})),
  ...(source.assets?.skills??[]).map(s=>({id:scopeKey('skills',s.name),category:'skills',categoryLabel:'Skills',label:'Skill · '+s.name,detail:s.files+' 文件 · '+s.bytes+' 字节'})),
  ...(source.assets?.mcps??[]).map(m=>({id:scopeKey('mcps',m.name),category:'mcps',categoryLabel:'MCP',label:'MCP · '+m.name,detail:'完整源定义；不自动执行'})),
 ]
})
const receiveKeys=computed(()=>[
 ...(receivePreferences.value?[scopeKey('manager','preferences')]:[]),...(receiveExtensions.value?[scopeKey('manager','extensions')]:[]),
 ...receiveEndpoints.value.map(id=>scopeKey('endpoints',id)),...receiveAccounts.value.map(id=>scopeKey('accounts',id)),...receiveRefs.value.map(id=>scopeKey('templates',id)),...receiveChannels.value.map(id=>scopeKey('channels',id)),...receiveSkills.value.map(id=>scopeKey('skills',id)),...receiveMcps.value.map(id=>scopeKey('mcps',id)),
])
function updateReceiveKeys(keys:string[]){
 const ids=(kind:string)=>keys.map(k=>JSON.parse(k) as [string,string]).filter(v=>v[0]===kind).map(v=>v[1]);
 receivePreferences.value=ids('manager').includes('preferences');receiveExtensions.value=ids('manager').includes('extensions');receiveEndpoints.value=ids('endpoints');receiveAccounts.value=ids('accounts');receiveRefs.value=ids('templates');receiveChannels.value=ids('channels');receiveSkills.value=ids('skills');receiveMcps.value=ids('mcps');receiveDirty.value=true;
}
const selectedCredentialKinds=computed(()=>[...(exportKeys.value?['渠道认证（'+keyChannelIds.value.length+'个渠道）']:[]),...(hasAssets.value&&extensionCredentials.value?['所选扩展授权']:[]),...(!props.remote&&managerWebdav.value&&activeWebdav.value&&webdavPasswords.value?['WebDAV连接密码']:[])])
watch(()=>JSON.stringify([templateIds.value,channelIds.value,channelModels.value,channelFamilies.value,includeGlobal.value,versions.value,accountIds.value,managerPreferences.value,managerExtensions.value,exportKeys.value,skillNames.value,mcpNames.value,extensionCredentials.value,allTemplates.value,allChannels.value,allSkills.value,allMcps.value]),()=>{scopeSaved.value=false},{flush:'sync'})
// SY-2：范围首项与原型一致从「默认配置范围」开始；读取失败也保留 all，并就地报错。
const scopePreset=ref('all');const catalogueLoading=ref(true);let fileScopeInitialized=false
// 搜索与范围预设同处一条工具栏；对象树只消费这个值。
const scopeQuery=ref('')
/** 留空写受管导出目录；选择后按该目录另存。 */
const exportTargetDir=ref('')
const remotePassword=ref('')
const remoteSecrets=computed(()=>exportKeys.value||hasAssets.value&&extensionCredentials.value||!props.remote&&managerWebdav.value&&activeWebdav.value&&webdavPasswords.value)
const remotePasswordLabel=computed(()=>'配置同步保护口令（可选，与连接密码独立）')
const remotePlaintextSecrets=computed(()=>remoteSecrets.value&&!remotePassword.value)
// SY-1 / N-SY-2：导出目的地与口令都在确认弹窗内；默认目录与自定义目录互斥，口令可选。
const destinationMode=ref<'default'|'custom'>('default')
const encryptExport=ref(false)
const exportPackagePassword=ref('');const exportPackagePasswordAgain=ref('')
const plaintextConfirmOpen=ref(false)
type PreparedExport={kind:'file'|'remote';scope:TransferScope;includeKeys:boolean;managerScope:ManagerScope;draft?:DraftCandidate;policy?:ModelPolicy;options?:RemoteOptions}
const preparedExport=ref<PreparedExport|null>(null)
const exportCredentialCount=computed(()=>{
 const reported=filePlan.value?.credentialCount
 if(typeof reported==='number')return reported
 return (exportKeys.value?keyChannelIds.value.length:0)+((hasAssets.value&&extensionCredentials.value)?1:0)+((!props.remote&&managerWebdav.value&&activeWebdav.value&&webdavPasswords.value)?1:0)
})
const exportCredentialSummary=computed(()=>[
 ['渠道 Key',exportKeys.value?keyChannelIds.value.length:0],
 ['账户登录态',0],
 ['扩展凭据',hasAssets.value&&extensionCredentials.value?1:0],
 ['WebDAV 密码',!props.remote&&managerWebdav.value&&activeWebdav.value&&webdavPasswords.value?1:0],
])
const exportPlaintextRisk=computed(()=>!incoming.value&&!props.remote&&!encryptExport.value&&exportCredentialCount.value>0)
watch(encryptExport,value=>{if(!value){exportPackagePassword.value='';exportPackagePasswordAgain.value=''}})
watch(destinationMode,value=>{if(value==='default')exportTargetDir.value=''})
const exportGroupLabels=computed(()=>[...new Set(scopeObjects.value.filter(item=>scopeKeys.value.includes(item.id)).map(item=>item.categoryLabel))].join(' / '))
// 原型里 WebDAV 只有两个主操作：保存本机范围 / 同步配置。
const remoteActionsDisabled=computed(()=>busy.value||!models.snapshot||catalogueLoading.value||!remoteScopeLoaded.value||!hasScope.value||!familyScopesValid.value||(hasAssets.value&&!assetsReviewed.value))
function updateScopeKeys(keys:string[]){
 const previous=new Set(scopeKeys.value);const next=new Set(keys);const changed=new Set([...previous,...next].filter(k=>previous.has(k)!==next.has(k)).map(k=>(JSON.parse(k) as [string,string])[0]))
 const ids=(category:string)=>keys.map(k=>JSON.parse(k) as [string,string]).filter(v=>v[0]===category).map(v=>v[1])
 if(changed.has('manager')){managerPreferences.value=ids('manager').includes('preferences');managerExtensions.value=ids('manager').includes('extensions');const selected=ids('manager').includes('webdav');if(selected&&!managerWebdav.value)webdavPasswords.value=true;managerWebdav.value=selected}
 if(changed.has('templates')){allTemplates.value=false;templateIds.value=ids('templates')}
 if(changed.has('channels')){allChannels.value=false;channelIds.value=ids('channels');channelModels.value=Object.fromEntries(Object.entries(channelModels.value).filter(([id])=>channelIds.value.includes(id)));channelFamilies.value=Object.fromEntries(Object.entries(channelFamilies.value).filter(([id])=>channelIds.value.includes(id)))}
 if(changed.has('accounts'))accountIds.value=ids('accounts')
 if(changed.has('skills')){allSkills.value=false;skillNames.value=ids('skills')}
 if(changed.has('mcps')){allMcps.value=false;mcpNames.value=ids('mcps')}
 scopePreset.value='custom'
}
function applyScopePreset(value:string){
 scopePreset.value=value;if(value==='custom')return;channelModels.value={};channelFamilies.value={};if(exportSource.value==='draft'){templateIds.value=[];channelIds.value=savedChannels.value.map(c=>c.id);includeGlobal.value=false;return}
 managerPreferences.value=true;managerExtensions.value=true;managerWebdav.value=!props.remote;webdavPasswords.value=value!=='share';
 templateIds.value=(models.snapshot?.library.templates??[]).map(t=>t.id);channelIds.value=savedChannels.value.map(c=>c.id);accountIds.value=accountCatalog.value.filter(a=>a.informationSupported).map(a=>a.account.id);skillNames.value=[...assetCatalog.value.skills];mcpNames.value=[...assetCatalog.value.mcps];includeGlobal.value=true;
 allTemplates.value=false;allChannels.value=false;allSkills.value=false;allMcps.value=false;exportKeys.value=value==='move';extensionCredentials.value=value==='move';assetsReviewed.value=false;
}

onMounted(async () => { if(props.importOnly)return; try{if (!models.snapshot) await models.load();if(!props.remote)await getSyncConfig().then(config=>{activeWebdav.value=!!config.endpoint}).catch(()=>{});else await loadScope()}finally{catalogueLoading.value=false;await begin()} })
async function loadAssets(){try{const catalog=await modelTransferApi.extensionCatalog();if(!catalog||!Array.isArray(catalog.skills)||!Array.isArray(catalog.mcps)||![...catalog.skills,...catalog.mcps].every(name=>typeof name==='string'))throw {message:'扩展目录返回结构无效，请重新读取；未扩大配置范围'};assetCatalog.value=catalog;assetError.value=''}catch(cause){assetError.value=errorText(cause)}}
function assetScope(){return hasAssets.value?{skillNames:skillNames.value,mcpNames:mcpNames.value,extensionCredentials:extensionCredentials.value,assetsReviewed:assetsReviewed.value}:{}}
function remoteOptions(){return {...(accountIds.value.length?{accountIds:accountIds.value}:{}),preferences:managerPreferences.value,extensions:managerExtensions.value,includeKeys:exportKeys.value,...assetScope(),...(allTemplates.value||allChannels.value||allSkills.value||allMcps.value?{allTemplates:allTemplates.value,allChannels:allChannels.value,allSkills:allSkills.value,allMcps:allMcps.value}:{}),passphrase:remotePassword.value||undefined}}
async function loadScope(){try{const scope=await modelTransferApi.loadRemoteScope();accountIds.value=scope.accountIds??[];templateIds.value=scope.templateIds;channelIds.value=scope.channelIds;channelModels.value=scope.channelModels??{};channelFamilies.value=scope.channelFamilies??{};includeGlobal.value=scope.includeGlobal;managerPreferences.value=scope.preferences;managerExtensions.value=scope.extensions;exportKeys.value=scope.includeKeys;skillNames.value=scope.skillNames??[];mcpNames.value=scope.mcpNames??[];extensionCredentials.value=scope.extensionCredentials??false;allTemplates.value=scope.allTemplates??false;allChannels.value=scope.allChannels??false;allSkills.value=scope.allSkills??false;allMcps.value=scope.allMcps??false;remoteScopeLoaded.value=true;scopeReadError.value=''}catch(cause){remoteScopeLoaded.value=false;scopeReadError.value=errorText(cause)}}
function transferScope(){return scopeFor(models.snapshot!,templateIds.value,allChannels.value?savedChannels.value.map(c=>c.id):channelIds.value,includeGlobal.value,versions.value,channelModels.value,channelFamilies.value)}
async function saveRemoteScope(){if(!models.snapshot||!remoteScopeLoaded.value||catalogueLoading.value)return;await perform(async()=>{await modelTransferApi.saveRemoteScope(remoteScope(transferScope(),remoteOptions()));scopeSaved.value=true;selectionOpen.value=false})}
async function perform(action: () => Promise<void>) {
  if (busy.value) return
  busy.value = true; error.value = ''
  try { await action() } catch (cause) { error.value = errorText(cause) } finally { busy.value = false }
}
async function begin() {
 if(catalogueLoading.value)return
 selectionOpen.value=true;accountError.value='';channelError.value='';catalogueLoading.value=true
 try {if(props.remote&&!remoteScopeLoaded.value)await loadScope();await Promise.all([modelTransferApi.channelCatalog().then(items=>{channelCatalog.value=items}).catch(cause=>{channelError.value=errorText(cause)}),modelTransferApi.accountCatalog().then(items=>{accountCatalog.value=items}).catch(cause=>{accountError.value=errorText(cause)}),loadAssets()]);if(!props.remote&&!fileScopeInitialized){applyScopePreset('all');fileScopeInitialized=true}}finally{catalogueLoading.value=false}
}
function managerScope():ManagerScope{return {...(accountIds.value.length?{accountIds:accountIds.value}:{}),preferences:managerPreferences.value,extensions:managerExtensions.value,webdav:managerWebdav.value,webdavPasswords:webdavPasswords.value,...assetScope()}}
async function prepare() {
  if (props.remote&&!remoteScopeLoaded.value||!models.snapshot || !hasScope.value||!familyScopesValid.value||catalogueLoading.value) return
  await perform(async () => {
    const scope = JSON.parse(JSON.stringify(transferScope())) as TransferScope
    if(exportSource.value==='draft'&&!props.remote){
      if(!models.snapshot||!models.channelDraft)throw {message:'工作区未读取'}
      const draft={expectedLibrary:models.snapshot.libraryDigest,expectedPolicy:models.snapshot.policyDigest,expectedWorkspace:models.snapshot.workspaceDigest}
      const policy=JSON.parse(JSON.stringify(models.channelDraft)) as ModelPolicy
      filePlan.value=await modelTransferApi.prepareDraft(scope,draft,undefined,policy)
      preparedExport.value={kind:'file',scope,includeKeys:false,managerScope:managerScope(),draft,policy}
    }else if (props.remote){
      const options=remoteOptions()
      upload.value = await modelTransferApi.prepareUpload(scope,options)
      preparedExport.value={kind:'remote',scope,includeKeys:exportKeys.value,managerScope:managerScope(),options}
      remotePassword.value=''
    }else{
      const manager=managerScope()
      filePlan.value = await modelTransferApi.prepare(scope,undefined,exportKeys.value,manager)
      preparedExport.value={kind:'file',scope,includeKeys:exportKeys.value,managerScope:manager}
    }
    destinationMode.value='default';encryptExport.value=exportCredentialCount.value>0;exportPackagePassword.value='';exportPackagePasswordAgain.value=''
    selectionOpen.value = false
  })
}
/** 用户在确认弹窗选择加密后，按同一冻结范围重新生成受口令保护的计划；未勾选则沿用已预览计划。 */
async function preparedWithPassphrase(passphrase?:string){const prepared=preparedExport.value;if(!prepared)throw {message:'导出范围已失效，请重新选择范围并导出。'};if(prepared.kind==='remote'){const options:RemoteOptions={...(prepared.options??remoteOptions()),passphrase};prepared.options=options;return await modelTransferApi.prepareUpload(prepared.scope,options)}if(prepared.draft)return await modelTransferApi.prepareDraft(prepared.scope,prepared.draft,passphrase,prepared.policy);return await modelTransferApi.prepare(prepared.scope,passphrase,prepared.includeKeys,prepared.managerScope)}
function openImport() { if (!busy.value) { importPath.value=''; error.value=''; pickOpen.value=true } }
async function chooseImportFile() {
  const selected = await open({ multiple:false, directory:false, filters:[{ name:'配置文件', extensions:['ocxdconf','json'] }] })
  if (typeof selected === 'string') importPath.value=selected
}
async function receive() {
  if (!props.remote && !importPath.value) return
  await perform(async () => {
    if(!models.snapshot) await models.load()
    if (props.remote){try{incoming.value = await modelTransferApi.download()}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&cause.code==='passphrase_required'){remoteUnlock.value=true;passwordOpen.value=true;importPassword.value='';return}throw cause}}
    else {
      recompareId.value='';openingOriginal.value=false;remoteUnlock.value=false
      try{incoming.value=await modelTransferApi.receive(importPath.value)}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&cause.code==='passphrase_required'){pickOpen.value=false;passwordOpen.value=true;importPassword.value='';return}throw cause}
    }
    pickOpen.value=false
    filePlan.value=null;upload.value=null;preparedExport.value=null;plaintextConfirmOpen.value=false;exportPackagePassword.value='';exportPackagePasswordAgain.value=''
    adoptKeys.value=false;unlockedPassword='';if(incoming.value)selectIncoming(incoming.value)
  })
}
watch(() => props.importOpen, value => { if(value)openImport() }, { immediate:true })
function closeImport() { if(!busy.value){pickOpen.value=false;emit('update:importOpen',false)} }
function selectIncoming(plan:IncomingPlan){
 receiveChannelModels.value={};
 receiveSkillTargets.value=Object.fromEntries((plan.assets?.skills??[]).map(s=>[s.name,plan.assetMappings?.skills[s.name]??s.name]));receiveMcpTargets.value=Object.fromEntries((plan.assets?.mcps??[]).map(m=>[m.name,plan.assetMappings?.mcps[m.name]??m.name]));receiveMappings.value=Object.fromEntries(Object.entries(plan.channelBindings??{}).map(([id,b])=>[id,b.keepLocalConnection?'local:'+b.targetId:'new']));receiveNewLabels.value=Object.fromEntries((plan.package.payload.policy?.channels??[]).map(c=>[c.id,plan.channelBindings?.[c.id]?.label??c.label+'（导入）']));receiveAccounts.value=plan.manager?.accounts?.filter(accountInformationSupported).map(a=>a.id)??[];incomingSource.value=plan;receiveRefs.value=plan.package.payload.templates.flatMap(t=>t.revisions.map(r=>JSON.stringify([t.id,r.id])));receiveChannels.value=plan.package.payload.policy?.channels.map(c=>c.id)??[];receiveGlobal.value=!!(plan.package.payload.policy?.templateRefs||plan.package.payload.policy?.channelOrder||plan.package.payload.policy?.defaultChannelId);receivePreferences.value=!!plan.manager?.preferences;receiveExtensions.value=!!plan.manager?.extensions;receiveEndpoints.value=plan.manager?.webdav.map(e=>e.id)??[];receiveSkills.value=plan.assets?.skills.map(skill=>skill.name)??[];receiveMcps.value=plan.assets?.mcps.map(mcp=>mcp.name)??[];adoptExtensionCredentials.value=plan.adoption?.extensionCredentials??false;adoptKeys.value=plan.adoption?.keys??false;adoptWebdavPasswords.value=plan.adoption?.webdav??true;receiveDirty.value=false
}
async function unlockFile(){await perform(async()=>{incoming.value=remoteUnlock.value?await modelTransferApi.download(importPassword.value):recompareId.value?(openingOriginal.value?await modelTransferApi.openReceived(recompareId.value,importPassword.value):await modelTransferApi.recompare(recompareId.value,importPassword.value)):await modelTransferApi.receive(importPath.value,importPassword.value);unlockedPassword=importPassword.value;passwordOpen.value=false;importPassword.value='';filePlan.value=null;upload.value=null;if(!openingOriginal.value)selectIncoming(incoming.value)})}
async function recompareOriginal(){if(!incomingSource.value)return;await perform(async()=>{recompareId.value=incomingSource.value!.id;openingOriginal.value=false;try{incoming.value=await modelTransferApi.recompare(recompareId.value);selectIncoming(incoming.value)}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&cause.code==='passphrase_required'){passwordOpen.value=true;importPassword.value='';return}throw cause}})}

async function confirm() {
  if(incoming.value?.protected&&!unlockedPassword&&((adoptKeys.value||adoptWebdavPasswords.value&&incoming.value.availableWebdavPasswords?.length||adoptExtensionCredentials.value))){recompareId.value=incoming.value.id;openingOriginal.value=true;passwordOpen.value=true;importPassword.value='';return}
  await perform(async () => {
    if (incoming.value) {
      if(receiveDirty.value||incoming.value.previewError)return
      const includeDav=adoptWebdavPasswords.value&&!!incoming.value.availableWebdavPasswords?.length
      const plan=incoming.value
      await models.adoptTransfer(plan,async()=>{
        try{return adoptKeys.value||includeDav||adoptExtensionCredentials.value||plan.protected&&!!(plan.assets?.skills.length||plan.assets?.mcps.length)?await modelTransferApi.accept(plan.id,{adoptKeys:adoptKeys.value,...(includeDav?{adoptWebdavPasswords:true}:{}),...(adoptExtensionCredentials.value?{adoptExtensionCredentials:true}:{}),passphrase:unlockedPassword}):await modelTransferApi.accept(plan.id)}catch(cause){try{const latest=await modelTransferApi.latestIncoming();if(latest?.id===plan.id&&latest.adoption)plan.adoption=latest.adoption}catch{}throw cause}
      })
      const updatePreferences=!!plan.manager?.preferences; emit('update:importOpen',false); app.showToast('配置已接收。'); incoming.value = null;incomingSource.value=null;unlockedPassword='';if(updatePreferences)await app.loadPreferences()
    } else if (upload.value) {
      if(preparedExport.value?.kind==='remote'&&remotePassword.value){const options:RemoteOptions={...(preparedExport.value.options??remoteOptions()),passphrase:remotePassword.value};upload.value=await modelTransferApi.prepareUpload(preparedExport.value.scope,options)}
      upload.value = await modelTransferApi.publish(upload.value.id)
      remotePassword.value='';preparedExport.value=null
    } else if (filePlan.value) await finishExport()
  })
}
/** 打开备份包所在目录：定位到单次包，便于整体拷贝。 */
async function openExportParent(){
  const path=exportedPath.value
  if(!path)return
  const parent=path.slice(0,path.lastIndexOf('/'))||path
  try{await app.openManagedPathByKey('exports')}catch{app.showToast('打开导出目录失败；可手动前往 '+parent)}
}
/** 选择导出目录；取消时保持当前选择。 */
async function chooseExportDir(){
  const picked=await open({directory:true,multiple:false,title:'选择导出目录'})
  if(!picked||Array.isArray(picked))return
  exportTargetDir.value=picked
}
async function finishExport(plaintextAccepted=false){
  if(!filePlan.value||!preparedExport.value)return
  if(encryptExport.value){
    if(exportPackagePassword.value.length<8||exportPackagePassword.value!==exportPackagePasswordAgain.value){error.value='请输入至少 8 位的加密口令，并保持两次一致。';return}
    filePlan.value=await preparedWithPassphrase(exportPackagePassword.value)
  }else if(exportCredentialCount.value>0&&!plaintextAccepted){plaintextConfirmOpen.value=true;return}
  else if(filePlan.value.protected)filePlan.value=await preparedWithPassphrase()
  if(destinationMode.value==='custom'&&!exportTargetDir.value){error.value='请选择自定义目录，或改回默认导出目录。';return}
  const result=await modelTransferApi.export(filePlan.value.id, exportTargetDir.value || undefined)
  exportedPath.value=result.path;filePlan.value=null;preparedExport.value=null;plaintextConfirmOpen.value=false;exportPackagePassword.value='';exportPackagePasswordAgain.value=''
}
async function acceptPlaintextExport(){await perform(()=>finishExport(true))}
function setModelScope(id:string,values:string[]|undefined,receive=false){const target=receive?receiveChannelModels:channelModels;const next={...target.value};if(values===undefined)delete next[id];else next[id]=values;target.value=next;if(receive)receiveDirty.value=true;else scopePreset.value='custom'}
function sourceFamilies(id:string){
 const policy=exportSource.value==='draft'?models.channelDraft:models.snapshot?.policy
 const channel=policy?.channels.find(c=>c.id===id);if(!channel)return []
 const ids=[...new Set([...(policy?.templateRefs??[]),...channel.templateRefs,...Object.values(channel.modelRefs)].map(r=>r.templateId))]
 return ids.map(id=>({id,label:models.snapshot?.library.templates.find(t=>t.id===id)?.label??id}))
}
function setFamilyScope(id:string,value:FamilyScope|undefined){
 const next={...channelFamilies.value};if(value===undefined)delete next[id];else{next[id]=value;const subset={...channelModels.value};delete subset[id];channelModels.value=subset}
 channelFamilies.value=next;scopePreset.value='custom'
}
function receivedFamily(id:string){return incomingSource.value?.package.payload.policy?.channels.find(c=>c.id===id)?.familyScope}
function sourceModels(id:string,receive=false){
 if(!receive){const channel=savedChannels.value.find(c=>c.id===id);if(!channel)return [];const ids=[...new Set([...channel.discovered.map(m=>m.id),...channel.modelOrder,...Object.keys(channel.modelRefs)])];return ids.map(id=>({id,label:channel.discovered.find(m=>m.id===id)?.label}))}
 const channel=incomingSource.value?.package.payload.policy?.channels.find(c=>c.id===id);if(!channel)return []
 const ids=[...new Set([...(channel.discovered??[]).map(m=>m.id),...(channel.modelOrder??[]),...((channel.connection?.provider?.models as string[]|undefined)??[])])];return ids.map(id=>({id,label:channel.discovered?.find(m=>m.id===id)?.label}))
}
function assetMappingScope(){
 const skills=Object.fromEntries(receiveSkills.value.filter(name=>receiveSkillTargets.value[name]!==undefined&&receiveSkillTargets.value[name]!==name).map(name=>[name,receiveSkillTargets.value[name]!]))
 const mcps=Object.fromEntries(receiveMcps.value.filter(name=>receiveMcpTargets.value[name]!==undefined&&receiveMcpTargets.value[name]!==name).map(name=>[name,receiveMcpTargets.value[name]!]))
 return Object.keys(skills).length||Object.keys(mcps).length?{assetMappings:{skills,mcps}}:{}
}
function channelMappingScope(){
 const channelMappings:Record<string,ChannelImportMapping>={}
 for(const id of receiveChannels.value){const mode=receiveMappings.value[id];if(mode==='new')channelMappings[id]={mode:'new',label:receiveNewLabels.value[id]??''};else if(mode?.startsWith('local:'))channelMappings[id]={mode:'local',targetId:mode.slice(6)}}
 return Object.keys(channelMappings).length?{channelMappings}:{}
}
async function compareReceived(){if(!incomingSource.value)return;await perform(async()=>{const scope={...(Object.keys(receiveChannelModels.value).length?{channelModels:Object.fromEntries(Object.entries(receiveChannelModels.value).filter(([id])=>receiveChannels.value.includes(id)))}:{}),...assetMappingScope(),...channelMappingScope(),templateRefs:receiveRefs.value.map(value=>{const [templateId,revisionId]=JSON.parse(value) as [string,string];return {templateId,revisionId}}),channelIds:receiveChannels.value,includeGlobal:receiveGlobal.value,...(incomingSource.value!.manager?{accountIds:receiveAccounts.value,preferences:receivePreferences.value,extensions:receiveExtensions.value,endpointIds:receiveEndpoints.value}:{}),...(incomingSource.value!.assets?{skillNames:receiveSkills.value,mcpNames:receiveMcps.value}:{})};incoming.value=unlockedPassword?await modelTransferApi.select(incomingSource.value!.id,scope,unlockedPassword):await modelTransferApi.select(incomingSource.value!.id,scope);if(!incoming.value.availableKeys?.length)adoptKeys.value=false;if(!incoming.value.availableExtensionCredentials)adoptExtensionCredentials.value=false;receiveDirty.value=false})}

function closePreview() { if (!busy.value) { emit('update:importOpen',false); incoming.value = null;incomingSource.value=null; filePlan.value = null; upload.value = null;preparedExport.value=null;plaintextConfirmOpen.value=false;exportPackagePassword.value='';exportPackagePasswordAgain.value='';remotePassword.value='';unlockedPassword='' } }
</script>
<template>
  <article class="model-transfer" :class="{ card: !importOnly }">
    <template v-if="!importOnly">
    <div class="card-head"><div><h2>{{ remote ? '设备间同步' : '导入与导出' }}</h2><p>{{ remote ? '本机范围同时用于上传与接收；下载的变化先预览，再应用。' : '把选定配置带到另一台设备，或用分享预设导出不含凭据的配置。' }}</p></div><div class="sync-head-actions"><template v-if="remote"><UiButton variant="primary" :disabled="remoteActionsDisabled" @click="saveRemoteScope">保存本机范围</UiButton><UiButton :disabled="remoteActionsDisabled" @click="prepare">同步配置</UiButton></template><template v-else><UiButton :disabled="busy||catalogueLoading||!models.snapshot" @click="openImport">导入配置</UiButton><UiButton variant="primary" :disabled="busy||!models.snapshot||catalogueLoading||!hasScope||!familyScopesValid||(exportSource==='draft'&&!channelIds.length)||(hasAssets&&!assetsReviewed)" @click="prepare">导出配置</UiButton></template></div></div>
    <p v-if="models.error && !models.snapshot" class="model-error" role="alert">{{ models.error }}</p>
    <p v-if="error && !preview" class="model-error" role="alert">{{ error }}</p>
<p v-if="remote&&scopeSaved" role="status">本设备范围已保存，尚未上传；采用结果另行核对。</p>
    <div v-if="exportedPath" class="model-transfer-path" role="status"><span>备份包</span><code class="local-path">{{ displayPath(exportedPath) }}</code><UiButton size="sm" variant="ghost" @click="openExportParent">打开备份包目录</UiButton></div>
    <!-- 范围是页面主体的一部分，不再收进弹窗：原型里它是列表旁的常驻两栏工作区。 -->
    <!-- 原型结构：一条工具栏（范围预设 + 搜索 + 清空）+ 两栏；右栏是凭据清单，不是操作列。 -->
    <section class="sync-scope-workspace" aria-label="同步范围" data-testid="sync-scope-workspace">
      <div class="sync-scope-main">
        <div class="sync-scope-toolbar">
          <UiSelectField class="sync-preset" :model-value="scopePreset" label="配置范围预设" :options="[{value:'all',label:'默认配置范围'},{value:'share',label:'分享配置 · 不含凭据'},{value:'move',label:'换设备 · 含可转移凭据'},{value:'custom',label:'自定义范围'}]" :disabled="busy||catalogueLoading" @update:model-value="applyScopePreset" />
          <input v-model="scopeQuery" class="input sync-scope-search" type="search" aria-label="搜索配置对象" placeholder="搜索配置对象" :disabled="busy||catalogueLoading" />
          <UiButton :disabled="busy||catalogueLoading||!scopeKeys.length" @click="updateScopeKeys([])">清空选择</UiButton>
        </div>
        <p v-if="exportSource==='draft'" class="model-muted">未提交草稿候选：仅含明确选择的渠道草稿和精确模版依赖，不携带凭据。</p>
        <p v-if="catalogueLoading" role="status">正在读取已保存配置对象…</p>
        <p v-for="message in [scopeReadError,assetError,accountError,channelError].filter(Boolean)" :key="message" class="model-error" role="alert">{{ message }}</p>
        <label v-if="accountCatalog.length&&exportSource!=='draft'" class="sync-account-filter">账户平台<UiSelectField v-model="accountPlatform" label="账户平台筛选" :options="[{value:'all',label:'全部平台'},...Object.entries(accountPlatforms).map(([value,label])=>({value,label})),{value:'legacy',label:'旧格式 · 来源未登记'}]" :disabled="busy||catalogueLoading" /></label>
        <ObjectScopeTree :objects="scopeObjects" :selected-ids="scopeKeys" :query="scopeQuery" label="导出配置对象" :disabled="busy||catalogueLoading" @update:selected-ids="updateScopeKeys">
          <template #options="{object}">
            <template v-if="object.category==='accounts'"><details v-for="account in scopeAccounts(object.id)" :key="account.id"><summary>账户详情与同步选项</summary><dl><dt>管理方</dt><dd>{{ account.origin?.owner==='opencodex'?'OpenCodex账户池':account.origin?.owner??'旧格式 · 来源未登记' }}</dd><dt>来源账户</dt><dd>{{ account.origin?.sourceAccountId??account.id }}</dd><dt>授权主体</dt><dd>{{ account.subject??'未提供；保留独立记录' }}</dd><dt>账户 / 组织</dt><dd>{{ account.origin?.tenant??'未提供' }}</dd></dl><label><input type="checkbox" :checked="accountInformationSupported(account)" disabled />仅账户信息</label><label><input type="checkbox" disabled />同步登录状态</label><p>{{ accountTransferLabel(account.provider) }}</p></details></template>
            <UiSelectField v-if="object.category==='templates'&&!remote" :model-value="versions[JSON.parse(object.id)[1]]??models.snapshot!.library.templates.find(t=>t.id===JSON.parse(object.id)[1])!.revisions.at(-1)!.id" :options="[...models.snapshot!.library.templates.find(t=>t.id===JSON.parse(object.id)[1])!.revisions].reverse().map(r=>({value:r.id,label:versionLabel(r)}))" :label="object.label+'导出版本'" :disabled="busy||catalogueLoading||!scopeKeys.includes(object.id)" @update:model-value="versions[JSON.parse(object.id)[1]]=$event" />
            <ChannelModelScope v-if="object.category==='channels'&&scopeKeys.includes(object.id)" :label="object.label" :models="sourceModels(JSON.parse(object.id)[1])" :families="sourceFamilies(JSON.parse(object.id)[1])" :family-scope="channelFamilies[JSON.parse(object.id)[1]]" @update:family-scope="setFamilyScope(JSON.parse(object.id)[1],$event)" :selected="channelModels[JSON.parse(object.id)[1]]" :disabled="busy||catalogueLoading" @update:selected="setModelScope(JSON.parse(object.id)[1],$event)" />
            <details v-if="object.id===scopeKey('manager','webdav')&&managerWebdav"><summary>同步选项</summary><label><input v-model="webdavPasswords" type="checkbox" :disabled="busy||catalogueLoading" />包含连接密码</label></details>
          </template>
        </ObjectScopeTree>
      </div>
      <aside class="sync-scope-credentials" aria-label="其他凭据" data-testid="sync-scope-credentials">
        <h3>其他凭据</h3>
        <p class="sync-credentials-note">{{ remote ? '账户按平台能力提供可选范围；当前仅账户信息，WebDAV 密码随配置选择；以下凭据可另选。' : '账户按平台能力提供可选范围；当前仅账户信息，WebDAV 密码随配置选择；以下凭据可另选。' }}</p>
        <section class="sync-credential-block">
          <label class="sync-credential-head"><input v-model="exportKeys" type="checkbox" :disabled="busy||catalogueLoading||!keyChannelIds.length" />渠道 API Key</label>
          <p class="sync-credential-hint">渠道可单独分享，接收方自行填 Key。</p>
          <p v-if="exportSource!=='draft'&&!keyChannelIds.length" class="sync-credential-hint">先在左栏选择含 Key 认证的渠道。</p>
        </section>
        <section class="sync-credential-block">
          <label class="sync-credential-head"><input v-model="extensionCredentials" type="checkbox" :disabled="busy||catalogueLoading||!hasAssets" />扩展凭据</label><label v-if="hasAssets" class="sync-credential-head"><input v-model="assetsReviewed" type="checkbox" :disabled="busy||catalogueLoading" aria-label="已复核所选 Skill 与 MCP 资源" />已复核所选资源</label><p v-if="hasAssets&&!assetsReviewed" class="model-muted">勾选 Skill 或 MCP 后需先复核资源内容，方可导出。</p>
          <p class="sync-credential-hint">例如 MCP 的 Authorization / 环境变量。</p>
          <p v-if="!hasAssets" class="sync-credential-hint">先在左栏选择 Skills 或 MCP。</p>
        </section>
        <details v-if="remote" class="sync-credential-more"><summary>后续新增对象</summary><label><input v-model="allTemplates" type="checkbox" :disabled="busy||catalogueLoading" />全部模版（含后续新增）</label><label><input v-model="allChannels" type="checkbox" :disabled="busy||catalogueLoading" />全部已保存渠道（含后续新增）</label><label><input v-model="allSkills" type="checkbox" :disabled="busy||catalogueLoading" />全部Skill（含后续新增）</label><label><input v-model="allMcps" type="checkbox" :disabled="busy||catalogueLoading" />全部MCP（含后续新增）</label></details>
        <label class="sync-credential-inline"><input v-model="includeGlobal" type="checkbox" :disabled="busy||catalogueLoading" />全局采用及渠道组顺序</label>
        <p class="sync-credentials-note">{{ selectedCredentialKinds.length ? '所选凭据将随文件传输；加密口令在确认弹窗内填写，可选。' : '当前不携带凭据' }}</p>
      </aside>
    </section>
    </template>
    <UiDialog panel-class="transfer-file-picker" hide-close :open="pickOpen" title="导入配置" :busy="busy" @close="closeImport"><label>配置文件<UiButton :disabled="busy" @click="chooseImportFile">选择文件</UiButton><span class="local-path">{{ displayPath(importPath) || '尚未选择文件' }}</span></label><p v-if="error" class="model-error" role="alert">{{ error }}</p><template #actions><UiButton :disabled="busy" @click="closeImport">取消</UiButton><UiButton variant="primary" :disabled="busy||!importPath" :loading="busy" @click="receive">读取并比较</UiButton></template></UiDialog>
    <UiDialog panel-class="transfer-review" :open="!!preview" :title="incoming ? '比较配置文件' : remote ? '确认配置上传' : '确认导出'" :busy="busy" wide @close="closePreview">
      <p v-if="error" class="model-error" role="alert">{{ error }}</p>
      <p v-if="preview?.payload.candidate" role="status">未提交草稿候选；采用后仍进入本机工作区，须另行预览并保存，不表示运行配置已写入。</p>

      <div v-if="!incoming&&!remote" class="export-destination"><span>保存位置</span><div class="export-destination-choice" role="radiogroup" aria-label="保存位置"><label><input v-model="destinationMode" type="radio" value="default" :disabled="busy" />默认导出目录</label><label><input v-model="destinationMode" type="radio" value="custom" :disabled="busy" />自定义目录</label></div><div v-if="destinationMode==='custom'" class="export-destination-custom"><div class="export-destination-row"><input class="input local-path" type="text" readonly :value="displayPath(exportTargetDir)" aria-label="自定义导出目录" placeholder="尚未选择目录"><UiButton size="sm" variant="ghost" :disabled="busy" @click="chooseExportDir">选择目录…</UiButton></div><p class="model-muted">未选择目录时按默认导出目录保存。</p></div></div>
      <div v-if="!incoming&&!remote" class="export-package-layout">
        <!-- 原型顺序：标题 → 导出目录 → 生成位置树 → 说明段 → 文件保护/携带凭据 → 口令 → 底部主按钮。 -->
        <div class="export-tree" aria-label="导出目录结构"><p class="export-tree-kicker">生成位置</p><pre>{{ exportTreeText }}</pre><p class="model-muted">按日期归组，一次导出一个可独立拷贝的备份包。同日多次导出各自建包，不混写、不覆盖。</p></div>
        <div class="export-summary"><strong>{{ scopeKeys.length }} 项配置 · 同一备份包</strong><p class="model-muted">{{ exportCredentialCount ? '含 '+exportCredentialCount+' 项凭据；可选择加密，不加密则凭据以明文写入，文件可被直接读取。口令始终不写入包。' : '不含凭据，可直接使用完整性校验。' }}每次导出生成一个独立备份包，按“日期 / 单次包”存放，可整体拷贝到其它设备再导入。</p><p class="model-muted">包内按 {{ exportGroupLabels || '所选范围' }} 分组，导入时仍按所选范围识别。</p></div>
      </div>
      <div v-if="!incoming&&!remote" class="export-info"><strong>{{ scopeKeys.length }} 项配置 · {{ exportCredentialCount?'含 '+exportCredentialCount+' 项凭据':'不含任何凭据' }}</strong><br><span v-for="([label,count],index) in exportCredentialSummary" :key="label">{{ index?' / ':'' }}{{ label }} {{ count }} 项</span></div>
      <label v-if="!incoming&&!remote" class="export-encrypt"><input v-model="encryptExport" type="checkbox" :disabled="busy" />{{ exportCredentialCount?'加密配置包（建议；不加密则凭据以明文写入）':'为配置包设置加密口令' }}</label>
      <div v-if="!incoming&&!remote&&(exportCredentialCount>0||encryptExport)" class="export-passphrase"><label>加密口令（可选，至少 8 位）<input v-model="exportPackagePassword" aria-label="加密口令" class="input" type="password" autocomplete="new-password" :disabled="busy||!encryptExport" :placeholder="encryptExport?'至少 8 位':'勾选加密后可填写'" /></label><label>再次输入<input v-model="exportPackagePasswordAgain" aria-label="再次输入加密口令" class="input" type="password" autocomplete="new-password" :disabled="busy||!encryptExport" :placeholder="encryptExport?'再次输入':'勾选加密后可填写'" /></label></div>
      <p v-if="exportPlaintextRisk" class="export-plaintext-risk" role="note">不加密时凭据以明文写入该文件，任何能读取文件的人都能看到这些凭据。</p>
      <label v-if="remote&&upload" class="export-encrypt">{{ remotePasswordLabel }}<input v-model="remotePassword" aria-label="导出保护口令" class="input" type="password" autocomplete="new-password" :disabled="busy" :placeholder="remotePlaintextSecrets?'留空则凭据以明文写入文件':'留空使用完整性校验'" /></label><p v-if="remote&&upload&&remoteSecrets" class="export-plaintext-risk" role="note">{{ remotePlaintextSecrets?'所选凭据将随同步快照传输；未填口令时以明文写入。':'所选凭据已用口令加密。' }}</p>
      <p v-if="incoming?.adoption">{{ incoming.adoption.completed?'原导入已完成；当前本机配置保持。':'原导入采用范围已冻结，请核对同一操作。' }}</p><label v-if="incoming?.availableExtensionCredentials"><input v-model="adoptExtensionCredentials" type="checkbox" :disabled="busy||receiveDirty||!!incoming.adoption" />采用所选扩展凭据（新本机引用及受保护资源）</label><label v-if="incoming?.availableWebdavPasswords?.length"><input v-model="adoptWebdavPasswords" type="checkbox" :disabled="busy||receiveDirty||!!incoming?.adoption" />采用包内连接密码（{{ incoming.availableWebdavPasswords.length }} 项），连接保持未启用</label><label v-if="incoming?.availableKeys?.length"><input v-model="adoptKeys" type="checkbox" :disabled="busy||receiveDirty||!!incoming?.adoption" />采用所选渠道的Key与认证配置（{{ incoming.availableKeys.length }} 个渠道），创建受保护本机材料并进入草稿</label>
      <details v-if="incomingSource" open class="model-transfer-scope"><summary>选择接收范围</summary><UiButton size="sm" variant="ghost" :disabled="busy||!!incoming?.adoption" @click="recompareOriginal">按当前基准重新比较原文件</UiButton><ObjectScopeTree :objects="receiveObjects" :selected-ids="receiveKeys" label="接收配置对象" :disabled="busy||!!incoming?.adoption" @update:selected-ids="updateReceiveKeys"><template #options="{object}">
       <template v-if="object.category==='skills'&&receiveSkills.includes(JSON.parse(object.id)[1])"><label>接收Skill名称<input v-model="receiveSkillTargets[JSON.parse(object.id)[1]]" class="input" maxlength="256" :aria-label="JSON.parse(object.id)[1]+'目标Skill名称'" :disabled="busy||!!incoming?.adoption" @input="receiveDirty=true" /></label></template>
       <template v-if="object.category==='mcps'&&receiveMcps.includes(JSON.parse(object.id)[1])"><label>接收MCP名称<input v-model="receiveMcpTargets[JSON.parse(object.id)[1]]" class="input" maxlength="256" :aria-label="JSON.parse(object.id)[1]+'目标MCP名称'" :disabled="busy||!!incoming?.adoption" @input="receiveDirty=true" /></label></template>
       <template v-if="object.category==='channels'&&receiveChannels.includes(JSON.parse(object.id)[1])"><p v-if="receivedFamily(JSON.parse(object.id)[1])">模型族片段：保留本机其他族、连接、Key和已有顺序；{{ receivedFamily(JSON.parse(object.id)[1])?.includeChannelDefaults?'另含渠道共用默认字段，影响全部型号':'共用默认字段沿本机值解析' }}</p><ChannelModelScope v-else :label="object.label" :models="sourceModels(JSON.parse(object.id)[1],true)" :selected="receiveChannelModels[JSON.parse(object.id)[1]]" :disabled="busy||!!incoming?.adoption" @update:selected="setModelScope(JSON.parse(object.id)[1],$event,true)" /><UiSelectField :model-value="receiveMappings[JSON.parse(object.id)[1]]??'identity'" :label="object.label+'接收方式'" :disabled="busy||!!incoming?.adoption" :options="[{value:'identity',label:'按原渠道身份合并'},...(incomingSource.package.payload.policy?.channels.find(c=>c.id===JSON.parse(object.id)[1])?.connection?.nodes.length?[{value:'new',label:'建立独立新渠道'}]:[]),...localChannels.map(c=>({value:'local:'+c.id,label:'映射到 '+c.label+' · 保留本机连接和Key'}))]" @update:model-value="receiveMappings[JSON.parse(object.id)[1]]=$event;receiveDirty=true"/><label v-if="receiveMappings[JSON.parse(object.id)[1]]==='new'">新渠道名称<input v-model="receiveNewLabels[JSON.parse(object.id)[1]]" class="input" maxlength="256" :disabled="busy||!!incoming?.adoption" @input="receiveDirty=true"/></label><small>{{ receiveMappings[JSON.parse(object.id)[1]]?.startsWith('local:')?'只接收模型规则，保留本机连接与Key':receiveMappings[JSON.parse(object.id)[1]]==='new'?'独立身份，保持停用；来源Key另行选择采用':'同身份的不同端点须先映射；同名不会自动合并' }}</small></template>
      </template></ObjectScopeTree><label v-if="incomingSource.package.payload.policy?.templateRefs||incomingSource.package.payload.policy?.channelOrder||incomingSource.package.payload.policy?.defaultChannelId"><input v-model="receiveGlobal" type="checkbox" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true"/>全局采用及渠道顺序</label><p>所选渠道需要的保存版本会一并接收。</p><UiButton :loading="busy" @click="compareReceived">比较所选范围</UiButton></details>
      <div v-if="(incoming??filePlan??upload)?.manager" class="model-transfer-manager"><strong>管理器配置</strong><p v-if="(incoming??filePlan??upload)?.manager?.preferences">桌面偏好</p><p v-if="(incoming??filePlan??upload)?.manager?.extensions">扩展设置 · Skill {{ (incoming??filePlan??upload)?.manager?.extensions?.skills.length }} · MCP {{ (incoming??filePlan??upload)?.manager?.extensions?.servers.length }}（资源与凭据按包内内容处理）</p><p v-for="endpoint in (incoming??filePlan??upload)?.manager?.webdav??[]" :key="endpoint.id">{{ endpoint.username }} · {{ endpoint.url }} / {{ endpoint.remotePath }}{{ incoming?' · 导入后未启用':'' }}</p></div>
      <div v-if="(incoming??filePlan??upload)?.manager?.accounts?.length"><h3>账户信息</h3><div v-for="account in (incoming??filePlan??upload)?.manager?.accounts??[]" :key="account.id" class="model-transfer-account-summary"><strong>{{ account.label }}</strong><span>{{ accountInformationLabel(account) }}</span><small v-if="incoming">仅登记账户信息；已有登录状态保持，不代表登录已恢复。</small></div></div><div v-if="(incoming??filePlan??upload)?.assets"><h3>扩展资源</h3><p v-for="skill in (incoming??filePlan??upload)?.assets?.skills??[]" :key="skill.name">Skill · {{ incoming?.assetMappings?.skills[skill.name]??skill.name }}{{ incoming?.assetMappings?.skills[skill.name]?'（来源：'+skill.name+'）':'' }} · {{ skill.files }} 文件 · {{ skill.bytes.toLocaleString() }} 字节{{ skill.protected?' · 受保护内容':'' }}</p><p v-for="mcp in (incoming??filePlan??upload)?.assets?.mcps??[]" :key="mcp.name">MCP · {{ incoming?.assetMappings?.mcps[mcp.name]??mcp.name }}{{ incoming?.assetMappings?.mcps[mcp.name]?'（来源：'+mcp.name+'）':'' }} · {{ mcp.secretFields.length }} 个凭据字段</p><details v-if="((incoming??filePlan??upload)?.assets?.skills??[]).some(skill=>skill.warnings.length)||((incoming??filePlan??upload)?.assets?.mcps??[]).some(mcp=>mcp.warnings.length)"><summary>资源检查结果</summary><p v-for="warning in [...((incoming??filePlan??upload)?.assets?.skills??[]).flatMap(skill=>skill.warnings),...((incoming??filePlan??upload)?.assets?.mcps??[]).flatMap(mcp=>mcp.warnings)]" :key="warning">{{ warning }}</p></details></div>
      <details v-if="preview" class="model-transfer-manifest-wrap"><summary>查看内容清单</summary><p>模版 {{ preview.payload.templates.length }} · 渠道 {{ preview.payload.policy?.channels.length ?? 0 }}</p>
        <ul class="model-transfer-manifest"><li v-for="template in preview.payload.templates" :key="template.id"><strong>{{ template.label }}</strong><span>{{ template.revisions.length }} 个保存版本</span><code>{{ template.id }}</code><small v-if="incoming">{{ models.snapshot?.library.templates.some(t => t.id === template.id) ? '合并保存版本，保持本机默认' : '新增模版，不自动采用' }}</small></li><li v-for="channel in preview.payload.policy?.channels ?? []" :key="channel.id"><strong>{{ channel.label }}</strong><small v-if="channel.familyScope">模型族片段 · {{ channel.familyScope.templateIds.length }} 族 · {{ channel.familyScope.includeChannelDefaults?'含渠道共用默认字段（影响全部型号）':'共用默认值不携带，沿接收方配置解析' }}</small><small v-else-if="channel.modelSubset">型号子集 · {{ channel.modelSubset.length }} 项</small><span v-if="!channel.familyScope">{{ channel.selection.mode === 'all' ? '全部模型' : channel.selection.mode === 'none' ? '不选择模型' : '指定模型' }}</span><code>{{ channel.id }}</code><small v-if="channel.connection?.provider">原渠道配置 · {{ Object.keys(channel.connection.provider).length }} 个公开字段 · 认证 {{ channel.connection.provider.authMode??'key' }}</small><small v-for="node in channel.connection?.nodes??[]" :key="node.id">{{ node.name }} · {{ node.url }}</small><details v-if="channel.connection?.provider"><summary>渠道原设置</summary><pre class="model-transfer-fields">{{ JSON.stringify(channel.connection.provider,null,2) }}</pre></details><small v-if="incoming&&channel.familyScope">仅合并所选族；其他族、连接、Key、已有选择和顺序保持</small><small v-else-if="incoming">{{ models.snapshot?.policy.channels.some(c => c.id === channel.id) ? '同身份原设置进入停用草稿；映射到本机渠道时保留本机连接和凭据' : '新增渠道草稿，确认节点及凭据后再保存，保持停用' }}</small></li></ul>
        <p v-for="(binding,sourceId) in incoming?.channelBindings??{}" :key="sourceId">{{ preview.payload.policy?.channels.find(c=>c.id===sourceId)?.label }} → {{ binding.keepLocalConnection?localChannels.find(c=>c.id===binding.targetId)?.label:binding.label }} · {{ binding.keepLocalConnection?'保留本机连接和Key':'独立停用草稿' }}</p><p v-for="notice in incoming?.notices ?? []" :key="notice">{{ notice }}</p><code class="model-transfer-digest">{{ preview.digest }}</code></details>
      <template #actions><UiButton v-if="upload?.phase === 'published'||incoming?.adoption?.completed" variant="primary" @click="closePreview">完成</UiButton><UiButton v-else variant="primary" :loading="busy" :disabled="!!incoming&&(receiveDirty||!!incoming.previewError)" @click="confirm">{{ incoming ? incoming.adoption?'核对原导入':'采用到本机' : remote ? (upload?.phase === 'prepared' ? '确认上传' : '核对原上传') : '生成配置包' }}</UiButton></template>
    </UiDialog>
    <UiDialog panel-class="transfer-plaintext" :open="plaintextConfirmOpen" title="凭据将以明文写入" :busy="busy" @close="plaintextConfirmOpen=false">
      <p>当前范围含 {{ exportCredentialCount }} 项凭据，不加密将以明文写入该文件，任何能读取文件的人都能看到这些凭据。继续？</p><p v-if="error" class="model-error" role="alert">{{ error }}</p>
      <template #actions><UiButton :disabled="busy" @click="plaintextConfirmOpen=false">返回设置口令</UiButton><UiButton variant="primary" :loading="busy" @click="acceptPlaintextExport">仍以明文导出</UiButton></template>
    </UiDialog>
    <UiDialog panel-class="transfer-password" :open="passwordOpen" :title="remoteUnlock?'读取受保护同步快照':'打开受保护配置文件'" :busy="busy" @close="passwordOpen=false;importPassword='';emit('update:importOpen',false)">
      <p>未加密的文件无需口令，直接继续即可；加密文件才需要填写原口令。</p><label>文件保护口令（可选）<input v-model="importPassword" aria-label="导入解密口令" class="input" type="password" autocomplete="off" :disabled="busy" /></label><p v-if="error" class="model-error" role="alert">{{ error }}</p><template #actions><UiButton variant="primary" :loading="busy" @click="unlockFile">{{ importPassword ? '解密并比较' : '直接读取并比较' }}</UiButton></template>
    </UiDialog>
  </article>
</template>
<style scoped>
:global(.transfer-file-picker.ui-dialog) {width:min(520px,calc(100vw / var(--ui-zoom) - 40px))}
:global(.transfer-file-picker.ui-dialog header) {border-bottom:0;padding:20px 20px 8px}
:global(.transfer-file-picker.ui-dialog .ui-dialog-content) {padding:0 20px 14px}
:global(.transfer-file-picker.ui-dialog footer) {padding:14px 20px 20px}

fieldset{border:1px solid var(--border);border-radius:var(--radius);margin:0 0 18px;padding:14px;display:grid;gap:12px;min-width:0}
.model-transfer-account span,.model-transfer-account-summary{display:grid;gap:4px;min-width:0}.model-transfer-account-summary{margin:12px 0;overflow-wrap:anywhere}.model-transfer-account small{color:var(--muted)}
label{display:flex;align-items:center;gap:8px}
.model-error{color:var(--red);overflow-wrap:anywhere}
.model-transfer-choice{display:grid;grid-template-columns:minmax(120px,1fr) minmax(0,2fr);align-items:center;gap:12px}
.model-transfer-path{display:grid;gap:8px;overflow-wrap:anywhere}
/* 原型 sync-config.js:169：内容清单是默认收起的 details，标题行同一行垂直居中。 */
.model-transfer-manifest-wrap{margin:var(--space-2) 0 0;font-size:var(--text-caption)}
.model-transfer-manifest-wrap>summary{display:flex;align-items:center;gap:6px;cursor:pointer;color:var(--muted);list-style:none}
/* display:flex 会吃掉默认三角；按原型用 ▸ 自绘，展开时旋转，对齐「›」同款。 */
.model-transfer-manifest-wrap>summary::-webkit-details-marker{display:none}
.model-transfer-manifest-wrap>summary::before{content:"▸";flex:none;font-size:var(--text-micro);transition:transform var(--motion-fast,.12s) ease-out}
.model-transfer-manifest-wrap[open]>summary::before{transform:rotate(90deg)}
.model-transfer-manifest-wrap>summary:hover{color:var(--text)}
.model-transfer-manifest{list-style:none;padding:0;display:grid;gap:12px}
.model-transfer-manifest li{display:grid;grid-template-columns:1fr auto;gap:6px;align-items:center;border-bottom:1px solid var(--border);padding:12px 0}
.model-transfer-manifest code,.model-transfer-manifest small{grid-column:1/-1;overflow-wrap:anywhere;color:var(--muted)}
.model-transfer-fields{max-height:240px;overflow:auto;white-space:pre-wrap;overflow-wrap:anywhere;font-family:var(--font-mono,monospace);font-size:12px;line-height:18px}
.model-transfer-digest{display:block;overflow-wrap:anywhere;font-size:var(--text-small)}
.model-import-asset{padding:10px 0;display:grid;gap:8px}.model-import-asset .input{flex:1;min-width:0}
.model-import-channel{border:1px solid var(--border);border-radius:var(--radius);padding:12px;display:grid;gap:10px}.model-import-channel small{color:var(--muted)}
.model-transfer-scope{border-bottom:1px solid var(--border);padding-bottom:16px;margin-bottom:16px}.model-transfer-scope>div{display:grid;gap:8px;margin:14px 0}.model-transfer-scope>label{margin:10px 0}
@media(max-width:600px){.model-transfer-choice{grid-template-columns:1fr}}
</style>

<style scoped>
.export-destination{display:grid;gap:var(--space-1);padding-block:var(--space-2);border-top:1px solid var(--border-soft)}
.export-destination>span{color:var(--muted);font-size:var(--text-label)}
.export-destination-choice{display:flex;gap:var(--space-4);align-items:center;flex-wrap:wrap}
.export-destination-choice label{display:flex;align-items:center;gap:var(--space-2)}
.export-destination-custom{display:grid;gap:var(--space-1)}
.export-destination-row{display:grid;grid-template-columns:minmax(0,1fr) auto auto;gap:var(--space-2);align-items:center}
.export-package-layout{display:grid;grid-template-columns:minmax(0,1.15fr) minmax(0,1fr);gap:var(--space-4);align-items:start;margin-block:var(--space-4);padding:var(--space-3);border:1px solid var(--border);border-radius:var(--radius-sm);background:var(--rail)}
.export-tree{padding:0;border:0;background:transparent}
.export-tree-kicker{margin:0 0 var(--space-1-5);color:var(--muted);font-size:var(--text-caption)}
.export-tree pre{margin:0 0 var(--space-2);font-family:var(--font-code);font-size:var(--text-caption);line-height:1.6;white-space:pre-wrap;overflow-wrap:anywhere;color:var(--text)}
.export-tree p{margin:0 0 var(--space-2)}
.export-summary{min-width:0}.export-summary strong{display:block;font-size:var(--text-label);margin-bottom:var(--space-1)}
.export-summary p{margin:0 0 var(--space-2)}
.export-info{padding:var(--space-3) var(--space-3);background:var(--rail);border-radius:var(--radius-sm);margin-block:var(--space-3);line-height:1.65}
.export-encrypt{display:flex;align-items:center;gap:var(--space-2);margin-block:var(--space-3)}
.export-passphrase{display:grid;grid-template-columns:1fr 1fr;gap:0 var(--space-4)}
.export-passphrase label{display:grid;align-items:stretch;gap:var(--space-1);margin-block:var(--space-3)}
.export-plaintext-risk{margin-block:var(--space-2);color:var(--red);font-size:var(--text-label);line-height:1.6}
.export-destination-row .input{min-width:0}
.sync-head-actions{display:flex;gap:var(--space-2);flex-wrap:wrap;justify-content:flex-end}
.sync-scope-workspace{display:grid;grid-template-columns:minmax(0,1fr) 220px;gap:var(--space-5);margin-top:var(--space-4);min-width:0}
.sync-scope-main{display:grid;gap:var(--space-3);align-content:start;min-width:0}
.sync-scope-toolbar{display:grid;grid-template-columns:minmax(0,200px) minmax(0,1fr) auto;gap:var(--space-2);align-items:end;min-width:0}
.sync-scope-toolbar .sync-scope-search{min-width:0}
.sync-credentials-note{margin:0;color:var(--muted);font-size:var(--text-label);line-height:1.6}
.sync-credential-block{display:grid;gap:var(--space-1);padding-block:var(--space-2);border-top:1px solid var(--border-soft)}
.sync-credential-head,.sync-credential-inline,.sync-credential-password{display:flex;gap:var(--space-2);align-items:center;font-size:var(--text-control)}
.sync-credential-hint{margin:0;color:var(--muted);font-size:var(--text-caption);line-height:1.5}
.sync-credential-more{font-size:var(--text-label)}
.sync-credential-password{display:grid;gap:var(--space-1);align-items:stretch}
.sync-account-filter{display:grid;gap:var(--space-1);justify-items:start}
@media(max-width:1000px){.sync-scope-workspace{grid-template-columns:minmax(0,1fr)}.sync-scope-toolbar{grid-template-columns:minmax(0,1fr)}}
.sync-scope-controls{display:flex;flex-wrap:wrap;gap:var(--space-2)}.sync-scope-controls>*{flex:1 1 200px;min-width:0}.sync-scope-objects,.sync-scope-credentials{display:grid;align-content:start;gap:var(--space-3);min-width:0}
.sync-scope-credentials{border-inline-start:1px solid var(--border);padding-inline-start:var(--space-4)}
.sync-scope-credentials h3{margin:0;font-size:var(--text-body)}.sync-scope-credentials label{font-size:var(--text-control)}
@media(max-width:1000px){.sync-scope-workspace{grid-template-columns:minmax(0,1fr)}.sync-scope-credentials{border-inline-start:0;border-top:1px solid var(--border);padding-inline-start:0;padding-top:var(--space-3)}}
@media(max-width:720px){.export-package-layout{grid-template-columns:minmax(0,1fr)}.export-passphrase{grid-template-columns:minmax(0,1fr)}}
</style>
