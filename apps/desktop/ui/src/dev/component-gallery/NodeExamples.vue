<script setup lang="ts">
import {ref} from 'vue'
import ModalContent from '@/components/patterns/ModalContent.vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
const props=defineProps<{long?:boolean}>()
const modal=ref(false);const values=ref({name:'原输入内容'});const result=ref('')
</script>
<template>
 <UiCard id="modal-content"><UiCardHeader><h2>应用确认与输入内容</h2><p>与应用Modal共用内容视图和按钮；应用仍由AppModal唯一处理状态、焦点与解析，本页没有应用store或真实副作用。</p></UiCardHeader><UiButton @click="modal=true">打开输入确认样例</UiButton><p role="status">{{ result }}</p><UiDialog :open="modal" title="应用确认内容样例" @close="modal=false"><ModalContent title-id="gallery-modal-content-title" :modal="{title:long?'确认较长的中文与 English 操作说明及目标名称':'确认配置',body:'<p>内容样例；表单取消保留原调用方状态。</p>',fields:[{key:'name',label:'名称'}],confirmLabel:'确认',cancelLabel:'取消',primaryKind:'primary'}" :values="values" @field="(key,value)=>values={...values,[key]:value}" @confirm="result='确认样例：'+values.name;modal=false" @cancel="modal=false" /></UiDialog></UiCard>
</template>
