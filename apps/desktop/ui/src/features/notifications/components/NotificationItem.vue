<script setup lang="ts">
import type { NotificationItem as NotificationItemModel } from '@/types/ui'
import { formatNotificationTime } from '../model'

// 通知中心不显示时间；通知历史复用同一组件并显示时间列。
defineProps<{ item: NotificationItemModel; showTime?: boolean }>()
defineEmits<{ open: [string]; delete: [string] }>()
</script>

<template>
  <div
    class="notification-item"
    :class="[item.kind, { 'is-read': item.read || item.resolved, resolved: item.resolved, 'with-time': showTime }]"
  >
    <span class="severity" aria-hidden="true"></span>
    <button class="n-main" type="button" @click="$emit('open', item.id)">
      <strong>
        {{ item.title }}
        <span v-if="item.resolved" class="n-tag">已解决</span>
        <span v-else-if="item.read" class="n-tag">未解决</span>
      </strong>
      <span class="n-detail">{{ item.detail }}</span>
    </button>
    <time v-if="showTime" class="n-time">{{ formatNotificationTime(item.time) }}</time>
    <div class="n-actions">
      <button
        class="n-del"
        type="button"
        title="删除这条通知"
        :aria-label="`删除这条通知：${item.title}`"
        @click.stop="$emit('delete', item.id)"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M4 7h16M9 7V5.5A1.5 1.5 0 0 1 10.5 4h3A1.5 1.5 0 0 1 15 5.5V7M6.5 7l.8 12a2 2 0 0 0 2 1.9h5.4a2 2 0 0 0 2-1.9L17.5 7"/>
        </svg>
      </button>
    </div>
  </div>
</template>
