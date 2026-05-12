export interface ConfigValue {
  value: string
  encrypted: boolean
  description?: string
}

export interface ConfigItem {
  key: string
  value: ConfigValue
  category: string
  updated_at: string
}

export interface ConfigCategory {
  name: string
  label: string
  description?: string
  items: string[]
}

export interface ConfigSnapshot {
  version: string
  items: ConfigItem[]
  created_at: string
  description?: string
}

export interface ConfigHistory {
  id: string
  snapshot: ConfigSnapshot
  action: 'Created' | 'Updated' | 'RolledBack' | 'Exported'
  operator?: string
}

export interface AppConfig {
  version: string
  app_name: string
  api_base_url: string
  environment: string
  categories: ConfigCategory[]
  items: Record<string, ConfigValue>
  last_updated: string
}

export interface TomlConfig {
  app: AppSection
  gzip: GzipSection
  cache: CacheSection
  pagination: PaginationSection
  request_merging: RequestMergingSection
  api: ApiSection
  performance: PerformanceSection
  window: WindowSection
  security: SecuritySection
  features: FeaturesSection
  editor: EditorSection
}

export interface AppSection {
  name: string
  version: string
  environment: string
  description: string
}

export interface GzipSection {
  enabled: boolean
  level: number
  min_size: number
  compress_types: string[]
}

export interface CacheSection {
  enabled: boolean
  max_entries: number
  ttl_seconds: number
  cached_commands: string[]
}

export interface PaginationSection {
  default_page_size: number
  max_page_size: number
}

export interface RequestMergingSection {
  enabled: boolean
  window_ms: number
  max_batch_size: number
}

export interface ApiSection {
  base_url: string
  timeout_ms: number
  max_retries: number
}

export interface PerformanceSection {
  monitoring_enabled: boolean
  slow_request_threshold_ms: number
  log_payload_size: boolean
}

export interface WindowSection {
  default_width: number
  default_height: number
  min_width: number
  min_height: number
  resizable: boolean
  max_width: number
  max_height: number
  portrait?: PortraitSection
}

export interface PortraitSection {
  enabled: boolean
  default_width: number
  default_height: number
  min_width: number
  min_height: number
}

export interface SecuritySection {
  api_key: string
  secret_token: string
}

export interface FeaturesSection {
  auto_save_enabled: boolean
  sync_enabled: boolean
  writing_stats_enabled: boolean
  inspiration_board_enabled: boolean
}

export interface EditorSection {
  default_font_size: number
  default_font: string
  line_spacing: number
  show_line_numbers: boolean
  spell_check_enabled: boolean
}