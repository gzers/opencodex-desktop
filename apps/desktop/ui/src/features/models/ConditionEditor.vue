<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from './ModelRuleFields.vue'
import {emptyRulesV2,type TemplateContentV2,type SelectorAtomV2,type NameLayoutV2,type MatchIssueV2,type PatternV2} from './api-v2'
const props=defineProps<{modelValue:TemplateContentV2;disabled?:boolean;issues?:readonly MatchIssueV2[]}>()
const emit=defineEmits<{'update:modelValue':[content:TemplateContentV2];preview:[]}>()
const atomKinds=[{value:'numeric_version',label:'数字版本'},{value:'literal_segment',label:'名称片段'},{value:'exact_id',label:'完整模型 ID'}]
const positions=[{value:'any',label:'不限位置'},{value:'before_version',label:'版本前'},{value:'after_version',label:'版本后'}]
const patternKinds=[{value:'exact',label:'精确 ID'},{value:'prefix',label:'字面前缀'},{value:'glob',label:'通配符'},{value:'ascii_numeric_prefix',label:'前缀后接 ASCII 数字'}]
const clone=<T>(value:T):T=>JSON.parse(JSON.stringify(value)) as T
function edit(change:(content:TemplateContentV2)=>void){if(props.disabled)return;const next=clone(props.modelValue);change(next);emit('update:modelValue',next)}
function text(event:Event){return (event.target as HTMLInputElement).value}
function layout(){edit(c=>c.nameLayouts.push({id:crypto.randomUUID(),literalPrefix:'',versionCapture:'complete_numeric_components',numericJoiners:['.','-'],tailCapture:'literal_remainder',trailingDatePolicy:'retain_valid_YYYYMMDD_after_hyphen',ambiguousNumericTail:'no_equivalence'}))}
function changeLayout(index:number,key:keyof Pick<NameLayoutV2,'literalPrefix'>,value:string){edit(c=>{c.nameLayouts[index]![key]=value})}
function addCondition(){edit(c=>c.conditionOverrides.push({id:crypto.randomUUID(),name:'',selector:{all:[]},rules:emptyRulesV2(),lockedFields:[],sourceExactId:'',scope:'known_identity_only'}))}
function addAtom(index:number){edit(c=>c.conditionOverrides[index]!.selector.all.push({kind:'literal_segment',value:'',position:'any',boundary:'hyphen_or_string_edge'}))}
function atomType(index:number,at:number,kind:string){edit(c=>{c.conditionOverrides[index]!.selector.all[at]=kind==='numeric_version'?{kind,components:[]}:kind==='exact_id'?{kind,value:''}:{kind:'literal_segment',value:'',position:'any',boundary:'hyphen_or_string_edge'}})}
function atomValue(index:number,at:number,value:string){edit(c=>{const atom=c.conditionOverrides[index]!.selector.all[at]!;if(atom.kind==='numeric_version')atom.components=value.split(/[.-]/);else atom.value=value})}
function setLayout(index:number,id:string){edit(c=>{const selector=c.conditionOverrides[index]!.selector;if(id){selector.layoutId=id;selector.literalTail??=''}else{delete selector.layoutId;delete selector.literalTail}})}
function removeLayout(id:string){edit(c=>{c.nameLayouts=c.nameLayouts.filter(l=>l.id!==id)})}
function atomText(atom:SelectorAtomV2){return atom.kind==='numeric_version'?atom.components.join('.'):atom.value}
function issuesFor(id:string){return props.issues?.filter(issue=>issue.conditionIds.includes(id))??[]}
</script>
<template>
  <div class="condition-editor" aria-label="条件覆盖编辑器">
    <fieldset :disabled="disabled">
      <legend>族匹配范围</legend>
      <p class="model-muted">排除优先于包含；完整 ID 保持原样。协议是匹配后的配置输出，不参与族识别。</p>
      <label>渠道范围<UiSelectField :model-value="modelValue.matching.channelMode" label="模版渠道范围" :disabled="disabled" :options="[{value:'all',label:'全部渠道'},{value:'whitelist',label:'仅所选渠道 UUID'},{value:'blacklist',label:'排除所选渠道 UUID'}]" @update:model-value="edit(c=>c.matching.channelMode=$event as 'all'|'whitelist'|'blacklist')" /></label>
      <label v-if="modelValue.matching.channelMode!=='all'">渠道 UUID（每行一个）<textarea class="input" aria-label="模版范围渠道UUID" :value="modelValue.matching.channels.join('\n')" @input="edit(c=>c.matching.channels=text($event).split(/\n/).map(v=>v.trim()).filter(Boolean))" /></label>
      <section v-for="kind in (['include','exclude'] as const)" :key="kind">
        <div class="model-field-title"><h3>{{kind==='include'?'包含规则':'排除规则'}}</h3><UiButton size="sm" :disabled="disabled" @click="edit(c=>c.matching[kind].push({kind:'prefix',value:''}))">添加{{kind==='include'?'包含':'排除'}}</UiButton></div>
        <div v-for="(pattern,index) in modelValue.matching[kind]" :key="index" class="condition-row">
          <UiSelectField :model-value="pattern.kind" :options="patternKinds" :label="kind+'匹配方式'+(index+1)" :disabled="disabled" @update:model-value="edit(c=>c.matching[kind][index]!.kind=$event as PatternV2['kind'])" />
          <input class="input" :aria-label="kind+'匹配文本'+(index+1)" :value="pattern.value" @input="edit(c=>c.matching[kind][index]!.value=text($event))" />
          <UiButton size="sm" :disabled="disabled" @click="edit(c=>c.matching[kind].splice(index,1))">移除规则</UiButton>
        </div>
      </section>
      <div class="model-columns"><label>优先级<input class="input" type="number" :value="modelValue.matching.priority" @input="edit(c=>c.matching.priority=Number(text($event)))" /></label><label><input type="checkbox" :checked="modelValue.matching.caseSensitive" @change="edit(c=>c.matching.caseSensitive=($event.target as HTMLInputElement).checked)" />区分大小写</label></div>
      <label><input type="checkbox" :checked="modelValue.matching.rejectRepeatedLiteralPrefix" @change="edit(c=>c.matching.rejectRepeatedLiteralPrefix=($event.target as HTMLInputElement).checked)" />拒绝重复的已声明族前缀</label>
    </fieldset>
    <section>
      <div class="model-field-title"><h2>名称结构</h2><UiButton size="sm" :disabled="disabled" @click="layout">添加名称结构</UiButton></div>
      <p class="model-muted">只在声明的数字槽位内将点与连字符视为等价。保留完整段数、前导零、日期、构建号和尾部规格。</p>
      <p v-if="!modelValue.nameLayouts.length" class="model-muted">尚无数字版本结构。完整 ID 条件可独立使用。</p>
      <article v-for="(item,index) in modelValue.nameLayouts" :key="item.id" class="condition-block">
        <label>字面前缀<input class="input" :aria-label="'结构前缀'+(index+1)" :value="item.literalPrefix" :disabled="disabled" placeholder="例如 gpt- 或 claude-opus-" @input="changeLayout(index,'literalPrefix',text($event))" /></label>
        <p class="model-muted">{{item.literalPrefix||'〈前缀〉'}}〈完整数字版本〉〈精确剩余文本〉 · . / -</p>
        <UiButton size="sm" :disabled="disabled" @click="removeLayout(item.id)">移除此结构</UiButton>
      </article>
    </section>
    <section>
      <div class="model-field-title"><h2>条件覆盖</h2><UiButton size="sm" :disabled="disabled" @click="addCondition">添加覆盖项</UiButton></div>
      <p class="model-muted">同一覆盖项内全部条件必须满足（AND）。未知型号只继承通用值；多个覆盖同时命中时需要解决冲突。</p>
      <p v-if="!modelValue.conditionOverrides.length" class="model-muted">暂无覆盖项，使用族通用规则。</p>
      <article v-for="(condition,index) in modelValue.conditionOverrides" :key="condition.id" class="condition-block" :aria-label="'覆盖项'+(index+1)">
        <div class="model-columns">
          <label>覆盖项名称<input class="input" :aria-label="'覆盖项名称'+(index+1)" :value="condition.name" :disabled="disabled" @input="edit(c=>c.conditionOverrides[index]!.name=text($event))" /></label>
          <label>已知来源精确 ID<input class="input" :aria-label="'来源精确ID'+(index+1)" :value="condition.sourceExactId" :disabled="disabled" @input="edit(c=>c.conditionOverrides[index]!.sourceExactId=text($event))" /></label>
          <label>数字版本结构<UiSelectField :model-value="condition.selector.layoutId??''" :options="[{value:'',label:'无数字结构'},...modelValue.nameLayouts.map(l=>({value:l.id,label:l.literalPrefix+'〈数字版本〉'}))]" :label="'数字版本结构'+(index+1)" :disabled="disabled" @update:model-value="setLayout(index,$event)" /></label>
          <label>精确剩余文本 / 规格<input class="input" :aria-label="'精确剩余文本'+(index+1)" :value="condition.selector.literalTail??''" :disabled="disabled||!condition.selector.layoutId" placeholder="例如 -sol；空表示精确空尾部" @input="edit(c=>c.conditionOverrides[index]!.selector.literalTail=text($event))" /></label>
        </div>
        <h3>以下条件全部满足（AND）</h3>
        <div v-for="(atom,at) in condition.selector.all" :key="at" class="condition-row">
          <UiSelectField :model-value="atom.kind" :options="atomKinds" :label="'覆盖'+(index+1)+'条件类型'+(at+1)" :disabled="disabled" @update:model-value="atomType(index,at,$event)" />
          <input class="input" :aria-label="'覆盖'+(index+1)+'条件值'+(at+1)" :value="atomText(atom)" :disabled="disabled" :placeholder="atom.kind==='numeric_version'?'完整版本，如 6.1':'完整片段或精确 ID'" @input="atomValue(index,at,text($event))" />
          <UiSelectField v-if="atom.kind==='literal_segment'" :model-value="atom.position" :options="positions.map(p=>({...p,disabled:p.value!=='any'&&!condition.selector.layoutId}))" :label="'覆盖'+(index+1)+'名称位置'+(at+1)" :disabled="disabled" @update:model-value="edit(c=>(c.conditionOverrides[index]!.selector.all[at] as Extract<SelectorAtomV2,{kind:'literal_segment'}>).position=$event as 'any'|'before_version'|'after_version')" />
          <UiButton size="sm" :disabled="disabled" @click="edit(c=>c.conditionOverrides[index]!.selector.all.splice(at,1))">删除条件</UiButton>
        </div>
        <UiButton size="sm" :disabled="disabled||condition.selector.all.length>=32" @click="addAtom(index)">添加 AND 条件</UiButton>
        <p v-if="condition.selector.layoutId&&!modelValue.nameLayouts.some(l=>l.id===condition.selector.layoutId)" class="model-error" role="alert">所引用的名称结构不存在，请重新选择。当前条件没有被自动改写。</p>
        <p v-for="issue in issuesFor(condition.id)" :key="issue.code+issue.fieldPath" role="alert" class="model-error">{{issue.message}} · {{issue.code}} <span v-if="issue.fieldPath">{{issue.fieldPath}}</span></p>
        <ModelRuleFields :model-value="condition.rules" :locked-fields="condition.lockedFields" :disabled="disabled" @update:model-value="edit(c=>c.conditionOverrides[index]!.rules=$event)" @update:locked-fields="edit(c=>c.conditionOverrides[index]!.lockedFields=$event)" />
        <UiButton size="sm" variant="danger" :disabled="disabled" @click="edit(c=>c.conditionOverrides.splice(index,1))">删除此覆盖项</UiButton>
      </article>
    </section>
    <UiButton :disabled="disabled" @click="emit('preview')">预览当前条件</UiButton>
  </div>
</template>
<style scoped>
.condition-editor{display:grid;gap:var(--space-5);min-width:0}.condition-editor fieldset{min-width:0;border:1px solid var(--border);border-radius:var(--radius-sm);padding:var(--space-3)}.condition-editor label{display:grid;gap:var(--space-2);min-width:0}.condition-editor input{min-width:0;width:100%;box-sizing:border-box}.condition-editor input[type=checkbox]{width:auto}.condition-row{display:flex;flex-wrap:wrap;gap:var(--space-2);align-items:center;margin-block:var(--space-3)}.condition-row>.input{flex:1;min-width:140px}.condition-row>.select{flex:1;min-width:130px}.condition-block{display:grid;gap:var(--space-3);padding-block:var(--space-4);border-bottom:1px solid var(--border);min-width:0}.condition-editor h2,.condition-editor h3{font-size:var(--text-body);margin:0}.condition-editor p{overflow-wrap:anywhere}
</style>
