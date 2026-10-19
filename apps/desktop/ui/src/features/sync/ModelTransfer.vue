<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref,watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import { useModelsStore } from '@/features/models/store'
import {useAppStore} from '@/stores/app'
import { errorText, versionLabel } from '@/features/models/api'
import { modelTransferApi, scopeFor,remoteScope, accountTransferLabel, type AccountCatalogEntry, type ChannelImportMapping, type FilePlan, type IncomingPlan, type UploadPlan } from './modelTransfer'
import {getSyncConfig} from './api'
const props = defineProps<{ remote?: boolean }>()
const models = useModelsStore()
const app=useAppStore()
const busy = ref(false)
const error = ref('')
const templateIds = ref<string[]>([])
const channelIds = ref<string[]>([])
const versions = ref<Record<string,string>>({})
const includeGlobal = ref(false)
const selectionOpen = ref(false)
const filePlan = ref<FilePlan | null>(null)
const upload = ref<UploadPlan | null>(null)
const incoming = ref<IncomingPlan | null>(null)
const incomingSource=ref<IncomingPlan|null>(null)
const receiveRefs=ref<string[]>([]);const receiveChannels=ref<string[]>([]);const receiveGlobal=ref(false);const receiveDirty=ref(false)
const exportedPath = ref('')
const exportPassword=ref('');const importPassword=ref('');const passwordOpen=ref(false);const importPath=ref('')
const recompareId=ref('');const openingOriginal=ref(false)
const remoteUnlock=ref(false);const scopeSaved=ref(false)
const exportKeys=ref(false);const adoptKeys=ref(false);let unlockedPassword=''
const managerPreferences=ref(true);const managerExtensions=ref(true);const managerWebdav=ref(true);const webdavPasswords=ref(true);const activeWebdav=ref(false)
const receivePreferences=ref(false);const receiveExtensions=ref(false);const receiveEndpoints=ref<string[]>([]);const adoptWebdavPasswords=ref(true)
const skillNames=ref<string[]>([]);const mcpNames=ref<string[]>([]);const extensionCredentials=ref(false);const assetsReviewed=ref(false);const assetCatalog=ref<{skills:string[];mcps:string[]}>({skills:[],mcps:[]});const assetError=ref('')
const receiveMappings=ref<Record<string,string>>({});const receiveNewLabels=ref<Record<string,string>>({})
const localChannels=computed(()=>models.channelDraft?.channels??models.snapshot?.policy.channels??[])
const receiveSkillTargets=ref<Record<string,string>>({});const receiveMcpTargets=ref<Record<string,string>>({})
const receiveSkills=ref<string[]>([]);const receiveMcps=ref<string[]>([]);const adoptExtensionCredentials=ref(false)
const accountIds=ref<string[]>([]);const accountCatalog=ref<AccountCatalogEntry[]>([]);const accountError=ref('');const receiveAccounts=ref<string[]>([])
const allTemplates=ref(false);const allChannels=ref(false);const allSkills=ref(false);const allMcps=ref(false)
const hasAssets=computed(()=>skillNames.value.length>0||mcpNames.value.length>0||props.remote&&(allSkills.value||allMcps.value))
watch(()=>channelIds.value.length,length=>{if(!length)exportKeys.value=false})
onBeforeUnmount(()=>{exportPassword.value='';importPassword.value='';unlockedPassword=''})
const hasScope = computed(() => accountIds.value.length>0 || templateIds.value.length > 0 || channelIds.value.length > 0 || includeGlobal.value||managerPreferences.value||managerExtensions.value||!props.remote&&managerWebdav.value&&activeWebdav.value||hasAssets.value||props.remote&&(allTemplates.value||allChannels.value))
const requiresPassword=computed(()=>exportKeys.value||hasAssets.value&&extensionCredentials.value||!props.remote&&managerWebdav.value&&activeWebdav.value&&webdavPasswords.value)
const preview = computed(() => incoming.value?.package ?? upload.value?.package ?? filePlan.value?.package)
onMounted(() => { if (!models.snapshot) void models.load();if(!props.remote)void getSyncConfig().then(config=>{activeWebdav.value=!!config.endpoint}).catch(()=>{});else void loadScope();void loadAssets() })
async function loadAssets(){try{assetCatalog.value=await modelTransferApi.extensionCatalog();assetError.value=''}catch(cause){assetError.value=errorText(cause)}}
function assetScope(){return hasAssets.value?{skillNames:skillNames.value,mcpNames:mcpNames.value,extensionCredentials:extensionCredentials.value,assetsReviewed:assetsReviewed.value}:{}}
function remoteOptions(){return {...(accountIds.value.length?{accountIds:accountIds.value}:{}),preferences:managerPreferences.value,extensions:managerExtensions.value,includeKeys:exportKeys.value,...assetScope(),...(allTemplates.value||allChannels.value||allSkills.value||allMcps.value?{allTemplates:allTemplates.value,allChannels:allChannels.value,allSkills:allSkills.value,allMcps:allMcps.value}:{}),passphrase:exportPassword.value||undefined}}
async function loadScope(){try{const scope=await modelTransferApi.loadRemoteScope();accountIds.value=scope.accountIds??[];templateIds.value=scope.templateIds;channelIds.value=scope.channelIds;includeGlobal.value=scope.includeGlobal;managerPreferences.value=scope.preferences;managerExtensions.value=scope.extensions;exportKeys.value=scope.includeKeys;skillNames.value=scope.skillNames??[];mcpNames.value=scope.mcpNames??[];extensionCredentials.value=scope.extensionCredentials??false;allTemplates.value=scope.allTemplates??false;allChannels.value=scope.allChannels??false;allSkills.value=scope.allSkills??false;allMcps.value=scope.allMcps??false}catch(cause){error.value=errorText(cause)}}
async function saveRemoteScope(){if(!models.snapshot)return;await perform(async()=>{await modelTransferApi.saveRemoteScope(remoteScope(scopeFor(models.snapshot!,templateIds.value,channelIds.value,includeGlobal.value,versions.value),remoteOptions()));scopeSaved.value=true;selectionOpen.value=false})}
async function perform(action: () => Promise<void>) {
  if (busy.value) return
  busy.value = true; error.value = ''
  try { await action() } catch (cause) { error.value = errorText(cause) } finally { busy.value = false }
}
async function begin() { selectionOpen.value = true; exportedPath.value = ''; error.value = '';accountError.value='';try{accountCatalog.value=await modelTransferApi.accountCatalog()}catch(cause){accountError.value=errorText(cause)} }
async function prepare() {
  if (!models.snapshot || !hasScope.value) return
  await perform(async () => {
    const scope = scopeFor(models.snapshot!, templateIds.value, channelIds.value, includeGlobal.value, versions.value)
    if (props.remote) upload.value = await modelTransferApi.prepareUpload(scope,remoteOptions())
    else filePlan.value = await modelTransferApi.prepare(scope,exportPassword.value||undefined,exportKeys.value,{...(accountIds.value.length?{accountIds:accountIds.value}:{}),preferences:managerPreferences.value,extensions:managerExtensions.value,webdav:managerWebdav.value,webdavPasswords:webdavPasswords.value,...assetScope()})
    exportPassword.value=''
    selectionOpen.value = false
  })
}
async function receive() {
  await perform(async () => {
    if (models.channelDirty) { await models.persistDraft(); if (models.channelError) throw { message: models.channelError } }
    if (props.remote){try{incoming.value = await modelTransferApi.download()}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&cause.code==='passphrase_required'){remoteUnlock.value=true;passwordOpen.value=true;importPassword.value='';return}throw cause}}
    else {
      const selected = await open({ multiple: false, directory: false, filters: [{ name: '配置文件', extensions: ['ocxdconf','json'] }] })
      if (!selected || Array.isArray(selected)) return
      importPath.value=selected
      recompareId.value='';openingOriginal.value=false
      remoteUnlock.value=false
      try{incoming.value = await modelTransferApi.receive(selected)}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&cause.code==='passphrase_required'){passwordOpen.value=true;importPassword.value='';return}throw cause}
    }
    filePlan.value = null; upload.value = null
    adoptKeys.value=false;unlockedPassword='';if(incoming.value)selectIncoming(incoming.value)
  })
}
function selectIncoming(plan:IncomingPlan){
 receiveSkillTargets.value=Object.fromEntries((plan.assets?.skills??[]).map(s=>[s.name,plan.assetMappings?.skills[s.name]??s.name]));receiveMcpTargets.value=Object.fromEntries((plan.assets?.mcps??[]).map(m=>[m.name,plan.assetMappings?.mcps[m.name]??m.name]));receiveMappings.value=Object.fromEntries(Object.entries(plan.channelBindings??{}).map(([id,b])=>[id,b.keepLocalConnection?'local:'+b.targetId:'new']));receiveNewLabels.value=Object.fromEntries((plan.package.payload.policy?.channels??[]).map(c=>[c.id,plan.channelBindings?.[c.id]?.label??c.label+'（导入）']));receiveAccounts.value=plan.manager?.accounts?.map(a=>a.id)??[];incomingSource.value=plan;receiveRefs.value=plan.package.payload.templates.flatMap(t=>t.revisions.map(r=>JSON.stringify([t.id,r.id])));receiveChannels.value=plan.package.payload.policy?.channels.map(c=>c.id)??[];receiveGlobal.value=!!(plan.package.payload.policy?.templateRefs||plan.package.payload.policy?.channelOrder||plan.package.payload.policy?.defaultChannelId);receivePreferences.value=!!plan.manager?.preferences;receiveExtensions.value=!!plan.manager?.extensions;receiveEndpoints.value=plan.manager?.webdav.map(e=>e.id)??[];receiveSkills.value=plan.assets?.skills.map(skill=>skill.name)??[];receiveMcps.value=plan.assets?.mcps.map(mcp=>mcp.name)??[];adoptExtensionCredentials.value=plan.adoption?.extensionCredentials??false;adoptKeys.value=plan.adoption?.keys??false;adoptWebdavPasswords.value=plan.adoption?.webdav??true;receiveDirty.value=false
}
async function unlockFile(){await perform(async()=>{incoming.value=remoteUnlock.value?await modelTransferApi.download(importPassword.value):recompareId.value?(openingOriginal.value?await modelTransferApi.openReceived(recompareId.value,importPassword.value):await modelTransferApi.recompare(recompareId.value,importPassword.value)):await modelTransferApi.receive(importPath.value,importPassword.value);unlockedPassword=importPassword.value;passwordOpen.value=false;importPassword.value='';filePlan.value=null;upload.value=null;if(!openingOriginal.value)selectIncoming(incoming.value)})}
async function restoreIncoming(){await perform(async()=>{const latest=await modelTransferApi.latestIncoming();if(!latest){error.value='没有可恢复的文件接收记录';return}incoming.value=latest;filePlan.value=null;upload.value=null;unlockedPassword='';adoptKeys.value=false;selectIncoming(latest)})}
async function restoreUpload(){await perform(async()=>{const original=await modelTransferApi.latestUpload();if(!original){error.value='没有可核对的配置上传计划';return}upload.value=original;incoming.value=null;incomingSource.value=null;filePlan.value=null})}
async function recompareOriginal(){if(!incomingSource.value)return;await perform(async()=>{recompareId.value=incomingSource.value!.id;openingOriginal.value=false;try{incoming.value=await modelTransferApi.recompare(recompareId.value);selectIncoming(incoming.value)}catch(cause){if(typeof cause==='object'&&cause!==null&&'code' in cause&&cause.code==='passphrase_required'){passwordOpen.value=true;importPassword.value='';return}throw cause}})}

async function confirm() {
  if(incoming.value&&!unlockedPassword&&((adoptKeys.value||adoptWebdavPasswords.value&&incoming.value.availableWebdavPasswords?.length||adoptExtensionCredentials.value))){recompareId.value=incoming.value.id;openingOriginal.value=true;passwordOpen.value=true;importPassword.value='';return}
  await perform(async () => {
    if (incoming.value) {
      if(receiveDirty.value||incoming.value.previewError)return
      if (models.channelDirty) { await models.persistDraft(); if (models.channelError) throw { message: models.channelError } }
      const includeDav=adoptWebdavPasswords.value&&!!incoming.value.availableWebdavPasswords?.length
      let saved
      try{saved=adoptKeys.value||includeDav||adoptExtensionCredentials.value||incoming.value.protected&&!!(incoming.value.assets?.skills.length||incoming.value.assets?.mcps.length)?await modelTransferApi.accept(incoming.value.id,{adoptKeys:adoptKeys.value,...(includeDav?{adoptWebdavPasswords:true}:{}),...(adoptExtensionCredentials.value?{adoptExtensionCredentials:true}:{}),passphrase:unlockedPassword}):await modelTransferApi.accept(incoming.value.id)}catch(cause){try{const latest=await modelTransferApi.latestIncoming();if(latest?.id===incoming.value.id&&latest.adoption)incoming.value.adoption=latest.adoption}catch{}throw cause}
      models.acceptTransfer(saved);const updatePreferences=!!incoming.value.manager?.preferences; incoming.value = null;incomingSource.value=null;unlockedPassword='';if(updatePreferences)await app.loadPreferences()
    } else if (upload.value) {
      upload.value = await modelTransferApi.publish(upload.value.id)
    } else if (filePlan.value) {
      const result = await modelTransferApi.export(filePlan.value.id)
      exportedPath.value = result.path; filePlan.value = null
    }
  })
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
async function compareReceived(){if(!incomingSource.value)return;await perform(async()=>{const scope={...assetMappingScope(),...channelMappingScope(),templateRefs:receiveRefs.value.map(value=>{const [templateId,revisionId]=JSON.parse(value) as [string,string];return {templateId,revisionId}}),channelIds:receiveChannels.value,includeGlobal:receiveGlobal.value,...(incomingSource.value!.manager?{accountIds:receiveAccounts.value,preferences:receivePreferences.value,extensions:receiveExtensions.value,endpointIds:receiveEndpoints.value}:{}),...(incomingSource.value!.assets?{skillNames:receiveSkills.value,mcpNames:receiveMcps.value}:{})};incoming.value=incomingSource.value!.assets?await modelTransferApi.select(incomingSource.value!.id,scope,unlockedPassword||undefined):await modelTransferApi.select(incomingSource.value!.id,scope);if(!incoming.value.availableKeys?.length)adoptKeys.value=false;if(!incoming.value.availableExtensionCredentials)adoptExtensionCredentials.value=false;receiveDirty.value=false})}

function closePreview() { if (!busy.value) { incoming.value = null;incomingSource.value=null; filePlan.value = null; upload.value = null;unlockedPassword='' } }
</script>
<template>
  <article class="card model-transfer">
    <div class="card-head"><div><h2>{{ remote ? '配置同步 · WebDAV' : '文件同步' }}</h2><p>{{ remote ? '本设备范围同时约束上传和接收；下载后比较并采用。' : '导入导出管理器配置和模型；模型采用后进入本机草稿。' }}</p></div></div>
    <p v-if="models.error && !models.snapshot" class="model-error" role="alert">{{ models.error }}</p>
    <p v-if="error && !selectionOpen && !preview" class="model-error" role="alert">{{ error }}</p>
    <div class="controls"><UiButton :disabled="busy || !models.snapshot" @click="begin">{{ remote ? '选择本设备范围' : '导出配置' }}</UiButton><UiButton :loading="busy" @click="receive">{{ remote ? '下载并比较' : '导入配置' }}</UiButton><UiButton v-if="!remote" variant="ghost" :disabled="busy" @click="restoreIncoming">恢复上次接收</UiButton><UiButton v-else variant="ghost" :disabled="busy" @click="restoreUpload">查看原上传计划</UiButton><UiButton variant="ghost" @click="models.load">重新读取</UiButton></div><p v-if="remote&&scopeSaved" role="status">本设备范围已保存，尚未上传。</p>
    <div v-if="exportedPath" class="model-transfer-path"><span>导出文件</span><code>{{ exportedPath }}</code><UiButton size="sm" variant="ghost" @click="app.openManagedPathByKey('exports')">打开导出目录</UiButton></div>
    <UiDialog :open="selectionOpen" :title="remote ? '本设备配置同步范围' : '选择配置导出范围'" :busy="busy" wide @close="selectionOpen = false">
      <p v-if="error" class="model-error" role="alert">{{ error }}</p>
      <fieldset><legend>管理器配置</legend><label><input v-model="managerPreferences" type="checkbox" :disabled="busy" />桌面偏好</label><label><input v-model="managerExtensions" type="checkbox" :disabled="busy" />扩展设置与名称清单</label><label v-if="!remote"><input v-model="managerWebdav" type="checkbox" :disabled="busy" />WebDAV连接配置</label><label v-if="!remote&&managerWebdav"><input v-model="webdavPasswords" type="checkbox" :disabled="busy" />包含连接密码</label><UiButton size="sm" variant="ghost" :disabled="busy" @click="exportKeys=false;webdavPasswords=false;extensionCredentials=false">分享预设：移除凭据</UiButton></fieldset><fieldset><legend>扩展资源</legend><p v-if="assetError" role="alert" class="model-error">{{ assetError }}</p><div class="model-inline"><UiButton size="sm" :disabled="busy" @click="loadAssets">读取扩展目录</UiButton><label v-if="remote"><input v-model="allSkills" type="checkbox" :disabled="busy" />全部Skill（含后续新增）</label><label v-if="remote"><input v-model="allMcps" type="checkbox" :disabled="busy" />全部MCP（含后续新增）</label></div><label v-for="skill in assetCatalog.skills" :key="'skill:'+skill"><input v-model="skillNames" type="checkbox" :value="skill" :disabled="busy||allSkills" />Skill · {{ skill }}</label><label v-for="mcp in assetCatalog.mcps" :key="'mcp:'+mcp"><input v-model="mcpNames" type="checkbox" :value="mcp" :disabled="busy||allMcps" />MCP · {{ mcp }}</label><template v-if="hasAssets"><label><input v-model="extensionCredentials" type="checkbox" :disabled="busy" />携带所选扩展凭据（独立口令保护）</label><label><input v-model="assetsReviewed" type="checkbox" :disabled="busy" />已检查所选Skill自定义文本、资源及MCP自定义字段</label></template></fieldset><fieldset><legend>账户信息</legend><p v-if="accountError" class="model-error" role="alert">{{ accountError }}</p><p v-else-if="!accountCatalog.length">暂无账户信息</p><label v-for="entry in accountCatalog" :key="entry.account.id" class="model-transfer-account"><input v-model="accountIds" type="checkbox" :value="entry.account.id" :disabled="busy" /><span><strong>{{ entry.account.label }}</strong><small>{{ entry.account.provider }} · {{ accountTransferLabel(entry.account.provider) }}</small></span></label><p v-if="accountCatalog.length">仅包含账户身份与显示信息；已有本机登录状态会保留。</p></fieldset><fieldset><legend>模版保存版本</legend><label v-if="remote"><input v-model="allTemplates" type="checkbox" :disabled="busy" />全部模版（含后续新增）</label><div v-for="template in models.snapshot?.library.templates ?? []" :key="template.id" class="model-transfer-choice"><label><input v-model="templateIds" type="checkbox" :value="template.id" :disabled="busy||allTemplates" />{{ template.label }}</label><UiSelectField v-if="!remote" :model-value="versions[template.id] ?? template.revisions.at(-1)!.id" :options="[...template.revisions].reverse().map(r => ({ value: r.id, label: versionLabel(r, template.source === 'builtin' && r === template.revisions[0]) }))" :label="template.label + '导出版本'" :disabled="busy || !templateIds.includes(template.id)" @update:model-value="versions[template.id] = $event" /><span v-else>全部已保存版本</span></div></fieldset>
      <fieldset><legend>已提交渠道</legend><label v-if="remote"><input v-model="allChannels" type="checkbox" :disabled="busy" />全部已提交渠道（含后续新增）</label><label v-for="channel in models.snapshot?.policy.channels ?? []" :key="channel.id"><input v-model="channelIds" type="checkbox" :value="channel.id" :disabled="busy||allChannels" />{{ channel.label }}</label><p v-if="!models.snapshot?.policy.channels.length">暂无已提交渠道</p></fieldset>
      <label><input v-model="includeGlobal" type="checkbox" :disabled="busy" />全局采用及渠道组顺序</label>
      <label>{{ remote?'配置同步保护口令（可选，与连接密码独立）':'文件保护口令（可选）' }}<input v-model="exportPassword" class="input" type="password" autocomplete="new-password" :disabled="busy" placeholder="留空使用完整性校验" /></label>
      <label><input v-model="exportKeys" type="checkbox" :disabled="busy||(!channelIds.length&&!allChannels)" />携带所选渠道的Key（须设置独立口令）</label>
      <template #actions><UiButton v-if="remote" :disabled="!hasScope||busy" @click="saveRemoteScope">仅保存范围</UiButton><UiButton variant="primary" :disabled="!hasScope||(requiresPassword&&!exportPassword)||(hasAssets&&!assetsReviewed)" :loading="busy" @click="prepare">预览清单</UiButton></template>
    </UiDialog>
    <UiDialog :open="!!preview" :title="incoming ? '比较配置文件' : remote ? '确认配置上传' : '确认配置导出'" :busy="busy" wide @close="closePreview">
      <p v-if="error" class="model-error" role="alert">{{ error }}</p>
      <p v-if="filePlan">文件保护：{{ filePlan.protected?'独立口令加密':'未加密，完整性校验' }}</p>
      <p v-if="filePlan?.credentialCount">携带凭据：{{ filePlan.credentialCount }} 项</p>
      <p v-if="incoming?.adoption">{{ incoming.adoption.completed?'原导入已完成；当前本机配置保持。':'原导入采用范围已冻结，请核对同一操作。' }}</p><label v-if="incoming?.availableExtensionCredentials"><input v-model="adoptExtensionCredentials" type="checkbox" :disabled="busy||receiveDirty||!!incoming.adoption" />采用所选扩展凭据（新本机引用及受保护资源）</label><label v-if="incoming?.availableWebdavPasswords?.length"><input v-model="adoptWebdavPasswords" type="checkbox" :disabled="busy||receiveDirty||!!incoming?.adoption" />采用包内连接密码（{{ incoming.availableWebdavPasswords.length }} 项），连接保持未启用</label><label v-if="incoming?.availableKeys?.length"><input v-model="adoptKeys" type="checkbox" :disabled="busy||receiveDirty||!!incoming?.adoption" />采用所选渠道的Key（{{ incoming.availableKeys.length }} 项），写入新本机引用和渠道草稿</label>
      <details v-if="incomingSource" open class="model-transfer-scope"><summary>选择接收范围</summary><UiButton size="sm" variant="ghost" :disabled="busy||!!incoming?.adoption" @click="recompareOriginal">按当前基准重新比较原文件</UiButton><label v-if="incomingSource.manager?.preferences"><input v-model="receivePreferences" type="checkbox" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" />桌面偏好</label><label v-if="incomingSource.manager?.extensions"><input v-model="receiveExtensions" type="checkbox" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" />扩展设置</label><label v-for="account in incomingSource.manager?.accounts??[]" :key="'account:'+account.id" class="model-transfer-account"><input v-model="receiveAccounts" type="checkbox" :value="account.id" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" /><span>{{ account.label }}<small>{{ account.provider }} · 仅账户信息，保留本机登录态</small></span></label><label v-for="endpoint in incomingSource.manager?.webdav??[]" :key="endpoint.id"><input v-model="receiveEndpoints" type="checkbox" :value="endpoint.id" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" />{{ endpoint.url }} · {{ endpoint.username }}</label><div v-for="skill in incomingSource.assets?.skills??[]" :key="'asset-skill:'+skill.name" class="model-import-asset"><label><input v-model="receiveSkills" type="checkbox" :value="skill.name" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" />Skill · {{ skill.name }} · {{ skill.files }} 文件</label><label v-if="receiveSkills.includes(skill.name)">目标Skill名称<input v-model="receiveSkillTargets[skill.name]" class="input" :aria-label="skill.name+'目标Skill名称'" maxlength="256" :disabled="busy||!!incoming?.adoption" @input="receiveDirty=true" /></label></div><div v-for="mcp in incomingSource.assets?.mcps??[]" :key="'asset-mcp:'+mcp.name" class="model-import-asset"><label><input v-model="receiveMcps" type="checkbox" :value="mcp.name" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" />MCP · {{ mcp.name }}</label><label v-if="receiveMcps.includes(mcp.name)">目标MCP名称<input v-model="receiveMcpTargets[mcp.name]" class="input" :aria-label="mcp.name+'目标MCP名称'" maxlength="256" :disabled="busy||!!incoming?.adoption" @input="receiveDirty=true" /></label></div><div v-for="template in incomingSource.package.payload.templates" :key="template.id"><strong>{{ template.label }}</strong><label v-for="revision in template.revisions" :key="revision.id"><input v-model="receiveRefs" type="checkbox" :value="JSON.stringify([template.id,revision.id])" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" />{{ versionLabel(revision) }}</label></div><div v-for="channel in incomingSource.package.payload.policy?.channels??[]" :key="channel.id" class="model-import-channel"><label><input v-model="receiveChannels" type="checkbox" :value="channel.id" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" />{{ channel.label }}</label><template v-if="receiveChannels.includes(channel.id)"><UiSelectField :model-value="receiveMappings[channel.id]??'identity'" :label="channel.label+'接收方式'" :disabled="busy||!!incoming?.adoption" :options="[{value:'identity',label:'按原渠道身份合并'},...(channel.connection?.nodes.length?[{value:'new',label:'建立独立新渠道'}]:[]),...localChannels.map(c=>({value:'local:'+c.id,label:'映射到 '+c.label+' · 保留本机连接和Key'}))]" @update:model-value="receiveMappings[channel.id]=$event;receiveDirty=true" /><label v-if="receiveMappings[channel.id]==='new'">新渠道名称<input v-model="receiveNewLabels[channel.id]" class="input" maxlength="256" :disabled="busy||!!incoming?.adoption" @input="receiveDirty=true" /></label><small v-if="receiveMappings[channel.id]?.startsWith('local:')">只接收模型规则；协议、服务节点、当前节点和Key使用所选本机渠道。</small><small v-else-if="receiveMappings[channel.id]==='new'">独立身份，保持停用；来源Key需另行选择采用。</small><small v-else>同身份的不同端点须先映射；同名渠道不会自动合并。</small></template></div><label v-if="incomingSource.package.payload.policy?.templateRefs||incomingSource.package.payload.policy?.channelOrder||incomingSource.package.payload.policy?.defaultChannelId"><input v-model="receiveGlobal" type="checkbox" :disabled="busy||!!incoming?.adoption" @change="receiveDirty=true" />全局采用及渠道顺序</label><p>所选渠道需要的保存版本会一并接收。</p><UiButton :loading="busy" @click="compareReceived">比较所选范围</UiButton></details>
      <div v-if="(incoming??filePlan??upload)?.manager" class="model-transfer-manager"><strong>管理器配置</strong><p v-if="(incoming??filePlan??upload)?.manager?.preferences">桌面偏好</p><p v-if="(incoming??filePlan??upload)?.manager?.extensions">扩展设置 · Skill {{ (incoming??filePlan??upload)?.manager?.extensions?.skills.length }} · MCP {{ (incoming??filePlan??upload)?.manager?.extensions?.servers.length }}（资源与凭据按包内内容处理）</p><p v-for="endpoint in (incoming??filePlan??upload)?.manager?.webdav??[]" :key="endpoint.id">{{ endpoint.username }} · {{ endpoint.url }} / {{ endpoint.remotePath }}{{ incoming?' · 导入后未启用':'' }}</p></div>
      <div v-if="(incoming??filePlan??upload)?.manager?.accounts?.length"><h3>账户信息</h3><div v-for="account in (incoming??filePlan??upload)?.manager?.accounts??[]" :key="account.id" class="model-transfer-account-summary"><strong>{{ account.label }}</strong><span>{{ account.provider }} · {{ accountTransferLabel(account.provider) }}</span><small v-if="incoming">仅登记账户信息；已有登录状态保持，不代表登录已恢复。</small></div></div><div v-if="(incoming??filePlan??upload)?.assets"><h3>扩展资源</h3><p v-for="skill in (incoming??filePlan??upload)?.assets?.skills??[]" :key="skill.name">Skill · {{ incoming?.assetMappings?.skills[skill.name]??skill.name }}{{ incoming?.assetMappings?.skills[skill.name]?'（来源：'+skill.name+'）':'' }} · {{ skill.files }} 文件 · {{ skill.bytes.toLocaleString() }} 字节{{ skill.protected?' · 受保护内容':'' }}</p><p v-for="mcp in (incoming??filePlan??upload)?.assets?.mcps??[]" :key="mcp.name">MCP · {{ incoming?.assetMappings?.mcps[mcp.name]??mcp.name }}{{ incoming?.assetMappings?.mcps[mcp.name]?'（来源：'+mcp.name+'）':'' }} · {{ mcp.secretFields.length }} 个凭据字段</p><details v-if="((incoming??filePlan??upload)?.assets?.skills??[]).some(skill=>skill.warnings.length)||((incoming??filePlan??upload)?.assets?.mcps??[]).some(mcp=>mcp.warnings.length)"><summary>资源检查结果</summary><p v-for="warning in [...((incoming??filePlan??upload)?.assets?.skills??[]).flatMap(skill=>skill.warnings),...((incoming??filePlan??upload)?.assets?.mcps??[]).flatMap(mcp=>mcp.warnings)]" :key="warning">{{ warning }}</p></details></div>
      <template v-if="preview"><p>模版 {{ preview.payload.templates.length }} · 渠道 {{ preview.payload.policy?.channels.length ?? 0 }}</p>
        <ul class="model-transfer-manifest"><li v-for="template in preview.payload.templates" :key="template.id"><strong>{{ template.label }}</strong><span>{{ template.revisions.length }} 个保存版本</span><code>{{ template.id }}</code><small v-if="incoming">{{ models.snapshot?.library.templates.some(t => t.id === template.id) ? '合并保存版本，保持本机默认' : '新增模版，不自动采用' }}</small></li><li v-for="channel in preview.payload.policy?.channels ?? []" :key="channel.id"><strong>{{ channel.label }}</strong><span>{{ channel.selection.mode === 'all' ? '全部模型' : channel.selection.mode === 'none' ? '不选择模型' : '指定模型' }}</span><code>{{ channel.id }}</code><small v-for="node in channel.connection?.nodes??[]" :key="node.id">{{ node.name }} · {{ node.url }}</small><small v-if="incoming">{{ models.snapshot?.policy.channels.some(c => c.id === channel.id) ? '保留本机连接，策略进入草稿' : '新增渠道草稿，确认节点及凭据后再保存，保持停用' }}</small></li></ul>
        <p v-for="(binding,sourceId) in incoming?.channelBindings??{}" :key="sourceId">{{ preview.payload.policy?.channels.find(c=>c.id===sourceId)?.label }} → {{ binding.keepLocalConnection?localChannels.find(c=>c.id===binding.targetId)?.label:binding.label }} · {{ binding.keepLocalConnection?'保留本机连接和Key':'独立停用草稿' }}</p><p v-for="notice in incoming?.notices ?? []" :key="notice">{{ notice }}</p><code class="model-transfer-digest">{{ preview.digest }}</code>
      </template>
      <template #actions><UiButton v-if="upload?.phase === 'published'||incoming?.adoption?.completed" variant="primary" @click="closePreview">完成</UiButton><UiButton v-else variant="primary" :loading="busy" :disabled="!!incoming&&(receiveDirty||!!incoming.previewError)" @click="confirm">{{ incoming ? incoming.adoption?'核对原导入':'采用到本机' : remote ? (upload?.phase === 'prepared' ? '确认上传' : '核对原上传') : '生成导出文件' }}</UiButton></template>
    </UiDialog>
    <UiDialog :open="passwordOpen" :title="remoteUnlock?'读取受保护同步快照':'打开受保护配置文件'" :busy="busy" @close="passwordOpen=false;importPassword=''">
      <label>文件保护口令<input v-model="importPassword" class="input" type="password" autocomplete="off" :disabled="busy" /></label><p v-if="error" class="model-error" role="alert">{{ error }}</p><template #actions><UiButton variant="primary" :disabled="!importPassword" :loading="busy" @click="unlockFile">解密并比较</UiButton></template>
    </UiDialog>
  </article>
</template>
<style scoped>
fieldset{border:1px solid var(--border);border-radius:var(--radius);margin:0 0 18px;padding:14px;display:grid;gap:12px;min-width:0}
.model-transfer-account span,.model-transfer-account-summary{display:grid;gap:4px;min-width:0}.model-transfer-account-summary{margin:12px 0;overflow-wrap:anywhere}.model-transfer-account small{color:var(--muted)}
label{display:flex;align-items:center;gap:8px}
.model-error{color:var(--red);overflow-wrap:anywhere}
.model-transfer-choice{display:grid;grid-template-columns:minmax(120px,1fr) minmax(0,2fr);align-items:center;gap:12px}
.model-transfer-path{display:grid;gap:8px;overflow-wrap:anywhere}
.model-transfer-manifest{list-style:none;padding:0;display:grid;gap:12px}
.model-transfer-manifest li{display:grid;grid-template-columns:1fr auto;gap:6px;border-bottom:1px solid var(--border);padding:12px 0}
.model-transfer-manifest code,.model-transfer-manifest small{grid-column:1/-1;overflow-wrap:anywhere;color:var(--muted)}
.model-transfer-digest{display:block;overflow-wrap:anywhere;font-size:var(--text-small)}
.model-import-asset{padding:10px 0;display:grid;gap:8px}.model-import-asset .input{flex:1;min-width:0}
.model-import-channel{border:1px solid var(--border);border-radius:var(--radius);padding:12px;display:grid;gap:10px}.model-import-channel small{color:var(--muted)}
.model-transfer-scope{border-bottom:1px solid var(--border);padding-bottom:16px;margin-bottom:16px}.model-transfer-scope>div{display:grid;gap:8px;margin:14px 0}.model-transfer-scope>label{margin:10px 0}
@media(max-width:600px){.model-transfer-choice{grid-template-columns:1fr}}
</style>
