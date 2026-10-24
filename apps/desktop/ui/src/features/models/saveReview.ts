import type {ChannelPolicy, Field, ModelPolicy} from './api'

/** 启用判定与执行目录口径一致：渠道启用 + 型号在公开范围内。 */
export function isModelActive(channel: ChannelPolicy, modelId: string) {
  if (channel.selection.mode === 'none') return false
  if (channel.selection.mode === 'all') return channel.discovered.some(model => model.id === modelId)
  return channel.selection.mode === 'whitelist' && channel.selection.models.includes(modelId)
}

/** 启停直接写当前草稿：全量范围改为明确白名单，避免“停一个”变成“全隐藏”。 */
export function setModelActive(channel: ChannelPolicy, ids: string[], active: boolean) {
  const current = channel.selection.mode === 'all'
    ? channel.discovered.map(model => model.id)
    : channel.selection.mode === 'whitelist' ? [...channel.selection.models] : []
  const next = active ? [...new Set([...current, ...ids])] : current.filter(id => !ids.includes(id))
  channel.selection = next.length ? {mode: 'whitelist', models: next} : {mode: 'none'}
}

export interface InlineChange {label: string;before: unknown;after: unknown}

function inherit(value: Field<unknown> | undefined) {
  return !value || value.mode === 'inherit' ? '继承' : value.value
}

/**
 * 标签表达不了的字段变化（名称、协议、节点、认证、模版引用等）。
 * 启停、白名单、顺序与发现清单已由渠道卡和型号标签直接表达，不在此重复（§38.3）。
 */
export function inlineChanges(saved: ModelPolicy, draft: ModelPolicy): InlineChange[] {
  const changes: InlineChange[] = []
  const rows = (label: string, before: unknown, after: unknown) => {
    if (JSON.stringify(before) !== JSON.stringify(after)) changes.push({label, before: before ?? null, after: after ?? null})
  }
  rows('全局族精确采用', saved.templateRefs, draft.templateRefs)
  rows('渠道顺序', saved.channels.map(channel => channel.id), draft.channels.map(channel => channel.id))
  for (const id of new Set([...saved.channels, ...draft.channels].map(channel => channel.id))) {
    const before = saved.channels.find(channel => channel.id === id)
    const after = draft.channels.find(channel => channel.id === id)
    const name = after?.label || before?.label || id
    if (!before || !after) {
      changes.push({label: name + ' · 渠道', before: before ? '存在' : '不存在', after: after ? '存在' : '不存在'})
      continue
    }
    rows(name + ' · 名称', before.label, after.label)
    rows(name + ' · 协议', before.connection.protocol, after.connection.protocol)
    rows(name + ' · 运行渠道标识', before.connection.providerId, after.connection.providerId)
    rows(name + ' · 服务节点', before.connection.nodes.map(node => [node.name, node.url]), after.connection.nodes.map(node => [node.name, node.url]))
    rows(name + ' · 当前节点', before.connection.nodes.find(node => node.id === before.connection.activeNodeId)?.name ?? '未选择', after.connection.nodes.find(node => node.id === after.connection.activeNodeId)?.name ?? '未选择')
    if (before.connection.credentialRef !== after.connection.credentialRef) changes.push({label: name + ' · 认证', before: '原本机认证', after: '拟更换本机认证'})
    rows(name + ' · 渠道默认', [inherit(before.defaults.contextWindow), inherit(before.defaults.inputModalities), inherit(before.defaults.reasoningEfforts), inherit(before.defaults.defaultEffort), inherit(before.defaults.fast)], [inherit(after.defaults.contextWindow), inherit(after.defaults.inputModalities), inherit(after.defaults.reasoningEfforts), inherit(after.defaults.defaultEffort), inherit(after.defaults.fast)])
    rows(name + ' · 渠道族采用', before.templateRefs, after.templateRefs)
    rows(name + ' · 族覆盖', Object.keys(before.familyOverrides).length, Object.keys(after.familyOverrides).length)
    rows(name + ' · 单模型覆盖', Object.keys(before.overrides).length, Object.keys(after.overrides).length)
    rows(name + ' · 别名', before.naming ?? {}, after.naming ?? {})
  }
  const removals = (policy: ModelPolicy) => (policy.pendingRemovals ?? []).map(item => item.channelId)
  rows('待移除的已读取渠道', removals(saved), removals(draft))
  rows('恢复来源', saved.recoveryId ?? null, draft.recoveryId ?? null)
  return changes
}

