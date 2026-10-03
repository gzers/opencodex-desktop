<script setup lang="ts">
/**
 * 卡片外观基础（`components/ui`）。
 *
 * - 只负责“表面”：`background` / 描边 / 圆角 / 阴影 / 内边距，全部沿用既有 `.card`。
 * - 不承担内容结构；标题、正文与操作区由调用方按语义放入插槽，避免出现
 *   `isRuntime/isSync/isDanger` 之类的万能开关。
 * - 额外类名（如 `ovb-status`、`mod`）通过属性透传合并到根元素，保持既有布局选择器可用。
 * - 语义元素可经 `as` 指定（默认 `article`）；标题级别由页面层级决定，组件不写死。
 * - 变体 API（§26.3）：`density`（comfortable/compact）、`surface`（panel/plain/overlay）。
 */
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    as?: string
    density?: 'comfortable' | 'compact'
    surface?: 'panel' | 'plain' | 'overlay'
  }>(),
  { as: 'article', density: 'comfortable', surface: 'panel' },
)

const classes = computed(() => [
  'card',
  props.density === 'compact' ? 'card-compact' : '',
  props.surface !== 'panel' ? `card-${props.surface}` : '',
])
</script>

<template>
  <component :is="as" :class="classes">
    <slot />
  </component>
</template>
