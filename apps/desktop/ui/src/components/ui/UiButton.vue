<script setup lang="ts">
/**
 * 基础按钮控件（`components/ui`）。
 *
 * - 统一承载既有 `.btn` 配方：尺寸/圆角/字重/焦点环/禁用透明度都由 `.btn` 提供。
 * - 变体 API（§26.3）：`variant`（default/primary/danger/ghost）、`size`（sm/md/lg）、
 *   `density`（compact/comfortable）、`surface`；不使用 `isGlassHigh` 之类组合布尔值。
 * - 向后兼容：既有调用方传 `class="primary"` / `class="btn ghost"` 仍可合并生效。
 * - 不读取任何业务 store，也不直接调用 Tauri；点击经事件向上抛出。
 * - `loading` 期间禁用并给出 `aria-busy` 与旋转指示；不改变按钮宽度（沿用 `.btn-spinner`）。
 */
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    type?: 'button' | 'submit' | 'reset'
    loading?: boolean
    disabled?: boolean
    variant?: 'default' | 'primary' | 'danger' | 'ghost'
    size?: 'sm' | 'md' | 'lg'
    density?: 'comfortable' | 'compact'
    surface?: 'control' | 'plain'
  }>(),
  {
    type: 'button',
    loading: false,
    disabled: false,
    variant: 'default',
    size: 'md',
    density: 'comfortable',
    surface: 'control',
  },
)

const classes = computed(() => [
  'btn',
  props.variant !== 'default' ? props.variant : '',
  props.size !== 'md' ? `btn-${props.size}` : '',
  props.density === 'compact' ? 'btn-compact' : '',
  props.surface === 'plain' ? 'btn-plain' : '',
])
</script>

<template>
  <button
    :class="classes"
    :type="type"
    :disabled="disabled || loading"
    :aria-busy="loading ? 'true' : undefined"
  >
    <span v-if="loading" class="btn-spinner" aria-hidden="true"></span>
    <slot />
  </button>
</template>
