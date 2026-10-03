import type { ExtensionsDto } from '@/features/extensions/api'

export type ExtensionViewData = {
  skills: ExtensionsDto['skills']
  servers: ExtensionsDto['servers']
} | null

export function projectExtensions(payload: ExtensionsDto | null): ExtensionViewData {
  if (!payload) return null
  return {
    skills: payload.skills,
    servers: payload.servers,
  }
}
