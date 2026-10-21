<script setup lang="ts">
import {computed,ref} from 'vue'
import UiTreeTable from '@/components/ui/UiTreeTable.vue'
import type {TreeTableNode} from '@/components/ui/treeTable'
export interface ScopeObject {id:string;category:string;categoryLabel:string;label:string;detail?:string;disabled?:boolean}
const props=defineProps<{objects:ScopeObject[];selectedIds:string[];label:string;disabled?:boolean}>()
const emit=defineEmits<{'update:selectedIds':[string[]]}>()
const query=ref('');const expanded=ref<string[]>([])
const matching=computed(()=>props.objects.filter(o=>(o.label+' '+o.detail+' '+o.categoryLabel).toLocaleLowerCase().includes(query.value.toLocaleLowerCase())))
const groups=computed(()=>[...new Set(matching.value.map(o=>o.category))])
const hidden=computed(()=>props.selectedIds.filter(id=>!matching.value.some(o=>o.id===id)).length)
function state(ids:string[]):boolean|'mixed'{if(!ids.length)return false;const count=ids.filter(id=>props.selectedIds.includes(id)).length;return count===0?false:count===ids.length?true:'mixed'}
const selectable=computed(()=>matching.value.filter(o=>!o.disabled).map(o=>o.id))
const nodes=computed<TreeTableNode[]>(()=>groups.value.map(category=>{const items=matching.value.filter(o=>o.category===category);return {id:'category:'+category,label:items[0]!.categoryLabel,selectable:true,selected:state(items.filter(o=>!o.disabled).map(o=>o.id)),disabled:props.disabled||!items.some(o=>!o.disabled),children:items.map(o=>({...o,selectable:true,selected:props.selectedIds.includes(o.id),disabled:props.disabled||o.disabled}))}}))
const openIds=computed(()=>query.value?groups.value.map(g=>'category:'+g):expanded.value)
function choose(id:string,selected:boolean){if(props.disabled)return;const ids=id==='all'?selectable.value:id.startsWith('category:')?matching.value.filter(o=>o.category===id.slice(9)&&!o.disabled).map(o=>o.id):[id];const next=new Set(props.selectedIds);for(const value of ids){if(selected)next.add(value);else next.delete(value)}emit('update:selectedIds',[...next])}
function object(id:string){return props.objects.find(o=>o.id===id)}
</script>
<template>
 <section class="scope-object-tree"><div class="scope-tree-toolbar"><input v-model="query" type="search" class="input" :aria-label="label+'搜索'" placeholder="搜索类别或配置对象" :disabled="disabled" /><label><input type="checkbox" :checked="state(selectable)===true" :indeterminate="state(selectable)==='mixed'" :aria-checked="state(selectable)==='mixed'?'mixed':state(selectable)===true" :disabled="disabled||!selectable.length" @change="choose('all',($event.target as HTMLInputElement).checked)" />全选当前结果</label><slot name="actions" /></div>
 <p class="scope-tree-summary" role="status">已选 {{ selectedIds.length }} 个对象<span v-if="hidden"> · 当前筛选外仍有 {{ hidden }} 个已选对象</span></p>
 <UiTreeTable :nodes="nodes" :expanded-ids="openIds" :initial-expanded-depth="0" :label="label" name-label="类别 / 配置对象" :columns="[{key:'details',label:'内容与同步选项'}]" @update:expanded-ids="expanded=$event" @select="choose"><template #cell-details="{node}"><template v-if="object(node.id)"><small>{{ object(node.id)!.detail }}</small><slot name="options" :object="object(node.id)!" /></template><span v-else>{{ matching.filter(o=>'category:'+o.category===node.id).length }} 个对象</span></template></UiTreeTable>
 </section>
</template>
<style scoped>
.scope-object-tree{min-width:0}.scope-tree-toolbar{display:flex;align-items:center;gap:var(--space-3);flex-wrap:wrap}.scope-tree-toolbar>.input{flex:1;min-width:180px}.scope-tree-toolbar label{display:flex;gap:var(--space-2);align-items:center;white-space:nowrap}.scope-tree-summary,small{font-size:var(--text-label);color:var(--muted);overflow-wrap:anywhere}.scope-object-tree :deep(td){min-width:200px}.scope-object-tree :deep(details){margin-top:var(--space-2)}
</style>
