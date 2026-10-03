<script setup lang="ts">
/**
 * 卡片标题区（`components/ui`）：承载既有 `<div class="card-head">` 配方。
 *
 * - 左槽（默认插槽）放标题与说明（`h2`/`h3` + `p`）；标题级别由页面层级决定，组件不写死。
 * - `actions` 槽放右侧操作：单个 `.btn`，或承载多个按钮的 `.toolbar`。
 * - 只是配方容器：不读业务 store、不调用 Tauri，额外类名/属性透传到根元素。
 */
</script>

<template>
  <div class="card-head">
    <div>
      <slot />
    </div>
    <slot name="actions" />
  </div>
</template>
