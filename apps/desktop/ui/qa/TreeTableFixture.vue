<script setup>
import { ref } from "vue"
import UiTreeTable from "../src/components/ui/UiTreeTable.vue"

const width = ref(420)
const dark = ref(false)
const loading = ref(false)
const selected = ref("mixed")
const makeNodes = () => [{ id: "root", label: "备份目录", children: [
  { id: "long-folder", label: "升级前的管理器偏好备份与完整性清单目录（用于验证多行名称首行对齐）", selectable: true, selected: selected.value, children: [
    { id: "long-file", label: "preferences-with-a-long-descriptive-file-name-for-wrapping.json", selectable: true },
    { id: "disabled", label: "禁用复选框的多行文件名称与说明.json", selectable: true, disabled: true },
  ] },
  { id: "no-selection", label: "不带复选框的长名称文件与占位对齐检查.json" },
] }]
const nodes = ref(makeNodes())
function theme() {
  dark.value = !dark.value
  document.documentElement.dataset.theme = dark.value ? "dark" : "light"
}
function select(id, checked) {
  if (id === "long-folder") selected.value = checked
  nodes.value = makeNodes()
}
function refresh(event) {
  if (event.key !== "r" || loading.value) return
  event.preventDefault()
  loading.value = true
  setTimeout(() => { nodes.value = makeNodes(); loading.value = false }, 600)
}
</script>

<template>
  <main class="fixture">
    <h1>共享树表组件 QA</h1>
    <p>合成数据，仅渲染生产组件；不读取或写入真实文件。</p>
    <div class="toolbar">
      <button type="button" @click="width = width === 420 ? 300 : 420">切换窄宽</button>
      <button type="button" @click="theme">{{ dark ? "切换浅色" : "切换深色" }}</button>
    </div>
    <p>聚焦行控件后按 R，验证异步刷新；箭头键验证树导航。</p>
    <section :style="{ width: `${width}px` }" @keydown="refresh">
      <UiTreeTable :nodes="nodes" label="多行树表测试" :initial-expanded-depth="8" :loading="loading" @select="select">
        <template #icon="{ node }">
          <svg aria-hidden="true" viewBox="0 0 16 16" width="16" height="16" fill="none" stroke="currentColor">
            <path v-if="node.children?.length" d="M1.5 4h5l1-1h7v10h-13z" />
            <path v-else d="M4 1.5h5l3 3v10H4z M9 1.5v3h3" />
          </svg>
        </template>
      </UiTreeTable>
    </section>
  </main>
</template>

<style scoped>
.fixture { padding: 24px; }
h1 { font-size: 20px; margin: 0 0 12px; }
p { color: var(--muted); }
.toolbar { display: flex; gap: 8px; }
.toolbar button { border: 1px solid var(--border); border-radius: 6px; padding: 6px 12px; background: var(--panel); color: var(--text); }
section { max-width: 100%; }
</style>
