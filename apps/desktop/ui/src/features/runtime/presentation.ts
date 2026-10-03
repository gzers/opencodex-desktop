// 运行来源与安装进度的展示投影（纯函数，便于测试）。

import type { InstallPhase, RuntimeSourceKind } from './api'

/** 来源类型的中文名；必须与后端解析结果同源，不在前端重新推断。 */
export function sourceKindLabel(kind: RuntimeSourceKind): string {
  switch (kind) {
    case 'explicit':
      return '用户指定'
    case 'managed':
      return '托管安装'
    case 'discovered':
      return '自动发现'
    default:
      return '未解析'
  }
}

/** 落点提示：来源路径是否在数据根内。 */
export function sourceLandingLabel(insideDataRoot: boolean): string {
  return insideDataRoot ? '数据根内' : '数据根外'
}

export function installPhaseLabel(phase: InstallPhase): string {
  switch (phase) {
    case 'preparing':
      return '正在校验安装位置'
    case 'downloading':
      return '正在从 registry 取包'
    case 'installing':
      return '正在安装进私有前缀'
    case 'validating':
      return '正在校验离线包'
    case 'extracting':
      return '正在展开到私有前缀'
    case 'verifying':
      return '正在校验入口与版本'
    case 'activating':
      return '正在切换运行来源'
    case 'done':
      return '安装完成'
    default:
      return '安装失败'
  }
}

/** 联网安装才显示命令行明细；离线只显示分包步骤（UI规范 §18.2 / §18.3）。 */
export function phaseShowsCommandLine(phase: InstallPhase): boolean {
  return phase === 'downloading' || phase === 'installing'
}

/** 离线分包进度列表（三步固定文案，与冻结契约一致）。 */
export function offlineSteps(phase: InstallPhase): Array<{ label: string; state: 'done' | 'active' | 'pending' }> {
  const order: InstallPhase[] = ['validating', 'extracting', 'verifying', 'activating', 'done']
  const labels = ['校验离线包', '展开到私有前缀并链接 bin', '校验并切换运行来源']
  const reached = (index: number) => {
    const target = order[index]
    const currentIndex = order.indexOf(phase)
    if (phase === 'failed') return 'pending'
    return currentIndex >= order.indexOf(target) ? 'done' : 'pending'
  }
  const states = labels.map((label, index) => {
    const state = reached(index)
    return { label, state: state as 'done' | 'active' | 'pending' }
  })
  // 当前阶段所在的那一步标为 active（未完成）。
  const currentIndex = order.indexOf(phase)
  const stepIndex = currentIndex >= 0 ? Math.min(labels.length - 1, Math.floor(currentIndex * labels.length / order.length)) : -1
  if (stepIndex >= 0 && states[stepIndex].state === 'pending') states[stepIndex].state = 'active'
  else if (stepIndex >= 0 && states[stepIndex].state === 'done' && phase !== 'done') states[stepIndex].state = 'active'
  return states
}

/** 代理展示一律走掩码；前端不接触明文凭据。 */
export function proxyMaskLabel(scheme: string, host: string): string {
  const clean = host.trim()
  if (!clean) return ''
  return `${scheme}://***@${clean}`
}
