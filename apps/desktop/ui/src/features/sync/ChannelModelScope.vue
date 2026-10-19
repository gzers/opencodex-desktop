<script setup lang="ts">
import {computed,ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
const props=defineProps<{label:string;models:{id:string;label?:string}[];selected?:string[];disabled?:boolean}>()
const emit=defineEmits<{'update:selected':[string[]|undefined]}>()
const query=ref('');const rows=computed(()=>props.models.filter(m=>(m.id+' '+m.label).toLocaleLowerCase().includes(query.value.toLocaleLowerCase())))
const chosen=(id:string)=>props.selected===undefined||props.selected.includes(id)
function toggle(id:string,value:boolean){const selected=new Set(props.selected??props.models.map(m=>m.id));if(value)selected.add(id);else selected.delete(id);emit('update:selected',[...selected])}
</script>
<template>
 <details class="channel-model-scope"><summary>型号范围 · {{ selected===undefined?'全部 '+models.length+' 项':'所选 '+selected.length+' / '+models.length+' 项' }}</summary><p>渠道共用连接与认证单独处理；已有目标仅合并所选型号的明确规则和引用。</p>
 <label><input type="checkbox" :checked="selected===undefined" :disabled="disabled" @change="emit('update:selected',($event.target as HTMLInputElement).checked?undefined:models.map(m=>m.id))"/>包内全部型号</label>
 <input v-model="query" type="search" class="input" :aria-label="label+'型号范围搜索'" placeholder="搜索型号或原始ID" :disabled="disabled"/>
 <div class="model-scope-actions"><UiButton size="sm" variant="ghost" :disabled="disabled" @click="emit('update:selected',[...new Set([...(selected??models.map(m=>m.id)),...rows.map(m=>m.id)])])">选择当前结果</UiButton><UiButton size="sm" variant="ghost" :disabled="disabled" @click="emit('update:selected',(selected??models.map(m=>m.id)).filter(id=>!rows.some(m=>m.id===id)))">取消当前结果</UiButton></div>
 <div class="model-scope-list"><label v-for="model in rows" :key="model.id"><input type="checkbox" :checked="chosen(model.id)" :disabled="disabled" :aria-label="label+'范围内型号 '+model.id" @change="toggle(model.id,($event.target as HTMLInputElement).checked)"/><span>{{ model.label&&model.label!==model.id?model.label:model.id }}<code v-if="model.label&&model.label!==model.id">{{ model.id }}</code></span></label><p v-if="!rows.length">没有匹配的源型号</p></div>
 </details>
</template>
<style scoped>
.channel-model-scope{min-width:0}.channel-model-scope p{color:var(--muted);font-size:var(--text-label);overflow-wrap:anywhere}.channel-model-scope label{display:flex;gap:var(--space-2);align-items:flex-start;margin-block:var(--space-2)}.channel-model-scope>.input{width:100%;min-width:0}.model-scope-actions{display:flex;gap:var(--space-2);flex-wrap:wrap;margin-block:var(--space-2)}.model-scope-list{max-height:240px;overflow:auto}span,code{overflow-wrap:anywhere;min-width:0}code{display:block;font-family:var(--font-code);color:var(--muted)}
</style>
