import { ref, computed, watch } from 'vue'
import type { AppConfig, ConfigValue, ConfigHistory } from './types'
import * as api from './api'

const config = ref<AppConfig | null>(null)
const isLoading = ref(false)
const error = ref<string | null>(null)

export function useConfigStore() {
  const appVersion = computed(() => config.value?.version || '1.0.0')
  const appName = computed(() => config.value?.app_name || 'iNovel')
  const apiBaseUrl = computed(() => config.value?.api_base_url || '')
  const environment = computed(() => config.value?.environment || 'Development')
  const categories = computed(() => config.value?.categories || [])

  async function loadConfig() {
    isLoading.value = true
    error.value = null
    try {
      config.value = await api.getConfig()
    } catch (e) {
      error.value = '加载配置失败: ' + (e as Error).message
    } finally {
      isLoading.value = false
    }
  }

  async function getValue(key: string): Promise<ConfigValue | undefined> {
    try {
      return await api.getConfigValue(key)
    } catch (e) {
      error.value = '获取配置项失败: ' + (e as Error).message
      return undefined
    }
  }

  async function setValue(key: string, value: string, encrypted: boolean = false): Promise<boolean> {
    try {
      const result = await api.setConfigValue(key, value, encrypted)
      if (result && config.value) {
        if (config.value.items[key]) {
          config.value.items[key].value = value
          config.value.items[key].encrypted = encrypted
        }
        config.value.last_updated = new Date().toISOString()
      }
      return result
    } catch (e) {
      error.value = '设置配置项失败: ' + (e as Error).message
      return false
    }
  }

  async function updateVersion(newVersion: string): Promise<boolean> {
    try {
      const result = await api.updateAppVersion(newVersion)
      if (result && config.value) {
        config.value.version = newVersion
        config.value.last_updated = new Date().toISOString()
      }
      return result
    } catch (e) {
      error.value = '更新版本失败: ' + (e as Error).message
      return false
    }
  }

  async function reload(): Promise<boolean> {
    try {
      const result = await api.reloadConfig()
      if (result) {
        await loadConfig()
      }
      return result
    } catch (e) {
      error.value = '重新加载配置失败: ' + (e as Error).message
      return false
    }
  }

  async function exportTo(path: string): Promise<boolean> {
    try {
      const result = await api.exportConfig(path)
      if (!result.success) {
        error.value = result.message
      }
      return result.success
    } catch (e) {
      error.value = '导出配置失败: ' + (e as Error).message
      return false
    }
  }

  async function importFrom(path: string): Promise<boolean> {
    try {
      const result = await api.importConfig(path)
      if (result.success && result.config) {
        config.value = result.config
      } else {
        error.value = result.message
      }
      return result.success
    } catch (e) {
      error.value = '导入配置失败: ' + (e as Error).message
      return false
    }
  }

  async function reset(): Promise<boolean> {
    try {
      const result = await api.resetConfig()
      if (result.success && result.config) {
        config.value = result.config
      } else {
        error.value = result.message
      }
      return result.success
    } catch (e) {
      error.value = '重置配置失败: ' + (e as Error).message
      return false
    }
  }

  async function getHistory(page: number = 0, pageSize: number = 10): Promise<ConfigHistory[]> {
    try {
      const result = await api.getConfigHistory(page, pageSize)
      if (!result.success) {
        error.value = result.message
      }
      return result.history
    } catch (e) {
      error.value = '获取历史记录失败: ' + (e as Error).message
      return []
    }
  }

  async function rollback(historyId: string): Promise<boolean> {
    try {
      const result = await api.rollbackConfig(historyId)
      if (result.success && result.config) {
        config.value = result.config
      } else {
        error.value = result.message
      }
      return result.success
    } catch (e) {
      error.value = '回滚失败: ' + (e as Error).message
      return false
    }
  }

  return {
    config,
    isLoading,
    error,
    appVersion,
    appName,
    apiBaseUrl,
    environment,
    categories,
    loadConfig,
    getValue,
    setValue,
    updateVersion,
    reload,
    exportTo,
    importFrom,
    reset,
    getHistory,
    rollback,
  }
}

watch(config, (newConfig) => {
  if (newConfig) {
    console.log('Config updated:', newConfig.version)
  }
})