<script setup lang="ts">
import AppTopbar from '@/components/AppTopbar.vue'
import UiButton from '@/components/ui/UiButton.vue'
import { useRouteStore } from '@/stores/routes'
const routes = useRouteStore()
// 渠道、模型、协议和密钥由 OpenCodex 自己管理。Desktop 只提供入口，
// 不读取、不编辑、不迁移也不恢复 OpenCodex 的原生配置。
</script>
<template>
  <section class="route-section model-entry-route">
    <AppTopbar />
    <article class="card model-entry">
      <div class="card-head">
        <div>
          <h2>模型与渠道</h2>
          <p>渠道、模型、协议和密钥由 OpenCodex 自己管理。</p>
        </div>
      </div>
      <p>Desktop 不再编辑、导入、回退或迁移 OpenCodex 的配置文件，也不会读写你的本机 <code>~/.codex</code>。请在官方界面中修改这些内容；保存后由 OpenCodex 自己让 Codex 生效。</p>
      <UiButton variant="primary" @click="routes.go('panel')">在 OpenCodex 中管理</UiButton>
    </article>
  </section>
</template>
<style scoped>
.model-entry-route{display:grid;gap:var(--space-4);min-width:0}
.model-entry{max-width:760px}
.model-entry p{color:var(--muted);line-height:1.7;overflow-wrap:anywhere}
.model-entry code{font-family:var(--font-code)}
</style>
