// 偏好功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片六）。
// 缩放应用（UI 唯一落点 `--ui-zoom`）随偏好读写一起放在这里，保持「显示值＝实际比例＝持久化值」。
import { defineStore } from 'pinia'
import { applyInterfaceScale, clampScale } from '@/app/appearance/scale'
import { clampVisualEffects, useEffectsStore } from '@/app/appearance/effects'
import { clampGlowRender, useGlowRenderStore } from '@/app/appearance/glowRender'
import { hasStoredThemeCache, readThemeCache, useThemeStore } from '@/app/appearance/theme'
import {
  getPreferences,
  restoreDefaultPreferences,
  savePreferences as savePreferencesCommand,
  type PreferencesDto,
} from './api'

export const usePreferencesStore = defineStore('preferences', {
  state: () => ({
    data: null as PreferencesDto | null,
    loading: false,
    error: false,
  }),
  actions: {
    async load() {
      if (this.loading) return
      this.loading = true
      try {
        this.data = await getPreferences()
        // 启动与重新加载偏好时立即把缩放应用到界面，不依赖用户打开设置页。
        applyInterfaceScale(this.data.interfaceScale)
        // 档位同理：读完偏好立即落到 `data-effects`，由外观策略折算系统减少动态。
        useEffectsStore().setSetting(this.data.visualEffects)
        useGlowRenderStore().setSetting(this.data.glowRender)
        // 主题以后端偏好为事实源：仅在偏好有值且与当前投影不同时写缓存，避免旧缓存反向覆盖。
        useThemeStore().apply(this.data.theme)
        // 一次性历史主题导入（H-15）：仅当旧偏好没有 theme 且本机确实缓存过历史选择时提交一次。
        if (this.data.themeNeedsImport && hasStoredThemeCache()) {
          const imported = readThemeCache()
          await this.save({ ...this.data, theme: imported })
        }
        this.error = false
      } catch {
        // IPC 失败时保留上次偏好；不臆造或重置用户配置。
        this.error = true
      } finally {
        this.loading = false
      }
    },
    async save(preferences: PreferencesDto) {
      this.error = false
      try {
        this.data = await savePreferencesCommand(preferences)
        // 以磁盘回读值为准应用缩放：显示值、实际比例与持久化值保持一致。
        applyInterfaceScale(this.data.interfaceScale)
        useEffectsStore().setSetting(this.data.visualEffects)
        useGlowRenderStore().setSetting(this.data.glowRender)
        return true
      } catch {
        // 保存失败保留当前 UI 值，下一次真实契约加载时再对齐。
        this.error = true
        return false
      }
    },
    // 面板 hub 与设置共用同一「界面缩放」偏好：先即时预览，再落盘并回读。
    async setScale(value: unknown) {
      const scale = clampScale(value)
      applyInterfaceScale(scale)
      const base = this.data
      if (!base) return false
      if (base.interfaceScale === scale) return true
      return await this.save({ ...base, interfaceScale: scale })
    },
    // 三档特效：即时预览 + 落盘回读；失败时回退到持久化值（`save` 成功才回读覆盖）。
    async setVisualEffects(value: unknown) {
      const setting = clampVisualEffects(value)
      useEffectsStore().setSetting(setting)
      const base = this.data
      if (!base) return false
      if (base.visualEffects === setting) return true
      const ok = await this.save({ ...base, visualEffects: setting })
      if (!ok) {
        // 保存失败：回退界面档位到持久化值，避免显示未持久化的选择。
        useEffectsStore().setSetting(base.visualEffects)
      }
      return ok
    },
    // 背景光渲染方式：即时预览 + 落盘回读；失败时回退到持久化值。
    async setGlowRender(value: unknown) {
      const setting = clampGlowRender(value)
      const glow = useGlowRenderStore()
      glow.setSetting(setting)
      const base = this.data
      if (!base) return false
      if (base.glowRender === setting) return true
      const ok = await this.save({ ...base, glowRender: setting })
      if (!ok) glow.setSetting(base.glowRender)
      return ok
    },
    async restore() {
      this.error = false
      try {
        this.data = await restoreDefaultPreferences()
        applyInterfaceScale(this.data.interfaceScale)
        useEffectsStore().setSetting(this.data.visualEffects)
        useGlowRenderStore().setSetting(this.data.glowRender)
        useThemeStore().apply(this.data.theme)
        return true
      } catch {
        this.error = true
        return false
      }
    },
  },
})
