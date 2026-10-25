<script setup lang="ts">
import {computed,ref} from 'vue'
import UiTreeTable from '@/components/ui/UiTreeTable.vue'
import type {TreeTableNode} from '@/components/ui/treeTable'
export interface ScopeObject {id:string;category:string;categoryLabel:string;label:string;detail?:string;mark?:string;disabled?:boolean}
const props=defineProps<{objects:ScopeObject[];selectedIds:string[];label:string;disabled?:boolean;query?:string}>()
const emit=defineEmits<{'update:selectedIds':[string[]]}>()
// 搜索词由所属页面持有（原型里搜索与范围预设同处一条工具栏），这里只消费。
const query=computed(()=>props.query??'')
const expanded=ref<string[]>([])
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
 <section class="scope-object-tree">
 <p class="scope-tree-summary" role="status">已选 {{ selectedIds.length }} 个对象<span v-if="hidden"> · 当前筛选外仍有 {{ hidden }} 个已选对象</span></p>
 <UiTreeTable :nodes="nodes" :expanded-ids="openIds" :initial-expanded-depth="0" :label="label" name-label="配置对象" :columns="[{key:'details',label:'范围与凭据'}]" @update:expanded-ids="expanded=$event" @select="choose"><template #name="{node}"><template v-if="object(node.id)"><span class="scope-object-name">{{ object(node.id)!.label }}</span><small v-if="object(node.id)!.detail" class="scope-object-detail">{{ object(node.id)!.detail }}</small><slot name="options" :object="object(node.id)!" /></template><span v-else>{{ node.label }}</span></template><template #cell-details="{node}"><span v-if="object(node.id)" class="scope-mark" :class="{secret:(object(node.id)!.mark??'').includes('含'),warn:object(node.id)!.mark==='暂不支持'}">{{ object(node.id)!.mark ?? '配置' }}</span><span v-else class="scope-group-count">{{ matching.filter(o=>'category:'+o.category===node.id).length }} 个对象</span></template></UiTreeTable>
 </section>
</template>
<style scoped>
.scope-object-tree{min-width:0}.scope-tree-toolbar{display:flex;align-items:center;gap:var(--space-3);flex-wrap:wrap}.scope-tree-toolbar>.input{flex:1;min-width:180px}.scope-tree-toolbar label{display:flex;gap:var(--space-2);align-items:center;white-space:nowrap}.scope-tree-summary,.scope-object-detail,.scope-group-count{font-size:var(--text-label);color:var(--muted);overflow-wrap:anywhere}.scope-object-tree :deep(details){margin-top:var(--space-2)}
/* 原型列宽：配置对象占主列，范围与凭据居右且不挤压；行内一律垂直居中。 */
.scope-object-tree :deep(.ui-tree-table table){table-layout:fixed}
.scope-object-tree :deep(.ui-tree-table th:first-child),.scope-object-tree :deep(.ui-tree-table td:first-child){width:auto}
.scope-object-tree :deep(.ui-tree-table th:last-child),.scope-object-tree :deep(.ui-tree-table td:last-child){width:120px;white-space:nowrap}
.scope-object-tree :deep(th),.scope-object-tree :deep(td){vertical-align:middle}
.scope-object-tree :deep(.tree-name),.scope-object-tree :deep(.tree-cell),.scope-object-tree :deep(.tree-cell-action){align-items:center}
.scope-object-name,.scope-object-detail{display:block}.scope-object-detail{line-height:1.5}
.scope-mark{display:inline-flex;align-items:center;border-radius:5px;padding:3px 7px;font-size:11px;line-height:1.5;background:var(--rail);color:var(--muted);white-space:nowrap}
.scope-mark.secret{color:var(--accent);background:var(--accent-soft)}.scope-mark.warn{color:var(--warning,#a96c19)}
</style>
