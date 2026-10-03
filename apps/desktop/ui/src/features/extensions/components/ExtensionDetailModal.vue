<script setup lang="ts">
import { onMounted, ref, useId } from 'vue'
import ClientIcon from './ClientIcon.vue'
import MarkdownContent from '@/components/patterns/MarkdownContent.vue'
import { clientLabels, skillClientOrder } from '../presentation'
import type { ExtensionClientId } from '@/features/extensions/api'

interface DetailFact {
  label: string
  value: string
  /** 整行显示（原型 `.span-2`）。 */
  wide?: boolean
  /** 取值区自带内部滚动（长名单，例如 MCP 环境变量键名）。 */
  scroll?: boolean
}

interface DetailAction {
  key: string
  title: string
  icon: 'refresh' | 'trash' | 'edit'
  danger?: boolean
}

const props = defineProps<{
  title: string
  /** 已就绪的 Markdown 正文（描述 + 条目正文）；为空时显示空态。 */
  markdown: string | null
  markdownTruncated?: boolean
  facts: DetailFact[]
  enabledClients: ExtensionClientId[]
  actions: DetailAction[]
  busy?: boolean
}>()

const emit = defineEmits<{
  (event: 'close'): void
  (event: 'toggle-client', client: ExtensionClientId): void
  (event: 'action', key: string): void
}>()

const titleId = `ext-detail-title-${useId()}`
const dialog = ref<HTMLElement | null>(null)

const clientOrder = skillClientOrder

// 标题动作按原型用图标按钮（刷新 / 垃圾桶 / 铅笔），路径与列表行内动作同源。
const ACTION_ICONS: Record<DetailAction['icon'], string[]> = {
  refresh: ['M20 11a8 8 0 1 0-2.34 5.66', 'M20 4v7h-7'],
  trash: ['M4 7h16', 'M10 11v6', 'M14 11v6', 'M6 7l1 13h10l1-13', 'M9 7V4h6v3'],
  edit: ['M4 20h4l10-10-4-4L4 16v4Z', 'm14 6 4 4'],
}

function actionIcon(action: DetailAction) {
  return ACTION_ICONS[action.icon]
}

function isEnabled(client: ExtensionClientId) {
  return props.enabledClients.includes(client)
}

function close() {
  emit('close')
}

onMounted(() => dialog.value?.focus())
</script>

<template>
  <div class="modal-mask" @click.self="close">
    <div
      ref="dialog"
      class="modal modal-wide modal-tall ext-detail"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="titleId"
      tabindex="-1"
      @keydown.esc.stop.prevent="close"
    >
      <div class="modal-head">
        <h3 :id="titleId">{{ title }}</h3>
        <div v-if="actions.length" class="modal-head-actions">
          <button
            v-for="action in actions"
            :key="action.key"
            type="button"
            class="icon-btn"
            :class="{ danger: action.danger }"
            :title="action.title"
            :aria-label="action.title"
            @click="emit('action', action.key)"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path v-for="(d, index) in actionIcon(action)" :key="index" :d="d" />
            </svg>
          </button>
        </div>
      </div>
      <div class="ext-detail-body">
        <section class="ext-detail-doc" aria-label="完整内容">
          <MarkdownContent v-if="markdown" class="ext-detail-md" :source="markdown" />
          <p v-else class="ext-detail-empty">暂无正文内容。</p>
          <p v-if="markdownTruncated" class="ext-detail-truncated">正文超过显示上限，已截断。</p>
        </section>
        <dl class="ext-detail-facts">
          <div v-for="fact in facts" :key="fact.label" :class="{ 'span-2': fact.wide }">
            <dt>{{ fact.label }}</dt>
            <dd :class="{ 'is-scrollable': fact.scroll }">{{ fact.value }}</dd>
          </div>
        </dl>
        <ul class="ext-detail-targets">
          <li v-for="client in clientOrder" :key="client" :class="{ on: isEnabled(client) }">
            <button
              type="button"
              class="ext-detail-target"
              :aria-pressed="isEnabled(client) ? 'true' : 'false'"
              :disabled="busy"
              :aria-label="`${clientLabels[client]}：${isEnabled(client) ? '已同步，点击关闭同步' : '未同步，点击同步'}`"
              @click="emit('toggle-client', client)"
            >
              <ClientIcon :client="client" />
              <span>{{ clientLabels[client] }}</span>
              <em>{{ isEnabled(client) ? '已同步' : '未同步' }}</em>
            </button>
          </li>
        </ul>
      </div>
      <div class="modal-actions">
        <button type="button" class="btn" @click="close">关闭</button>
      </div>
    </div>
  </div>
</template>
