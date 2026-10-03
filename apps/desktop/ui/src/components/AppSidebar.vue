<script setup lang="ts">
import { useRouteStore } from '@/stores/routes'
import BrandMark from '@/components/BrandMark.vue'

const routes = useRouteStore()
const items = [
  { route: 'overview', label: '概览', icon: 'grid' },
  { route: 'panel', label: '面板', icon: 'panel' },
  { route: 'extensions', label: '拓展', icon: 'extensions' },
  { route: 'logs', label: '诊断', icon: 'logs' },
  { route: 'settings', label: '设置', icon: 'settings' },
] as const
</script>

<template>
  <aside class="side">
    <div class="brand">
      <BrandMark />
      <div class="brand-copy">
        <strong>OpenCodeX-Desktop</strong>
        <span>Desktop Manager</span>
      </div>
    </div>
    <nav class="nav">
      <button
        v-for="item in items"
        :key="item.route"
        :class="{ active: routes.current === item.route }"
        :title="item.label"
        :aria-label="item.label"
        @click="routes.go(item.route)"
      >
        <svg v-if="item.icon === 'grid'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="3" width="7" height="7" rx="1.5"/><rect x="14" y="3" width="7" height="7" rx="1.5"/><rect x="3" y="14" width="7" height="7" rx="1.5"/><rect x="14" y="14" width="7" height="7" rx="1.5"/></svg>
        <svg v-else-if="item.icon === 'panel'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2.5"/><path d="M9.5 4v16"/></svg>
        <svg v-else-if="item.icon === 'extensions'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M10 3.5a2.5 2.5 0 0 1 4 0V5h3A1.5 1.5 0 0 1 18.5 6.5V10h1.5a2.5 2.5 0 0 1 0 4h-1.5v3.5A1.5 1.5 0 0 1 17 19h-3v-1.5a2.5 2.5 0 0 0-5 0V19H6.5A1.5 1.5 0 0 1 5 17.5V14H3.5a2.5 2.5 0 0 1 0-4H5V6.5A1.5 1.5 0 0 1 6.5 5H10V3.5Z"/></svg>
        <svg v-else-if="item.icon === 'logs'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M14 3v4a1 1 0 0 0 1 1h4"/><path d="M17 21H7a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7l5 5v11a2 2 0 0 1-2 2Z"/><path d="M9 12h6M9 16h6"/></svg>
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
        <span class="nav-label">{{ item.label }}</span>
      </button>
    </nav>
  </aside>
</template>
