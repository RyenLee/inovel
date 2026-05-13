import { createI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import zhCN from '../locales/zh-CN'
import enUS from '../locales/en-US'
import zhTW from '../locales/zh-TW'

export type MessageSchema = typeof zhCN

export type AppLocale = 'zh-CN' | 'en-US' | 'zh-TW'

export interface LanguageOption {
  value: string
  label: string
}

const LOCALE_STORAGE_KEY = 'inovel_locale'
const SUPPORTED_LOCALES: AppLocale[] = ['zh-CN', 'en-US', 'zh-TW']

const missingKeys = new Set<string>()

function handleMissing(locale: string, key: string): string {
  const cacheKey = `${locale}::${key}`
  if (!missingKeys.has(cacheKey)) {
    missingKeys.add(cacheKey)
    console.warn(`[i18n] Missing translation: locale="${locale}", key="${key}"`)
  }
  return key
}

function isValidLocale(locale: string): locale is AppLocale {
  return SUPPORTED_LOCALES.includes(locale as AppLocale)
}

function getStoredLocale(): AppLocale {
  const stored = localStorage.getItem(LOCALE_STORAGE_KEY)
  if (stored && isValidLocale(stored)) {
    return stored
  }
  return 'zh-CN'
}

const i18n = createI18n({
  legacy: false,
  locale: getStoredLocale(),
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
    'zh-TW': zhTW,
  },
  missing: import.meta.env.DEV ? handleMissing : undefined,
  missingWarn: import.meta.env.DEV,
  fallbackWarn: import.meta.env.DEV,
  messageFormat: false,
})

export async function initializeLocale(): Promise<void> {
  try {
    const locale = await invoke<string>('get_locale')
    if (isValidLocale(locale)) {
      ; (i18n.global.locale as unknown as { value: AppLocale }).value = locale
      localStorage.setItem(LOCALE_STORAGE_KEY, locale)
      document.documentElement.setAttribute('lang', locale)
    } else {
      console.warn(`[i18n] Invalid locale from backend: "${locale}", using stored/fallback locale`)
    }
  } catch (error) {
    console.warn('[i18n] Failed to load locale from backend, using stored/fallback locale:', error)
  }
}

export async function getLanguageList(): Promise<LanguageOption[]> {
  try {
    const list = await invoke<LanguageOption[]>('get_language_list')
    if (list && list.length > 0) {
      return list
    }
    console.warn('[i18n] Backend returned empty language list, using fallback')
    return getDefaultLanguageList()
  } catch (error) {
    console.warn('[i18n] Failed to get language list from backend, using fallback:', error)
    return getDefaultLanguageList()
  }
}

function getDefaultLanguageList(): LanguageOption[] {
  return [
    { value: 'zh-CN', label: '简体中文' },
    { value: 'en-US', label: 'English' },
    { value: 'zh-TW', label: '繁体中文' },
  ]
}

export async function syncLocaleToBackend(locale: AppLocale): Promise<void> {
  if (!isValidLocale(locale)) {
    console.warn(`[i18n] Invalid locale to sync: "${locale}"`)
    return
  }
  try {
    await invoke('set_locale', { locale })
  } catch (error) {
    console.warn('[i18n] Failed to sync locale to backend:', error)
  }
}

export async function setLocale(locale: AppLocale): Promise<void> {
  if (!isValidLocale(locale)) {
    console.warn(`[i18n] Invalid locale to set: "${locale}"`)
    return
  }
  ; (i18n.global.locale as unknown as { value: AppLocale }).value = locale
  localStorage.setItem(LOCALE_STORAGE_KEY, locale)
  document.documentElement.setAttribute('lang', locale)
  await syncLocaleToBackend(locale)
}

export function getLocale(): AppLocale {
  return (i18n.global.locale as unknown as { value: AppLocale }).value
}

export function getMissingKeys(): string[] {
  return Array.from(missingKeys)
}

export function getSupportedLocales(): AppLocale[] {
  return [...SUPPORTED_LOCALES]
}

export default i18n