<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref,watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ObjectScopeTree,{type ScopeObject} from './ObjectScopeTree.vue'
import ChannelModelScope from './ChannelModelScope.vue'
import { useModelsStore } from '@/features/models/store'
import {useAppStore} from '@/stores/app'
import { errorText, versionLabel, type ChannelPolicy, type ModelPolicy } from '@/features/models/api'
import { modelTransferApi,assetDetailApi,type AssetDetail, transferFileDialog, cloudCatalogApi,type CloudSyncCatalog, transferResultApi, importCredentialChoiceApi, transferProtectionApi, transferChoiceApi, type TransferChoice,type TransferResult, scopeFor,remoteScope,defaultIncludedMcp,type McpConflict, accountInformationLabel, accountInformationSupported, accountTransferLabel, accountPlatforms, type AccountCatalogEntry, type ChannelImportMapping, type DraftCandidate, type FamilyScope, type FilePlan, type IncomingPlan, type ManagerScope, type RemoteOptions, type TransferScope, type UploadPlan, type SyncChange, type SyncPreview } from './modelTransfer'
import {getSyncConfig} from './api'
import {useSyncStore} from './store'
import {recordSyncActivity,recordRemoteVersion} from './activity'
import {clientLabels} from '@/features/extensions/presentation'
import type {ExtensionClientId} from '@/features/extensions/api'
import { displayPath } from '@/lib/paths'
const props = defineProps<{ retryReceipt?:TransferResult; refreshReceipt?:TransferResult; source?:'saved'|'draft'; remote?: boolean; importOnly?: boolean; importOpen?: boolean; actionOnly?: boolean }>()
const emit = defineEmits<{ 'update:importOpen': [value: boolean]; close:[] }>()
const pickOpen = ref(false)
const models = useModelsStore()
const app=useAppStore();const syncStore=useSyncStore()
const busy = ref(false)
const error = ref('')
const previewInvalid=ref(false)
let requestedResultId:string|null=null
const currentResultId=computed(()=>props.remote?syncStore.webdavResultId:syncStore.fileResultId)
const templateIds = ref<string[]>([])
const channelIds = ref<string[]>([])
const channelFamilies=ref<Record<string,FamilyScope>>({});
const channelModels=ref<Record<string,string[]>>({});const receiveChannelModels=ref<Record<string,string[]>>({})
const channelCatalog=ref<ChannelPolicy[]|null>(null);const channelError=ref('')
const savedChannels=computed(()=>exportSource.value==='draft'?(models.channelDraft?.channels??[]):channelCatalog.value??models.snapshot?.policy.channels??[])
const versions = ref<Record<string,string>>({})
const includeGlobal = computed(()=>exportSource.value!=='draft'&&channelIds.value.length>0&&Object.keys(channelFamilies.value).length===0)
const cloudCatalog=ref<CloudSyncCatalog|null>(null);const cloudCatalogError=ref('')
async function loadCloudCatalog(){if(!props.remote)return;try{cloudCatalog.value=await cloudCatalogApi.load();cloudCatalogError.value=''}catch(cause){cloudCatalogError.value=errorText(cause)}}
const assetDetails=ref<Record<string,AssetDetail>>({});const assetDetailError=ref<Record<string,string>>({});const assetDetailLoading=ref('')
async function readObjectDetail(object:ScopeObject){if(!['skills','mcps'].includes(object.category))return;const remote=cloudCatalog.value?.objects.find(o=>o.id===object.id);if(remote&&!localScopeObjects.value.some(o=>o.id===object.id)){assetDetails.value={...assetDetails.value,[object.id]:{source:'云端已保存内容',fields:['实际资源与所选凭据'],files:remote.files??[],bytes:0,clients:[],conflicts:[],warnings:[]}};return}assetDetailLoading.value=object.id;assetDetailError.value={...assetDetailError.value,[object.id]:''};try{const [,name]=JSON.parse(object.id) as [string,string];const value=await assetDetailApi.load(object.category,name,mcpSources.value[name]);assetDetails.value={...assetDetails.value,[object.id]:value}}catch(cause){assetDetailError.value={...assetDetailError.value,[object.id]:errorText(cause)}}finally{if(assetDetailLoading.value===object.id)assetDetailLoading.value=''}}
const selectionReady=ref(false);const choiceError=ref('');const credentialIds=ref<string[]>([])
let choiceWrite=Promise.resolve();let choiceRevision=0
const selectionOpen = ref(true)
const filePlan = ref<FilePlan | null>(null)
// Frozen packages are necessary for confirmation/retry identity, but an
// unattempted preview must not remain as a plaintext cache after cancellation.
// Keep every prepared/protected plan created by this dialog instance; backend
// discard is idempotent and refuses plans that already have an export attempt.
const pendingFilePlanIds = new Set<string>()
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
const exportSource=computed(()=>props.source??'saved')
const importPassword=ref('');const passwordOpen=ref(false);const importPath=ref('')
const recompareId=ref('');const openingOriginal=ref(false)
const remoteUnlock=ref(false);const remoteScopeLoaded=ref(!props.remote);const scopeReadError=ref('')
const receiveCredentialIds=ref<string[]>([])
const importCredentialPrefs=ref({keys:true,extensions:true});const importPrefsReady=ref(false)
const exportKeys=ref(false);const adoptKeys=ref(false);let unlockedPassword=''
const managerPreferences=ref(true);const managerExtensions=ref(false);const managerWebdav=ref(true);const webdavPasswords=ref(true);const activeWebdav=ref(false)
const receivePreferences=ref(false);const receiveExtensions=ref(false);const receiveEndpoints=ref<string[]>([]);const adoptWebdavPasswords=ref(true)
const skillNames=ref<string[]>([]);const mcpNames=ref<string[]>([]);const extensionCredentials=ref(false);const assetCatalog=ref<{skills:string[];mcps:string[];credentialSkills?:string[];credentialMcps?:string[]}>({skills:[],mcps:[]});const assetError=ref('')
const receiveMappings=ref<Record<string,string>>({});const receiveNewLabels=ref<Record<string,string>>({})
const localChannels=computed(()=>models.channelDraft?.channels??models.snapshot?.policy.channels??[])
const receiveSkillTargets=ref<Record<string,string>>({});const receiveMcpTargets=ref<Record<string,string>>({})
const receiveSkills=ref<string[]>([]);const receiveMcps=ref<string[]>([]);const adoptExtensionCredentials=ref(false)
const accountPlatform=ref('all');const accountIds=ref<string[]>([]);const accountCatalog=ref<AccountCatalogEntry[]>([]);const accountError=ref('');const receiveAccounts=ref<string[]>([])
function scopeAccounts(id:string){const value=accountCatalog.value.find(a=>a.account.id===JSON.parse(id)[1])?.account??cloudCatalog.value?.accounts?.find(a=>a.id===JSON.parse(id)[1]);return value?[value]:[]}
const allTemplates=ref(false);const allChannels=ref(false);const allSkills=ref(false);const allMcps=ref(false)
// S18-03：同名 MCP 在多个客户端定义不同时由用户选「采用来源」；未选的只跳过该 MCP。
const mcpSources=ref<Record<string,string>>({});const mcpConflicts=ref<McpConflict[]>([])
function selectedMcpSources(){return Object.fromEntries(Object.entries(mcpSources.value).filter(([name])=>mcpNames.value.includes(name)||allMcps.value&&defaultIncludedMcp(name)))}
function noteConflicts(plan?:{assets?:{conflicts?:McpConflict[]}|null}|null){const merged=new Map(mcpConflicts.value.map(c=>[c.name,c]));for(const c of plan?.assets?.conflicts??[])merged.set(c.name,c);mcpConflicts.value=[...merged.values()]}
const clientName=(c:string)=>clientLabels[c as ExtensionClientId]??c
const hasAssets=computed(()=>skillNames.value.length>0||mcpNames.value.length>0||props.remote&&(allSkills.value||allMcps.value))
const keyChannelIds=computed(()=>(allChannels.value?savedChannels.value.map(c=>c.id):channelIds.value).filter(id=>!channelFamilies.value[id]&&!['oauth','forward'].includes(savedChannels.value.find(c=>c.id===id)?.connection.authMode??'')));
const credentialChannels=computed(()=>keyChannelIds.value.map(id=>({id:'channel:'+id,label:savedChannels.value.find(c=>c.id===id)?.label??cloudCatalog.value?.objects.find(o=>o.id===scopeKey('channels',id))?.label??id})))
const credentialExtensions=computed(()=>[...skillNames.value.filter(id=>assetCatalog.value.credentialSkills?.includes(id)??true).map(id=>({id:'skill:'+id,label:'Skill · '+id})),...mcpNames.value.filter(id=>assetCatalog.value.credentialMcps?.includes(id)||cloudCatalog.value?.credentialIds.includes('mcp:'+id)||assetCatalog.value.credentialMcps===undefined).map(id=>({id:'mcp:'+id,label:'MCP · '+id}))])
function setCredentialGroup(kind:'channel'|'extension',checked:boolean){const rows=kind==='channel'?credentialChannels.value:credentialExtensions.value;const ids=new Set(credentialIds.value);for(const row of rows){if(checked)ids.add(row.id);else ids.delete(row.id)}credentialIds.value=[...ids];if(kind==='channel')exportKeys.value=checked;else extensionCredentials.value=checked}
function toggleCredential(id:string,checked:boolean){const next=new Set(credentialIds.value);if(checked)next.add(id);else next.delete(id);credentialIds.value=[...next];exportKeys.value=credentialChannels.value.some(x=>next.has(x.id));extensionCredentials.value=credentialExtensions.value.some(x=>next.has(x.id))}

watch(()=>JSON.stringify([credentialChannels.value,credentialExtensions.value,credentialIds.value]),()=>{exportKeys.value=credentialChannels.value.some(x=>credentialIds.value.includes(x.id));extensionCredentials.value=credentialExtensions.value.some(x=>credentialIds.value.includes(x.id))})
function credentialState(kind:'channel'|'extension'){const rows=kind==='channel'?credentialChannels.value:credentialExtensions.value;const count=rows.filter(x=>credentialIds.value.includes(x.id)).length;return !count?false:count===rows.length?true:'mixed'}
let disposed=false
onBeforeUnmount(()=>{disposed=true;if(!busy.value)void discardPendingFilePlans();exportPackagePassword.value='';exportPackagePasswordAgain.value='';importPassword.value='';unlockedPassword=''})
const hasScope = computed(() => accountIds.value.length>0 || templateIds.value.length > 0 || channelIds.value.length > 0 || includeGlobal.value||managerPreferences.value||managerExtensions.value||!props.remote&&managerWebdav.value&&activeWebdav.value||hasAssets.value||props.remote&&(allTemplates.value||allChannels.value))
const familyScopesValid=computed(()=>Object.values(channelFamilies.value).every(scope=>scope.templateIds.length>0));
const preview = computed(() => incoming.value?.package ?? upload.value?.package ?? filePlan.value?.package)
const reviewOpen=computed(()=>pickOpen.value||passwordOpen.value||!!preview.value||!!(props.actionOnly&&error.value))
const scopeKey=(category:string,id:string)=>JSON.stringify([category,id])
const dependencyTemplates=computed(()=>{const out=new Map<string,Set<string>>();for(const id of channelIds.value){const channel=savedChannels.value.find(c=>c.id===id);for(const r of [...(models.snapshot?.policy.templateRefs??[]),...(channel?.templateRefs??[]),...Object.values(channel?.modelRefs??{})]){if(!out.has(r.templateId))out.set(r.templateId,new Set());out.get(r.templateId)!.add(r.revisionId)}}return out})
function choiceScopeKeys(){return scopeKeys.value.filter(key=>{const [kind,id]=JSON.parse(key) as [string,string];return kind!=='templates'||templateIds.value.includes(id)||!dependencyTemplates.value.has(id)})}
const localScopeObjects=computed<ScopeObject[]>(()=>[
 {id:scopeKey('manager','preferences'),category:'manager',categoryLabel:'桌面偏好',label:'通用偏好',detail:'主题与可移植显示偏好',mark:'配置'},
 ...(!props.remote&&activeWebdav.value?[{id:scopeKey('manager','webdav'),category:'endpoints',categoryLabel:'WebDAV 配置',label:'WebDAV连接配置',detail:'文件接收后为未启用连接',mark:webdavPasswords.value?'含密码':'仅配置'}]:[]),
 ...(models.snapshot?.library.templates??[]).map(t=>({id:scopeKey('templates',t.id),category:'templates',categoryLabel:'模型模版',label:t.label,detail:dependencyTemplates.value.has(t.id)&&!templateIds.value.includes(t.id)?'所选渠道需要的精确保存版本':'全部已保存版本；未保存编辑不进入',mark:dependencyTemplates.value.has(t.id)&&!templateIds.value.includes(t.id)?'依赖补齐':'配置',disabled:props.remote&&allTemplates.value||dependencyTemplates.value.has(t.id)&&!templateIds.value.includes(t.id)})),
 ...savedChannels.value.map(c=>({id:scopeKey('channels',c.id),category:'channels',categoryLabel:exportSource.value==='draft'?'渠道草稿':'渠道与模型',label:c.label,detail:exportSource.value==='draft'?'未提交草稿；精确模版依赖一并导出':'原设置、服务节点、模型规则及精确模版依赖',mark:credentialIds.value.includes('channel:'+c.id)&&!channelFamilies.value[c.id]?'含凭据':'不带 Key',disabled:props.remote&&allChannels.value})),
 ...accountCatalog.value.map(a=>({id:scopeKey('accounts',a.account.id),category:'accounts',categoryLabel:'账户',label:a.account.label,detail:accountInformationLabel(a.account),mark:a.informationSupported?'仅账户信息':'暂不支持',disabled:!a.informationSupported})),
 ...assetCatalog.value.skills.map(name=>({id:scopeKey('skills',name),category:'skills',categoryLabel:'Skills',label:'Skill · '+name,detail:'实际SKILL.md与所需资源',mark:credentialIds.value.includes('skill:'+name)?'含凭据':'配置',disabled:props.remote&&allSkills.value})),
 ...assetCatalog.value.mcps.map(name=>({id:scopeKey('mcps',name),category:'mcps',categoryLabel:'MCP',label:'MCP · '+name,detail:defaultIncludedMcp(name)?'完整源定义；不自动执行':'由客户端应用维护 · 含本机路径',mark:credentialIds.value.includes('mcp:'+name)?'含凭据':'配置',disabled:props.remote&&allMcps.value&&defaultIncludedMcp(name)})),
].filter(item=>exportSource.value!=='draft'||['templates','channels'].includes(item.category)))
const scopeObjects=computed<ScopeObject[]>(()=>[...localScopeObjects.value,...(props.remote?cloudCatalog.value?.objects??[]:[]).filter(o=>!localScopeObjects.value.some(local=>local.id===o.id))])
const unselectedCloudCount=computed(()=>(cloudCatalog.value?.objects??[]).filter(o=>!scopeKeys.value.includes(o.id)).length)
const accountFilterIds=computed(()=>accountPlatform.value==='all'?undefined:scopeObjects.value.filter(o=>o.category!=='accounts'||accountCatalog.value.find(a=>a.account.id===JSON.parse(o.id)[1])?.account.origin?.accountPlatform===accountPlatform.value).map(o=>o.id))
const scopeKeys=computed(()=>[
 ...(managerPreferences.value||managerExtensions.value?[scopeKey('manager','preferences')]:[]),...(!props.remote&&managerWebdav.value&&activeWebdav.value?[scopeKey('manager','webdav')]:[]),
 ...[...new Set([...(allTemplates.value?(models.snapshot?.library.templates??[]).map(t=>t.id):templateIds.value),...dependencyTemplates.value.keys()])].map(id=>scopeKey('templates',id)),
 ...(allChannels.value?savedChannels.value.map(c=>c.id):channelIds.value).map(id=>scopeKey('channels',id)),...accountIds.value.map(id=>scopeKey('accounts',id)),
 ...(allSkills.value?assetCatalog.value.skills:skillNames.value).map(id=>scopeKey('skills',id)),...(allMcps.value?assetCatalog.value.mcps.filter(n=>defaultIncludedMcp(n)||mcpNames.value.includes(n)):mcpNames.value).map(id=>scopeKey('mcps',id)),
])
const receiveObjects=computed<ScopeObject[]>(()=>{
 const source=incomingSource.value;if(!source)return []
 return [
  ...(source.manager?.preferences||source.manager?.extensions&&(!!props.retryReceipt||openingOriginal.value)?[{id:scopeKey('manager','preferences'),category:'manager',categoryLabel:'桌面偏好',label:'通用偏好'}]:[]),
  ...(source.manager?.webdav??[]).map(e=>({id:scopeKey('endpoints',e.id),category:'endpoints',categoryLabel:'WebDAV连接',label:e.username+' · '+e.url,detail:e.remotePath+' · 采用后未启用'})),
  ...(source.manager?.accounts??[]).map(a=>({id:scopeKey('accounts',a.id),category:'accounts',categoryLabel:'账户',label:a.label,detail:accountInformationLabel(a)+'；不代表登录已恢复',disabled:!accountInformationSupported(a)})),
  ...source.package.payload.templates.map(t=>({id:scopeKey('templates',t.id),category:'templates',categoryLabel:'模型模版',label:t.label,detail:t.revisions.length+' 个已保存版本；不切换本机主版本'})),
  ...(source.package.payload.policy?.channels??[]).map(c=>({id:scopeKey('channels',c.id),category:'channels',categoryLabel:'渠道与模型',label:c.label,detail:'所需精确模版版本一并接收'})),
  ...(source.assets?.skills??[]).map(s=>({id:scopeKey('skills',s.name),category:'skills',categoryLabel:'Skills',label:'Skill · '+s.name,detail:s.files+' 文件 · '+s.bytes+' 字节'})),
  ...(source.assets?.mcps??[]).map(m=>({id:scopeKey('mcps',m.name),category:'mcps',categoryLabel:'MCP',label:'MCP · '+m.name,detail:'完整源定义；不自动执行'})),
 ]
})
const receiveKeys=computed(()=>[
 ...(receivePreferences.value||receiveExtensions.value?[scopeKey('manager','preferences')]:[]),
 ...receiveEndpoints.value.map(id=>scopeKey('endpoints',id)),...receiveAccounts.value.map(id=>scopeKey('accounts',id)),...[...new Set(receiveRefs.value.map(id=>(JSON.parse(id) as [string,string])[0]))].map(id=>scopeKey('templates',id)),...receiveChannels.value.map(id=>scopeKey('channels',id)),...receiveSkills.value.map(id=>scopeKey('skills',id)),...receiveMcps.value.map(id=>scopeKey('mcps',id)),
])
function updateReceiveKeys(keys:string[]){
 const ids=(kind:string)=>keys.map(k=>JSON.parse(k) as [string,string]).filter(v=>v[0]===kind).map(v=>v[1]);
 receivePreferences.value=!!incomingSource.value?.manager?.preferences&&ids('manager').includes('preferences');receiveExtensions.value=!!incomingSource.value?.manager?.extensions&&(!!props.retryReceipt||openingOriginal.value);receiveEndpoints.value=ids('endpoints');receiveAccounts.value=ids('accounts');receiveRefs.value=(incomingSource.value?.package.payload.templates??[]).filter(t=>ids('templates').includes(t.id)).flatMap(t=>t.revisions.map(r=>JSON.stringify([t.id,r.id])));receiveChannels.value=ids('channels');receiveSkills.value=ids('skills');receiveMcps.value=ids('mcps');receiveDirty.value=true;queueComparison();
}
const incomingCredentialRows=computed(()=>[...(incoming.value?.availableKeys??[]).map(id=>({id:'channel:'+id,label:'渠道 · '+(incoming.value?.package.payload.policy?.channels.find(c=>c.id===id)?.label??id)})),...(incoming.value?.availableExtensionIds??[]).map(id=>({id,label:(id.startsWith('skill:')?'Skill · ':'MCP · ')+id.slice(id.indexOf(':')+1)}))])
function setIncomingCredentialGroup(kind:'channel'|'extension',checked:boolean){const rows=incomingCredentialRows.value.filter(r=>(kind==='channel')===r.id.startsWith('channel:'));const ids=new Set(receiveCredentialIds.value);for(const row of rows){if(checked)ids.add(row.id);else ids.delete(row.id)}receiveCredentialIds.value=[...ids];if(!props.remote&&importPrefsReady.value){const prefs={...importCredentialPrefs.value,[kind==='channel'?'keys':'extensions']:checked};importCredentialPrefs.value=prefs;void importCredentialChoiceApi.save(prefs).catch(()=>{error.value='导入凭据偏好未保存，当前选择保留。'})}}
function syncIncomingCredentialFlags(){adoptKeys.value=incomingCredentialRows.value.some(r=>r.id.startsWith('channel:')&&receiveCredentialIds.value.includes(r.id));adoptExtensionCredentials.value=incomingCredentialRows.value.some(r=>!r.id.startsWith('channel:')&&receiveCredentialIds.value.includes(r.id))}
watch(()=>JSON.stringify([receiveCredentialIds.value,incomingCredentialRows.value]),syncIncomingCredentialFlags)
const selectedCredentialKinds=computed(()=>[...(exportKeys.value?['渠道认证（'+keyChannelIds.value.length+'个渠道）']:[]),...(hasAssets.value&&extensionCredentials.value?['所选扩展凭据']:[]),...(!props.remote&&managerWebdav.value&&activeWebdav.value&&webdavPasswords.value?['WebDAV连接密码']:[])])
// S18-01：同步预览按原型列出变化项；下载与导入默认勾选更新前备份。
const backupBeforeUpdate=ref(true)
const directionReason=ref('')
const syncPreview=computed<SyncPreview|null>(()=>props.remote?((incoming.value??upload.value)?.sync??null):null)
const syncChanges=computed(()=>syncPreview.value?.changes)
const importChanges=computed(()=>incoming.value?.comparison?.changes)
const pendingCount=computed(()=>mcpConflicts.value.filter(c=>!mcpSources.value[c.name]).length)
function pickerClients(c:SyncChange){return c.mark==='conflict'?c.clients??[]:c.kind==='mcp'?mcpConflicts.value.find(x=>x.name===c.id)?.clients??[]:[]}
function endpointLabel(value:string){try{const url=new URL(value);return url.origin+url.pathname+(url.search?'（请求参数省略）':'')}catch{return '地址格式待检查'}}
function syncTime(value:string){const date=new Date(value);return Number.isNaN(date.getTime())?'—':date.toLocaleString('sv-SE').slice(0,16)}
function changeSummary(c:SyncChange){
 if(c.mark==='conflict')return '同名定义在 '+(c.clients??[]).map(clientName).join('、')+' 中不同（差异字段：'+(c.fields??[]).join('、')+'），请选择来源'
 if(c.mark==='new')return '新增配置'
 switch(c.kind){
  case 'channels':if(c.fields?.includes('credentials')&&c.before===undefined)return '更新所选渠道 API Key 与认证配置';return typeof c.before==='number'&&typeof c.after==='number'?'更新已选模型 '+c.before+' → '+c.after+'；应用来源中的模型选择、顺序与服务节点':'更新服务节点、模型选择与顺序'+(c.fields?.includes('credentials')?'及所选认证配置':'')
  case 'templates':return typeof c.version==='number'?'更新模版内容至 v'+c.version+'，已有历史版本保留':'更新已保存模版内容'
  case 'accounts':return '更新账户信息，保留本机登录'
  case 'skills':return [['新增',c.filesAdded],['更新',c.filesUpdated],['移除',c.filesRemoved]].filter(([,files])=>Array.isArray(files)&&files.length).map(([verb,files])=>verb+' '+(files as string[]).join('、')).join('；')||'更新 Skill 文件内容'
  case 'mcp':return '替换MCP定义与所选凭据'
  case 'preferences':return '更新所选桌面偏好'
  case 'protection':return '按本机选择更新同步内容保护'
  default:return '更新所选配置'
 }
}
function markText(c:SyncChange){return c.mark==='conflict'?'选择来源':c.mark==='new'?'新增':'更新'}
// SY-2：范围首项与原型一致从「默认配置范围」开始；读取失败也保留 all，并就地报错。
const catalogueLoading=ref(true);let fileScopeInitialized=false
// 搜索与范围预设同处一条工具栏；对象树只消费这个值。
const scopeQuery=ref('')
/** 留空写受管导出目录；选择后按该目录另存。 */
const exportTargetDir=ref('')
const remotePassword=ref('');const remoteEncrypted=ref(false);const remoteHasPassword=ref(false);const protectionError=ref('')
// SY-1 / N-SY-2：导出目的地与口令都在确认弹窗内；默认目录与自定义目录互斥，口令可选。
const destinationMode=ref<'default'|'custom'>('default')
const encryptExport=ref(false)
const exportPackagePassword=ref('');const exportPackagePasswordAgain=ref('')
type PreparedExport={kind:'file'|'remote';scope:TransferScope;includeKeys:boolean;managerScope:ManagerScope;draft?:DraftCandidate;policy?:ModelPolicy;options?:RemoteOptions}
const preparedExport=ref<PreparedExport|null>(null)
const exportCredentialCount=computed(()=>{
 const counts=filePlan.value?.credentialCounts;if(counts)return counts.channelKeys+counts.extensions+counts.webdavPasswords
 const reported=filePlan.value?.credentialCount
 if(typeof reported==='number')return reported
 return (exportKeys.value?keyChannelIds.value.length:0)+((hasAssets.value&&extensionCredentials.value)?1:0)+((!props.remote&&managerWebdav.value&&activeWebdav.value&&webdavPasswords.value)?1:0)
})
const exportCredentialSummary=computed(()=>[
 ['渠道 Key',filePlan.value?.credentialCounts?.channelKeys??(exportKeys.value?credentialChannels.value.filter(x=>credentialIds.value.includes(x.id)).length:0)],
 ['账户登录态',0],
 ['扩展凭据',filePlan.value?.credentialCounts?.extensions??(extensionCredentials.value?credentialExtensions.value.filter(x=>credentialIds.value.includes(x.id)).length:0)],
 ['WebDAV 密码',filePlan.value?.credentialCounts?.webdavPasswords??(!props.remote&&managerWebdav.value&&activeWebdav.value&&webdavPasswords.value?1:0)],
])
watch(encryptExport,value=>{if(!value){exportPackagePassword.value='';exportPackagePasswordAgain.value=''}})
watch(destinationMode,value=>{if(value==='default')exportTargetDir.value=''})
const exportGroupLabels=computed(()=>[...new Set(scopeObjects.value.filter(item=>scopeKeys.value.includes(item.id)).map(item=>item.categoryLabel))].join(' / '))
// 原型里 WebDAV 只有一个「同步」按钮，方向由较新版本决定。
const remoteActionsDisabled=computed(()=>!selectionReady.value||!!choiceError.value||busy.value||!models.snapshot||catalogueLoading.value||!remoteScopeLoaded.value||!hasScope.value||!familyScopesValid.value)
function updateScopeKeys(keys:string[]){
 const required=new Set(dependencyTemplates.value.keys());const explicitTemplates=new Set(templateIds.value);const previous=new Set(scopeKeys.value);const next=new Set(keys);const changed=new Set([...previous,...next].filter(k=>previous.has(k)!==next.has(k)).map(k=>(JSON.parse(k) as [string,string])[0]))
 const ids=(category:string)=>keys.map(k=>JSON.parse(k) as [string,string]).filter(v=>v[0]===category).map(v=>v[1])
 if(changed.has('manager')){managerPreferences.value=ids('manager').includes('preferences');managerExtensions.value=false;const selected=ids('manager').includes('webdav');if(selected&&!managerWebdav.value)webdavPasswords.value=true;managerWebdav.value=selected}
 if(changed.has('templates')){allTemplates.value=false;templateIds.value=ids('templates').filter(id=>!required.has(id)||explicitTemplates.has(id))}
 if(changed.has('channels')){allChannels.value=false;channelIds.value=ids('channels');channelModels.value=Object.fromEntries(Object.entries(channelModels.value).filter(([id])=>channelIds.value.includes(id)));channelFamilies.value=Object.fromEntries(Object.entries(channelFamilies.value).filter(([id])=>channelIds.value.includes(id)))}
 if(changed.has('accounts'))accountIds.value=ids('accounts')
 if(changed.has('skills')){allSkills.value=false;skillNames.value=ids('skills')}
 if(changed.has('mcps')){const toggled=[...previous,...next].filter(k=>previous.has(k)!==next.has(k)).map(k=>JSON.parse(k) as [string,string]).filter(v=>v[0]==='mcps').map(v=>v[1]);if(toggled.some(defaultIncludedMcp))allMcps.value=false;mcpNames.value=ids('mcps')}

}
function initializeDefaultScope(){
 channelModels.value={};channelFamilies.value={};if(exportSource.value==='draft'){templateIds.value=[];channelIds.value=savedChannels.value.map(c=>c.id);return}
 managerPreferences.value=true;managerExtensions.value=false;managerWebdav.value=!props.remote;webdavPasswords.value=true;
 templateIds.value=(models.snapshot?.library.templates??[]).map(t=>t.id);channelIds.value=savedChannels.value.map(c=>c.id);accountIds.value=accountCatalog.value.filter(a=>a.informationSupported).map(a=>a.account.id);skillNames.value=[...assetCatalog.value.skills];mcpNames.value=assetCatalog.value.mcps.filter(defaultIncludedMcp);
 allTemplates.value=false;allChannels.value=false;allSkills.value=false;allMcps.value=false;exportKeys.value=false;extensionCredentials.value=false;
}

onMounted(async () => { if(!props.importOnly&&!props.actionOnly){void transferResultApi.readMode(props.remote?'webdav':'file').then(value=>{if(value)syncStore.rememberResult(value.id,!!props.remote)}).catch(()=>{})}if(props.retryReceipt||props.refreshReceipt){await loadRetry();return}if(props.importOnly&&!props.actionOnly)return; try{if (!models.snapshot) await models.load();if(!props.remote)await getSyncConfig().then(config=>{activeWebdav.value=!!config.endpoint}).catch(()=>{});else await loadScope()}finally{catalogueLoading.value=false;await begin()} })
// S18-01：概览卡片的「同步 / 导出」进入本页后执行一次；条件未满足时就地提示，不静默忽略。
watch(()=>[syncStore.pendingAction,catalogueLoading.value,remoteScopeLoaded.value,selectionReady.value] as const,()=>{
 if(props.retryReceipt||props.refreshReceipt||props.importOnly&&!props.actionOnly||catalogueLoading.value||!selectionReady.value)return
 if(props.remote){
  if(remoteScopeLoaded.value&&syncStore.takeAction('sync')){
   if(remoteActionsDisabled.value)error.value='请先选择同步范围。'
   else startSync()
  }
 }else if(syncStore.takeAction('export')){
  if(!(exportSource.value==='draft'&&!channelIds.value.length))prepare()
 }
},{immediate:true})
async function loadAssets(){try{const catalog=await modelTransferApi.extensionCatalog();if(!catalog||!Array.isArray(catalog.skills)||!Array.isArray(catalog.mcps)||![...catalog.skills,...catalog.mcps].every(name=>typeof name==='string'))throw {message:'扩展目录返回结构无效，请重新读取；未扩大配置范围'};assetCatalog.value=catalog;assetError.value=''}catch(cause){assetError.value=errorText(cause)}}
function assetScope(){return hasAssets.value?{skillNames:skillNames.value,mcpNames:mcpNames.value,extensionCredentials:extensionCredentials.value,credentialIds:credentialIds.value,...(Object.keys(selectedMcpSources()).length?{mcpSources:selectedMcpSources()}:{})}:{}}
function remoteOptions(){return {...(accountIds.value.length?{accountIds:accountIds.value}:{}),preferences:managerPreferences.value,extensions:managerExtensions.value,includeKeys:exportKeys.value,credentialIds:credentialIds.value,...assetScope(),...(allTemplates.value||allChannels.value||allSkills.value||allMcps.value?{allTemplates:allTemplates.value,allChannels:allChannels.value,allSkills:allSkills.value,allMcps:allMcps.value}:{}),passphrase:remoteEncrypted.value?remotePassword.value||undefined:undefined}}
async function loadScope(){try{const scope=await modelTransferApi.loadRemoteScope();accountIds.value=scope.accountIds??[];templateIds.value=scope.templateIds;channelIds.value=scope.channelIds;channelModels.value=scope.channelModels??{};channelFamilies.value=scope.channelFamilies??{};managerPreferences.value=scope.preferences;managerExtensions.value=false;exportKeys.value=scope.includeKeys;skillNames.value=scope.skillNames??[];mcpNames.value=scope.mcpNames??[];mcpSources.value=scope.mcpSources??{};extensionCredentials.value=scope.extensionCredentials??false;allTemplates.value=scope.allTemplates??false;allChannels.value=scope.allChannels??false;allSkills.value=scope.allSkills??false;allMcps.value=scope.allMcps??false;remoteScopeLoaded.value=true;scopeReadError.value=''}catch(cause){remoteScopeLoaded.value=false;scopeReadError.value=errorText(cause)}}
function currentChoice():TransferChoice{return {preferences:managerPreferences.value,extensions:managerExtensions.value,selectedIds:choiceScopeKeys(),credentialIds:[...credentialIds.value],versions:{...versions.value},mcpSources:{...mcpSources.value},includeGlobal:includeGlobal.value,webdavPasswords:webdavPasswords.value,channelModels:{...channelModels.value},channelFamilies:{...channelFamilies.value}}}
function restoreChoice(choice:TransferChoice){updateScopeKeys(choice.selectedIds.filter(id=>id!==scopeKey('manager','extensions')));managerPreferences.value=choice.preferences??choice.selectedIds.includes(scopeKey('manager','preferences'));managerExtensions.value=false;channelModels.value=choice.channelModels??{};channelFamilies.value=choice.channelFamilies??{};credentialIds.value=[...choice.credentialIds];versions.value={...choice.versions};mcpSources.value={...choice.mcpSources};webdavPasswords.value=choice.webdavPasswords;exportKeys.value=credentialChannels.value.some(x=>credentialIds.value.includes(x.id));extensionCredentials.value=credentialExtensions.value.some(x=>credentialIds.value.includes(x.id))}
async function persistChoice(){if(!selectionReady.value||props.importOnly||exportSource.value==='draft'||props.remote&&!remoteScopeLoaded.value)return;const current=++choiceRevision;const choice=JSON.parse(JSON.stringify(currentChoice()));choiceWrite=choiceWrite.catch(()=>{}).then(async()=>{try{await transferChoiceApi.save(!!props.remote,choice);if(current===choiceRevision)choiceError.value=''}catch{choiceError.value='本机范围未保存，请重试；当前选择已保留。'}});await choiceWrite}
watch(()=>JSON.stringify([scopeKeys.value,exportKeys.value,extensionCredentials.value,credentialIds.value,versions.value,mcpSources.value,webdavPasswords.value,channelModels.value,channelFamilies.value]),()=>{void persistChoice()})
function transferScope(){return scopeFor(models.snapshot!,templateIds.value,allChannels.value?savedChannels.value.map(c=>c.id):channelIds.value,includeGlobal.value,exportSource.value==='draft'?versions.value:{},channelModels.value,channelFamilies.value)}
/** S18-01：保存本机范围后按较新版本决定方向：本机新则预览上传，云端新则预览下载。 */
async function startSync(){
 if(remoteActionsDisabled.value)return
 if(remoteEncrypted.value&&!remoteHasPassword.value&&!remotePassword.value){protectionError.value='请填写保护口令';return}
 try{const protection=await transferProtectionApi.save(remoteEncrypted.value,remotePassword.value||undefined);remoteHasPassword.value=protection.hasPassword;protectionError.value=''}catch(cause){protectionError.value=errorText(cause);return}
 const state={source:''}
 await perform(async()=>{
  await persistChoice()
  if(choiceError.value)throw {message:choiceError.value}
  await modelTransferApi.saveRemoteScope(remoteScope(transferScope(),remoteOptions()))
  const check=await modelTransferApi.checkSync();await loadCloudCatalog();state.source=check.source;directionReason.value=check.reason==='both_changed_remote_commit'?'两端均在共同基线后修改；本次以云端已提交快照为来源，更新前可备份本机所选内容。':check.reason==='no_common_baseline'?'尚无共同内容基线；本次读取云端已有快照，确认后建立基线。':''
 })
 if(state.source==='local')await prepare()
 else if(state.source==='remote')await receive()
}
async function perform(action: () => Promise<void>) {
  if (busy.value) return
  busy.value = true; error.value = '';requestedResultId=null
  try { await action() } catch (cause) { error.value = errorText(cause);const id=requestedResultId;if(id){void transferResultApi.read(id).then(value=>{if(value)syncStore.rememberResult(id,!!props.remote)}).catch(()=>{})} } finally { busy.value = false;if(disposed)void discardPendingFilePlans() }
}
async function begin() {
 if(catalogueLoading.value)return
 selectionOpen.value=true;accountError.value='';channelError.value='';catalogueLoading.value=true
 try {if(props.remote){if(!remoteScopeLoaded.value)await loadScope();await loadCloudCatalog()}await Promise.all([modelTransferApi.channelCatalog().then(items=>{if(!Array.isArray(items))throw {message:'渠道目录返回无效，请重新读取。'};channelCatalog.value=items}).catch(cause=>{channelError.value=errorText(cause)}),modelTransferApi.accountCatalog().then(items=>{if(!Array.isArray(items))throw {message:'账户目录返回无效，请重新读取。'};accountCatalog.value=items}).catch(cause=>{accountError.value=errorText(cause)}),loadAssets()]);if(props.remote){try{const protection=await transferProtectionApi.load();remoteEncrypted.value=protection.enabled;remoteHasPassword.value=protection.hasPassword}catch(cause){protectionError.value=errorText(cause)}}
 if(!selectionReady.value){
 if(exportSource.value==='draft'){initializeDefaultScope();selectionReady.value=true;return}
 let choice:TransferChoice|null=null
 try{choice=await transferChoiceApi.load(!!props.remote);choiceError.value=''}catch{choiceError.value='本机范围读取失败，当前选择保持；请重新读取。';return}
 if(choice)restoreChoice(choice)
 else if(!props.remote&&!fileScopeInitialized){initializeDefaultScope();fileScopeInitialized=true}
 else if(props.remote){
  if(allTemplates.value)templateIds.value=(models.snapshot?.library.templates??[]).map(t=>t.id)
  if(allChannels.value)channelIds.value=savedChannels.value.map(c=>c.id)
  if(allSkills.value)skillNames.value=[...assetCatalog.value.skills]
  if(allMcps.value)mcpNames.value=[...new Set([...mcpNames.value,...assetCatalog.value.mcps.filter(defaultIncludedMcp)])]
 }
 allTemplates.value=false;allChannels.value=false;allSkills.value=false;allMcps.value=false
 if(!choice)credentialIds.value=[...(exportKeys.value?keyChannelIds.value.map(id=>'channel:'+id):[]),...(extensionCredentials.value?[...skillNames.value.map(id=>'skill:'+id),...mcpNames.value.map(id=>'mcp:'+id)]:[])]
 selectionReady.value=true
}else if(choiceError.value){await persistChoice()} }finally{catalogueLoading.value=false}
}
function managerScope():ManagerScope{return {...(accountIds.value.length?{accountIds:accountIds.value}:{}),preferences:managerPreferences.value,extensions:managerExtensions.value,webdav:managerWebdav.value,webdavPasswords:webdavPasswords.value,credentialIds:credentialIds.value,...assetScope()}}
function rememberFilePlan(plan:FilePlan){pendingFilePlanIds.add(plan.id);return plan}
async function discardPendingFilePlans(keepId?:string){
 const ids=[...pendingFilePlanIds].filter(id=>id!==keepId);for(const id of ids)pendingFilePlanIds.delete(id)
 const outcomes=await Promise.all(ids.map(async id=>{try{await modelTransferApi.discardUnattempted(id);return {id,ok:true}}catch{return {id,ok:false}}}))
 let failed=false
 for(const outcome of outcomes)if(!outcome.ok){pendingFilePlanIds.add(outcome.id);failed=true}
 if(keepId)pendingFilePlanIds.add(keepId)
 if(failed)app.showToast('已关闭预览，但部分未执行的临时导出快照清理失败；可重试或稍后检查。')
 return !failed
}
async function prepare() {
  if (!selectionReady.value||choiceError.value||props.remote&&!remoteScopeLoaded.value||!models.snapshot || !hasScope.value||!familyScopesValid.value||catalogueLoading.value) return
  await perform(async () => {
    mcpConflicts.value=[];backupBeforeUpdate.value=true
    const scope = JSON.parse(JSON.stringify(transferScope())) as TransferScope
    if(exportSource.value==='draft'&&!props.remote){
      if(!models.snapshot||!models.channelDraft)throw {message:'工作区未读取'}
      const draft={expectedLibrary:models.snapshot.libraryDigest,expectedPolicy:models.snapshot.policyDigest,expectedWorkspace:models.snapshot.workspaceDigest}
      const policy=JSON.parse(JSON.stringify(models.channelDraft)) as ModelPolicy
      filePlan.value=rememberFilePlan(await modelTransferApi.prepareDraft(scope,draft,undefined,policy))
      preparedExport.value={kind:'file',scope,includeKeys:false,managerScope:managerScope(),draft,policy}
    }else if (props.remote){
      const options=remoteOptions()
      upload.value = await modelTransferApi.prepareUpload(scope,options)
      noteConflicts(upload.value)
      preparedExport.value={kind:'remote',scope,includeKeys:exportKeys.value,managerScope:managerScope(),options}
      remotePassword.value=''
    }else{
      const manager=managerScope()
      filePlan.value = rememberFilePlan(await modelTransferApi.prepare(scope,undefined,exportKeys.value,manager))
      noteConflicts(filePlan.value)
      preparedExport.value={kind:'file',scope,includeKeys:exportKeys.value,managerScope:manager}
    }
    destinationMode.value='default';encryptExport.value=false;exportPackagePassword.value='';exportPackagePasswordAgain.value=''
    selectionOpen.value = false
  })
}
/** 用户在确认弹窗选择加密后，按同一冻结范围重新生成受口令保护的计划；未勾选则沿用已预览计划。 */
async function preparedWithPassphrase(passphrase?:string){if(!filePlan.value)throw {message:'原导出计划已失效，请重新导出。'};return rememberFilePlan(await modelTransferApi.protect(filePlan.value.id,passphrase))}

/** S18-03：选定来源后按同一冻结范围重算计划，选中的来源作为该 MCP 的定义；清空即本次不同步该 MCP。 */
async function rebuildPreview(){const prepared=preparedExport.value;if(!prepared||prepared.draft)return;if(prepared.kind==='remote'){upload.value=await modelTransferApi.prepareUpload(prepared.scope,prepared.options??remoteOptions());noteConflicts(upload.value)}else{const next=rememberFilePlan(await modelTransferApi.prepare(prepared.scope,undefined,prepared.includeKeys,prepared.managerScope));filePlan.value=next;noteConflicts(next)}previewInvalid.value=false}
async function retryPreview(){await perform(rebuildPreview)}
async function setMcpSource(name:string,value:string){
 const next={...mcpSources.value};if(value)next[name]=value;else delete next[name];mcpSources.value=next
 const prepared=preparedExport.value;if(!prepared||prepared.draft)return
 if(prepared.kind==='remote'){const options:RemoteOptions={...(prepared.options??remoteOptions())};const sources=selectedMcpSources();if(Object.keys(sources).length)options.mcpSources=sources;else delete options.mcpSources;prepared.options=options}else prepared.managerScope={...managerScope()}
 previewInvalid.value=true;await perform(rebuildPreview)
}

/** 所选范围只剩未选来源的同名 MCP 时不能继续：原型要求先选来源或调整范围。 */
function onlyUnresolvedMcps(){const unresolved=mcpConflicts.value.map(c=>c.name).filter(name=>!mcpSources.value[name]);if(!unresolved.length)return false;const keys=new Set(unresolved.map(name=>scopeKey('mcps',name)));return scopeKeys.value.every(k=>keys.has(k))}
function applyRetryIntent(){const value=props.retryReceipt?.retry;if(!value)return;receiveCredentialIds.value=value.credentialIds??[...(value.adoptKeys?(incoming.value?.availableKeys??[]).map(id=>'channel:'+id):[]),...(value.adoptExtensionCredentials?incoming.value?.availableExtensionIds??[]:[])];adoptKeys.value=value.adoptKeys;adoptExtensionCredentials.value=value.adoptExtensionCredentials;adoptWebdavPasswords.value=value.adoptWebdavPasswords;backupBeforeUpdate.value=value.backupBeforeUpdate}
async function loadRetry(){const receipt=props.retryReceipt??props.refreshReceipt;if(!receipt)return;await perform(async()=>{if(!models.snapshot)await models.load();recompareId.value=receipt.id;openingOriginal.value=!!props.retryReceipt;remoteUnlock.value=false;try{incoming.value=props.retryReceipt?await modelTransferApi.openReceived(receipt.id):await modelTransferApi.recompare(receipt.id);selectIncoming(incoming.value);applyRetryIntent()}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&['passphrase_required','portable_authentication'].includes(String(cause.code))){passwordOpen.value=true;return}throw cause}});catalogueLoading.value=false}

async function openImport() { if (busy.value)return;previewInvalid.value=false;importPath.value='';error.value='';pickOpen.value=true;try{importCredentialPrefs.value=await importCredentialChoiceApi.load();importPrefsReady.value=true}catch{error.value='导入凭据偏好读取失败，请重试。';importPrefsReady.value=false} }
async function chooseImportFile() {
  if(!importPrefsReady.value&&!props.remote){error.value='导入凭据偏好尚未读取，请重新打开。';return}
  try {
    const selected = await transferFileDialog.open({ multiple:false, directory:false, filters:[{ name:'配置文件', extensions:['ocxdconf','json'] }] })
    if (typeof selected === 'string'){importPath.value=selected;incoming.value=null;incomingSource.value=null;await receive()}
  } catch(cause) { error.value=errorText(cause) }
}
async function receive() {
  if (!props.remote && !importPath.value) return
  await perform(async () => {
    if(!models.snapshot) await models.load()
    if (props.remote){try{incoming.value = await modelTransferApi.download()}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&['passphrase_required','portable_authentication'].includes(String(cause.code))){remoteUnlock.value=true;passwordOpen.value=true;importPassword.value='';return}throw cause}}
    else {
      recompareId.value='';openingOriginal.value=false;remoteUnlock.value=false
      try{incoming.value=await modelTransferApi.receive(importPath.value)}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&['passphrase_required','portable_authentication'].includes(String(cause.code))){pickOpen.value=true;passwordOpen.value=true;importPassword.value='';return}throw cause}
    }
    pickOpen.value=false
    filePlan.value=null;upload.value=null;preparedExport.value=null;exportPackagePassword.value='';exportPackagePasswordAgain.value=''
    adoptKeys.value=false;unlockedPassword='';if(incoming.value)selectIncoming(incoming.value)
  })
}
watch(() => props.importOpen, value => { if(value)openImport() }, { immediate:true })
function selectIncoming(plan:IncomingPlan){
 receiveChannelModels.value={};
 receiveSkillTargets.value=Object.fromEntries((plan.assets?.skills??[]).map(s=>[s.name,plan.assetMappings?.skills[s.name]??s.name]));receiveMcpTargets.value=Object.fromEntries((plan.assets?.mcps??[]).map(m=>[m.name,plan.assetMappings?.mcps[m.name]??m.name]));receiveMappings.value=Object.fromEntries(Object.entries(plan.channelBindings??{}).map(([id,b])=>[id,b.keepLocalConnection?'local:'+b.targetId:'new']));receiveNewLabels.value=Object.fromEntries((plan.package.payload.policy?.channels??[]).map(c=>[c.id,plan.channelBindings?.[c.id]?.label??c.label+'（导入）']));receiveAccounts.value=plan.manager?.accounts?.filter(accountInformationSupported).map(a=>a.id)??[];incomingSource.value=plan;receiveRefs.value=plan.package.payload.templates.flatMap(t=>t.revisions.map(r=>JSON.stringify([t.id,r.id])));receiveChannels.value=plan.package.payload.policy?.channels.map(c=>c.id)??[];receiveGlobal.value=!!(plan.package.payload.policy?.templateRefs||plan.package.payload.policy?.channelOrder||plan.package.payload.policy?.defaultChannelId);receivePreferences.value=!!plan.manager?.preferences;receiveExtensions.value=!!plan.manager?.extensions&&(!!props.retryReceipt||openingOriginal.value);receiveEndpoints.value=plan.manager?.webdav.map(e=>e.id)??[];receiveSkills.value=plan.assets?.skills.map(skill=>skill.name)??[];receiveMcps.value=plan.assets?.mcps.map(mcp=>mcp.name)??[];adoptExtensionCredentials.value=plan.adoption?.extensionCredentials??true;adoptKeys.value=plan.adoption?.keys??true;receiveCredentialIds.value=[...(importCredentialPrefs.value.keys||props.remote?(plan.availableKeys??[]).map(id=>'channel:'+id):[]),...(importCredentialPrefs.value.extensions||props.remote?plan.availableExtensionIds??[]:[])];adoptWebdavPasswords.value=plan.adoption?.webdav??true;receiveDirty.value=false;backupBeforeUpdate.value=true;if(props.remote&&!plan.adoption){receiveCredentialIds.value=receiveCredentialIds.value.filter(id=>credentialIds.value.includes(id));adoptKeys.value=exportKeys.value&&!!plan.availableKeys?.length;adoptExtensionCredentials.value=extensionCredentials.value&&!!plan.availableExtensionCredentials}
 // Ordinary file/download scope only exposes portable display preferences.
 // Re-select older packages before confirm so their whole-extension settings
 // cannot be adopted by a display-only checkbox. Original retries keep their
 // frozen scope; a failed re-selection remains dirty and blocks confirmation.
 if(plan.manager?.extensions&&!props.retryReceipt&&!openingOriginal.value){receiveDirty.value=true;queueComparison()}
 syncIncomingCredentialFlags()
}
async function unlockFile(){await perform(async()=>{incoming.value=remoteUnlock.value?await modelTransferApi.download(importPassword.value):recompareId.value?(openingOriginal.value?await modelTransferApi.openReceived(recompareId.value,importPassword.value):await modelTransferApi.recompare(recompareId.value,importPassword.value)):await modelTransferApi.receive(importPath.value,importPassword.value);unlockedPassword=importPassword.value;pickOpen.value=false;passwordOpen.value=false;importPassword.value='';filePlan.value=null;upload.value=null;if(!openingOriginal.value||props.retryReceipt){selectIncoming(incoming.value);applyRetryIntent()}})}
async function recompareOriginal(){if(!incomingSource.value)return;await perform(async()=>{recompareId.value=incomingSource.value!.id;openingOriginal.value=false;try{incoming.value=await modelTransferApi.recompare(recompareId.value);selectIncoming(incoming.value)}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&['passphrase_required','portable_authentication'].includes(String(cause.code))){passwordOpen.value=true;importPassword.value='';return}throw cause}})}

async function confirm() {
  if(previewInvalid.value)return
  if(incoming.value?.protected&&!unlockedPassword&&!props.remote&&((adoptKeys.value||adoptWebdavPasswords.value&&incoming.value.availableWebdavPasswords?.length||adoptExtensionCredentials.value))){recompareId.value=incoming.value.id;openingOriginal.value=true;passwordOpen.value=true;importPassword.value='';return}
  await perform(async () => {
    if(props.remote&&!props.retryReceipt&&syncPreview.value?.changes===null){error.value='无法比较变化，请重新读取后再确认。';return}
    if (incoming.value) {
      if(receiveDirty.value||incoming.value.previewError||incoming.value.comparison?.changes===null)return
      const includeDav=adoptWebdavPasswords.value&&!!incoming.value.availableWebdavPasswords?.length
      const plan=incoming.value
      const syncResult=props.remote?plan.sync??null:null;const count=receiveKeys.value.length
      const needBackup=backupBeforeUpdate.value
      // 更新前备份放在草稿冲突预检之后、写入之前：被冲突拦下的导入不留下多余备份。
      await models.adoptTransfer(props.retryReceipt&&plan.modelsAccepted?{expectedPolicy:models.snapshot!.policyDigest,mergedPolicy:null}:plan,async()=>{
        try{requestedResultId=plan.id;return await modelTransferApi.accept(plan.id,{backupBeforeUpdate:needBackup,credentialIds:receiveCredentialIds.value.filter(id=>incomingCredentialRows.value.some(r=>r.id===id)),adoptKeys:adoptKeys.value,...(includeDav?{adoptWebdavPasswords:true}:{}),...(adoptExtensionCredentials.value?{adoptExtensionCredentials:true}:{}),passphrase:unlockedPassword})}catch(cause){try{const latest=await modelTransferApi.latestIncoming();if(latest?.id===plan.id&&latest.adoption)plan.adoption=latest.adoption}catch{}throw cause}
      })
      const updatePreferences=!!plan.manager?.preferences; emit('update:importOpen',false); if(syncResult){recordRemoteVersion(syncResult.createdAt);syncStore.setWebdavState('synced');syncStore.rememberResult(plan.id,!!props.remote)}else{if(!props.remote)recordSyncActivity('import',count);syncStore.rememberResult(plan.id,!!props.remote)}; incoming.value = null;incomingSource.value=null;unlockedPassword='';if(updatePreferences)await app.loadPreferences();if(props.actionOnly)emit('close')
    } else if ((upload.value||filePlan.value)&&onlyUnresolvedMcps()) {error.value='请为同名 MCP 选择来源，或调整导出范围。';return
    } else if (upload.value) {
      if(preparedExport.value?.kind==='remote'&&remotePassword.value){const options:RemoteOptions={...(preparedExport.value.options??remoteOptions()),passphrase:remotePassword.value};upload.value=await modelTransferApi.prepareUpload(preparedExport.value.scope,options)}
      const syncResult=upload.value.sync??null
      requestedResultId=upload.value.id;upload.value = await modelTransferApi.publish(upload.value.id)
      if(syncResult){recordRemoteVersion(upload.value.package.createdAt);syncStore.setWebdavState(pendingCount.value?'conflict':'synced');syncStore.rememberResult(upload.value.id,true);upload.value=null;mcpConflicts.value=[]}
      remotePassword.value='';preparedExport.value=null;if(props.actionOnly)emit('close')
    } else if (filePlan.value) await finishExport()
  })
}
/** 选择导出目录；取消时保持当前选择。 */
async function chooseExportDir(){
  const picked=await transferFileDialog.open({directory:true,multiple:false,title:'选择导出目录'})
  if(!picked||Array.isArray(picked))return
  exportTargetDir.value=picked
}
async function finishExport(){
  if(!filePlan.value||!preparedExport.value)return
  if(encryptExport.value){
    if(exportPackagePassword.value.length<8||exportPackagePassword.value!==exportPackagePasswordAgain.value){error.value='请输入至少 8 位的加密口令，并保持两次一致。';return}
    filePlan.value=await preparedWithPassphrase(exportPackagePassword.value)
  }else if(filePlan.value.protected)filePlan.value=await preparedWithPassphrase()
  if(destinationMode.value==='custom'&&!exportTargetDir.value){error.value='请选择自定义目录，或改回默认导出目录。';return}
  const id=filePlan.value.id;requestedResultId=id;await modelTransferApi.export(id, exportTargetDir.value || undefined)
  await discardPendingFilePlans(id);pendingFilePlanIds.delete(id)
  recordSyncActivity('export',Math.max(0,scopeKeys.value.length-pendingCount.value));syncStore.rememberResult(id,false);filePlan.value=null;preparedExport.value=null;exportPackagePassword.value='';exportPackagePasswordAgain.value='';if(props.actionOnly)emit('close')
}

function setModelScope(id:string,values:string[]|undefined,receive=false){const target=receive?receiveChannelModels:channelModels;const next={...target.value};if(values===undefined)delete next[id];else next[id]=values;target.value=next;if(receive)receiveDirty.value=true}
function sourceFamilies(id:string){
 const policy=exportSource.value==='draft'?models.channelDraft:models.snapshot?.policy
 const channel=policy?.channels.find(c=>c.id===id);if(!channel)return []
 const ids=[...new Set([...(policy?.templateRefs??[]),...channel.templateRefs,...Object.values(channel.modelRefs)].map(r=>r.templateId))]
 return ids.map(id=>({id,label:models.snapshot?.library.templates.find(t=>t.id===id)?.label??id}))
}
function setFamilyScope(id:string,value:FamilyScope|undefined){
 const next={...channelFamilies.value};if(value===undefined)delete next[id];else{next[id]=value;const subset={...channelModels.value};delete subset[id];channelModels.value=subset}
 channelFamilies.value=next;
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
let comparisonQueued=false,comparisonWaiting=false
function scheduleComparison(){if(comparisonQueued||disposed)return;comparisonQueued=true;void nextTick(async()=>{comparisonQueued=false;if(disposed||!receiveDirty.value||!incomingSource.value)return;if(busy.value){comparisonWaiting=true;return}await compareReceived()})}
function queueComparison(){receiveRevision++;if(busy.value){comparisonWaiting=true;return}scheduleComparison()}
watch(busy,value=>{if(!value&&comparisonWaiting){comparisonWaiting=false;scheduleComparison()}})
watch(()=>JSON.stringify([receiveRefs.value,receiveChannels.value,receiveGlobal.value,receivePreferences.value,receiveExtensions.value,receiveEndpoints.value,receiveAccounts.value,receiveSkills.value,receiveMcps.value,receiveMappings.value,receiveNewLabels.value,receiveSkillTargets.value,receiveMcpTargets.value,receiveChannelModels.value]),()=>{if(receiveDirty.value)queueComparison()})
let receiveRevision=0
async function compareReceived(){if(!incomingSource.value)return;await perform(async()=>{const revision=receiveRevision;const scope={...(Object.keys(receiveChannelModels.value).length?{channelModels:Object.fromEntries(Object.entries(receiveChannelModels.value).filter(([id])=>receiveChannels.value.includes(id)))}:{}),...assetMappingScope(),...channelMappingScope(),templateRefs:receiveRefs.value.map(value=>{const [templateId,revisionId]=JSON.parse(value) as [string,string];return {templateId,revisionId}}),channelIds:receiveChannels.value,includeGlobal:receiveGlobal.value&&receiveChannels.value.length>0,...(incomingSource.value!.manager?{accountIds:receiveAccounts.value,preferences:receivePreferences.value,extensions:receiveExtensions.value,endpointIds:receiveEndpoints.value}:{}),...(incomingSource.value!.assets?{skillNames:receiveSkills.value,mcpNames:receiveMcps.value}:{})};const selected=unlockedPassword?await modelTransferApi.select(incomingSource.value!.id,scope,unlockedPassword):await modelTransferApi.select(incomingSource.value!.id,scope);if(revision!==receiveRevision)return;incoming.value=selected;if(!incoming.value.availableKeys?.length)adoptKeys.value=false;if(!incoming.value.availableExtensionCredentials)adoptExtensionCredentials.value=false;receiveDirty.value=false})}

async function closePreview() { if (!busy.value) { await discardPendingFilePlans();previewInvalid.value=false;pickOpen.value=false;passwordOpen.value=false;importPassword.value=''; const same=props.remote&&incoming.value?.sync&&incoming.value.sync.changes?.length===0?incoming.value.sync:null;if(same){/* 后端读取云端快照时已记为已一致；概览卡片同步展示。 */recordRemoteVersion(same.createdAt);syncStore.setWebdavState('synced')} emit('update:importOpen',false); incoming.value = null;incomingSource.value=null; filePlan.value = null; upload.value = null;preparedExport.value=null;exportPackagePassword.value='';exportPackagePasswordAgain.value='';remotePassword.value='';unlockedPassword='';if(props.actionOnly)emit('close') } }
</script>
<template>
  <article class="model-transfer">
    <template v-if="!importOnly&&!actionOnly">
    <div class="card-head"><div><h2>{{ remote ? '设备间同步' : '导入与导出' }}</h2><p>{{ remote ? '较新版本覆盖所选范围，确认前可查看变化。' : '按下方范围导出配置；导入时在所选文件内选择范围。' }}</p></div><div class="sync-head-actions"><UiButton v-if="currentResultId" variant="ghost" :disabled="busy" @click="syncStore.openResult(currentResultId)">查看结果</UiButton><template v-if="remote"><UiButton variant="primary" :disabled="remoteActionsDisabled" :loading="busy" @click="startSync">{{ busy?'同步中…':'同步' }}</UiButton></template><template v-else><UiButton :disabled="busy||catalogueLoading||!models.snapshot" @click="openImport">导入配置</UiButton><UiButton variant="primary" :disabled="!selectionReady||!!choiceError||busy||!models.snapshot||catalogueLoading||!hasScope||!familyScopesValid||(exportSource==='draft'&&!channelIds.length)" @click="prepare">导出配置</UiButton></template></div></div>
    <slot name="connection" />
    <p v-if="models.error && !models.snapshot" class="model-error" role="alert">{{ models.error }}</p>
    <div v-if="choiceError"><p class="model-error" role="alert">{{ choiceError }}</p><UiButton :disabled="busy||catalogueLoading" @click="begin">{{ selectionReady?'重试保存选择':'重新读取' }}</UiButton></div>
    <p v-if="error && !preview" class="model-error" role="alert">{{ error }}</p>
    <!-- 范围是页面主体的一部分，不再收进弹窗：原型里它是列表旁的常驻两栏工作区。 -->
    <!-- 原型结构：搜索与表头选择 + 两栏；右栏只表达凭据与保护选择。 -->
    <section class="sync-scope-workspace" aria-label="同步范围" data-testid="sync-scope-workspace">
      <div class="sync-scope-main">
        <div class="sync-scope-toolbar">
          <input v-model="scopeQuery" class="input sync-scope-search" type="search" aria-label="搜索配置对象" placeholder="搜索配置对象" :disabled="busy||catalogueLoading" />
        </div>
        <p v-if="exportSource==='draft'" class="model-muted">未提交草稿候选：仅含明确选择的渠道草稿和精确模版依赖，不携带凭据。</p>
        <p v-if="catalogueLoading" role="status">正在读取已保存配置对象…</p>
        <p v-for="message in [scopeReadError,cloudCatalogError,assetError,accountError,channelError].filter(Boolean)" :key="message" class="model-error" role="alert">{{ message }}</p><UiButton v-if="[scopeReadError,cloudCatalogError,assetError,accountError,channelError].some(Boolean)" :disabled="busy||catalogueLoading" @click="begin">重新读取范围目录</UiButton>

        <ObjectScopeTree :objects="scopeObjects" :visible-ids="accountFilterIds" :selected-ids="scopeKeys" :query="scopeQuery" :name-label="remote?'配置对象':'导出范围'" label="导出配置对象" :disabled="busy||catalogueLoading" @update:selected-ids="updateScopeKeys" @detail="readObjectDetail">
          <template #group-options="{category}"><label v-if="category==='accounts'&&accountCatalog.length&&exportSource!=='draft'" class="sync-account-filter">账户平台<UiSelectField v-model="accountPlatform" label="账户平台筛选" :options="[{value:'all',label:'全部平台'},...Object.entries(accountPlatforms).map(([value,label])=>({value,label})),{value:'legacy',label:'旧格式 · 来源未登记'}]" :disabled="busy||catalogueLoading" /></label></template>
          <template #options="{object}">
            <template v-if="['skills','mcps'].includes(object.category)"><p v-if="assetDetailLoading===object.id" role="status">正在读取资源详情…</p><p v-if="assetDetailError[object.id]" class="model-error" role="alert">{{ assetDetailError[object.id] }}</p><dl v-if="assetDetails[object.id]" class="sx-detail-list"><dt>归属与来源</dt><dd>{{ assetDetails[object.id]!.source }}</dd><dt>包含字段</dt><dd>{{ assetDetails[object.id]!.fields.join('、') }}</dd><dt>当前文件</dt><dd><div v-for="file in assetDetails[object.id]!.files" :key="file">{{ file }}</div></dd><template v-if="assetDetails[object.id]!.clients.length"><dt>客户端来源</dt><dd>{{ assetDetails[object.id]!.clients.map(clientName).join('、') }}</dd></template><template v-if="assetDetails[object.id]!.conflicts.length"><dt>定义差异</dt><dd>{{ assetDetails[object.id]!.conflicts.join('、') }}；在同步或导出预览中选择采用来源。</dd></template><dt>凭据</dt><dd>凭据单独选择，详情不展示秘密值。</dd></dl><p v-for="warning in assetDetails[object.id]?.warnings??[]" :key="warning" class="sx-muted">{{ warning }}</p></template>

            <template v-if="object.category==='channels'"><dl v-for="channel in savedChannels.filter(c=>c.id===JSON.parse(object.id)[1])" :key="channel.id" class="sx-detail-list"><dt>归属</dt><dd>OpenCodex 已保存渠道</dd><dt>包含字段</dt><dd>服务节点、当前节点、协议、启停、模型清单、选择、顺序、命名与精确模版引用</dd><dt>当前连接地址</dt><dd>{{ endpointLabel(channel.connection.nodes.find(n=>n.id===channel.connection.activeNodeId)?.url??'') }}</dd><dt>服务节点</dt><dd><div v-for="node in channel.connection.nodes" :key="node.id">{{ node.name }}{{ node.id===channel.connection.activeNodeId?' · 当前':'' }}：{{ endpointLabel(node.url) }}</div></dd><dt>协议与启停</dt><dd>{{ channel.connection.protocol }} · {{ channel.connection.enabled?'启用':'停用' }}</dd><dt>已选模型</dt><dd>{{ channel.selection.mode==='all'?'全部已保存型号':channel.selection.mode==='none'?'未选择型号':('models' in channel.selection?channel.selection.models.join('、'):'未选择型号') }}</dd><dt>精确依赖</dt><dd>{{ sourceFamilies(channel.id).map(t=>t.label).join('、')||'无额外模版依赖' }}</dd><dt>凭据</dt><dd>API Key 单独选择，不展示秘密值。</dd></dl></template>
            <dl v-if="object.category==='manager'" class="sx-detail-list"><dt>归属</dt><dd>Desktop</dd><dt>包含字段</dt><dd>主题、界面缩放、材质与背景显示偏好；已选扩展启用及分发设置</dd><dt>设备边界</dt><dd>本机路径、代理、启动、执行绑定、当前连接和同步操作偏好由接收设备保留。</dd></dl>
            <dl v-if="object.category==='endpoints'" class="sx-detail-list"><dt>包含字段</dt><dd>服务地址、目录、用户名与另行选择的连接密码</dd><dt>接收方式</dt><dd>导入后为未启用连接，不改当前连接或同步范围。</dd><dt>内容保护</dt><dd>同步内容保护口令不进入配置包。</dd></dl>
            <template v-if="object.category==='accounts'"><details v-for="account in scopeAccounts(object.id)" :key="account.id"><summary>账户详情与同步选项</summary><dl><dt>管理方</dt><dd>{{ account.origin?.owner==='opencodex'?'OpenCodex账户池':account.origin?.owner??'旧格式 · 来源未登记' }}</dd><dt>来源账户</dt><dd>{{ account.origin?.sourceAccountId??account.id }}</dd><dt>授权主体</dt><dd>{{ account.subject??'未提供；保留独立记录' }}</dd><dt>账户 / 组织</dt><dd>{{ account.origin?.tenant??'未提供' }}</dd></dl><label><input type="checkbox" :checked="accountInformationSupported(account)" disabled />仅账户信息</label><label><input type="checkbox" disabled />同步登录状态</label><p>{{ accountTransferLabel(account.provider) }}</p></details></template>
            <p v-if="object.category==='templates'">已保存版本：{{ models.snapshot!.library.templates.find(t=>t.id===JSON.parse(object.id)[1])!.revisions.map(r=>versionLabel(r)).join(' / ') }}</p>
            <ChannelModelScope v-if="object.category==='channels'&&scopeKeys.includes(object.id)&&(source==='draft'||channelFamilies[JSON.parse(object.id)[1]]||channelModels[JSON.parse(object.id)[1]])" :label="object.label" :models="sourceModels(JSON.parse(object.id)[1])" :families="sourceFamilies(JSON.parse(object.id)[1])" :family-scope="channelFamilies[JSON.parse(object.id)[1]]" @update:family-scope="setFamilyScope(JSON.parse(object.id)[1],$event)" :selected="channelModels[JSON.parse(object.id)[1]]" :disabled="busy||catalogueLoading" @update:selected="setModelScope(JSON.parse(object.id)[1],$event)" />
            <details v-if="object.id===scopeKey('manager','webdav')&&managerWebdav"><summary>同步选项</summary><label><input v-model="webdavPasswords" type="checkbox" :disabled="busy||catalogueLoading" />包含连接密码</label></details>
          </template>
        </ObjectScopeTree>
      </div>
      <aside v-if="exportSource!=='draft'" class="sync-scope-credentials" aria-label="其他凭据" data-testid="sync-scope-credentials">
        <header class="sync-credentials-intro"><h3>其他凭据</h3>
        <p class="sync-credential-hint">{{ remote ? '账户按平台能力提供可选范围；当前仅账户信息；以下凭据可另选。' : '账户按平台能力提供可选范围；当前仅账户信息，WebDAV 密码随配置选择；以下凭据可另选。' }}</p></header>
        <section v-if="remote" class="sync-credential-block"><label><input v-model="remoteEncrypted" type="checkbox" :disabled="busy" />加密同步内容</label><label v-if="remoteEncrypted" class="sx-field"><span>保护口令</span><input v-model="remotePassword" class="input" type="password" autocomplete="off" aria-label="同步保护口令" :placeholder="remoteHasPassword?'本机已保存 · 留空保留':'填写保护口令'" :disabled="busy" /></label><p v-if="protectionError" class="model-error" role="alert">{{ protectionError }}</p><p class="sync-credential-hint">可选，与连接密码独立。</p></section>
        <section class="sync-credential-block">
          <label class="sync-credential-head"><input :checked="credentialState('channel')===true" :indeterminate="credentialState('channel')==='mixed'" :aria-checked="credentialState('channel')" @change="setCredentialGroup('channel',($event.target as HTMLInputElement).checked)" type="checkbox" :disabled="busy||catalogueLoading||!keyChannelIds.length" />渠道 API Key</label>
          <p class="sync-credential-hint">渠道可单独分享，接收方自行填 Key。</p>
          <div v-if="exportKeys" class="sx-subpicks"><label v-for="item in credentialChannels" :key="item.id"><input type="checkbox" :checked="credentialIds.includes(item.id)" :disabled="busy" @change="toggleCredential(item.id,($event.target as HTMLInputElement).checked)" />{{ item.label }}</label></div>

        </section>
        <section class="sync-credential-block">
          <label class="sync-credential-head"><input :checked="credentialState('extension')===true" :indeterminate="credentialState('extension')==='mixed'" :aria-checked="credentialState('extension')" @change="setCredentialGroup('extension',($event.target as HTMLInputElement).checked)" type="checkbox" :disabled="busy||catalogueLoading||!hasAssets" />扩展凭据</label>
          <p class="sync-credential-hint">例如 MCP 的 Authorization / 环境变量。</p>
          <div v-if="extensionCredentials" class="sx-subpicks"><label v-for="item in credentialExtensions" :key="item.id"><input type="checkbox" :checked="credentialIds.includes(item.id)" :disabled="busy" @change="toggleCredential(item.id,($event.target as HTMLInputElement).checked)" />{{ item.label }}</label></div>

        </section>
        <p class="sync-credentials-note">{{ selectedCredentialKinds.length ? '所选凭据随范围同步' : '当前不携带凭据' }}</p>
      </aside>
    </section>
    </template>
    <UiDialog :hide-close="!remote&&!incoming&&!pickOpen&&!passwordOpen" icon-close panel-class="transfer-review" :open="reviewOpen" :title="remote?'同步预览':pickOpen||passwordOpen||incoming?'导入配置':'确认导出'" :busy="busy" wide @close="closePreview">
      <p v-if="error" class="model-error" role="alert">{{ error }}</p><p v-if="previewInvalid" class="sx-muted">选择已改变，当前预览尚未重新生成。<UiButton v-if="preparedExport" size="sm" :disabled="busy" @click="retryPreview">重新生成预览</UiButton></p><p v-if="retryReceipt" class="sx-muted">继续原操作，范围与凭据选择沿原确认。</p>
      <div v-if="pickOpen||!remote&&incoming&&!retryReceipt" class="transfer-file-field"><span class="transfer-file-label">配置文件</span><div class="transfer-file-row"><UiButton :disabled="busy" @click="chooseImportFile">选择文件</UiButton><span class="local-path">{{ displayPath(importPath)||'尚未选择文件' }}</span></div></div>
      <div v-if="passwordOpen" class="transfer-file-field"><label><span>解密口令</span><input v-model="importPassword" type="password" class="input" aria-label="导入解密口令" autocomplete="off" :disabled="busy" /></label></div>

      <p v-if="preview?.payload.candidate" role="status">未提交草稿候选；采用后仍进入本机工作区，须另行预览并保存，不表示运行配置已写入。</p>
      <template v-if="syncPreview"><p v-if="upload" class="sx-muted">传输保护：{{ upload.protected?'加密':'不加密' }}</p><p v-if="upload&&!upload.protected&&(upload.credentialCount??0)>0" class="export-plaintext-risk">不加密时，所传内容中的凭据以明文保存。</p><p v-if="directionReason" class="sx-muted">{{ directionReason }}</p><p v-if="unselectedCloudCount" class="sx-muted">云端另有 {{ unselectedCloudCount }} 项未选择。本次保持；可取消后在范围中选择。</p>
        <dl class="sx-detail-list sx-snapshot-facts"><dt>新版来源</dt><dd>{{ syncPreview.source==='local'?'本机配置 → 云端':'云端配置 → 本机' }}</dd><dt>版本时间</dt><dd>{{ syncTime(syncPreview.createdAt) }}</dd></dl>
        <label v-if="syncPreview.source==='remote'&&syncChanges?.length!==0&&!incoming?.adoption"><input v-model="backupBeforeUpdate" type="checkbox" :disabled="busy||!!retryReceipt" />更新前备份</label>
        <p v-if="incoming?.previewError" class="model-error" role="alert">{{ incoming.previewError }}</p>
        <p v-if="syncChanges===null" class="sx-muted">无法比较变化</p>
        <p v-else-if="syncChanges?.length===0" class="sx-muted">所选内容已一致，无需更新。</p>
        <template v-else>
          <div class="sx-preview-summary"><strong>{{ syncChanges?.length }} 项变化</strong><span class="sx-muted">{{ syncPreview.unchanged }} 项保持不变</span></div>
          <ul class="sx-outcomes sx-change-list"><li v-for="c in syncChanges??[]" :key="c.kind+c.id"><div><strong>{{ c.label }}</strong><small>{{ changeSummary(c) }}</small><label v-if="upload&&pickerClients(c).length" class="sx-mcp-source"><span>采用来源</span><UiSelectField size="sm" :label="c.label+' 采用来源'" :model-value="mcpSources[c.id]??''" :options="[{value:'',label:'暂不同步'},...pickerClients(c).map(v=>({value:v,label:clientName(v)}))]" :disabled="busy" @update:model-value="setMcpSource(c.id,$event)"/></label></div><span class="sx-mark">{{ markText(c) }}</span></li></ul>
        </template>

      </template>
      <template v-else>
      <div v-if="filePlan&&!remote" class="export-destination"><span>保存位置</span><div class="export-destination-choice" role="radiogroup" aria-label="保存位置"><label><input v-model="destinationMode" type="radio" value="default" :disabled="busy" />默认导出目录</label><label><input v-model="destinationMode" type="radio" value="custom" :disabled="busy" />自定义目录</label></div><div v-if="destinationMode==='custom'" class="export-destination-custom"><div class="export-destination-row"><input class="input local-path" type="text" readonly :value="displayPath(exportTargetDir)" aria-label="自定义导出目录" placeholder="尚未选择目录"><UiButton size="sm" variant="ghost" :disabled="busy" @click="chooseExportDir">选择目录…</UiButton></div><p class="model-muted">未选择目录时按默认导出目录保存。</p></div></div>
      <div v-if="filePlan&&!remote" class="export-package-layout">
        <!-- 原型顺序：标题 → 导出目录 → 生成位置树 → 说明段 → 文件保护/携带凭据 → 口令 → 底部主按钮。 -->
        <div class="export-tree" aria-label="导出目录结构"><p class="export-tree-kicker">生成位置</p><pre>{{ exportTreeText }}</pre><p class="model-muted">按日期归组，一次导出一个可独立拷贝的备份包。同日多次导出各自建包，不混写、不覆盖。</p></div>
        <div class="export-summary"><strong>{{ scopeKeys.length }} 项配置 · 同一备份包</strong><p class="model-muted">{{ exportCredentialCount ? '含 '+exportCredentialCount+' 项凭据；可选择加密，不加密则凭据以明文写入，文件可被直接读取。口令始终不写入包。' : '不含凭据，可直接使用完整性校验。' }}复制整个备份包即可带到另一台设备。</p><p class="model-muted">包内按 {{ exportGroupLabels || '所选范围' }} 分组，导入时仍按所选范围识别。</p></div>
      </div>
      <div v-if="!incoming&&mcpConflicts.length" class="transfer-mcp-conflicts" role="group" aria-label="同名 MCP 定义不同"><strong>同名 MCP 定义不同</strong><ul><li v-for="c in mcpConflicts" :key="c.name"><strong>{{ c.name }}</strong><small>同名定义在 {{ c.clients.map(clientName).join('、') }} 中不同（差异字段：{{ c.fields.join('、') }}），请选择来源</small><label class="sx-mcp-source"><span>采用来源</span><UiSelectField size="sm" :label="c.name+' 采用来源'" :model-value="mcpSources[c.name]??''" :options="[{value:'',label:remote?'暂不同步':'暂不导出'},...c.clients.map(v=>({value:v,label:clientName(v)}))]" :disabled="busy" @update:model-value="setMcpSource(c.name,$event)"/></label></li></ul></div>
      <div v-if="filePlan&&!remote" class="export-info"><strong>{{ scopeKeys.length }} 项配置 · {{ exportCredentialCount?'含 '+exportCredentialCount+' 项凭据':'不含任何凭据' }}</strong><br><span v-for="([label,count],index) in exportCredentialSummary" :key="label">{{ index?' / ':'' }}{{ label }} {{ count }} 项</span></div>
      <label v-if="filePlan&&!remote" class="export-encrypt"><input v-model="encryptExport" type="checkbox" :disabled="busy" />加密配置包</label>
      <div v-if="filePlan&&!remote&&encryptExport" class="export-passphrase"><label>加密口令（至少 8 位）<input v-model="exportPackagePassword" aria-label="加密口令" class="input" type="password" autocomplete="new-password" :disabled="busy||!encryptExport" :placeholder="encryptExport?'至少 8 位':'勾选加密后可填写'" /></label><label>再次输入<input v-model="exportPackagePasswordAgain" aria-label="再次输入加密口令" class="input" type="password" autocomplete="new-password" :disabled="busy||!encryptExport" :placeholder="encryptExport?'再次输入':'勾选加密后可填写'" /></label></div>

      <template v-if="incoming&&!remote&&incoming.comparison"><p class="sx-muted">文件版本：{{ syncTime(incoming.comparison.createdAt) }}</p><p v-if="importChanges===null" class="model-error">无法比较当前内容，请重新读取后确认。</p><p v-else-if="importChanges?.length===0" class="sx-muted">所选内容已一致，无需更新。</p><ul v-else class="sx-outcomes"><li v-for="change in importChanges??[]" :key="change.kind+change.id"><div><strong>{{ change.label }}</strong><small>{{ changeSummary(change) }}</small></div><span class="sx-mark">{{ markText(change) }}</span></li></ul></template>
      <label v-if="incoming&&!remote&&!incoming.adoption"><input v-model="backupBeforeUpdate" type="checkbox" :disabled="busy||!!retryReceipt" />更新前备份</label>
      <p v-if="incoming?.adoption">{{ incoming.adoption.completed?'原导入已完成；当前本机配置保持。':'原导入采用范围已冻结，请核对同一操作。' }}</p><label v-if="incoming?.availableExtensionCredentials"><input v-model="adoptExtensionCredentials" @change="setIncomingCredentialGroup('extension',adoptExtensionCredentials)" type="checkbox" :disabled="busy||receiveDirty||!!incoming.adoption||!!retryReceipt" />采用所选扩展凭据（新本机引用及受保护资源）</label><label v-if="incoming?.availableWebdavPasswords?.length"><input v-model="adoptWebdavPasswords" type="checkbox" :disabled="busy||receiveDirty||!!incoming?.adoption||!!retryReceipt" />采用包内连接密码（{{ incoming.availableWebdavPasswords.length }} 项），连接保持未启用</label><label v-if="incoming?.availableKeys?.length"><input v-model="adoptKeys" @change="setIncomingCredentialGroup('channel',adoptKeys)" type="checkbox" :disabled="busy||receiveDirty||!!incoming?.adoption||!!retryReceipt" />采用所选渠道的Key与认证配置（{{ incoming.availableKeys.length }} 个渠道）</label>
      <div v-if="incomingCredentialRows.length" class="sx-credential-items"><label v-for="item in incomingCredentialRows" :key="item.id"><input v-model="receiveCredentialIds" type="checkbox" :value="item.id" :disabled="busy||!!incoming?.adoption||!!retryReceipt" />{{ item.label }} · 凭据</label></div><details v-if="incomingSource" open class="model-transfer-scope"><summary>选择接收范围</summary><UiButton v-if="incoming?.previewError" size="sm" variant="ghost" :disabled="busy||!!incoming?.adoption||!!retryReceipt" @click="recompareOriginal">重新读取当前配置</UiButton><ObjectScopeTree :objects="receiveObjects" :selected-ids="receiveKeys" label="接收配置对象" :disabled="busy||!!incoming?.adoption||!!retryReceipt" @update:selected-ids="updateReceiveKeys"><template #options="{object}">
       <template v-if="incomingSource.package.payload.candidate&&object.category==='skills'&&receiveSkills.includes(JSON.parse(object.id)[1])"><label>接收Skill名称<input v-model="receiveSkillTargets[JSON.parse(object.id)[1]]" class="input" maxlength="256" :aria-label="JSON.parse(object.id)[1]+'目标Skill名称'" :disabled="busy||!!incoming?.adoption||!!retryReceipt" @input="receiveDirty=true" /></label></template>
       <template v-if="incomingSource.package.payload.candidate&&object.category==='mcps'&&receiveMcps.includes(JSON.parse(object.id)[1])"><label>接收MCP名称<input v-model="receiveMcpTargets[JSON.parse(object.id)[1]]" class="input" maxlength="256" :aria-label="JSON.parse(object.id)[1]+'目标MCP名称'" :disabled="busy||!!incoming?.adoption||!!retryReceipt" @input="receiveDirty=true" /></label></template>
       <template v-if="object.category==='channels'&&receiveChannels.includes(JSON.parse(object.id)[1])"><p v-if="receivedFamily(JSON.parse(object.id)[1])">模型族片段：保留本机其他族、连接、Key和已有顺序；{{ receivedFamily(JSON.parse(object.id)[1])?.includeChannelDefaults?'另含渠道共用默认字段，影响全部型号':'共用默认字段沿本机值解析' }}</p><ChannelModelScope v-else-if="incomingSource.package.payload.candidate||incomingSource.package.payload.policy?.channels.find(c=>c.id===JSON.parse(object.id)[1])?.modelSubset" :label="object.label" :models="sourceModels(JSON.parse(object.id)[1],true)" :selected="receiveChannelModels[JSON.parse(object.id)[1]]" :disabled="busy||!!incoming?.adoption||!!retryReceipt" @update:selected="setModelScope(JSON.parse(object.id)[1],$event,true)" /><UiSelectField v-if="incomingSource.package.payload.candidate||incoming?.previewError==='channel_endpoint_conflict'" :model-value="receiveMappings[JSON.parse(object.id)[1]]??'identity'" :label="object.label+'接收方式'" :disabled="busy||!!incoming?.adoption||!!retryReceipt" :options="[{value:'identity',label:'按原渠道身份合并'},...(incomingSource.package.payload.policy?.channels.find(c=>c.id===JSON.parse(object.id)[1])?.connection?.nodes.length?[{value:'new',label:'建立独立新渠道'}]:[]),...localChannels.map(c=>({value:'local:'+c.id,label:'映射到 '+c.label+' · 保留本机连接和Key'}))]" @update:model-value="receiveMappings[JSON.parse(object.id)[1]]=$event;receiveDirty=true"/><label v-if="receiveMappings[JSON.parse(object.id)[1]]==='new'">新渠道名称<input v-model="receiveNewLabels[JSON.parse(object.id)[1]]" class="input" maxlength="256" :disabled="busy||!!incoming?.adoption||!!retryReceipt" @input="receiveDirty=true"/></label><small>{{ receiveMappings[JSON.parse(object.id)[1]]?.startsWith('local:')?'只接收模型规则，保留本机连接与Key':receiveMappings[JSON.parse(object.id)[1]]==='new'?'独立身份，保持停用；来源Key另行选择采用':'同身份的不同端点须先映射；同名不会自动合并' }}</small></template>
      </template></ObjectScopeTree><p>所选渠道需要的保存版本会一并接收。</p></details>
      <details v-if="preview" class="model-transfer-manifest-wrap"><summary>查看内容清单</summary>      <div v-if="(incoming??filePlan??upload)?.manager" class="model-transfer-manager"><strong>管理器配置</strong><p v-if="(incoming??filePlan??upload)?.manager?.preferences">桌面偏好</p><p v-if="(incoming??filePlan??upload)?.manager?.extensions">扩展设置 · Skill {{ (incoming??filePlan??upload)?.manager?.extensions?.skills.length }} · MCP {{ (incoming??filePlan??upload)?.manager?.extensions?.servers.length }}（资源与凭据按包内内容处理）</p><p v-for="endpoint in (incoming??filePlan??upload)?.manager?.webdav??[]" :key="endpoint.id">{{ endpoint.username }} · {{ endpoint.url }} / {{ endpoint.remotePath }}{{ incoming?' · 导入后未启用':'' }}</p></div>
      <div v-if="(incoming??filePlan??upload)?.manager?.accounts?.length"><h3>账户信息</h3><div v-for="account in (incoming??filePlan??upload)?.manager?.accounts??[]" :key="account.id" class="model-transfer-account-summary"><strong>{{ account.label }}</strong><span>{{ accountInformationLabel(account) }}</span><small v-if="incoming">仅登记账户信息；已有登录状态保持，不代表登录已恢复。</small></div></div><div v-if="(incoming??filePlan??upload)?.assets"><h3>扩展资源</h3><p v-for="skill in (incoming??filePlan??upload)?.assets?.skills??[]" :key="skill.name">Skill · {{ incoming?.assetMappings?.skills[skill.name]??skill.name }}{{ incoming?.assetMappings?.skills[skill.name]?'（来源：'+skill.name+'）':'' }} · {{ skill.files }} 文件 · {{ skill.bytes.toLocaleString() }} 字节{{ skill.protected?' · 受保护内容':'' }}</p><p v-for="mcp in (incoming??filePlan??upload)?.assets?.mcps??[]" :key="mcp.name">MCP · {{ incoming?.assetMappings?.mcps[mcp.name]??mcp.name }}{{ incoming?.assetMappings?.mcps[mcp.name]?'（来源：'+mcp.name+'）':'' }} · {{ mcp.secretFields.length }} 个凭据字段</p><details v-if="((incoming??filePlan??upload)?.assets?.skills??[]).some(skill=>skill.warnings.length)||((incoming??filePlan??upload)?.assets?.mcps??[]).some(mcp=>mcp.warnings.length)"><summary>资源检查结果</summary><p v-for="warning in [...((incoming??filePlan??upload)?.assets?.skills??[]).flatMap(skill=>skill.warnings),...((incoming??filePlan??upload)?.assets?.mcps??[]).flatMap(mcp=>mcp.warnings)]" :key="warning">{{ warning }}</p></details></div>
<p>模版 {{ preview.payload.templates.length }} · 渠道 {{ preview.payload.policy?.channels.length ?? 0 }}</p>
        <ul class="model-transfer-manifest"><li v-for="template in preview.payload.templates" :key="template.id"><strong>{{ template.label }}</strong><span>{{ template.revisions.length }} 个保存版本</span><code>{{ template.id }}</code><small v-if="incoming">{{ models.snapshot?.library.templates.some(t => t.id === template.id) ? '合并保存版本，保持本机默认' : '新增模版，不自动采用' }}</small></li><li v-for="channel in preview.payload.policy?.channels ?? []" :key="channel.id"><strong>{{ channel.label }}</strong><small v-if="channel.familyScope">模型族片段 · {{ channel.familyScope.templateIds.length }} 族 · {{ channel.familyScope.includeChannelDefaults?'含渠道共用默认字段（影响全部型号）':'共用默认值不携带，沿接收方配置解析' }}</small><small v-else-if="channel.modelSubset">型号子集 · {{ channel.modelSubset.length }} 项</small><span v-if="!channel.familyScope">{{ channel.selection.mode === 'all' ? '全部模型' : channel.selection.mode === 'none' ? '不选择模型' : '指定模型' }}</span><code>{{ channel.id }}</code><small v-if="channel.connection?.provider">原渠道配置 · {{ Object.keys(channel.connection.provider).length }} 个公开字段 · 认证 {{ channel.connection.provider.authMode??'key' }}</small><small v-for="node in channel.connection?.nodes??[]" :key="node.id">{{ node.name }} · {{ node.url }}</small><details v-if="channel.connection?.provider"><summary>渠道原设置</summary><pre class="model-transfer-fields">{{ JSON.stringify(channel.connection.provider,null,2) }}</pre></details><small v-if="incoming&&channel.familyScope">仅合并所选族；其他族、连接、Key、已有选择和顺序保持</small><small v-else-if="incoming">{{ models.snapshot?.policy.channels.some(c => c.id === channel.id) ? '按同一身份更新已保存配置；未选凭据保留本机值' : '新增渠道配置；运行状态沿来源' }}</small></li></ul>
        <p v-for="(binding,sourceId) in incoming?.channelBindings??{}" :key="sourceId">{{ preview.payload.policy?.channels.find(c=>c.id===sourceId)?.label }} → {{ binding.keepLocalConnection?localChannels.find(c=>c.id===binding.targetId)?.label:binding.label }} · {{ binding.keepLocalConnection?'保留本机连接和Key':'独立停用草稿' }}</p><p v-for="notice in incoming?.notices ?? []" :key="notice">{{ notice }}</p><code class="model-transfer-digest">{{ preview.digest }}</code></details>
      </template>
      <template #actions><template v-if="actionOnly&&error&&!preview&&!passwordOpen&&!pickOpen"><UiButton @click="closePreview">关闭</UiButton><UiButton :disabled="busy" @click="retryReceipt||refreshReceipt?loadRetry():remote?startSync():prepare()">重新读取</UiButton></template><template v-else-if="passwordOpen"><UiButton :disabled="busy" @click="closePreview">取消</UiButton><UiButton variant="primary" :loading="busy" :disabled="!importPassword||busy" @click="unlockFile">读取配置</UiButton></template><template v-else-if="pickOpen&&!preview"><UiButton :disabled="busy" @click="closePreview">取消</UiButton><UiButton variant="primary" :disabled="busy||!importPath" :loading="busy" @click="receive">读取配置</UiButton></template><template v-else-if="syncPreview&&!(upload?.phase==='published'||incoming?.adoption?.completed&&!retryReceipt)"><UiButton :disabled="busy" @click="closePreview">{{ syncChanges?.length===0?'关闭':'取消' }}</UiButton><UiButton variant="primary" :loading="busy" :disabled="previewInvalid||busy||!retryReceipt&&(syncChanges===null||syncChanges?.length===0)||!!incoming?.previewError||(!!incoming&&receiveDirty)" @click="confirm">确认更新</UiButton></template><UiButton v-else-if="upload?.phase === 'published'||incoming?.adoption?.completed&&!retryReceipt" variant="primary" @click="closePreview">完成</UiButton><template v-else><UiButton :disabled="busy" @click="closePreview">取消</UiButton><UiButton variant="primary" :loading="busy" :disabled="previewInvalid||!!incoming&&(receiveDirty||!!incoming.previewError||incoming.comparison?.changes===null||incoming.comparison?.changes?.length===0&&!retryReceipt)" @click="confirm">{{ incoming ? incoming.adoption?'核对原导入':'确认更新' : remote ? (upload?.phase === 'prepared' ? '确认上传' : '核对原上传') : '生成配置包' }}</UiButton></template></template>
    </UiDialog>

  </article>
</template>
<style scoped>
.transfer-file-field {display:grid;gap:6px;margin:14px 0}
.transfer-file-label {color:var(--muted);font-size:12px}
.transfer-file-row {display:flex;align-items:center;gap:12px;flex-wrap:wrap;padding:12px;border:1px solid var(--border);border-radius:8px}
.transfer-file-row .local-path {min-width:0;overflow-wrap:anywhere;color:var(--muted);font-size:12px}

:global(dialog.ui-dialog.transfer-review.wide) {width:min(720px,calc(86vw / var(--ui-zoom)));max-height:calc(78vh / var(--ui-zoom))}
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
.sx-outcomes{list-style:none;margin:0;padding:0}
.sx-outcomes li{display:flex;align-items:center;justify-content:space-between;gap:14px;padding:12px 0;border-bottom:1px solid var(--border)}
.sx-outcomes li>div{min-width:0;overflow-wrap:anywhere}
.sx-outcomes small{display:block;color:var(--muted);margin-top:4px}
.sx-preview-summary{display:flex;align-items:center;gap:12px;margin:16px 0 4px}
.sx-snapshot-facts{margin:0 0 16px;gap:8px 14px}
.sx-change-list{max-height:220px;overflow:auto;overscroll-behavior:contain;padding-right:6px}
.sx-change-list li{padding:9px 0}
.sx-detail-list{display:grid;grid-template-columns:110px minmax(0,1fr)}
.sx-detail-list dt{color:var(--muted)}
.sx-detail-list dd{margin:0;overflow-wrap:anywhere;line-height:1.6}
.sx-mark{display:inline-flex;align-items:center;border-radius:5px;padding:3px 7px;font-size:11px;line-height:1.5;background:var(--rail);color:var(--muted);white-space:nowrap}
.sx-muted{color:var(--muted)}
.export-destination{display:grid;gap:var(--space-1)}
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
.transfer-mcp-conflicts{padding:var(--space-3);background:var(--rail);border-radius:var(--radius-sm);margin-block:var(--space-3)}
.transfer-mcp-conflicts ul{list-style:none;padding:0;margin:var(--space-2) 0 0;display:grid;gap:var(--space-2)}
.transfer-mcp-conflicts small{display:block;color:var(--muted)}
.sx-mcp-source{display:flex;align-items:center;gap:8px;margin-top:6px;font-size:12px;color:var(--muted)}
.sx-mcp-source :deep(.select){min-width:140px}
.sync-head-actions{display:flex;gap:var(--space-2);flex-wrap:wrap;justify-content:flex-end}
.model-transfer{min-width:0;min-height:0;container-type:inline-size;display:flex;flex:1;flex-direction:column;gap:12px}.model-transfer>.card-head{flex:none;margin:0}.model-transfer>.card-head h2{font-size:var(--text-subtitle);margin:0}.model-transfer>.card-head p{font-size:var(--text-caption);margin:6px 0 0}.sync-scope-workspace{display:grid;flex:1;min-height:0;grid-template-rows:minmax(0,1fr);grid-template-columns:minmax(0,1fr) 220px;gap:20px;margin-top:0;min-width:0}
.sync-scope-main{display:flex;flex-direction:column;gap:10px;min-width:0;min-height:0}.sync-scope-main :deep(.scope-object-tree){display:flex;flex-direction:column;flex:1;min-height:0}.sync-scope-main :deep(.ui-tree-table){flex:1;min-height:0;overflow:auto}.sync-scope-main :deep(thead){position:sticky;top:0;z-index:1;background:var(--rail)}
.sync-scope-toolbar{display:grid;grid-template-columns:minmax(0,1fr);gap:var(--space-2);align-items:end;min-width:0}
.sync-scope-toolbar .sync-scope-search{min-width:0}
.sync-credentials-note{margin:14px 0 0;color:var(--muted);font-size:var(--text-label);line-height:1.65}
.sync-credential-block{padding-block:15px;border-bottom:1px solid var(--border-soft)}
.sync-credential-head,.sync-credential-inline,.sync-credential-password{display:flex;gap:var(--space-2);align-items:center;font-size:var(--text-control)}
.sync-credential-hint{margin:6px 0 0;color:var(--muted);font-size:var(--text-caption);line-height:1.6}
.sync-credential-head{gap:9px;align-items:flex-start}.sync-credential-head input{margin-top:3px}
.sync-credential-more{font-size:var(--text-label)}
.sync-credential-password{display:grid;gap:var(--space-1);align-items:stretch}
.sync-account-filter{display:grid;gap:var(--space-1);justify-items:start}

.sync-scope-controls{display:flex;flex-wrap:wrap;gap:var(--space-2)}.sync-scope-controls>*{flex:1 1 200px;min-width:0}.sync-scope-objects,.sync-scope-credentials{display:grid;align-content:start;gap:var(--space-3);min-width:0}
.sync-scope-credentials{display:block;min-height:0;overflow:auto;overscroll-behavior:contain;border-inline-start:1px solid var(--border);padding-inline-start:18px}
.sync-scope-credentials h3{margin:0 0 5px;font-size:var(--text-label);font-weight:650}.sync-scope-credentials label{font-size:var(--text-label);line-height:1.5}.sx-subpicks{display:grid;gap:9px;margin:12px 0 0 23px}.sx-subpicks label{font-size:12px}
@container(max-width:760px){.sync-scope-workspace{grid-template-columns:minmax(0,1fr) 180px;gap:14px}.sync-scope-credentials{padding-inline-start:12px}.sync-scope-credentials .sync-credentials-note{margin-top:10px}}
@container sync-workspace (max-height:600px){.sync-credential-block{padding-block:10px}}
@media(max-width:720px){.export-package-layout{grid-template-columns:minmax(0,1fr)}.export-passphrase{grid-template-columns:minmax(0,1fr)}}
</style>
