import type { EnvironmentReport } from './api'

interface EnvironmentCheck {
  name: string
  value: string
  tone: 'ok' | 'bad' | 'busy' | 'idle'
}

export interface EnvironmentPresentation {
  state: EnvironmentReport['gate'] | 'checking' | 'not_ready'
  title: string
  description: string
  checks: EnvironmentCheck[]
}

const dependencies = [
  { key: 'node', name: 'Node.js', gate: 'missing_node' },
  { key: 'npm', name: 'npm', gate: 'missing_npm' },
  { key: 'ocx', name: 'OpenCodex', gate: 'missing_ocx' },
] as const

export function buildEnvironmentPresentation(
  report: EnvironmentReport | null,
  loading = false,
  error = '',
): EnvironmentPresentation {
  if (!loading && error) {
    return {
      state: 'not_ready', title: '集成运行程序检查未就绪',
      description: '暂时无法确认内置依赖。可以重新检查，或到安装配置查看当前程序。',
      checks: dependencies.map(({ name }) => ({ name, value: '未声明', tone: 'idle' })),
    }
  }
  if (loading || !report) {
    return {
      state: 'checking', title: '正在检查内置运行环境',
      description: '依次检查随应用提供的 Node.js、npm 和 OpenCodex。',
      checks: dependencies.map(({ name }, index) => index === 0
        ? { name, value: '检查中', tone: 'busy' }
        : { name, value: '待检查', tone: 'idle' }),
    }
  }

  const missing = dependencies.findIndex(dependency => dependency.gate === report.gate)
  const checks = dependencies.map(({ key, name }, index): EnvironmentCheck => {
    if (missing >= 0 && index > missing) return { name, value: '待检查', tone: 'idle' }
    if (index === missing || !report[key].found) return { name, value: '未发现', tone: 'bad' }
    return { name, value: '已发现', tone: 'ok' }
  })
  if (report.gate === 'ready') {
    return {
      state: 'ready', title: '运行环境检查通过', checks,
      description: '内置 Node.js、npm 和 OpenCodex 已通过检查。',
    }
  }
  return {
    state: report.gate, title: '未发现内置 ' + dependencies[missing].name, checks,
    description: '这些程序随桌面应用提供。请查看安装配置中的内置程序状态，再重新检查。',
  }
}
