<script setup lang="ts">
/**
 * 纵向排列原语（`components/layout`）：统一常用间距与对齐，避免每处自写 flex+gap。
 *
 * - `gap` 接收设计刻度（默认 `var(--space-4)`）；间距只由本容器拥有，子级不再叠加 margin。
 * - `as` 可选语义元素（默认 `div`）；不制造多余 landmark。
 */
withDefaults(defineProps<{ gap?: string; as?: string }>(), {
  gap: 'var(--space-4)',
  as: 'div',
})
</script>

<template>
  <component :is="as" class="u-stack" :style="{ gap }">
    <slot />
  </component>
</template>

<style scoped>
.u-stack {
  display: flex;
  flex-direction: column;
  align-items: stretch;
}
</style>
