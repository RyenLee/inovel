import { createI18n } from 'vue-i18n'
import zhCN from '../locales/zh-CN'
import enUS from '../locales/en-US'

export type MessageSchema = typeof zhCN

export type AppLocale = 'zh-CN' | 'en-US'

const LOCALE_STORAGE_KEY = 'inovel_locale'

const missingKeys = new Set<string>()

function handleMissing(locale: string, key: string): string {
  const cacheKey = `${locale}::${key}`
  if (!missingKeys.has(cacheKey)) {
    missingKeys.add(cacheKey)
    console.warn(`[i18n] Missing translation: locale="${locale}", key="${key}"`)
  }
  return key
}

function detectSystemLocale(): AppLocale {
  const lang = navigator.language
  if (lang.startsWith('zh')) return 'zh-CN'
  return 'en-US'
}

function getStoredLocale(): AppLocale {
  const stored = localStorage.getItem(LOCALE_STORAGE_KEY)
  if (stored === 'zh-CN' || stored === 'en-US') return stored
  return detectSystemLocale()
}

const i18n = createI18n({
  legacy: false,
  locale: getStoredLocale(),
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
  missing: import.meta.env.DEV ? handleMissing : undefined,
  missingWarn: import.meta.env.DEV,
  fallbackWarn: import.meta.env.DEV,
})

export function setLocale(locale: AppLocale): void {
  ;(i18n.global.locale as unknown as { value: AppLocale }).value = locale
  localStorage.setItem(LOCALE_STORAGE_KEY, locale)
  document.documentElement.setAttribute('lang', locale)
}

export function getLocale(): AppLocale {
  return (i18n.global.locale as unknown as { value: AppLocale }).value
}

export function getMissingKeys(): string[] {
  return Array.from(missingKeys)
}

export default i18n
