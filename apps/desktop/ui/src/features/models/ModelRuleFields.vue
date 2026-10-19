<script setup lang="ts">
import UiSelectField from '@/components/ui/UiSelectField.vue'
import type { Field, ModelRules, ModelProjection } from './api'
const props = defineProps<{ modelValue: ModelRules; disabled?: boolean; hideProtocol?: boolean; lockedFields?:string[];effective?:ModelProjection }>()
const emit = defineEmits<{ 'update:modelValue': [rules: ModelRules];'update:lockedFields':[fields:string[]] }>()
const lockNames=[['protocol','协议'],['contextWindow','上下文'],['inputModalities','模态'],['reasoningEfforts','可选档位'],['defaultEffort','默认档位'],['fast','Fast']] as const
function lock(field:string){const current=props.lockedFields??[];emit('update:lockedFields',current.includes(field)?current.filter(f=>f!==field):[...current,field])}
function update<K extends keyof ModelRules>(key: K, value: ModelRules[K]) { emit('update:modelValue', { ...props.modelValue, [key]: value }) }
function selected<T>(field: Field<T>): string { return field.mode === 'inherit' ? 'inherit' : String(field.value) }
function scalar(key: 'protocol' | 'defaultEffort', value: string) { update(key, value === 'inherit' ? { mode: 'inherit' } : { mode: 'custom', value }) }
const protocols = [{ value: 'inherit', label: '继承' }, { value: 'openai-responses', label: 'OpenAI Responses' }, { value: 'openai-chat', label: 'OpenAI Chat' }, { value: 'anthropic', label: 'Anthropic' }, { value: 'google', label: 'Google' }]
const fieldLabels=[['protocol','协议'],['contextWindow','上下文窗口'],['inputModalities','输入模态'],['reasoningEfforts','可选推理档位'],['defaultEffort','默认推理档位'],['fast','Fast支持']] as const
function effectiveText(field:Field<unknown>){if(field.mode==='inherit')return '未知（无声明或覆盖）';if(Array.isArray(field.value))return field.value.length?field.value.join('、'):'不显示档位';if(typeof field.value==='boolean')return field.value?'支持':'不支持';return String(field.value)}
const efforts = ['none', 'minimal', 'low', 'medium', 'high', 'xhigh', 'max', 'ultra']
function toggle(key: 'inputModalities' | 'reasoningEfforts', value: string) {
  const field = props.modelValue[key]
  const values = field.mode === 'custom' ? field.value : []
  update(key, { mode: 'custom', value: values.includes(value) ? values.filter(item => item !== value) : [...values, value] })
}
function checked(key: 'inputModalities' | 'reasoningEfforts', value: string) { const field = props.modelValue[key]; return field.mode === 'custom' && field.value.includes(value) }
</script>
<template>
  <fieldset class="model-rule-fields" :disabled="disabled">
    <label v-if="!hideProtocol">协议<UiSelectField :model-value="selected(modelValue.protocol)" :options="protocols" label="协议" :disabled="disabled" @update:model-value="scalar('protocol', $event)" /></label>
    <label>上下文窗口<input class="input" type="number" min="1" :value="modelValue.contextWindow.mode === 'custom' ? modelValue.contextWindow.value : ''" placeholder="继承" @input="update('contextWindow', ($event.target as HTMLInputElement).value === '' ? { mode: 'inherit' } : { mode: 'custom', value: Number(($event.target as HTMLInputElement).value) })" /></label>
    <div class="model-field-wide">
      <div class="model-field-title">输入模态<label><input type="checkbox" :checked="modelValue.inputModalities.mode === 'inherit'" @change="update('inputModalities', ($event.target as HTMLInputElement).checked ? { mode: 'inherit' } : { mode: 'custom', value: ['text'] })" />继承</label></div>
      <div v-if="modelValue.inputModalities.mode === 'custom'" class="model-inline"><label v-for="value in ['text', 'image', 'audio', 'video']" :key="value"><input type="checkbox" :checked="checked('inputModalities', value)" @change="toggle('inputModalities', value)" />{{ { text: '文本', image: '图像', audio: '音频', video: '视频' }[value] }}</label></div>
    </div>
    <div class="model-field-wide">
      <div class="model-field-title">可选推理档位<label><input type="checkbox" :checked="modelValue.reasoningEfforts.mode === 'inherit'" @change="update('reasoningEfforts', ($event.target as HTMLInputElement).checked ? { mode: 'inherit' } : { mode: 'custom', value: [] })" />继承</label></div>
      <div v-if="modelValue.reasoningEfforts.mode === 'custom'" class="model-inline"><label v-for="value in efforts" :key="value"><input type="checkbox" :checked="checked('reasoningEfforts', value)" @change="toggle('reasoningEfforts', value)" />{{ value }}</label></div>
    </div>
    <label>默认推理档位<UiSelectField :model-value="selected(modelValue.defaultEffort)" :options="[{ value: 'inherit', label: '继承' }, ...efforts.map(value => ({ value, label: value }))]" label="默认推理档位" :disabled="disabled" @update:model-value="scalar('defaultEffort', $event)" /></label>
    <label>Fast支持<UiSelectField :model-value="selected(modelValue.fast)" :options="[{ value: 'inherit', label: '继承' }, { value: 'true', label: '支持' }, { value: 'false', label: '不支持' }]" label="Fast支持" :disabled="disabled" @update:model-value="update('fast', $event === 'inherit' ? { mode: 'inherit' } : { mode: 'custom', value: $event === 'true' })" /></label>
    <div v-if="effective" class="model-field-wide model-table-scroll"><table class="model-table"><caption>当前草稿有效值与来源</caption><thead><tr><th>字段</th><th>本层设置</th><th>当前解析结果</th><th>来源</th></tr></thead><tbody><tr v-for="[key,name] in fieldLabels.filter(([key])=>!hideProtocol||key!=='protocol')" :key="key"><th>{{ name }}</th><td>{{ modelValue[key].mode==='inherit'?'继承':effectiveText(modelValue[key]) }}</td><td>{{ effectiveText(effective.rules[key]) }}</td><td>{{ effective.source[key]??'无可核验声明' }}</td></tr></tbody></table></div>
    <details v-if="lockedFields" class="model-field-wide"><summary>人工锁</summary><div class="model-inline"><label v-for="item in lockNames.filter(item=>!hideProtocol||item[0]!=='protocol')" :key="item[0]"><input type="checkbox" :checked="lockedFields.includes(item[0])" @change="lock(item[0])" />{{ item[1] }}</label></div></details>
  </fieldset>
</template>
