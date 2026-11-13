<script setup lang="ts">
import {computed, onMounted, ref, watch} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import UiButton from '@/components/ui/UiButton.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import {useLifecycleStore} from '@/app/lifecycle/store'
import {useAppStore} from '@/stores/app'
type Mode='auto'|'fixed'
interface PortSnapshot {mode:Mode;port:number|null;revision:string;reconciliationRequired:boolean}
const props=defineProps<{disabled?:boolean}>()
const emit=defineEmits<{saved:[]}>()
const lifecycle=useLifecycleStore(),app=useAppStore()
const saved=ref<PortSnapshot|null>(null),mode=ref<Mode>('auto'),port=ref(''),loading=ref(true),saving=ref(false),error=ref(''),restartOpen=ref(false)
const running=computed(()=>lifecycle.statusSnapshot?.matrix.runtime==='running')
const actual=computed(()=>running.value?lifecycle.statusSnapshot?.port:null)
const busy=computed(()=>loading.value||saving.value||lifecycle.processActionBusy||props.disabled)
const valid=computed(()=>mode.value==='auto'||/^\d+$/.test(port.value)&&Number(port.value)>0&&Number(port.value)<=65535)
const dirty=computed(()=>!!saved.value&&(!saved.value.port||mode.value!==saved.value.mode||mode.value==='fixed'&&Number(port.value)!==saved.value.port))
const pending=computed(()=>running.value&&!!saved.value?.port&&actual.value!==saved.value.port)
const summary=computed(()=>loading.value?'正在读取端口设置…':saving.value?'正在检查并保存端口…':!saved.value?'端口设置尚未读取。':saved.value.reconciliationRequired?'上次端口保存结果待核对。':!saved.value.port?'尚未保存端口，请检查并保存后启动。':'已保存：'+(saved.value.mode==='auto'?'自动分配':'指定端口')+' · '+saved.value.port+'。'+(pending.value?'当前运行端口 '+(actual.value??'读取中')+'，下次启动生效。':running.value?'当前设置已生效。':'下次启动生效。'))
function accept(value:PortSnapshot){saved.value=value;mode.value=value.mode;port.value=String(value.port??'');}
function message(e:unknown){return typeof e==='object'&&e!==null&&'detail' in e&&typeof e.detail==='string'?e.detail:'端口操作未确认完成，请刷新核对。';}
async function load(){loading.value=true;try{accept(await invoke<PortSnapshot>('runtime_port_load'));error.value='';}catch(e){error.value=message(e);}finally{loading.value=false;}}
async function save(){if(busy.value||!saved.value||!valid.value)return;saving.value=true;error.value='';try{const value=await invoke<PortSnapshot>('runtime_port_save',{request:{mode:mode.value,port:mode.value==='fixed'?Number(port.value):null,expectedRevision:saved.value.revision}});accept(value);if(!value.reconciliationRequired)emit('saved');app.showToast('端口 '+value.port+' 已保存。');}catch(e){error.value=message(e);try{accept(await invoke<PortSnapshot>('runtime_port_load'));}catch{/* retain error and last readable snapshot */}}finally{saving.value=false;}}
async function reconcile(){if(busy.value)return;saving.value=true;try{accept(await invoke<PortSnapshot>('runtime_port_reconcile'));error.value='';}catch(e){error.value=message(e);}finally{saving.value=false;}}
function revert(){if(saved.value){accept(saved.value);error.value='';}}
async function action(restart=false){restartOpen.value=false;try{const result=await lifecycle.requestProcessAction(restart?'restart':'start');await load();if(result?.failure)error.value=result.failure.message;}catch(e){error.value=message(e);}}
watch(()=>lifecycle.statusSnapshot?.port,()=>{if(!dirty.value&&!saving.value)void load();})
onMounted(load)
</script>
<template>
  <UiCard id="runtime-port-settings" data-testid="runtime-port-settings">
    <UiCardHeader><h2>OpenCodex 服务</h2><p>设置代理服务的运行端口。</p></UiCardHeader>
    <fieldset class="port-field" :disabled="busy"><legend>运行端口</legend>
      <div class="port-modes"><label><input v-model="mode" type="radio" value="auto" name="runtime-port-mode">自动分配</label><label><input v-model="mode" type="radio" value="fixed" name="runtime-port-mode">指定端口</label></div>
      <div v-if="mode==='fixed'" class="port-input"><label for="runtime-port-input">端口号</label><input id="runtime-port-input" v-model="port" class="text-input" inputmode="numeric" autocomplete="off" :aria-invalid="!valid" aria-describedby="runtime-port-error"></div>
      <p v-if="!valid" id="runtime-port-error" role="alert">请输入1～65535之间的整数端口。</p>
    </fieldset>
    <p class="port-summary" role="status" aria-live="polite">{{summary}}</p><p v-if="error" role="alert">{{error}}</p>
    <div class="port-actions">
      <UiButton variant="primary" :disabled="busy||!valid||!dirty||saved?.reconciliationRequired" @click="save">{{saving?'正在检查空闲端口…':mode==='auto'?'检查并保存':'保存设置'}}</UiButton>
      <UiButton variant="ghost" :disabled="busy||!dirty" @click="revert">还原修改</UiButton>
      <UiButton v-if="mode==='auto'&&saved?.port" variant="ghost" :disabled="busy||saved.reconciliationRequired" @click="save">重新分配</UiButton>
      <UiButton v-if="pending" :disabled="busy||dirty||saved?.reconciliationRequired||lifecycle.processActionBlocked('restart')" @click="restartOpen=true">重启并应用</UiButton>
      <UiButton v-if="!running" :disabled="busy||dirty||!saved?.port||saved.reconciliationRequired||!lifecycle.statusSnapshot?.can_start||lifecycle.processActionBlocked('start')" @click="action()">启动 OpenCodex</UiButton>
      <UiButton v-if="saved?.reconciliationRequired" :disabled="busy" @click="reconcile">核对上次保存</UiButton>
      <UiButton v-if="!saved||error" variant="ghost" :disabled="busy" @click="load">刷新设置</UiButton>
    </div>
    <UiDialog :open="restartOpen" title="重启并应用端口设置" @close="restartOpen=false">
      <p>将停止当前用户的 OpenCodex 服务，按已保存的设置重新启动。进行中的请求可能中断。</p>
      <dl class="port-facts"><dt>当前端口</dt><dd>{{actual??'读取中'}}</dd><dt>下次启动</dt><dd>{{saved?.port}}</dd></dl>
      <template #actions><UiButton variant="ghost" @click="restartOpen=false">取消</UiButton><UiButton :disabled="busy||lifecycle.processActionBlocked('restart')" @click="action(true)">确认重启</UiButton></template>
    </UiDialog>
  </UiCard>
</template>
<style scoped>
.port-field{border:0;padding:0;margin:18px 0 0;min-width:0}.port-field legend{font-weight:600;margin-bottom:10px}.port-modes{display:flex;flex-wrap:wrap;gap:12px;margin-bottom:14px}.port-modes label{display:flex;gap:7px;align-items:center;border:1px solid var(--border);border-radius:8px;padding:8px 12px;cursor:pointer}.port-modes label:has(:checked){border-color:var(--text)}.port-modes input{margin:0;accent-color:var(--text)}.port-input{max-width:280px}.port-input label{display:block;margin-bottom:6px}.port-input input{width:100%;box-sizing:border-box;min-height:36px;padding:8px 12px;border:1px solid var(--border);border-radius:8px;background:var(--control-bg,var(--surface));color:var(--text);font:inherit}.port-input input[aria-invalid=true]{border-color:var(--red)}.port-summary{color:var(--muted);margin:14px 0;line-height:1.6}.port-actions{display:flex;gap:8px;flex-wrap:wrap}.port-facts{display:grid;grid-template-columns:100px minmax(0,1fr);gap:12px}.port-facts dd{margin:0}[role=alert]{color:var(--red)}
</style>
