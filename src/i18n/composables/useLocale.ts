import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLocale, getLocale } from '../index'
import type { AppLocale } from '../index'

export function useLocale() {
  const { t, locale } = useI18n()

  const isZhCN = computed(() => locale.value === 'zh-CN')
  const isEnUS = computed(() => locale.value === 'en-US')

  const availableLocales: { label: string; value: AppLocale }[] = [
    { label: '简体中文', value: 'zh-CN' },
    { label: 'English', value: 'en-US' },
  ]

  const switchLocale = (newLocale: AppLocale) => {
    setLocale(newLocale)
  }

  const currentLocale = computed(() => getLocale())

  return {
    t,
    locale,
    isZhCN,
    isEnUS,
    availableLocales,
    switchLocale,
    currentLocale,
  }
}
