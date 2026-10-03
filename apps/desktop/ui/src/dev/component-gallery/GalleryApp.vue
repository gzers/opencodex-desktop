<script setup lang="ts">
// 开发态组件预览：直接引用生产公共组件，展示变体、状态、亮暗主题与长文本。
// 不触发真实 IPC；数据是本地固定样例，不代表生产默认值。
import { ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiCard from '@/components/ui/UiCard.vue'
import UiCardHeader from '@/components/ui/UiCardHeader.vue'
import SettingRow from '@/components/patterns/SettingRow.vue'
import Stack from '@/components/layout/Stack.vue'
import Inline from '@/components/layout/Inline.vue'

const theme = ref<'light' | 'dark'>('light')
const effects = ref<'high' | 'mid' | 'low'>('high')
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

    <Stack>
      <UiCard>
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
        </Inline>
      </UiCard>

      <UiCard>
        <UiCardHeader>
          <h2>卡片 UiCard / UiCardHeader</h2>
          <p>只承载表面配方；标题与操作由插槽给出。</p>
          <template #actions>
            <div class="toolbar"><UiButton class="ghost">操作</UiButton></div>
          </template>
        </UiCardHeader>
        <p>正文直接作为卡片内容流动，不在卡片的每个内容块上重复内边距。</p>
      </UiCard>

      <UiCard surface="overlay">
        <UiCardHeader>
          <h2>表面 variant</h2>
          <p>panel（默认）/ plain / overlay 与 density=compact；低档下 overlay 退实底。</p>
        </UiCardHeader>
        <Stack gap="var(--space-2)">
          <UiCard density="compact"><p>紧凑卡片密度</p></UiCard>
          <UiCard surface="plain"><p>无表面卡片（仅语义容器）</p></UiCard>
        </Stack>
      </UiCard>

      <UiCard>
        <UiCardHeader>
          <h2>设置行 SettingRow</h2>
          <p>左列标题/说明，右列控件；行内说明由默认插槽追加。</p>
        </UiCardHeader>
        <div class="setting-list">
          <SettingRow title="示例开关" description="说明与控件在同一行；窄容器不截断多行说明。">
            <template #actions>
              <div class="controls"><button class="toggle" role="switch" aria-checked="false" aria-label="示例开关"></button></div>
            </template>
          </SettingRow>
          <SettingRow title="只读结论" description="无法兑现为行为时只陈述事实，不提供假开关。">
            <template #actions>
              <div class="controls"><span class="setting-readonly">官方 CLI 未就绪</span></div>
            </template>
          </SettingRow>
        </div>
      </UiCard>

      <UiCard>
        <UiCardHeader>
          <h2>布局 Stack / Inline</h2>
          <p>统一间距与换行；间距只由容器拥有，避免与子级 margin 叠加。</p>
        </UiCardHeader>
        <Stack gap="var(--space-2)">
          <UiButton>Stack 子项 1</UiButton>
          <UiButton>Stack 子项 2</UiButton>
        </Stack>
      </UiCard>

      <UiCard>
        <UiCardHeader><h2>长文本与溢出</h2><p>长路径应可读、可复制，不撑破容器。</p></UiCardHeader>
        <p class="mono gallery-path" :title="longPath">{{ longPath }}</p>
      </UiCard>
    </Stack>
  </main>
</template>

<style scoped>
.gallery {
  max-width: 900px;
  margin: 0 auto;
  padding: 24px 22px;
}
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
