<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount, onMounted, nextTick } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import UiButton from '@/components/ui/UiButton.vue'
import { useAppStore } from '@/stores/app'
import { useRuntimeStore } from '@/features/runtime/store'
import { useEffectsStore } from '@/app/appearance/effects'
import type { InstallSourceKind, ProxyScheme } from '@/features/runtime/api'
import { displayInstallPrefix, installPrefixIssue, prefixInsideDataRoot } from '@/features/runtime/installPath'
import { displayPath } from '@/lib/paths'
import { installPhaseLabel, offlineSteps, phaseShowsCommandLine, proxyMaskLabel } from '@/features/runtime/presentation'
const app = useAppStore()
const runtimeProtection = useRuntimeStore()
const installDialog = ref<HTMLElement | null>(null)
let origin: HTMLElement | null = null
function onDialogKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') { event.stopPropagation(); event.preventDefault(); closeInstallModal(); return }
  if (event.key !== 'Tab') return
  const list = Array.from(installDialog.value?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), select:not(:disabled), summary, [href]') ?? [])
  const first = list[0], last = list.at(-1)
  if (event.shiftKey ? document.activeElement === first || document.activeElement === installDialog.value : document.activeElement === last || document.activeElement === installDialog.value) {
    event.preventDefault(); (event.shiftKey ? last : first)?.focus()
  }
}
const installOpen = ref(false)
watch(installOpen, async value => {
  if (value) { origin = document.activeElement instanceof HTMLElement ? document.activeElement : null; await nextTick(); installDialog.value?.focus() }
  else { origin?.focus(); origin = null; installProxy.value.secret = '' }
})
onBeforeUnmount(() => { origin?.focus(); installProxy.value.secret = '' })
const installStep = ref(1)
const installPrefix = ref('')
const installSource = ref<InstallSourceKind>('registry')
const installVersion = ref('latest')
const installOfflinePath = ref('')
const installConfirmed = ref(false)
const installProxy = ref({ scheme: 'http' as ProxyScheme, host: '', username: '', secret: '' })
const installAllowScripts = ref(false)

const installPrefixError = computed(() => installPrefixIssue(installPrefix.value, app.aboutApp?.platform))
const installLandingText = computed(() => {
  const inside = prefixInsideDataRoot(app.dataRootConfig?.currentDataRoot ?? '', installPrefix.value, app.aboutApp?.platform)
  return inside === null ? '' : inside ? '落点：数据根内私有前缀（推荐）。' : '落点：数据根外；仅管理本次安装。'
})
const proxyHint = computed(() => {
  const host = installProxy.value.host.trim()
  if (!host) return '留空直连；代理只作用于本次安装，不写系统代理与全局 npm 配置。'
  return `本次安装将通过 ${proxyMaskLabel(installProxy.value.scheme, host)}（SOCKS5 使用 socks5h://）。`
})
const installCanAdvance = computed(() => !!installPrefix.value.trim() && !installPrefixError.value
  && installConfirmed.value && (installSource.value === 'registry' || app.offlinePreview?.ok === true))
const installProgress = computed(() => app.runtimeInstall)
const installEffects = useEffectsStore()
const installCompleted = computed(() => !app.runtimeInstalling && !app.runtimeInstallError
  && !!app.runtimeInstallOutcome && installProgress.value?.phase === 'done')
const installStatusLabel = computed(() => {
  if (app.runtimeInstallError || installProgress.value?.phase === 'failed') return '安装失败'
  if (installProgress.value?.phase === 'done' && !installCompleted.value) return '正在核对安装结果'
  return installPhaseLabel(installProgress.value?.phase ?? 'preparing')
})
const offlineStepList = computed(() => offlineSteps(app.runtimeInstall?.phase ?? 'preparing', runtimeProtection.installLastPhase))
const showInstallCommandLine = computed(() =>
  installSource.value === 'registry' && phaseShowsCommandLine(app.runtimeInstall?.phase ?? 'preparing'),
)
function defaultInstallPrefix() {
  // 重装落在**登记过的**前缀上（自定义前缀安装后不要悄悄搬回默认位置）。
  return displayInstallPrefix(app.runtimeSource?.managedPrefix ?? '', app.aboutApp?.platform)
}

/** 「恢复默认」只认数据根默认落点，不受自定义前缀影响。 */
function managedDefaultPrefix() {
  return displayInstallPrefix(app.runtimeSource?.defaultPrefix ?? '', app.aboutApp?.platform)
}

function openInstallModal(source: InstallSourceKind, version?: string) {
  if (app.runtimeInstalling) { installOpen.value = true; return }
  installOpen.value = true
  installStep.value = 1
  installPrefix.value = defaultInstallPrefix()
  if (!installPrefix.value) void app.loadRuntimeSource().then(() => {
    if (installOpen.value && !installPrefix.value) installPrefix.value = defaultInstallPrefix()
  })
  installSource.value = source
  installVersion.value = version ?? 'latest'
  installOfflinePath.value = ''
  installConfirmed.value = false
  installProofReset()
  app.clearOfflinePreview()
}

function installProofReset() {
  installProxy.value = { scheme: 'http', host: '', username: '', secret: '' }
  installAllowScripts.value = false
}

function closeInstallModal() {
  if (app.runtimeInstalling) return
  installOpen.value = false
}

async function chooseInstallPrefix() {
  const picked = await open({ directory: true, multiple: false, title: '选择 OpenCodex 托管安装位置' })
  if (typeof picked === 'string') installPrefix.value = displayInstallPrefix(picked, app.aboutApp?.platform)
}

function resetInstallPrefix() {
  installPrefix.value = managedDefaultPrefix()
}

let stopDrop: (() => void) | undefined
onMounted(async () => {
  if (!('__TAURI_INTERNALS__' in window)) return
  try {
    const { getCurrentWebview } = await import('@tauri-apps/api/webview')
    const stop = await getCurrentWebview().onDragDropEvent(event => {
      if (installOpen.value && installStep.value < 4 && installSource.value === 'offline' && event.payload.type === 'drop' && event.payload.paths.length === 1) {
        installOfflinePath.value = event.payload.paths[0]!
        void app.previewOfflinePackage(installOfflinePath.value)
      }
    })
    stopDrop = stop
  } catch { /* 普通浏览器无原生拖放；选择文件入口始终可用。 */ }
})
onBeforeUnmount(() => stopDrop?.())
async function chooseOfflinePackage() {
  const picked = await open({
    multiple: false,
    title: '选择 OpenCodex 离线包',
    filters: [{ name: '离线包', extensions: ['tgz', 'gz'] }],
  })
  if (typeof picked !== 'string') return
  installOfflinePath.value = picked
  await app.previewOfflinePackage(picked)
}

function advanceInstall() {
  if (installCanAdvance.value) void runInstall()
}

async function runInstall() {
  installStep.value = 4
  const useProxy = installSource.value === 'registry' && installProxy.value.host.trim() !== ''
  await app.installRuntime({
    prefix: installPrefix.value.trim() || null,
    source: installSource.value,
    version: installSource.value === 'registry' ? (installVersion.value.trim() || 'latest') : null,
    offlinePath: installSource.value === 'offline' ? installOfflinePath.value || null : null,
    proxyScheme: useProxy ? installProxy.value.scheme : null,
    proxyHost: useProxy ? installProxy.value.host.trim() : null,
    proxyUsername: useProxy && installProxy.value.username.trim() ? installProxy.value.username.trim() : null,
    proxySecret: useProxy && installProxy.value.username.trim() ? installProxy.value.secret : null,
    allowScripts: installAllowScripts.value,
  })
}

async function cancelInstall() {
  if (app.runtimeInstalling) {
    const cancelled = await app.cancelRuntimeInstall()
    app.showToast(cancelled ? '已请求取消安装。' : '当前没有可取消的安装。')
    return
  }
  closeInstallModal()
}

function finishInstall() {
  if (!installCompleted.value) return
  const version = app.runtimeInstallOutcome?.version
  installOpen.value = false
  if (version) app.showToast(`OpenCodex ${version} 已就绪。`)
}

watch(() => app.installModalRequest, value => {
  if (!value) return
  const source = app.consumeInstallModalRequest()
  if (source) openInstallModal(source)
}, { immediate: true })
function formatBytes(size: number) {
  if (size < 1024) return size + ' B'
  return (size / 1024 / 1024).toFixed(1) + ' MB'
}
</script>
<template>
  <div v-if="installOpen" class="modal-mask" data-testid="runtime-install-modal">
    <div ref="installDialog" class="modal modal-wide wiz runtime-install-dialog" tabindex="-1" @keydown="onDialogKeydown" role="dialog" aria-modal="true" aria-labelledby="wiz-title">
      <h3 id="wiz-title">安装 OpenCodex</h3>
      <ol class="wiz-steps" aria-label="安装步骤">
        <li v-for="(label, index) in ['安装位置', '安装源', '确认', '安装进度']" :key="label" :class="{ active: installStep < 4 ? index < 2 || index === 2 && installConfirmed : index === 3, done: installStep === 4 && index < 3 }"><span>{{ index + 1 }}</span>{{ label }}</li>
      </ol>

      <div v-if="installStep < 4" class="wiz-body">
        <div class="wiz-block"><label for="wiz-prefix" class="wiz-label">安装位置</label><div class="wiz-inline"><input id="wiz-prefix" v-model="installPrefix" class="input local-path" type="text" autocapitalize="off" autocorrect="off" spellcheck="false"><UiButton class="btn ghost" type="button" @click="chooseInstallPrefix">选择…</UiButton><UiButton class="btn ghost" type="button" :disabled="installPrefix === managedDefaultPrefix()" @click="resetInstallPrefix">恢复默认</UiButton></div><p v-if="installPrefixError" class="wiz-error">{{ installPrefixError }}</p><p v-else class="wiz-hint">{{ installLandingText }}</p></div>
      <div class="wiz-block">
        <span class="wiz-label">安装源</span>
        <label class="wiz-choice install-source-choice" :class="{ on: installSource === 'registry' }"><input v-model="installSource" type="radio" value="registry"><span><b>联网安装</b><small>官方包 @bitkyc08/opencodex</small></span></label>
        <label class="wiz-choice install-source-choice" :class="{ on: installSource === 'offline' }"><input v-model="installSource" type="radio" value="offline"><span><b>导入离线包</b><small>网络不通时用官方 .tgz</small></span></label>

        <div v-if="installSource === 'offline'" class="wiz-drop" data-testid="wiz-offline">
          <p>把 <code>opencodex-&lt;version&gt;.tgz</code> 拖到这里，或选择文件</p>
          <div class="wiz-actions"><UiButton class="btn ghost" type="button" @click="chooseOfflinePackage">选择文件…</UiButton></div>
          <p v-if="app.offlinePreview" class="wiz-hint" :class="{ 'wiz-error': !app.offlinePreview.ok }">
            {{ app.offlinePreview.fileName }} · {{ formatBytes(app.offlinePreview.fileSize) }} ·
            {{ app.offlinePreview.ok ? `SHA-256 ${app.offlinePreview.sha256Prefix}…` : app.offlinePreview.message }}
          </p>
          <p class="wiz-hint">离线包不联网，但仍会写入数据根内的私有前缀；不匹配时拒绝，不回退成「任意 tgz 都装」。</p>
          <details class="wiz-more"><summary>下载地址</summary><p>npm 包页面：https://www.npmjs.com/package/@bitkyc08/opencodex</p></details>
        </div>

        <div v-else class="wiz-proxy" data-testid="wiz-proxy">
          <div class="field"><label>代理（可选）</label>
            <div class="wiz-inline">
              <select v-model="installProxy.scheme" class="input"><option value="http">HTTP</option><option value="socks5h">SOCKS5</option></select>
              <input v-model="installProxy.host" class="input" placeholder="host:port" type="text" autocapitalize="off" autocorrect="off" spellcheck="false">
            </div>
          </div>
          <details class="wiz-more"><summary>代理需要认证</summary>
            <div class="wiz-inline">
              <input v-model="installProxy.username" class="input" placeholder="用户名" type="text" autocapitalize="off" autocorrect="off" spellcheck="false">
              <input v-model="installProxy.secret" class="input" placeholder="密码" type="password">
            </div>
            <p class="wiz-hint">凭据只用于本次安装：写进临时配置文件（Unix 0600，用完即删），不进命令行参数、不写系统代理、不落盘。</p>
          </details>
          <p class="wiz-hint">{{ proxyHint }}</p>
        </div>
      </div>
      <div class="wiz-block">
        <span class="wiz-label">版本</span>
        <div class="wiz-inline"><input v-model="installVersion" aria-label="版本" class="input" placeholder="latest 或具体版本号" type="text" autocapitalize="off" autocorrect="off" spellcheck="false" :disabled="installSource === 'offline'"><span class="wiz-hint">{{ installSource === 'offline' ? '版本由离线包决定' : 'latest 或具体版本号' }}</span></div>
        <label class="wiz-choice"><input v-model="installConfirmed" type="checkbox" data-testid="wiz-ack">我已确认安装位置与安装源</label>
        <details class="wiz-more"><summary>说明 / 安全边界</summary>
          <p class="wiz-hint">默认以 <code>--ignore-scripts</code> 安装；若包必须执行安装脚本才能生成入口，会先回来征求你的二次确认。</p>
        </details>
      </div>

      </div>
      <div v-else class="wiz-body">
        <div v-if="!app.runtimeInstallError && installProgress?.phase !== 'failed'" class="wiz-progress" role="progressbar" aria-label="OpenCodex 安装进度"
          :aria-valuenow="installCompleted ? 100 : undefined" :aria-valuemin="0" :aria-valuemax="100"
          :aria-valuetext="installCompleted ? '安装已完成' : installStatusLabel + '，等待实际完成'">
          <div class="wiz-progress-bar" :class="{ 'is-indeterminate': !installCompleted, 'is-animated': !installCompleted && installEffects.strategy.animated, 'is-complete': installCompleted }"></div>
        </div>
        <p class="wiz-status" data-testid="wiz-status">
          {{ installStatusLabel }}
          <template v-if="app.runtimeInstallOutcome"> · {{ app.runtimeInstallOutcome.version }} · {{ installSource === 'registry' ? '联网安装' : '离线导入' }}</template>

        </p>
        <pre v-if="showInstallCommandLine" class="wiz-console" data-testid="wiz-console">{{ app.runtimeInstallLines.join('\n') || '等待 npm 输出…' }}</pre>
        <ul v-if="installSource === 'offline'" class="wiz-substeps" data-testid="wiz-substeps">
          <li v-for="step in offlineStepList" :key="step.label" :class="`is-${step.state}`">{{ step.label }}</li>
        </ul>
        <details v-if="installSource === 'offline' && app.runtimeInstallLines.length && (app.runtimeInstallError || installCompleted)" class="wiz-more">
          <summary>安装输出</summary>
          <pre class="wiz-console" data-testid="wiz-console">{{ app.runtimeInstallLines.join('\n') }}</pre>
        </details>
        <p v-if="app.runtimeInstallError" class="wiz-error" data-testid="wiz-error">{{ app.runtimeInstallError }}</p>
        <p v-if="app.runtimeInstallOutcome" class="wiz-hint" data-testid="wiz-outcome">
          入口 <span class="local-path">{{ displayPath(app.runtimeInstallOutcome.entry) }}</span> · SHA-256 {{ app.runtimeInstallOutcome.tarballSha256.slice(0, 16) }}…
          <template v-if="app.runtimeInstallOutcome.needsRestart"> · 代理正在运行，重启后生效。</template>
          <template v-if="app.runtimeInstallOutcome.protectionReconciliationPending"> · 保护备份仍保留，完整性核对未完成。请查看诊断日志。</template>
        </p>
      </div>

      <div class="wiz-foot">
        <UiButton class="btn ghost" type="button" @click="cancelInstall()">{{ app.runtimeInstalling ? '取消安装' : '取消' }}</UiButton>
        <UiButton v-if="installStep < 4" class="btn primary" type="button" :disabled="!installCanAdvance" @click="advanceInstall()">开始安装</UiButton>
        <template v-else-if="app.runtimeInstallError">
          <UiButton class="btn primary" type="button" :disabled="!app.runtimeInstallScriptsRequired || app.runtimeInstalling" data-testid="wiz-allow-scripts" @click="installAllowScripts = true; runInstall()">允许安装脚本并重试</UiButton>
          <UiButton class="btn ghost" type="button" @click="closeInstallModal()">关闭</UiButton>
        </template>
        <UiButton v-else class="btn primary" type="button" :disabled="!installCompleted" data-testid="wiz-finish" @click="finishInstall()">完成</UiButton>
      </div>
    </div>
  </div>

</template>

<style scoped>
.runtime-install-dialog { width:min(520px,calc(100% - 40px)); max-height:min(calc(88vh / var(--ui-zoom)),calc(100% - 40px)); display:flex; flex-direction:column; overflow:hidden; }
.wiz-body { overflow:auto; min-height:0; padding-right:4px; gap:14px; }
.wiz-steps { flex-wrap:wrap; flex:none; gap:6px; }
.wiz-steps li {padding:5px 10px;border:1px solid var(--border);border-radius:var(--radius-pill);font-size:var(--text-caption)}
.wiz-steps li.active {border-color:var(--accent);color:var(--text);background:color-mix(in srgb,var(--accent) 8%,var(--panel))}
.wiz-steps li span {width:16px;height:16px;background:var(--raised);font-size:10px;border:0;}
.wiz-foot { flex:none; }
.wiz-block { display:grid; gap:8px; }
.wiz-label { color:var(--muted); font-size:var(--text-label); }
.install-source-choice { border:1px solid var(--border); border-radius:var(--radius); padding:10px 12px; }
.install-source-choice.on { border-color:var(--accent); background:color-mix(in srgb,var(--accent) 8%,var(--panel)); }
.install-source-choice span { display:grid; gap:4px; }
.install-source-choice b {font-size:var(--text-label);font-weight:650;}
.wiz-proxy .wiz-inline select {flex:none;width:105px;}
.wiz-inline {align-items:center;}
.install-source-choice small { color:var(--muted); font-size:var(--text-label); }
.wiz-console { font-family:var(--font-path); flex:none; }
.wiz-error { white-space:pre-wrap; font-family:var(--font-path); }
</style>
