<script setup lang="ts">
import {computed,ref} from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import NewChannelDialog from './NewChannelDialog.vue'
import ChannelModelTable from './ChannelModelTable.vue'
import {useModelsV2Store} from './store-v2'
import {channelNameErrorV2} from './channelRename'
import type {ChannelV2,EffectiveModelV2,ChannelIntentV2,ModelEditV2,ModelSelectionV2} from './api-v2'
import type {ChannelPolicy,ModelPolicy,ModelProjection,TemplateRef} from './api'
import type {ModelRowsState} from './modelRows'
import type {StatusTone} from '@/lib/saveStates'
const props=defineProps<{channel:ChannelV2|ChannelPolicy;effective?:readonly EffectiveModelV2[];channels?:readonly (ChannelV2|ChannelPolicy)[];busy?:boolean;templates?:readonly unknown[];existing?:boolean;globalRefs?:TemplateRef[];projection?:ModelProjection[];projectionBusy?:boolean;projectionError?:string;policy?:ModelPolicy;rowState?:ModelRowsState;navigation?:{collapsed:boolean;activeId:string};saveDisabled?:boolean;dirty?:boolean;saveStatus?:string;saveTone?:StatusTone;activeCount?:number;viewState?:{search:string;selectedOnly:boolean;family:string}}>()
const emit=defineEmits<{close:[];save:[];switch:[id:string];intent:[value:ChannelIntentV2];edit:[value:ModelEditV2]}>()
const store=useModelsV2Store(),editConnection=ref(false),query=ref(''),selectedOnly=ref(false)
const channel=computed(()=>('models'in props.channel?props.channel:null) as ChannelV2|null)
const nameError=computed(()=>channelNameErrorV2(props.channel.label,props.channel.id,props.channels??store.channels))
const selectedCount=computed(()=>!channel.value?0:channel.value.selection.mode==='all'?channel.value.models.length:channel.value.selection.mode==='whitelist'?channel.value.selection.models.length:0)
function rename(event:Event){if(props.busy)return;emit('intent',{kind:'rename',channelId:props.channel.id,name:(event.target as HTMLInputElement).value})}
function selection(value:ModelSelectionV2){emit('intent',{kind:'set_selection',channelId:props.channel.id,selection:value})}
</script>
<template>
  <section class="channel-editor-v2">
    <div class="model-toolbar"><UiButton size="sm" variant="ghost" @click="emit('close')">返回渠道列表</UiButton><h2>{{channel?.label||'渠道'}}</h2><UiButton variant="primary" :disabled="busy||saveDisabled||!!nameError" @click="emit('save')">保存当前草稿</UiButton></div>
    <p v-if="!channel" role="alert" class="model-error">此渠道不是新版格式，请重新读取；旧原件没有被转换。</p>
    <template v-else>
      <p role="status">{{busy?'正在处理':dirty?'有未保存修改':'当前已保存状态'}} · 已选 {{selectedCount}} / {{channel.models.length}}</p>
      <section><h3>渠道与连接</h3><label>渠道名称<input class="input" aria-label="渠道名称" :value="channel.label" :disabled="busy" maxlength="80" @input="rename" /></label><p v-if="nameError" role="alert" class="model-error">{{nameError}}</p><p class="model-muted">保存由同一事务处理真实名称、运行键及引用；不会改写原始模型 ID 或凭据。</p><label class="model-inline"><input type="checkbox" aria-label="启用渠道" :checked="channel.connection.enabled" :disabled="busy" @change="emit('intent',{kind:'set_enabled',channelId:channel.id,enabled:($event.target as HTMLInputElement).checked})" />启用渠道</label>
        <dl><dt>认证方式</dt><dd>{{channel.connection.authMode==='none'?'无需认证':'API Key / 实际来源'}}</dd><dt>凭据</dt><dd>{{channel.connection.credentialRef?'已配置（不会在页面读取明文）':'未配置或由实际受控来源提供'}}</dd></dl>
        <UiButton :disabled="busy" @click="editConnection=true">编辑渠道连接</UiButton>
      </section>
      <section><h3>服务节点</h3><ul><li v-for="node in channel.connection.nodes" :key="node.id"><strong>{{node.name}}</strong><span v-if="node.id===channel.connection.activeNodeId"> · 当前节点</span><code>{{node.url}}</code></li></ul><p class="model-muted">编辑连接时完整保留节点集合。清单获取方式只在本次读取选择，不保存为请求协议。</p></section>
      <details><summary>非协议渠道默认与族设置</summary><p class="model-muted">现有明确的非协议设置保留；请求协议只来自实际模型来源、模版、条件及单模型人工值。</p><ModelRuleFields :model-value="channel.defaults" hide-protocol disabled /><section v-for="(rules,id) in channel.familyOverrides" :key="id"><h4>{{id}}</h4><ModelRuleFields :model-value="rules" hide-protocol disabled /></section></details>
      <section><h3>模型列表</h3><div class="model-inline"><input v-model="query" class="input" type="search" aria-label="渠道模型搜索" placeholder="搜索原始模型 ID / 名称" /><label><input v-model="selectedOnly" type="checkbox" />仅已选</label><UiSelectField :model-value="channel.selection.mode" :disabled="busy" label="模型公开范围" :options="[{value:'all',label:'全部模型（含以后发现）'},{value:'none',label:'全部隐藏'},{value:'whitelist',label:'明确白名单'}]" @update:model-value="selection($event==='whitelist'?{mode:'whitelist',models:channel.models.filter(m=>channel!.selection.mode==='all'||channel!.selection.mode==='whitelist'&&channel!.selection.models.includes(m.rawModelId)).map(m=>m.rawModelId)}:{mode:$event as 'all'|'none'})" /></div><ChannelModelTable :channel="channel" :effective="effective" :query="query" :selected-only="selectedOnly" :disabled="busy" @edit="emit('edit',$event)" @selection="selection" @order="emit('intent',{kind:'set_order',channelId:channel.id,rawModelIds:$event})" /></section>
      <NewChannelDialog v-if="editConnection" :original="channel" @close="editConnection=false" @added="editConnection=false" />
    </template>
  </section>
</template>
<style scoped>
.channel-editor-v2{display:grid;gap:var(--space-5);min-width:0}.channel-editor-v2 h2{font-size:var(--text-title)}.channel-editor-v2 h3{font-size:var(--text-body)}.channel-editor-v2 label{display:grid;gap:var(--space-2);min-width:0}.channel-editor-v2 input{min-width:0}.channel-editor-v2 .model-inline label{display:flex;align-items:center}.channel-editor-v2 code{display:block;overflow-wrap:anywhere}.channel-editor-v2 dl{display:grid;grid-template-columns:auto minmax(0,1fr);gap:var(--space-2)}.channel-editor-v2 dd{margin:0;overflow-wrap:anywhere}
</style>
