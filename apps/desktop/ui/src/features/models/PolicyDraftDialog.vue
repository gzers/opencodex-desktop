<script setup lang="ts">
import {computed} from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiButton from '@/components/ui/UiButton.vue'
import {policyChanges} from './policyChanges'
import type {ModelPolicy} from './api'
const props=defineProps<{saved:ModelPolicy;draft:ModelPolicy;pending?:number;canSave?:boolean;busy?:boolean}>(),emit=defineEmits<{close:[];manage:[string];save:[]}>()
const report=computed(()=>policyChanges(props.saved,props.draft))
const anyChanges=computed(()=>report.value.global.length>0||report.value.channels.length>0)
const text=(value:unknown)=>value===null?'未设置':JSON.stringify(value,null,2)
</script>
<template>
 <UiDialog :open="true" title="查看渠道草稿" wide @close="emit('close')">
  <p>比较最近成功保存与当前已接受草稿，包含搜索和勾选范围外的修改；查看不会保存或改变客户端状态。</p>
  <p v-if="pending" role="status">另有 {{ pending }} 项型号输入未确认，尚未加入以下草稿差异；请返回对应行处理。</p>
  <p v-if="!anyChanges">暂无未保存改动。</p>
  <section v-if="report.global.length"><h3>全局与顺序</h3><details v-for="change in report.global" :key="change.field"><summary>{{ change.field }}</summary><div class="policy-change-values"><div><strong>已保存</strong><pre>{{ text(change.before) }}</pre></div><div><strong>当前草稿</strong><pre>{{ text(change.after) }}</pre></div></div></details></section>
  <section v-for="channel in report.channels" :key="channel.id"><div class="model-inline"><h3>{{ channel.label }} · {{ {added:'新增',removed:'删除',changed:'修改'}[channel.kind] }}</h3><UiButton v-if="channel.kind!=='removed'" size="sm" @click="emit('manage',channel.id)">管理此渠道</UiButton></div><details v-for="change in channel.changes" :key="change.field"><summary>{{ change.field }}</summary><div class="policy-change-values"><div><strong>已保存</strong><pre>{{ text(change.before) }}</pre></div><div><strong>当前草稿</strong><pre>{{ text(change.after) }}</pre></div></div></details></section>
  <template #actions><UiButton @click="emit('close')">继续编辑</UiButton><UiButton v-if="anyChanges" variant="primary" :disabled="busy||!canSave||!!pending" @click="emit('save')">去保存</UiButton></template>
 </UiDialog>
</template>
<style scoped>
section{border-top:1px solid var(--border);margin-top:var(--space-4);padding-top:var(--space-3)}h3{overflow-wrap:anywhere}details{padding-block:var(--space-2)}.policy-change-values{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:var(--space-4)}.policy-change-values>div{min-width:0}pre{white-space:pre-wrap;overflow-wrap:anywhere;font-family:var(--font-code);font-size:var(--text-caption)}@media(max-width:700px){.policy-change-values{grid-template-columns:minmax(0,1fr)}}
</style>
