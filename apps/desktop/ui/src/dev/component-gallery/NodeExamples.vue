<script setup lang="ts">
import {computed,ref} from 'vue'
import NodePicker from '@/features/models/NodePicker.vue'
import ChannelNodeSelect from '@/features/models/ChannelNodeSelect.vue'
import ModalContent from '@/components/patterns/ModalContent.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import {emptyRules,type ServiceNode,type NodeMeasurement,type ChannelPolicy} from '@/features/models/api'
const props=defineProps<{long?:boolean}>()
const mode=ref('results');const active=ref('one');const modal=ref(false);const values=ref({name:'原输入内容'});const result=ref('')
const nodes=ref<ServiceNode[]>([{id:'one',name:'主节点',url:'https://primary.example.test/v1'},{id:'two',name:'备用节点',url:'https://secondary.example.test/v1'},{id:'three',name:'暂不可用',url:'https://failed.example.test/v1'}])
const shown=computed(()=>props.long?nodes.value.map(n=>({...n,name:n.name+' · 较长的中文与 English 服务节点说明'})):nodes.value)
const results=computed<Record<string,NodeMeasurement>>(()=>mode.value==='empty'||mode.value==='running'?{} as Record<string,NodeMeasurement>:{one:{nodeId:'one',elapsedMs:42,modelCount:18,error:null},two:{nodeId:'two',elapsedMs:85,modelCount:36,error:null},three:{nodeId:'three',elapsedMs:null,modelCount:null,error:'discovery_timeout'}})
const menuChannel=computed<ChannelPolicy>(()=>({id:'gallery-node-channel',label:'只读节点菜单',templateRefs:[],modelRefs:{},connection:{providerId:'fixture',protocol:'openai-chat',nodes:mode.value==='empty'?[]:shown.value,activeNodeId:active.value,credentialRef:null,enabled:false,allowPrivateNetwork:false},selection:{mode:'none'},discovered:[],modelOrder:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}))
function add(){nodes.value.push({id:crypto.randomUUID(),name:'新节点',url:''})}
</script>
<template>
 <UiCard id="nodes"><UiCardHeader><h2>节点选择与检测</h2><p>复用渠道编辑器的正式纯视图；检测由业务适配器执行，本页只演示状态和选择意图。</p></UiCardHeader><UiSelectField v-model="mode" label="节点样例状态" :options="[{value:'results',label:'完成与失败'},{value:'running',label:'检测中'},{value:'cancelled',label:'取消后保留结果'},{value:'disabled',label:'禁用'},{value:'empty',label:'空节点'}]" /><p>实际列表节点菜单（只读展示，不调用检测服务）</p><ChannelNodeSelect :channel="menuChannel" readonly :disabled="mode==='disabled'" /><NodePicker :nodes="mode==='empty'?[]:shown" :active-id="active" :measuring="mode==='running'" :disabled="mode==='disabled'" :pending="mode==='running'?['one','two']:[]" :results="results" :stopped="mode==='cancelled'" @update:nodes="nodes=$event" @update:active-id="active=$event" @measure="mode='results'" @stop="mode='cancelled'" @add="add" /></UiCard>
 <UiCard id="modal-content"><UiCardHeader><h2>应用确认与输入内容</h2><p>与应用Modal共用内容视图和按钮；应用仍由AppModal唯一处理状态、焦点与解析，本页没有应用store或真实副作用。</p></UiCardHeader><UiButton @click="modal=true">打开输入确认样例</UiButton><p role="status">{{ result }}</p><UiDialog :open="modal" title="应用确认内容样例" @close="modal=false"><ModalContent title-id="gallery-modal-content-title" :modal="{title:long?'确认较长的中文与 English 操作说明及目标名称':'确认配置',body:'<p>内容样例；表单取消保留原调用方状态。</p>',fields:[{key:'name',label:'名称'}],confirmLabel:'确认',cancelLabel:'取消',primaryKind:'primary'}" :values="values" @field="(key,value)=>values={...values,[key]:value}" @confirm="result='确认样例：'+values.name;modal=false" @cancel="modal=false" /></UiDialog></UiCard>
</template>
