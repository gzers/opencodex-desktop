<script setup lang="ts">
// 受限 Markdown 正文（`components/patterns`）：唯一的 `v-html` 边界。
//
// - 渲染契约由 `@/markdown` 决定：先整段转义再套标记，链接只保留文字、不生成 `href`；
//   不执行内联 HTML，也不对外授予打开任意目标的能力（IMP-04 §7.11 / §13.6）。
// - 组件只负责「受限渲染」；正文外观由所属容器经传入的类名与命名空间样式提供
//   （例如详情的 `.ext-detail-md`），不在公共组件内新造富文本色板/字号。
import { computed } from 'vue'
import { renderMarkdown } from '@/lib/markdown'

const props = defineProps<{ source: string | null }>()
const html = computed(() => (props.source ? renderMarkdown(props.source) : ''))
</script>

<template>
  <div class="markdown-content" v-html="html"></div>
</template>
