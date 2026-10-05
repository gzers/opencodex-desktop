import type { EnvironmentReport } from './api'
import type { RuntimeSourceKind } from '@/features/runtime/api'

export interface EnvironmentCommand {
  label: string
  command: string
}

export interface EnvironmentLink {
  text: string
  note: string
}

export interface EnvironmentCheckPresentation {
  name: string
  value: string
  state: 'running' | 'bad' | 'muted'
  label: '通过' | '未通过' | '未检查' | '检查中'
}

export interface EnvironmentPresentation {
  state:
    | 'checking'
    | 'ready'
    | 'ready_via_source'
    | 'missing_node_brew'
    | 'missing_node_nobrew'
    | 'missing_npm'
    | 'missing_ocx'
  title: string
  description: string
  tag: string
  statusClass: '' | 'busy' | 'ok'
  hidden: boolean
  blocked: boolean
  checks: EnvironmentCheckPresentation[]
  commands: EnvironmentCommand[]
  link?: EnvironmentLink
  contractNote: boolean
  installEmptyMessage?: string
}

const CHECK_NAMES = ['Node.js', 'npm', 'OpenCodex'] as const

export type EnvironmentCheckTone = 'ok' | 'bad' | 'busy' | 'idle'

/** 概览环境摘要 / 环境弹窗的展示行：值 + 色调（与已确认原型 `checks()` 一致）。 */
export interface EnvironmentCheckDisplay {
  name: string
  value: string
  tone: EnvironmentCheckTone
}

function toneOf(state: EnvironmentCheckPresentation['state']): EnvironmentCheckTone {
  if (state === 'running') return 'ok'
  if (state === 'bad') return 'bad'
  if (state === 'muted') return 'idle'
  return 'busy'
}

/**
 * 概览按**检查顺序**呈现：检查进行中只有当前一项是「检查中」，其后各项为「待检查」；
 * 未执行到的项也显示「待检查」而不是含糊的「未检查」。顺序为 Node.js → npm → OpenCodex，
 * 与 `buildEnvironmentPresentation` 给出的检查数组同序（UI规范 §19 / 原型 `envStates`）。
 */
export function environmentChecksOrdered(
  checks: EnvironmentCheckPresentation[],
  checking: boolean,
): EnvironmentCheckDisplay[] {
  return checks.map((check, index) => {
    if (checking) {
      return index === 0
        ? { name: check.name, value: '检查中', tone: 'busy' }
        : { name: check.name, value: '待检查', tone: 'idle' }
    }
    if (check.value === '未检查') return { name: check.name, value: '待检查', tone: 'idle' }
    return { name: check.name, value: check.value, tone: toneOf(check.state) }
  })
}

function checkToPresentation(name: (typeof CHECK_NAMES)[number], check: EnvironmentReport['node']) {
  if (check.found) {
    return { name, value: '已发现', state: 'running', label: '通过' } as const
  }
  return { name, value: '未发现', state: 'bad', label: '未通过' } as const
}

function notChecked(name: (typeof CHECK_NAMES)[number]) {
  return { name, value: '未检查', state: 'muted', label: '未检查' } as const
}

/**
 * 运行来源已经解析出 `ocx` 时，本机「自动发现」为空不再构成待处理项
 * （`UI规范` §18.5）：`missing_ocx` 门禁只说明发现候选为空，不代表没有可用的
 * OpenCodex——托管安装落在数据根内，本来就不在发现候选里。
 */
function runtimeProvidesOcx(kind: RuntimeSourceKind | null | undefined): boolean {
  return kind === 'managed' || kind === 'explicit' || kind === 'discovered'
}

export function buildEnvironmentPresentation(
  report: EnvironmentReport | null,
  loading = false,
  runtimeKind: RuntimeSourceKind | null = null,
  platform: string | null = null,
): EnvironmentPresentation {
  if (loading || !report) {
    return {
      state: 'checking',
      title: '正在检查运行环境',
      description: '按 Node.js → npm → OpenCodex 顺序检查；未通过时不继续发现安装。',
      tag: '检查中',
      statusClass: 'busy',
      hidden: false,
      blocked: false,
      checks: CHECK_NAMES.map(name => ({ name, value: '检查中', state: 'muted', label: '检查中' })),
      commands: [],
      contractNote: false,
    }
  }

  const base: Pick<EnvironmentPresentation, 'tag' | 'statusClass' | 'hidden' | 'blocked' | 'contractNote' | 'installEmptyMessage'> = {
    tag: report.gate === 'ready' ? '环境通过' : '环境未通过',
    statusClass: report.gate === 'ready' ? 'ok' : '',
    hidden: report.gate === 'ready',
    blocked: report.gate !== 'ready',
    contractNote: false,
    installEmptyMessage:
      report.gate === 'ready'
        ? undefined
        : '已按前置条件暂停 OpenCodex 发现；请先处理概览中的环境门禁。',
  }

  if (report.gate === 'missing_node') {
    return {
      ...base,
      state: report.brewFound ? 'missing_node_brew' : 'missing_node_nobrew',
      title: '缺少 Node.js，OpenCodex 发现已暂停',
      description: platform === 'Windows'
        ? '请安装 Windows 版 Node.js LTS（包含 npm），或使用受支持的 NVM 安装目录，完成后重新检查。'
        : report.brewFound
        ? '检测到 Homebrew。可以先安装 Node.js LTS，安装完成后再回来重新检查。'
        : '未检测到 Homebrew。请从 Node.js LTS 官方渠道安装；应用不会代装，也不会写入系统环境。',
      checks: [
        checkToPresentation('Node.js', report.node),
        notChecked('npm'),
        notChecked('OpenCodex'),
      ],
      commands: [
        ...(report.brewFound && platform !== 'Windows' ? [{ label: '安装 Node.js', command: 'brew install node' }] : []),
        { label: '安装后校验', command: platform === 'Windows' ? 'node -v; npm -v' : 'node -v && npm -v' },
      ],
      link: report.brewFound
        ? undefined
        : { text: 'Node.js LTS 官方安装渠道', note: '仅展示渠道；真实实现提供打开官网入口。' },
    }
  }

  if (report.gate === 'missing_npm') {
    return {
      ...base,
      state: 'missing_npm',
      title: 'Node.js 已发现，但 npm 不可用',
      description: 'npm 通常随 Node.js 提供。建议重新安装 Node.js LTS，或检查 PATH 与 Node 安装完整性。',
      checks: [checkToPresentation('Node.js', report.node), checkToPresentation('npm', report.npm), notChecked('OpenCodex')],
      commands: [{ label: '安装后校验', command: platform === 'Windows' ? 'node -v; npm -v' : 'node -v && npm -v' }],
    }
  }

  if (report.gate === 'missing_ocx') {
    if (runtimeProvidesOcx(runtimeKind)) {
      return {
        ...base,
        tag: '环境通过',
        statusClass: 'ok',
        hidden: false,
        blocked: false,
        installEmptyMessage: undefined,
        state: 'ready_via_source',
        title: '运行环境已就绪，OpenCodex 由运行来源提供',
        description:
          '本机未发现 npm 全局安装；当前 OpenCodex 由「运行来源」提供（托管安装 / 指定路径 / 自动发现），无需再安装。需要更换或移除时到设置 → 安装配置。',
        checks: [
          checkToPresentation('Node.js', report.node),
          checkToPresentation('npm', report.npm),
          { name: 'OpenCodex', value: '运行来源', state: 'running', label: '通过' },
        ],
        commands: [],
        contractNote: false,
      }
    }
    return {
      ...base,
      state: 'missing_ocx',
      title: '运行环境已就绪，未发现 OpenCodex',
      // UI规范 §18.5：门禁动作升级为「安装 / 导入离线包」，并去掉 PATH 假设
      // （不再写「安装后校验 `ocx --version`」）。官方 npm 命令保留为终端备选。
      description: '可以把官方包装进数据根内的私有前缀（联网安装，或导入官方离线包）；安装完成后由应用重新解析运行来源。',
      checks: [checkToPresentation('Node.js', report.node), checkToPresentation('npm', report.npm), checkToPresentation('OpenCodex', report.ocx)],
      commands: [
        { label: '官方 npm 命令（终端备选）', command: 'npm install -g @bitkyc08/opencodex' },
      ],
      contractNote: false,
    }
  }

  return {
    ...base,
    state: 'ready',
    title: '运行环境检查通过',
    description: '前置条件全部通过，现在继续发现本机 OpenCodex 安装。',
    checks: [checkToPresentation('Node.js', report.node), checkToPresentation('npm', report.npm), checkToPresentation('OpenCodex', report.ocx)],
    commands: [],
  }
}
