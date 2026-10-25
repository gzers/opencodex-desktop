<script setup lang="ts">
/**
 * 状态标签（公共组件）。
 *
 * 与原型 `.state-pill` + `.state-*` 同形：胶囊、8px 圆点、五类语义色。
 * 入参只接**语义 tone**（或直接给 SaveState），不接中文 label —— 文案由
 * `lib/saveStates.ts` / `lib/labels.ts` 单一提供，避免同一状态两处翻译。
 *
 * 依据：2026-10-25 裁决 #2（保留 state-* 命名）、#1（阶段名不常驻，走 title）。
 */
import { computed } from 'vue'
import { TONE_CLASS, type SaveState, type StatusTone } from '@/lib/saveStates'

const props = withDefaults(
  defineProps<{
    /** 语义 tone；给了 state 时可省略。 */
    tone?: StatusTone
    /** 直接传结论级状态（含 label / tone）。 */
    state?: Pick<SaveState, 'label' | 'tone'> | null
    /** 覆盖文案（同一 tone 的不同说法，例如「已写入 · 用户已确认」）。 */
    label?: string
    /** hover / focus 显示的补充说明；不占常驻宽度（裁决 #1）。 */
    hint?: string
    /** 子标签：同一状态区表达「上次结果」（原型 .mp-policy-pending）。 */
    sub?: string | null
    /** 计数等附加信息，弱化显示。 */
    count?: string | number | null
  }>(),
  { tone: undefined, state: null, label: undefined, hint: undefined, sub: null, count: null },
)

const resolvedTone = computed<StatusTone>(() => props.tone ?? props.state?.tone ?? 'muted')
const text = computed(() => props.label ?? props.state?.label ?? '')
const title = computed(() => props.hint ?? undefined)
</script>

<template>
  <span class="state-pill ui-status" :class="TONE_CLASS[resolvedTone]" :title="title" role="status">
    <span class="ui-status-text">{{ text }}<span v-if="sub" class="ui-status-sub">{{ sub }}</span></span>
    <span v-if="count !== null && count !== undefined && count !== ''" class="ui-status-count">{{ count }}</span>
  </span>
</template>

<style scoped>
/*
 * 只补状态标签**自身**的排布；胶囊外形、圆点与语义色由公共 .state-pill / .state-* 提供，
 * 不在这里复制一份配色（否则又会出现第二套 tone）。
 */
.ui-status{max-width:100%}
.ui-status-text{min-width:0;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.ui-status-sub{margin-left:6px;opacity:.85}
.ui-status-sub::before{content:"·";margin-right:6px}
.ui-status-count{flex:none;font-variant-numeric:tabular-nums;opacity:.75}
</style>

