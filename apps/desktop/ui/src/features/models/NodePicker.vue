<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import type {NodeMeasurement,ServiceNode} from './api'
import {nodeMeasurementError} from './useNodeMeasurements'
const props=withDefaults(defineProps<{nodes:ServiceNode[];activeId:string;disabled?:boolean;readonly?:boolean;measuring?:boolean;measurementDisabled?:boolean;pending?:string[];results?:Record<string,NodeMeasurement>;error?:string;stopped?:boolean}>(),{pending:()=>[],results:()=>({})})
const emit=defineEmits<{'update:nodes':[ServiceNode[]];'update:activeId':[string];add:[];measure:[string[]];stop:[]}>()
function update(id:string,key:'name'|'url',event:Event){if(props.disabled||props.readonly)return;emit('update:nodes',props.nodes.map(node=>node.id===id?{...node,[key]:(event.target as HTMLInputElement).value}:node))}
function remove(id:string){if(props.disabled||props.readonly||props.nodes.length<=1||id===props.activeId)return;emit('update:nodes',props.nodes.filter(n=>n.id!==id))}
function choose(id:string){if(!props.disabled&&id!==props.activeId)emit('update:activeId',id)}
</script>
<template>
 <section class="node-picker" aria-label="服务节点">
  <header><h2>服务节点</h2><div class="node-actions"><UiButton size="sm" :disabled="disabled||measuring||measurementDisabled||!nodes.length" @click="emit('measure',nodes.map(n=>n.id))">检测全部节点</UiButton><UiButton v-if="measuring" size="sm" @click="emit('stop')">取消检测</UiButton><UiButton v-if="!readonly" size="sm" :disabled="disabled" @click="emit('add')">新增节点</UiButton></div></header>
  <p class="node-muted">检测模型清单请求的耗时；当前节点由你手动选择。</p><p v-if="error" class="node-error" role="alert">{{ error }}</p><p v-if="stopped" role="status" class="node-muted">检测已取消，已完成结果保留。</p><p v-if="!nodes.length" class="node-muted">暂无服务节点</p>
  <article v-for="(node,index) in nodes" :key="node.id" class="node-row"><div class="node-fields"><label>名称<input class="input" :value="node.name" :aria-label="'节点'+(index+1)+'名称'" :disabled="disabled||readonly" @input="update(node.id,'name',$event)" /></label><label>服务基地址<input class="input" type="url" :value="node.url" :aria-label="'节点'+(index+1)+'服务基地址'" :disabled="disabled||readonly" placeholder="https://api.example.com/v1" @input="update(node.id,'url',$event)" /></label></div>
   <p v-if="results[node.id]" :role="results[node.id]!.error?'status':undefined" class="node-result" :class="{'node-error':results[node.id]!.error}">{{ results[node.id]!.error?nodeMeasurementError(results[node.id]!.error!):results[node.id]!.elapsedMs+' ms · '+results[node.id]!.modelCount+' 个模型' }}</p>
   <div class="node-actions"><UiButton size="sm" :loading="pending.includes(node.id)" :disabled="disabled||measuring||measurementDisabled" :aria-label="'检测节点'+(index+1)" @click="emit('measure',[node.id])">检测节点</UiButton><UiButton size="sm" :disabled="disabled||activeId===node.id" :aria-label="'设节点'+(index+1)+'为当前'" @click="choose(node.id)">{{ activeId===node.id?'当前节点':'设为当前' }}</UiButton><UiButton v-if="!readonly" size="sm" variant="danger" :disabled="disabled||nodes.length===1||activeId===node.id" :aria-label="'删除节点'+(index+1)" @click="remove(node.id)">删除</UiButton></div>
  </article>
 </section>
</template>
<style scoped>
.node-picker{min-width:0;container-type:inline-size}.node-picker header{display:flex;align-items:center;justify-content:space-between;gap:var(--space-3);flex-wrap:wrap;margin-block:var(--space-3)}h2{margin:0;font-size:var(--text-subtitle)}.node-actions{display:flex;gap:var(--space-2);align-items:center;flex-wrap:wrap}.node-row{border:1px solid var(--border);border-radius:var(--radius);padding:var(--space-4);margin-top:var(--space-4)}.node-fields{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:var(--space-4);margin-bottom:var(--space-3)}label{display:grid;gap:var(--space-2);min-width:0}.input{width:100%;min-width:0}.node-muted,.node-result{color:var(--muted);font-size:var(--text-label);overflow-wrap:anywhere}.node-error{color:var(--danger)}@container(max-width:600px){.node-fields{grid-template-columns:minmax(0,1fr)}}
</style>
