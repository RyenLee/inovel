import { invoke } from '@tauri-apps/api/core';
import type { TomlConfig } from '../config/types';

const DEFAULT_CONFIG: Partial<TomlConfig> = {
    app: {
        name: 'iNovel',
        version: '1.0.0',
        environment: 'development',
        description: '一款现代化的小说创作工具',
    },
    api: {
        base_url: 'http://localhost:3000',
        timeout_ms: 30000,
        max_retries: 3,
    },
    window: {
        default_width: 1200,
        default_height: 800,
        min_width: 600,
        min_height: 800,
        max_width: 1920,
        max_height: 1200,
        resizable: true,
    },
    editor: {
        default_font_size: 16,
        default_font: '微软雅黑',
        line_spacing: 1.5,
        show_line_numbers: true,
        spell_check_enabled: true,
    },
    features: {
        auto_save_enabled: true,
        sync_enabled: false,
        writing_stats_enabled: true,
        inspiration_board_enabled: true,
    },
    performance: {
        monitoring_enabled: true,
        slow_request_threshold_ms: 500,
        log_payload_size: false,
    },
    cache: {
        enabled: true,
        max_entries: 1000,
        ttl_seconds: 300,
        cached_commands: [],
    },
    gzip: {
        enabled: true,
        level: 6,
        min_size: 1024,
        compress_types: [
            'application/json',
            'text/plain',
            'text/html',
            'text/css',
            'text/javascript',
            'application/javascript',
            'application/xml',
            'text/xml',
        ],
    },
    pagination: {
        default_page_size: 20,
        max_page_size: 100,
    },
    request_merging: {
        enabled: true,
        window_ms: 100,
        max_batch_size: 10,
    },
    security: {
        api_key: '',
        secret_token: '',
    },
};

type ConfigChangeListener = (config: TomlConfig) => void;

class ConfigService {
    private config: TomlConfig = {} as TomlConfig;
    private listeners: ConfigChangeListener[] = [];
    private isLoaded = false;
    private loadError: Error | null = null;

    public async loadConfig(): Promise<TomlConfig> {
        try {
            const result = await invoke<TomlConfig>('read_toml_config');
            this.config = this.mergeWithDefaults(result);
            this.isLoaded = true;
            this.loadError = null;
            this.notifyListeners();
            return this.config;
        } catch (error) {
            console.warn('Failed to load config from backend, using defaults:', error);
            this.loadError = error as Error;
            this.config = this.getFullDefaults();
            this.isLoaded = true;
            this.notifyListeners();
            return this.config;
        }
    }

    public async saveConfig(config: Partial<TomlConfig>): Promise<void> {
        try {
            await invoke('write_toml_config', { newConfig: config });
            this.config = { ...this.config, ...config };
            this.notifyListeners();
        } catch (error) {
            console.error('Failed to save config:', error);
            throw error;
        }
    }

    public getConfig(): TomlConfig {
        return { ...this.config };
    }

    public getAppConfig() {
        return {
            ...DEFAULT_CONFIG.app,
            ...this.config.app,
        };
    }

    public getApiConfig() {
        return {
            ...DEFAULT_CONFIG.api,
            ...this.config.api,
        };
    }

    public addChangeListener(listener: ConfigChangeListener): () => void {
        this.listeners.push(listener);
        return () => {
            const index = this.listeners.indexOf(listener);
            if (index > -1) {
                this.listeners.splice(index, 1);
            }
        };
    }

    public removeChangeListener(listener: ConfigChangeListener): void {
        const index = this.listeners.indexOf(listener);
        if (index > -1) {
            this.listeners.splice(index, 1);
        }
    }

    public isConfigLoaded(): boolean {
        return this.isLoaded;
    }

    public getLoadError(): Error | null {
        return this.loadError;
    }

    private mergeWithDefaults(config: Partial<TomlConfig>): TomlConfig {
        return {
            ...DEFAULT_CONFIG,
            ...config,
            app: {
                ...DEFAULT_CONFIG.app,
                ...config.app,
            },
            api: {
                ...DEFAULT_CONFIG.api,
                ...config.api,
            },
            window: {
                ...DEFAULT_CONFIG.window,
                ...config.window,
                portrait: {
                    ...DEFAULT_CONFIG.window?.portrait,
                    ...config.window?.portrait,
                },
            },
            editor: {
                ...DEFAULT_CONFIG.editor,
                ...config.editor,
            },
            features: {
                ...DEFAULT_CONFIG.features,
                ...config.features,
            },
            performance: {
                ...DEFAULT_CONFIG.performance,
                ...config.performance,
            },
            cache: {
                ...DEFAULT_CONFIG.cache,
                ...config.cache,
            },
            gzip: {
                ...DEFAULT_CONFIG.gzip,
                ...config.gzip,
            },
            pagination: {
                ...DEFAULT_CONFIG.pagination,
                ...config.pagination,
            },
            request_merging: {
                ...DEFAULT_CONFIG.request_merging,
                ...config.request_merging,
            },
            security: {
                ...DEFAULT_CONFIG.security,
                ...config.security,
            },
        } as TomlConfig;
    }

    private getFullDefaults(): TomlConfig {
        return DEFAULT_CONFIG as TomlConfig;
    }

    private notifyListeners(): void {
        const configCopy = { ...this.config };
        this.listeners.forEach((listener) => {
            try {
                listener(configCopy);
            } catch (error) {
                console.error('Error in config change listener:', error);
            }
        });
    }
}

export const configService = new ConfigService();

export const loadConfig = async (): Promise<TomlConfig> => {
    return configService.loadConfig();
};

export const saveConfig = async (config: Partial<TomlConfig>): Promise<void> => {
    return configService.saveConfig(config);
};

export const getConfig = (): TomlConfig => {
    return configService.getConfig();
};
