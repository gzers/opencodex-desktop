<script setup lang="ts">
// 开发态组件预览：直接引用生产公共组件，展示变体、状态、亮暗主题与长文本。
// 不触发真实 IPC；数据是本地固定样例，不代表生产默认值。
import { computed,ref } from 'vue'
import {componentRegistry} from '@/components/registry'
import FoundationExamples from './FoundationExamples.vue'
import ModelPicker,{type ModelChoice} from '@/features/models/ModelPicker.vue'
import {emptyRules,type ModelProjection} from '@/features/models/api'
import UiButton from '@/components/ui/UiButton.vue'
import UiSwitch from '@/components/ui/UiSwitch.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import SettingRow from '@/components/patterns/SettingRow.vue'
import Stack from '@/components/layout/Stack.vue'
import Inline from '@/components/layout/Inline.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
import UiDiscreteSlider from '@/components/ui/UiDiscreteSlider.vue'
const switchExample=ref(false)
const pickerOpen=ref(false);const pickerMultiple=ref(false);const pickerState=ref('normal');const galleryChoice=ref<ModelChoice[]>([])
const pickerStates=[{value:'normal',label:'正常完整目录'},{value:'empty',label:'空目录'},{value:'loading',label:'读取中'},{value:'error',label:'读取失败'},{value:'missing',label:'已选引用缺失'}]
const pickerChannels=[{id:'native',label:'官方 / 原生'},{id:'primary',label:'自建渠道'},{id:'long',label:'较长名称 English 与中文渠道'}]
const pickerRows=computed<ModelProjection[]>(()=>pickerState.value==='empty'||pickerState.value==='error'?[]:Array.from({length:18},(_,i)=>({channelId:i===0?'native':i%2?'primary':'long',providerId:i===0?'openai':'provider-'+(i%2),modelId:i===0?'gpt-native':('model/'+i),label:i===0?'原生模型':i===3?'较长的中文与 English 型号展示名称':'型号 '+i,enabled:i!==4,selected:i<3,discovered:true,templateRef:null,rules:{...emptyRules(),...(i%3===0?{contextWindow:{mode:'custom' as const,value:128000},reasoningEfforts:{mode:'custom' as const,value:['none','low','high']},defaultEffort:{mode:'custom' as const,value:'high'},fast:{mode:'custom' as const,value:true}}:{})},source:{},...(i===0?{nativeModelId:'gpt-native'}:{}),...(i===4?{unavailableReason:'调用方明确标为不可用'}:{}),aliases:i===3?['short-alias']:[]})))
function showPicker(multiple:boolean){pickerMultiple.value=multiple;galleryChoice.value=pickerState.value==='missing'?[{channelId:'removed',modelId:'missing'}]:[];pickerOpen.value=true}
const effortExample=ref<string|undefined>()

const theme = ref<'light' | 'dark'>('light')
const effects = ref<'high' | 'mid' | 'low'>('high')
const selectedExample = ref('one')
const dialogOpen = ref(false)
const loadingExample=ref(false)
const longPath = ref('/Users/example/Library/Application Support/OpenCodex/data-root/logs/2026-09-27-application.log')

function setTheme(next: 'light' | 'dark') {
  theme.value = next
  document.documentElement.dataset.theme = next
}

// 档位矩阵：直接写 `data-effects` 以预览高/中/低三档；生产由 `app/appearance/effects.ts` 唯一写入。
function setEffects(next: 'high' | 'mid' | 'low') {
  effects.value = next
  document.documentElement.setAttribute('data-effects', next)
}
</script>

<template>
  <main class="gallery">
    <header class="gallery-head">
      <h1>公共组件预览</h1>
      <Stack>
        <Inline>
          <span class="gallery-label">主题</span>
          <UiButton :variant="theme === 'light' ? 'primary' : 'default'" @click="setTheme('light')">亮色</UiButton>
          <UiButton :variant="theme === 'dark' ? 'primary' : 'default'" @click="setTheme('dark')">暗色</UiButton>
        </Inline>
        <Inline>
          <span class="gallery-label">档位</span>
          <UiButton :variant="effects === 'high' ? 'primary' : 'default'" @click="setEffects('high')">高</UiButton>
          <UiButton :variant="effects === 'mid' ? 'primary' : 'default'" @click="setEffects('mid')">中</UiButton>
          <UiButton :variant="effects === 'low' ? 'primary' : 'default'" @click="setEffects('low')">低</UiButton>
        </Inline>
      </Stack>
    </header>

    <div class="gallery-layout"><nav class="gallery-nav" aria-label="组件与规格目录"><strong>组件目录</strong><a v-for="item in componentRegistry" :key="item.id" :href="'#'+(item.gallery??'entry-'+item.id)">{{ item.name }}</a></nav>
    <Stack>
      <FoundationExamples />
      <UiCard id="fields">
        <UiCardHeader><h2>选择字段与弹窗</h2><p>真实选择字段复用浮层、键盘和边缘定位；弹窗沿同一材质及焦点约束。</p></UiCardHeader>
        <Inline><UiSelectField v-model="selectedExample" label="标准选择字段" :options="[{ value: 'one', label: '默认选项' }, { value: 'long', label: '较长的中文与 English 模型版本说明' }]" /><UiSelectField v-model="selectedExample" size="sm" label="小号选择字段" :options="[{ value: 'one', label: '小号' }, { value: 'long', label: '长值' }]" /><UiButton @click="dialogOpen = true">打开弹窗</UiButton></Inline>
        <UiDialog :open="dialogOpen" title="组件弹窗" @close="dialogOpen = false"><p>确认、关闭、Escape及焦点返回共用该控件。</p><template #actions><UiButton variant="primary" @click="dialogOpen = false">确定</UiButton></template></UiDialog>
        <UiDiscreteSlider v-model="effortExample" :values="['none','low','high','max']" default-value="high" label="离散推理档位" />
        <UiDiscreteSlider :values="[]" label="未知档位" disabled />
      </UiCard>
      <UiCard id="tabs">
        <UiCardHeader><h2>主Tab固定容器</h2><p>到滚动顶部固定，换行后自动更新下层标题偏移。</p></UiCardHeader>
        <StickyTabs><div role="tablist" class="diag-tabs"><button class="active" role="tab" aria-selected="true">当前分区</button><button role="tab" aria-selected="false">长中文 English 分区名称</button></div></StickyTabs>
      </UiCard>
      <UiCard id="buttons">
        <UiCardHeader>
          <h2>按钮 UiButton</h2>
          <p>变体 API：variant / size / density / surface；loading 不改变宽度；禁用与焦点规则一致。</p>
        </UiCardHeader>
        <Inline>
          <UiButton>默认</UiButton>
          <UiButton variant="primary">主操作</UiButton>
          <UiButton variant="ghost">幽灵</UiButton>
          <UiButton variant="danger">危险</UiButton>
          <UiButton size="sm">小号</UiButton>
          <UiButton size="lg">大号</UiButton>
          <UiButton density="compact" variant="ghost">紧凑</UiButton>
          <UiButton surface="plain" variant="ghost">无表面</UiButton>
          <UiButton disabled>禁用</UiButton>
          <UiButton :loading="true">进行中…</UiButton>
          <UiButton :loading="loadingExample" @click="loadingExample=true">保存这份示例</UiButton><UiButton variant="ghost" @click="loadingExample=false">重置进行状态</UiButton>
        </Inline>
      </UiCard>

      <UiCard id="cards">
        <UiCardHeader>
          <h2>卡片 UiCard / UiCardHeader</h2>
          <p>只承载表面配方；标题与操作由插槽给出。</p>
          <template #actions>
            <div class="toolbar"><UiButton class="ghost">操作</UiButton></div>
          </template>
        </UiCardHeader>
        <p>正文直接作为卡片内容流动，不在卡片的每个内容块上重复内边距。</p>
      </UiCard>

      <UiCard id="materials" surface="overlay">
        <UiCardHeader>
          <h2>表面 variant</h2>
          <p>panel（默认）/ plain / overlay 与 density=compact；低档下 overlay 退实底。</p>
        </UiCardHeader>
        <Stack gap="var(--space-2)">
          <UiCard density="compact"><p>紧凑卡片密度</p></UiCard>
          <UiCard surface="plain"><p>无表面卡片（仅语义容器）</p></UiCard>
        </Stack>
      </UiCard>

      <UiCard id="settings">
        <UiCardHeader>
          <h2>设置行 SettingRow</h2>
          <p>左列标题/说明，右列控件；行内说明由默认插槽追加。</p>
        </UiCardHeader>
        <div class="setting-list">
          <SettingRow title="示例开关" description="说明与控件在同一行；窄容器不截断多行说明。">
            <template #actions>
              <div class="controls"><UiSwitch v-model="switchExample" label="示例开关" /></div>
            </template>
          </SettingRow>
          <SettingRow title="只读结论" description="无法兑现为行为时只陈述事实，不提供假开关。">
            <template #actions>
              <div class="controls"><span class="setting-readonly">官方 CLI 未就绪</span></div>
            </template>
          </SettingRow>
        </div>
      </UiCard>

      <UiCard id="layout">
        <UiCardHeader>
          <h2>布局 Stack / Inline</h2>
          <p>统一间距与换行；间距只由容器拥有，避免与子级 margin 叠加。</p>
        </UiCardHeader>
        <Stack gap="var(--space-2)">
          <UiButton>Stack 子项 1</UiButton>
          <UiButton>Stack 子项 2</UiButton>
        </Stack>
      </UiCard>

      <UiCard id="switches"><UiCardHeader><h2>开关 UiSwitch</h2><p>视觉轨道 38×22，点击高度28；受控值、可访问名称、禁用和忙碌状态由同一组件承载。</p></UiCardHeader><Inline><UiSwitch v-model="switchExample" label="可切换样例" /><UiSwitch :model-value="false" label="关闭样例" disabled /><UiSwitch :model-value="true" label="开启样例" disabled /><UiSwitch :model-value="true" label="保存中样例" busy /></Inline></UiCard>
      <UiCard>
        <UiCardHeader><h2>长文本与溢出</h2><p>长路径应可读、可复制，不撑破容器。</p></UiCardHeader>
        <p class="mono gallery-path" :title="longPath">{{ longPath }}</p>
      </UiCard>
      <UiCard id="model-picker"><UiCardHeader><h2>公共模型选择器</h2><p>真实组件的原生身份、渠道引用、未知能力、失效选择、明确推荐、读取失败与临时组合；下面的固定样例不连接服务。</p></UiCardHeader><Inline><UiSelectField v-model="pickerState" label="模型选择器样例状态" :options="pickerStates" /><UiButton @click="showPicker(false)">单选打样</UiButton><UiButton @click="showPicker(true)">多选与优先级打样</UiButton></Inline><p>返回：{{ galleryChoice.length?galleryChoice:'尚未确认' }}</p><ModelPicker :open="pickerOpen" :multiple="pickerMultiple" ordered reasoning :rows="pickerRows" :channels="pickerChannels" :current="galleryChoice" :recommendations="[{channelId:'primary',modelId:'model/3'},{nativeModelId:'gpt-native'}]" :loading="pickerState==='loading'" :error="pickerState==='error'?'固定读取失败样例，已选项保持':undefined" @close="pickerOpen=false" @confirm="galleryChoice=$event;pickerOpen=false" @retry="pickerState='normal'" /></UiCard>
      <UiCard id="registration"><UiCardHeader><h2>规格与采用登记</h2><p>43项稳定ID对应真实源码。已有配方、实际组件、平台实现和原型专用说明分别记录；完整尺寸/主题/画质/原生验证尚未全部完成。</p></UiCardHeader><div class="gallery-registry-scroll"><table><thead><tr><th>组件</th><th>正式来源</th><th>实际消费者</th><th>检查状态</th></tr></thead><tbody><tr v-for="item in componentRegistry" :id="'entry-'+item.id" :key="item.id"><th scope="row">{{ item.name }}<small>{{ item.id }}</small><small>{{ item.specification.document }}</small></th><td><code v-for="file in item.implementations" :key="file">{{ file }}</code><span v-if="item.reason">{{ item.reason }}</span></td><td><code v-for="file in item.consumers" :key="file">{{ file }}</code></td><td>完整矩阵待验证{{ item.gallery?' · 有组件样例':' · 样例待补/原生独立核验' }}</td></tr></tbody></table></div></UiCard>
    </Stack></div>
  </main>
</template>

<style scoped>
.gallery {
  width: 100%;
  margin: 0 auto;
  padding: 24px 22px;
}
.gallery-layout{display:grid;grid-template-columns:220px minmax(0,1fr);gap:24px}.gallery-nav{position:sticky;top:20px;align-self:start;max-height:calc(100vh - 40px);overflow:auto;display:grid;gap:8px;font-size:var(--text-label)}.gallery-nav a{padding:6px;border-radius:var(--radius-sm);text-decoration:none}.gallery-nav a:hover,.gallery-nav a:focus-visible{background:var(--hover)}.gallery-registry-scroll{overflow:auto}table{width:100%;border-collapse:collapse;font-size:var(--text-small)}th,td{text-align:left;vertical-align:top;border-bottom:1px solid var(--border);padding:12px;min-width:140px}code,small{display:block;overflow-wrap:anywhere}small{color:var(--muted)}[id]{scroll-margin-top:20px}@media(max-width:850px){.gallery-layout{grid-template-columns:1fr}.gallery-nav{position:static;max-height:180px;grid-template-columns:repeat(auto-fit,minmax(140px,1fr))}}
.gallery-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}
.gallery-head h1 {
  font-size: var(--text-display);
  margin: 0;
}
.gallery-path {
  word-break: break-all;
  color: var(--muted);
  font-size: var(--text-label);
}
.gallery-label {
  color: var(--muted);
  font-size: var(--text-label);
  align-self: center;
}
</style>
