<script setup lang="ts">
// 概览「环境准备」门禁卡（UI规范 §25.1 / §25.3）。
//
// 结构对齐已确认原型 `#envGate`：标题 + 通过计数；左列纵向检查，右列当前指引 +
// 最相关一条命令 + 处理动作；底部一行说明与「完整指引」出口。
// 只消费 `buildEnvironmentPresentation` 的既有结论与既有业务动作（重新检查 / 安装 /
// 导入离线包 / 设置指引），不新增状态机、不伪造结论。
import { computed, ref } from 'vue'
import { useRouteStore } from '@/stores/routes'
import { useAppStore } from '@/stores/app'
import { buildEnvironmentPresentation, environmentChecksOrdered } from '../presentation'
import type { InstallSourceKind } from '@/features/runtime/api'

const props = defineProps<{ onRecheck?: () => void }>()
const app = useAppStore()
const routes = useRouteStore()
// 门禁要读运行来源：托管安装落在数据根内、不在发现候选里，只按「自动发现」
// 判断会误报「未发现 OpenCodex」（`UI规范` §18.5）。
const presentation = computed(() =>
  buildEnvironmentPresentation(app.environment, app.environmentLoading, app.runtimeSource?.kind ?? null, app.aboutApp?.platform),
)
const checks = computed(() =>
  environmentChecksOrdered(presentation.value.checks, presentation.value.state === 'checking'),
)
const passedCount = computed(() => checks.value.filter(check => check.tone === 'ok').length)
const checking = computed(() => presentation.value.state === 'checking')

// 门禁卡标题与修复区标题分开（原型 gateTitle / guidance 两组文案）。
const gateTitle: Record<string, string> = {
  checking: '检查启动条件',
  missing_node_brew: '缺少 Node.js',
  missing_node_nobrew: '缺少 Node.js',
  missing_npm: 'npm 不可用',
  missing_ocx: '尚未接入 OpenCodex',
}
const repairTitle: Record<string, string> = {
  checking: '检查顺序',
  missing_node_brew: '安装 Node.js',
  missing_node_nobrew: '安装 Node.js',
  missing_npm: '修复 npm',
  missing_ocx: '接入 OpenCodex',
}
const title = computed(() => gateTitle[presentation.value.state] ?? '检查启动条件')
const repair = computed(() => repairTitle[presentation.value.state] ?? '检查顺序')
const subtitle = computed(() =>
  checking.value ? '确认完成前暂不提供运行操作。' : '补全启动条件后，即可继续使用 OpenCodex。',
)
// 主卡只显示最相关的一条命令；其余校验命令在完整指引里查看（原型同款）。
// `missing_ocx` 也保留官方 npm 命令作为**终端备选**（安全边界要求可核对真实命令）。
const command = computed(() => presentation.value.commands[0] ?? null)
const note = computed(() =>
  presentation.value.state === 'missing_ocx'
    ? '安装仅写管理器数据目录；已有安装可在设置中指定来源。'
    : '未执行的项目显示「待检查」；补全前置条件后继续。',
)

const copiedCommand = ref<string | null>(null)

// 复制的是**原始命令**：HTML 属性里的 & 等实体必须还原，否则复制出去的是 `&amp;&amp;`。
function escape(value: string): string {
  return value.replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'")
}

async function copyCommand(cmd: string, event: MouseEvent) {
  const target = event.currentTarget as HTMLButtonElement
  try {
    // 没有剪贴板能力时不谎称「已复制」，只说明不会代执行。
    if (!navigator.clipboard) {
      app.showToast(`剪贴板不可用；不会执行 ${cmd}，请手动运行。`)
      return
    }
    await navigator.clipboard.writeText(escape(cmd))
    copiedCommand.value = cmd
    window.setTimeout(() => {
      if (copiedCommand.value === cmd) copiedCommand.value = null
    }, 1400)
    app.showToast(`已复制 ${escape(cmd)}；不会代你执行，请手动运行。`)
  } catch {
    app.showToast(`复制失败；不会执行 ${escape(cmd)}，请手动运行。`)
  } finally {
    target.blur()
  }
}

function recheck() {
  props.onRecheck?.()
}

function guide() {
  routes.go('settings', { section: 'installation' })
}

/**
 * 未安装引导的两个出口（UI规范 §18.5）：置位后由设置页消费并打开对应向导。
 * 概览不自己实现安装流程，避免与设置页出现两套口径。
 */
function installManaged(source: InstallSourceKind) {
  app.requestInstallModal(source)
  routes.go('settings', { section: 'installation' })
}
</script>

<template>
  <section class="card env-gate" data-testid="environment-gate">
    <div class="overview-gate-head">
      <div>
        <h2>{{ title }}</h2>
        <p>{{ subtitle }}</p>
      </div>
      <span class="overview-gate-count">{{ checking ? '检查中' : `${passedCount} / 3 项通过` }}</span>
    </div>
    <div class="overview-gate-body">
      <ol class="overview-checks" aria-label="环境检查结果">
        <li v-for="(check, index) in checks" :key="check.name">
          <span class="overview-check-index">0{{ index + 1 }}</span>
          <b>{{ check.name }}</b>
          <span class="overview-check-value" :data-tone="check.tone">
            <svg v-if="check.tone === 'ok'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="m5 12 4 4L19 6" /></svg>
            {{ check.value }}
          </span>
        </li>
      </ol>
      <section class="overview-repair">
        <h3>{{ repair }}</h3>
        <p>{{ presentation.description }}</p>
        <div v-if="command" class="env-command">
          <div class="env-command-label">{{ command.label }}</div>
          <button
            class="cli-copy-btn"
            :class="{ copied: copiedCommand === command.command }"
            type="button"
            @click="copyCommand(command.command, $event)"
          ><code>{{ command.command }}</code></button>
        </div>
        <div class="overview-repair-actions" :data-testid="presentation.state === 'missing_ocx' ? 'env-install-actions' : undefined">
          <button v-if="checking" class="btn" type="button" disabled>检查中…</button>
          <template v-else-if="presentation.state === 'missing_ocx'">
            <button class="btn primary" type="button" @click="installManaged('registry')">安装 OpenCodex</button>
            <button class="btn" type="button" @click="installManaged('offline')">导入离线包</button>
            <button class="btn ghost" type="button" @click="recheck">重新检查</button>
          </template>
          <template v-else>
            <button v-if="presentation.state === 'missing_node_nobrew'" class="btn primary" type="button" @click="guide">安装指引</button>
            <button class="btn" type="button" @click="recheck">重新检查</button>
          </template>
        </div>
      </section>
    </div>
    <div class="overview-gate-footer">
      <p>{{ note }}</p>
      <button type="button" @click="guide">完整指引 ›</button>
    </div>
    <p class="env-bound">应用不读取 PATH；不执行全局 npm 安装、不写系统目录与全局 npm 前缀，也不修改 OpenCodex 配置。托管安装只写自有数据根内的私有前缀。</p>
  </section>
</template>
