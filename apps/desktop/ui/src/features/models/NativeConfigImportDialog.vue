<script setup lang="ts">
import {ref,watch} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import {open as chooseFile} from '@tauri-apps/plugin-dialog'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import {useModelsV2Store} from './store-v2'
import {useFeedbackStore} from '@/app/feedback/store'

const props=defineProps<{open:boolean}>()
const emit=defineEmits<{close:[]}>()
const models=useModelsV2Store()
const selected=ref<{id:string;name:string;context:string}|null>(null)
const selecting=ref(false), applying=ref(false), error=ref('')
let generation=0
watch(()=>props.open,()=>{generation++;selected.value=null;error.value='';selecting.value=false})
const errors:Record<string,string>={
  native_import_source_invalid:'无法读取所选 JSON 文件，请重新选择。',
  native_import_source_is_target:'所选文件就是当前配置，请选择要导入的文件。',
  native_import_config_invalid:'所选文件未通过当前 OpenCodex 的配置校验，请检查后重新选择。',
  native_import_port_required:'请先在安装配置中保存并核对本机运行端口。',
  native_import_draft_active:'当前有未保存的渠道更改，请先保存或还原。',
  native_import_runtime_unavailable:'当前官方运行程序未准备完成，暂时无法校验配置。',
  native_import_runtime_not_stopped:'请先停止 OpenCodex 并确认服务已退出，然后重新选择文件。',
  native_import_backup_failed:'未能完整备份当前配置，原配置已保留。',
  native_import_stale:'所选文件或本机配置已变化，请重新选择文件。',
  model_transaction_cas:'本机配置已变化，请重新选择文件。',
  model_transaction_pending:'上一次配置操作仍需核对，请先处理后再导入。',
}
function message(cause:unknown){
  const code=typeof cause==='object'&&cause!==null&&'code' in cause?String(cause.code):''
  return errors[code]??'未能确认导入结果，请核对当前配置和备份。'
}
function hasEdits(){return models.fileBusy||models.fileNeedsReconcile||!!models.pendingRows.length||!!models.channelIntents.length||!!models.modelEdits.length||!!Object.keys(models.pendingCredentialKinds).length}
async function select(){
  if(selecting.value||applying.value)return
  if(hasEdits()){error.value=errors.native_import_draft_active!;return}
  selecting.value=true;selected.value=null;error.value=''
  const attempt=++generation, context=models.fileContext()
  try{
    const path=await chooseFile({title:'选择配置文件',multiple:false,directory:false,filters:[{name:'JSON',extensions:['json']}]})
    if(typeof path!=='string'||attempt!==generation||!props.open)return
    const id=await invoke<string>('model_v2_native_import_select',{path})
    if(attempt!==generation||!props.open)return
    if(context!==models.fileContext()||hasEdits()){error.value=errors.native_import_stale!;return}
    selected.value={id,name:path.split(/[\\/]/).pop()??'JSON',context}
  }catch(cause){if(attempt===generation)error.value=message(cause)}
  finally{if(attempt===generation)selecting.value=false}
}
async function apply(){
  const item=selected.value
  if(!item||applying.value||selecting.value)return
  if(item.context!==models.fileContext()||hasEdits()){selected.value=null;error.value=errors.native_import_stale!;return}
  applying.value=true;models.fileBusy=true;error.value=''
  let success=false
  try{
    const outcome=await invoke<{backupId:string|null;changed:boolean;externalAuth:boolean}>('model_v2_native_import_apply',{id:item.id})
    success=true
    const info=outcome.changed?'配置已导入，原文件已备份。下次启动 OpenCodex 时生效。':'所选文件与本机配置一致。'
    useFeedbackStore().showToast(info+(outcome.externalAuth?'外部账户或系统凭据未随文件导入，请核对。':''),outcome.externalAuth?{kind:'info'}:undefined)
    emit('close')
  }catch(cause){error.value=message(cause)}
  finally{models.fileBusy=false;applying.value=false;selected.value=null}
  if(success)await models.loadChannels()
}
</script>

<template>
  <UiDialog :open="open" title="导入配置" :busy="selecting||applying" @close="emit('close')">
    <div class="native-config-file">
      <span>{{selected?.name??'尚未选择文件'}}</span>
      <UiButton :disabled="selecting||applying" :loading="selecting" @click="select">选择文件</UiButton>
    </div>
    <p v-if="error" class="model-error" role="alert">{{error}}</p>
    <template #actions="{close}">
      <UiButton :disabled="selecting||applying" @click="close">取消</UiButton>
      <UiButton variant="primary" :loading="applying" :disabled="!selected||selecting||applying" @click="apply">导入</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
.native-config-file{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);min-width:0}
.native-config-file>span{min-width:0;overflow-wrap:anywhere}
</style>
