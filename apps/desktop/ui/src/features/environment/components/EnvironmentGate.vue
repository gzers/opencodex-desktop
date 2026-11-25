<script setup lang="ts">
import { computed, ref } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import { useRouteStore } from '@/stores/routes'
import { usePreparationStore } from '../preparation'
import { useEnvironmentStore } from '../store'
import IntegratedPreparation from './IntegratedPreparation.vue'

const props = defineProps<{ onRecheck?: () => void }>()
const environment = useEnvironmentStore()
const preparation = usePreparationStore()
const routes = useRouteStore()
const showPreparation = computed(() => environment.checkingPreparation || preparation.needsAttention ||
  (preparation.error && !environment.report))
const presentation = computed(() => environment.presentation)
const checking = computed(() => presentation.value.state === 'checking')
const passedCount = computed(() => presentation.value.checks.filter(check => check.tone === 'ok').length)
const deferred = ref(false)
const canDefer = computed(() => !preparation.active && !preparation.retrying &&
  (showPreparation.value || (!checking.value && !environment.ready)))

function recheck() {
  if (props.onRecheck) props.onRecheck()
  else void environment.refresh()
}
</script>

<template>
  <section class="card env-gate" data-testid="environment-gate">
    <div v-if="deferred && canDefer" class="overview-gate-head">
      <h2>运行环境待处理</h2>
      <UiButton @click="deferred = false">查看环境准备</UiButton>
    </div>
    <IntegratedPreparation v-else-if="showPreparation">
      <UiButton v-if="canDefer" variant="ghost" @click="deferred = true">稍后处理</UiButton>
    </IntegratedPreparation>
    <template v-else>
      <div class="overview-gate-head">
        <h2>{{ presentation.title }}</h2>
        <span class="overview-gate-count">{{ checking ? '检查中' : `${passedCount} / 3 项通过` }}</span>
      </div>
      <div class="overview-gate-body">
        <ol class="overview-checks" aria-label="环境检查结果">
          <li v-for="(check, index) in presentation.checks" :key="check.name">
            <span class="overview-check-index">0{{ index + 1 }}</span>
            <b>{{ check.name }}</b>
            <span class="overview-check-value" :data-tone="check.tone">
              <svg v-if="check.tone === 'ok'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="m5 12 4 4L19 6" /></svg>
              {{ check.value }}
            </span>
          </li>
        </ol>
        <div class="overview-repair">
          <p>{{ presentation.description }}</p>
          <div class="overview-repair-actions">
            <UiButton :disabled="environment.loading" @click="recheck">重新检查</UiButton>
            <UiButton variant="ghost" @click="routes.go('settings', { section: 'installation' })">查看安装配置</UiButton>
            <UiButton v-if="canDefer" variant="ghost" @click="deferred = true">稍后处理</UiButton>
          </div>
        </div>
      </div>
    </template>
  </section>
</template>
