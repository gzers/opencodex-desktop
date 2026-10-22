<template>
  <div class="model-picker-dialog">
    <div class="dialog-header">
      <h2>选择渠道模型</h2>
      <button @click="$emit('close')" class="close-btn">关闭</button>
    </div>

    <div class="dialog-body">
      <div class="filter-section">
        <div class="filter-group">
          <label>渠道</label>
          <input
            v-model="channelFilter"
            type="text"
            placeholder="搜索渠道"
            class="search-input"
          />
        </div>
        <div class="filter-group">
          <label>模型</label>
          <input
            v-model="modelFilter"
            type="text"
            placeholder="搜索模型名称或ID"
            class="search-input"
          />
        </div>
      </div>

      <div class="channels-section">
        <button
          v-for="ch in channelRows"
          :key="ch.id"
          :class="['channel-btn', { active: selectedChannel === ch.id }]"
          @click="selectedChannel = ch.id"
        >
          {{ ch.name }}
        </button>
      </div>

      <div class="models-section">
        <div class="models-header">
          <span>{{ filteredRows.length }} 个结果</span>
        </div>
        <table class="models-table">
          <thead>
            <tr>
              <th @click="toggleSort('model')">模型 ↕</th>
              <th @click="toggleSort('channel')">渠道 ↕</th>
              <th @click="toggleSort('context')">上下文 ↕</th>
              <th @click="toggleSort('tier')">推理档位 ↕</th>
              <th @click="toggleSort('status')">状态 ↕</th>
              <th>设置</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="row in paginatedRows"
              :key="row.id"
              :class="{ selected: selectedModels.has(row.id) }"
              @click="toggleModel(row.id)"
            >
              <td>{{ row.model }}</td>
              <td>{{ row.channel }}</td>
              <td>{{ row.context || '未知' }}</td>
              <td>{{ row.tier || '未知' }}</td>
              <td>{{ row.status || '可选' }}</td>
              <td>
                <button @click.stop="showModelSettings(row)">⚙️</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="dialog-footer">
      <div class="selection-info">已选 {{ selectedModels.size }} 项</div>
      <button @click="$emit('close')" class="btn-secondary">取消</button>
      <button @click="confirmSelection" class="btn-primary">确定选择</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useModelsStore } from './store'

const emit = defineEmits(['close', 'confirm'])

const store = useModelsStore()

// 数据源从 store 同步读取
const rows = computed(() => store.visibleModels)
const channels = computed(() => store.visibleChannels)

// 过滤和搜索
const channelFilter = ref('')
const modelFilter = ref('')
const selectedChannel = ref('all')
const selectedModels = ref(new Set<string>())

// 排序
const sortBy = ref('model')
const sortOrder = ref<'asc' | 'desc'>('asc')

// 过滤渠道（仅显示启用的）
const channelRows = computed(() => {
  const all = { id: 'all', name: '全部渠道' }
  const filtered = channels.value.filter(ch =>
    ch.name.toLowerCase().includes(channelFilter.value.toLowerCase())
  )
  return [all, ...filtered]
})

// 过滤模型
const filteredRows = computed(() => {
  let result = rows.value

  // 按渠道筛选
  if (selectedChannel.value !== 'all') {
    result = result.filter(r => r.channelId === selectedChannel.value)
  }

  // 按模型名称筛选
  if (modelFilter.value) {
    const query = modelFilter.value.toLowerCase()
    result = result.filter(r =>
      r.model.toLowerCase().includes(query) ||
      r.id.toLowerCase().includes(query)
    )
  }

  // 排序
  result = [...result].sort((a, b) => {
    const aVal = a[sortBy.value] || ''
    const bVal = b[sortBy.value] || ''
    const cmp = aVal < bVal ? -1 : aVal > bVal ? 1 : 0
    return sortOrder.value === 'asc' ? cmp : -cmp
  })

  return result
})

// 分页（可选）
const paginatedRows = computed(() => filteredRows.value)

// 切换排序
function toggleSort(field: string) {
  if (sortBy.value === field) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = field
    sortOrder.value = 'asc'
  }
}

// 切换选中
function toggleModel(id: string) {
  if (selectedModels.value.has(id)) {
    selectedModels.value.delete(id)
  } else {
    selectedModels.value.add(id)
  }
}

// 确认选择
function confirmSelection() {
  emit('confirm', Array.from(selectedModels.value))
  emit('close')
}

// 显示模型设置
function showModelSettings(row: any) {
  console.log('Settings for', row)
}
</script>

<style scoped>
.model-picker-dialog {
  display: flex;
  flex-direction: column;
  height: 80vh;
  max-width: 1200px;
  margin: 0 auto;
  background: var(--surface);
  border-radius: 8px;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid var(--border);
}

.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.filter-section {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.filter-group {
  flex: 1;
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 4px;
}

.channels-section {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 16px;
  max-height: 120px;
  overflow-y: auto;
}

.channel-btn {
  padding: 6px 12px;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--surface);
  cursor: pointer;
  transition: all 0.2s;
}

.channel-btn.active {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
}

.models-section {
  border: 1px solid var(--border);
  border-radius: 4px;
  overflow: hidden;
}

.models-header {
  padding: 8px 12px;
  background: var(--surface-variant);
  border-bottom: 1px solid var(--border);
}

.models-table {
  width: 100%;
  border-collapse: collapse;
}

.models-table th {
  padding: 12px;
  text-align: left;
  background: var(--surface-variant);
  cursor: pointer;
  user-select: none;
}

.models-table td {
  padding: 12px;
  border-top: 1px solid var(--border);
}

.models-table tr.selected {
  background: var(--primary-light);
}

.models-table tr:hover {
  background: var(--hover);
}

.dialog-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-top: 1px solid var(--border);
}

.btn-primary,
.btn-secondary {
  padding: 8px 16px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
}

.btn-primary {
  background: var(--primary);
  color: white;
}

.btn-secondary {
  background: var(--surface-variant);
}
</style>

