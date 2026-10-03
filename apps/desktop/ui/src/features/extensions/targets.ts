// 扩展目标客户端清单（原 data/mock 的扩展部分）。
export const skillTargets = ['claude', 'codex', 'gemini', 'grok', 'opencode', 'hermes'] as const
export type SkillTarget = (typeof skillTargets)[number]
