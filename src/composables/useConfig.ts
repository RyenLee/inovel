import { ref, watch, onMounted, onUnmounted } from 'vue'
import { readTomlConfig, writeTomlConfig } from '../config/api'
import type { TomlConfig } from '../config/types'

const config = ref<TomlConfig | null>(null)
const isLoading = ref(false)
const error = ref<string | null>(null)

let updateInterval: ReturnType<typeof setInterval> | null = null

export function useConfig() {
  const appName = ref('iNovel')
  const appVersion = ref('1.1.0')
  const environment = ref('development')
  const appDescription = ref('一款现代化的小说创作工具')

  async function loadConfig() {
    isLoading.value = true
    error.value = null
    try {
      config.value = await readTomlConfig()
      updateDerivedValues()
    } catch (e) {
      error.value = '加载配置失败: ' + (e as Error).message
      console.error('Config load error:', e)
    } finally {
      isLoading.value = false
    }
  }

  function updateDerivedValues() {
    if (config.value) {
      appName.value = config.value.app.name
      appVersion.value = config.value.app.version
      environment.value = config.value.app.environment
      appDescription.value = config.value.app.description
    }
  }

  async function saveConfig(newConfig: TomlConfig): Promise<boolean> {
    isLoading.value = true
    try {
      const result = await writeTomlConfig(newConfig)
      if (result.success) {
        config.value = newConfig
        updateDerivedValues()
        return true
      }
      error.value = result.message
      return false
    } catch (e) {
      error.value = '保存配置失败: ' + (e as Error).message
      return false
    } finally {
      isLoading.value = false
    }
  }

  async function updateAppInfo(info: Partial<TomlConfig['app']>): Promise<boolean> {
    if (!config.value) {
      await loadConfig()
    }
    
    if (config.value) {
      config.value.app = { ...config.value.app, ...info }
      return saveConfig(config.value)
    }
    return false
  }

  function startWatching() {
    updateInterval = setInterval(async () => {
      await loadConfig()
    }, 5000)
  }

  function stopWatching() {
    if (updateInterval) {
      clearInterval(updateInterval)
      updateInterval = null
    }
  }

  watch(config, () => {
    updateDerivedValues()
  }, { deep: true })

  return {
    config,
    isLoading,
    error,
    appName,
    appVersion,
    environment,
    appDescription,
    loadConfig,
    saveConfig,
    updateAppInfo,
    startWatching,
    stopWatching,
  }
}

export function useAppInfo() {
  const { appName, appVersion, environment, appDescription, loadConfig } = useConfig()

  onMounted(() => {
    loadConfig()
  })

  return {
    appName,
    appVersion,
    environment,
    appDescription,
  }
}
