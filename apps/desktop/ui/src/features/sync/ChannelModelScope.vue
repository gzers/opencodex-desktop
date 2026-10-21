<script setup lang="ts">
import {computed,ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import type {FamilyScope} from './modelTransfer'
const props=defineProps<{label:string;models:{id:string;label?:string}[];selected?:string[];disabled?:boolean;families?:{id:string;label:string}[];familyScope?:FamilyScope}>()
const emit=defineEmits<{'update:selected':[string[]|undefined];'update:familyScope':[FamilyScope|undefined]}>()
function toggleFamily(id:string,checked:boolean){const ids=new Set(props.familyScope?.templateIds??[]);if(checked)ids.add(id);else ids.delete(id);emit('update:familyScope',{templateIds:[...ids],includeChannelDefaults:props.familyScope?.includeChannelDefaults??false})}
const query=ref('');const rows=computed(()=>props.models.filter(m=>(m.id+' '+m.label).toLocaleLowerCase().includes(query.value.toLocaleLowerCase())))
const chosen=(id:string)=>props.selected===undefined||props.selected.includes(id)
function toggle(id:string,value:boolean){const selected=new Set(props.selected??props.models.map(m=>m.id));if(value)selected.add(id);else selected.delete(id);emit('update:selected',[...selected])}
</script>
<template>
 <div class="channel-model-scope">
 <UiSelectField v-if="families?.length" :model-value="familyScope?'families':'models'" :label="label+'策略范围'" :disabled="disabled" :options="[{value:'models',label:'整渠道 / 所选型号'},{value:'families',label:'所选模型族'}]" @update:model-value="emit('update:familyScope',$event==='families'?{templateIds:[],includeChannelDefaults:false}:undefined)" />
 <div v-if="familyScope">
 <p v-if="!familyScope.templateIds.length">请选择至少一个已采用的模型族。</p>
 <label v-for="family in families??[]" :key="family.id"><input type="checkbox" :aria-label="label+'范围内模型族 '+family.label" :checked="familyScope.templateIds.includes(family.id)" :disabled="disabled" @change="toggleFamily(family.id,($event.target as HTMLInputElement).checked)"/>{{ family.label }}</label>
 <p>只携带所选族的精确采用版本、族规则和所属型号例外；接收后保留其他族、连接、Key及已有选择和顺序。</p>
 <label><input type="checkbox" :aria-label="label+'包含渠道共用默认字段'" :checked="familyScope.includeChannelDefaults" :disabled="disabled" @change="emit('update:familyScope',{...familyScope,includeChannelDefaults:($event.target as HTMLInputElement).checked})"/>另含渠道共用默认字段（影响该渠道全部型号）</label>
 <p>未携带的共用值沿接收方配置解析，最终效果可能不同。</p>
 </div>
 <details v-else><summary>型号范围 · {{ selected===undefined?'全部 '+models.length+' 项':'所选 '+selected.length+' / '+models.length+' 项' }}</summary><p>渠道共用连接与认证单独处理；已有目标仅合并所选型号的明确规则和引用。</p>
 <label><input type="checkbox" :checked="selected===undefined" :disabled="disabled" @change="emit('update:selected',($event.target as HTMLInputElement).checked?undefined:models.map(m=>m.id))"/>包内全部型号</label>
 <input v-model="query" type="search" class="input" :aria-label="label+'型号范围搜索'" placeholder="搜索型号或原始ID" :disabled="disabled"/>
 <div class="model-scope-actions"><UiButton size="sm" variant="ghost" :disabled="disabled" @click="emit('update:selected',[...new Set([...(selected??models.map(m=>m.id)),...rows.map(m=>m.id)])])">选择当前结果</UiButton><UiButton size="sm" variant="ghost" :disabled="disabled" @click="emit('update:selected',(selected??models.map(m=>m.id)).filter(id=>!rows.some(m=>m.id===id)))">取消当前结果</UiButton></div>
 <div class="model-scope-list"><label v-for="model in rows" :key="model.id"><input type="checkbox" :checked="chosen(model.id)" :disabled="disabled" :aria-label="label+'范围内型号 '+model.id" @change="toggle(model.id,($event.target as HTMLInputElement).checked)"/><span>{{ model.label&&model.label!==model.id?model.label:model.id }}<code v-if="model.label&&model.label!==model.id">{{ model.id }}</code></span></label><p v-if="!rows.length">没有匹配的源型号</p></div>
 </details>
 </div>
</template>
<style scoped>
.channel-model-scope{min-width:0}.channel-model-scope p{color:var(--muted);font-size:var(--text-label);overflow-wrap:anywhere}.channel-model-scope label{display:flex;gap:var(--space-2);align-items:flex-start;margin-block:var(--space-2)}.channel-model-scope>.input{width:100%;min-width:0}.model-scope-actions{display:flex;gap:var(--space-2);flex-wrap:wrap;margin-block:var(--space-2)}.model-scope-list{max-height:240px;overflow:auto}span,code{overflow-wrap:anywhere;min-width:0}code{display:block;font-family:var(--font-code);color:var(--muted)}
</style>
