<script setup lang="ts">
/**
 * 设置行（`components/patterns`）：承载既有 `<div class="setting-row">` 配方。
 *
 * - 左列由 `title` / `description` 渲染，默认插槽追加行内说明等额外内容。
 * - 右列由 `actions` 槽放置控件（调用方给出 `.controls`，保持与既有布局一致）。
 * - 数据由调用方传入、动作经事件抛出；组件不读业务 store、不调用 Tauri。
 */
defineProps<{ title?: string; description?: string }>()
</script>

<template>
  <div class="setting-row">
    <div>
      <div class="setting-title">{{ title }}<slot name="title-extra" /></div>
      <div v-if="description" class="setting-desc">{{ description }}</div>
      <slot />
    </div>
    <slot name="actions" />
  </div>
</template>
