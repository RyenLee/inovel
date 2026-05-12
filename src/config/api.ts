import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, ConfigValue, ConfigItem, ConfigHistory, TomlConfig } from './types'

const STORAGE_KEY = 'inovel_dev_config'

function loadDevConfig(): AppConfig {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      return JSON.parse(stored)
    }
  } catch (e) {
    console.warn('Failed to load dev config from localStorage:', e)
  }
  return getDefaultConfig()
}

function saveDevConfig(config: AppConfig): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config))
  } catch (e) {
    console.warn('Failed to save dev config to localStorage:', e)
  }
}

function getDefaultConfig(): AppConfig {
  return {
    version: '1.1.0',
    app_name: 'iNovel',
    api_base_url: 'http://localhost:8080',
    environment: 'development',
    categories: [
      {
        name: 'app',
        label: '应用配置',
        description: '应用基础配置',
        items: ['app_name', 'version', 'environment', 'app_description']
      },
      {
        name: 'api',
        label: 'API配置',
        description: '接口相关配置',
        items: ['api_base_url', 'api_timeout', 'api_max_retries']
      },
      {
        name: 'security',
        label: '安全配置',
        description: '敏感配置项',
        items: ['api_key', 'secret_token']
      },
      {
        name: 'feature',
        label: '功能开关',
        description: '功能特性配置',
        items: ['auto_save_enabled', 'sync_enabled', 'writing_stats_enabled', 'inspiration_board_enabled']
      },
      {
        name: 'editor',
        label: '编辑器配置',
        description: '编辑器相关设置',
        items: ['editor_font_size', 'editor_font', 'editor_line_spacing', 'editor_show_line_numbers', 'editor_spell_check']
      },
      {
        name: 'window',
        label: '窗口配置',
        description: '窗口尺寸设置',
        items: ['window_default_width', 'window_default_height', 'window_min_width', 'window_min_height', 'window_max_width', 'window_max_height', 'window_resizable']
      },
      {
        name: 'performance',
        label: '性能配置',
        description: '性能监控设置',
        items: ['perf_monitoring_enabled', 'perf_slow_threshold', 'perf_log_payload']
      },
      {
        name: 'cache',
        label: '缓存配置',
        description: '缓存相关设置',
        items: ['cache_enabled', 'cache_max_entries', 'cache_ttl']
      }
    ],
    items: {
      'app_name': { value: 'iNovel', encrypted: false, description: '应用名称' },
      'version': { value: '1.1.0', encrypted: false, description: '应用版本号' },
      'environment': { value: 'development', encrypted: false, description: '运行环境' },
      'app_description': { value: '一款现代化的小说创作工具', encrypted: false, description: '应用描述' },
      'api_base_url': { value: 'http://localhost:8080', encrypted: false, description: 'API基础地址' },
      'api_timeout': { value: '30000', encrypted: false, description: 'API超时时间(毫秒)' },
      'api_max_retries': { value: '3', encrypted: false, description: '最大重试次数' },
      'api_key': { value: '', encrypted: true, description: 'API密钥' },
      'secret_token': { value: '', encrypted: true, description: '安全令牌' },
      'auto_save_enabled': { value: 'true', encrypted: false, description: '自动保存开关' },
      'sync_enabled': { value: 'false', encrypted: false, description: '云同步开关' },
      'writing_stats_enabled': { value: 'true', encrypted: false, description: '写作统计开关' },
      'inspiration_board_enabled': { value: 'true', encrypted: false, description: '灵感面板开关' },
      'editor_font_size': { value: '16', encrypted: false, description: '默认字体大小' },
      'editor_font': { value: '微软雅黑', encrypted: false, description: '默认字体' },
      'editor_line_spacing': { value: '1.5', encrypted: false, description: '行间距' },
      'editor_show_line_numbers': { value: 'true', encrypted: false, description: '显示行号' },
      'editor_spell_check': { value: 'true', encrypted: false, description: '拼写检查' },
      'window_default_width': { value: '1200', encrypted: false, description: '默认窗口宽度' },
      'window_default_height': { value: '800', encrypted: false, description: '默认窗口高度' },
      'window_min_width': { value: '600', encrypted: false, description: '最小窗口宽度' },
      'window_min_height': { value: '800', encrypted: false, description: '最小窗口高度' },
      'window_max_width': { value: '1920', encrypted: false, description: '最大窗口宽度' },
      'window_max_height': { value: '1200', encrypted: false, description: '最大窗口高度' },
      'window_resizable': { value: 'false', encrypted: false, description: '允许调整大小' },
      'perf_monitoring_enabled': { value: 'true', encrypted: false, description: '性能监控' },
      'perf_slow_threshold': { value: '1000', encrypted: false, description: '慢请求阈值(毫秒)' },
      'perf_log_payload': { value: 'true', encrypted: false, description: '记录请求大小' },
      'cache_enabled': { value: 'true', encrypted: false, description: '启用缓存' },
      'cache_max_entries': { value: '1000', encrypted: false, description: '最大缓存条目' },
      'cache_ttl': { value: '300', encrypted: false, description: '缓存过期时间(秒)' }
    },
    last_updated: new Date().toISOString()
  }
}

let devConfig: AppConfig = loadDevConfig()

function isTauriAvailable(): boolean {
  if (typeof window !== 'undefined') {
    return !!(window as unknown as { isTauri?: boolean }).isTauri
  }
  return false
}

async function callTauriCommand<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!isTauriAvailable()) {
    console.debug(`[Dev Mode] Executing command: ${command}`)

    switch (command) {
      case 'get_config':
        return { ...devConfig } as unknown as T

      case 'get_config_value': {
        const key = (args as { key: string }).key
        return { ...(devConfig.items[key] || { value: '', encrypted: false, description: '' }) } as unknown as T
      }

      case 'get_config_by_category': {
        const category = (args as { category: string }).category
        const cat = devConfig.categories.find(c => c.name === category)
        if (!cat) return [] as unknown as T
        return cat.items.map(key => ({
          key,
          value: { ...(devConfig.items[key] || { value: '', encrypted: false, description: '' }) },
          category,
          updated_at: devConfig.last_updated
        })) as unknown as T
      }

      case 'set_config_value': {
        const key = (args as { key: string }).key
        const value = (args as { value: string }).value
        const encrypted = (args as { encrypted: boolean }).encrypted

        if (devConfig.items[key]) {
          devConfig.items[key].value = value
          devConfig.items[key].encrypted = encrypted
          devConfig.last_updated = new Date().toISOString()
          saveDevConfig(devConfig)
          return true as unknown as T
        }
        return false as unknown as T
      }

      case 'set_config_values': {
        const values = args as Record<string, string>
        const updated_items: string[] = []

        for (const [key, value] of Object.entries(values)) {
          if (devConfig.items[key]) {
            devConfig.items[key].value = value
            updated_items.push(key)
          }
        }

        if (updated_items.length > 0) {
          devConfig.last_updated = new Date().toISOString()
          saveDevConfig(devConfig)
        }

        return {
          success: true,
          message: '配置更新成功',
          updated_items
        } as unknown as T
      }

      case 'update_app_version': {
        const newVersion = (args as { newVersion: string }).newVersion
        devConfig.version = newVersion
        if (devConfig.items['version']) {
          devConfig.items['version'].value = newVersion
        }
        devConfig.last_updated = new Date().toISOString()
        saveDevConfig(devConfig)
        return true as unknown as T
      }

      case 'reload_config': {
        devConfig = loadDevConfig()
        return true as unknown as T
      }

      case 'export_config': {
        const path = (args as { path: string }).path
        const blob = new Blob([JSON.stringify(devConfig, null, 2)], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = path
        document.body.appendChild(a)
        a.click()
        document.body.removeChild(a)
        URL.revokeObjectURL(url)
        return { success: true, path, message: '配置导出成功' } as unknown as T
      }

      case 'import_config': {
        return { success: true, config: { ...devConfig }, message: '配置导入成功' } as unknown as T
      }

      case 'reset_config': {
        devConfig = getDefaultConfig()
        saveDevConfig(devConfig)
        return { success: true, config: { ...devConfig }, message: '配置已重置为默认值' } as unknown as T
      }

      case 'get_config_history': {
        return { success: true, history: [] as ConfigHistory[], total: 0, message: '查询成功' } as unknown as T
      }

      case 'rollback_config': {
        return { success: true, config: { ...devConfig }, message: '回滚成功' } as unknown as T
      }

      case 'read_toml_config': {
        console.log('[Dev Mode] Reading TOML config from localStorage...')
        const fallbackConfig: TomlConfig = {
          app: {
            name: devConfig.items['app_name']?.value || 'iNovel',
            version: devConfig.version,
            environment: devConfig.environment,
            description: devConfig.items['app_description']?.value || '一款现代化的小说创作工具'
          },
          gzip: {
            enabled: true,
            level: 6,
            min_size: 1024,
            compress_types: ['application/json', 'text/plain', 'text/html', 'text/css', 'text/javascript', 'application/javascript', 'application/xml', 'text/xml']
          },
          cache: {
            enabled: devConfig.items['cache_enabled']?.value === 'true',
            max_entries: parseInt(devConfig.items['cache_max_entries']?.value || '1000'),
            ttl_seconds: parseInt(devConfig.items['cache_ttl']?.value || '300'),
            cached_commands: ['get_chapter_tree', 'list_characters', 'list_locations', 'list_organizations', 'get_relationships', 'list_events', 'get_builtin_templates', 'get_inspiration_board', 'get_writing_stats', 'get_recent_projects']
          },
          pagination: {
            default_page_size: 20,
            max_page_size: 100
          },
          request_merging: {
            enabled: true,
            window_ms: 300,
            max_batch_size: 50
          },
          api: {
            base_url: devConfig.items['api_base_url']?.value || 'http://localhost:8080',
            timeout_ms: parseInt(devConfig.items['api_timeout']?.value || '30000'),
            max_retries: parseInt(devConfig.items['api_max_retries']?.value || '3')
          },
          performance: {
            monitoring_enabled: devConfig.items['perf_monitoring_enabled']?.value === 'true',
            slow_request_threshold_ms: parseInt(devConfig.items['perf_slow_threshold']?.value || '1000'),
            log_payload_size: devConfig.items['perf_log_payload']?.value === 'true'
          },
          window: {
            default_width: parseInt(devConfig.items['window_default_width']?.value || '1200'),
            default_height: parseInt(devConfig.items['window_default_height']?.value || '800'),
            min_width: parseInt(devConfig.items['window_min_width']?.value || '600'),
            min_height: parseInt(devConfig.items['window_min_height']?.value || '800'),
            resizable: devConfig.items['window_resizable']?.value === 'true',
            max_width: parseInt(devConfig.items['window_max_width']?.value || '1920'),
            max_height: parseInt(devConfig.items['window_max_height']?.value || '1200'),
            portrait: {
              enabled: true,
              default_width: 800,
              default_height: 1200,
              min_width: 600,
              min_height: 800
            }
          },
          security: {
            api_key: '',
            secret_token: ''
          },
          features: {
            auto_save_enabled: devConfig.items['auto_save_enabled']?.value === 'true',
            sync_enabled: devConfig.items['sync_enabled']?.value === 'true',
            writing_stats_enabled: devConfig.items['writing_stats_enabled']?.value === 'true',
            inspiration_board_enabled: devConfig.items['inspiration_board_enabled']?.value === 'true'
          },
          editor: {
            default_font_size: parseInt(devConfig.items['editor_font_size']?.value || '16'),
            default_font: devConfig.items['editor_font']?.value || '微软雅黑',
            line_spacing: parseFloat(devConfig.items['editor_line_spacing']?.value || '1.5'),
            show_line_numbers: devConfig.items['editor_show_line_numbers']?.value === 'true',
            spell_check_enabled: devConfig.items['editor_spell_check']?.value === 'true'
          }
        }
        console.log('[Dev Mode] TOML config built from localStorage:', fallbackConfig)
        return fallbackConfig as unknown as T
      }

      case 'write_toml_config': {
        const configToWrite = args as unknown as TomlConfig
        console.log('[Dev Mode] Writing TOML config:', configToWrite)

        try {
          const response = await fetch('/api/config', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json'
            },
            body: JSON.stringify(configToWrite)
          })

          if (response.ok) {
            const result = await response.json()
            console.log('[Dev Mode] Successfully wrote config to backend:', result)
            return result as unknown as T
          } else {
            console.warn('[Dev Mode] Failed to write config to backend, updating local storage')
          }
        } catch (e) {
          console.warn('[Dev Mode] Failed to write config to backend, updating local storage:', e)
        }

        devConfig.items['app_name'].value = configToWrite.app.name
        devConfig.items['app_description'].value = configToWrite.app.description
        devConfig.version = configToWrite.app.version
        devConfig.environment = configToWrite.app.environment

        devConfig.items['api_base_url'].value = configToWrite.api.base_url
        devConfig.items['api_timeout'].value = configToWrite.api.timeout_ms.toString()
        devConfig.items['api_max_retries'].value = configToWrite.api.max_retries.toString()

        devConfig.items['auto_save_enabled'].value = configToWrite.features.auto_save_enabled.toString()
        devConfig.items['sync_enabled'].value = configToWrite.features.sync_enabled.toString()
        devConfig.items['writing_stats_enabled'].value = configToWrite.features.writing_stats_enabled.toString()
        devConfig.items['inspiration_board_enabled'].value = configToWrite.features.inspiration_board_enabled.toString()

        devConfig.items['editor_font_size'].value = configToWrite.editor.default_font_size.toString()
        devConfig.items['editor_font'].value = configToWrite.editor.default_font
        devConfig.items['editor_line_spacing'].value = configToWrite.editor.line_spacing.toString()
        devConfig.items['editor_show_line_numbers'].value = configToWrite.editor.show_line_numbers.toString()
        devConfig.items['editor_spell_check'].value = configToWrite.editor.spell_check_enabled.toString()

        devConfig.items['window_default_width'].value = configToWrite.window.default_width.toString()
        devConfig.items['window_default_height'].value = configToWrite.window.default_height.toString()
        devConfig.items['window_min_width'].value = configToWrite.window.min_width.toString()
        devConfig.items['window_min_height'].value = configToWrite.window.min_height.toString()
        devConfig.items['window_max_width'].value = configToWrite.window.max_width.toString()
        devConfig.items['window_max_height'].value = configToWrite.window.max_height.toString()
        devConfig.items['window_resizable'].value = configToWrite.window.resizable.toString()

        devConfig.items['perf_monitoring_enabled'].value = configToWrite.performance.monitoring_enabled.toString()
        devConfig.items['perf_slow_threshold'].value = configToWrite.performance.slow_request_threshold_ms.toString()
        devConfig.items['perf_log_payload'].value = configToWrite.performance.log_payload_size.toString()

        devConfig.items['cache_enabled'].value = configToWrite.cache.enabled.toString()
        devConfig.items['cache_max_entries'].value = configToWrite.cache.max_entries.toString()
        devConfig.items['cache_ttl'].value = configToWrite.cache.ttl_seconds.toString()

        devConfig.last_updated = new Date().toISOString()
        saveDevConfig(devConfig)

        return { success: true, message: 'TOML配置写入成功' } as unknown as T
      }

      default:
        throw new Error(`Unknown command: ${command}`)
    }
  }

  return invoke(command, args) as Promise<T>
}

export async function getConfig(): Promise<AppConfig> {
  try {
    const result = await callTauriCommand<AppConfig>('get_config')
    console.log('get_config result:', result)
    return result
  } catch (e) {
    console.error('get_config error:', e)
    throw e
  }
}

export async function getConfigValue(key: string): Promise<ConfigValue | undefined> {
  return await callTauriCommand<ConfigValue | undefined>('get_config_value', { key })
}

export async function getConfigByCategory(category: string): Promise<ConfigItem[]> {
  return await callTauriCommand<ConfigItem[]>('get_config_by_category', { category })
}

export async function setConfigValue(key: string, value: string, encrypted: boolean = false): Promise<boolean> {
  return await callTauriCommand<boolean>('set_config_value', { key, value, encrypted })
}

export async function setConfigValues(values: Record<string, string>): Promise<{ success: boolean; message: string; updated_items: string[] }> {
  return await callTauriCommand<{ success: boolean; message: string; updated_items: string[] }>('set_config_values', { values })
}

export async function updateAppVersion(newVersion: string): Promise<boolean> {
  return await callTauriCommand<boolean>('update_app_version', { newVersion })
}

export async function reloadConfig(): Promise<boolean> {
  return await callTauriCommand<boolean>('reload_config')
}

export async function exportConfig(path: string): Promise<{ success: boolean; path: string; message: string }> {
  return await callTauriCommand<{ success: boolean; path: string; message: string }>('export_config', { path })
}

export async function importConfig(path: string): Promise<{ success: boolean; config?: AppConfig; message: string }> {
  return await callTauriCommand<{ success: boolean; config?: AppConfig; message: string }>('import_config', { path })
}

export async function resetConfig(): Promise<{ success: boolean; config?: AppConfig; message: string }> {
  return await callTauriCommand<{ success: boolean; config?: AppConfig; message: string }>('reset_config')
}

export async function getConfigHistory(page: number, pageSize: number): Promise<{ success: boolean; history: ConfigHistory[]; total: number; message: string }> {
  return await callTauriCommand<{ success: boolean; history: ConfigHistory[]; total: number; message: string }>('get_config_history', { page, pageSize })
}

export async function rollbackConfig(historyId: string): Promise<{ success: boolean; config?: AppConfig; message: string }> {
  return await callTauriCommand<{ success: boolean; config?: AppConfig; message: string }>('rollback_config', { historyId })
}

export async function readTomlConfig(): Promise<TomlConfig> {
  return await callTauriCommand<TomlConfig>('read_toml_config')
}

export async function writeTomlConfig(config: TomlConfig): Promise<{ success: boolean; message: string }> {
  return await callTauriCommand<{ success: boolean; message: string }>('write_toml_config', config as unknown as Record<string, unknown>)
}