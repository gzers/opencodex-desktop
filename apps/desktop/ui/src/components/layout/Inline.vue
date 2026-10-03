<script setup lang="ts">
/**
 * 横向排列原语（`components/layout`）：统一常用间距、对齐与换行。
 *
 * - `gap` 接收设计刻度（默认 `var(--space-2)`）；`wrap` 控制是否换行（默认换行，窄容器不挤压）。
 * - `as` 可选语义元素（默认 `div`）。
 */
withDefaults(defineProps<{ gap?: string; wrap?: boolean; align?: string; as?: string }>(), {
  gap: 'var(--space-2)',
  wrap: true,
  align: 'center',
  as: 'div',
})
</script>

<template>
  <component
    :is="as"
    class="u-inline"
    :style="{ gap, alignItems: align, flexWrap: wrap ? 'wrap' : 'nowrap' }"
  >
    <slot />
  </component>
</template>

<style scoped>
.u-inline {
  display: flex;
}
</style>
