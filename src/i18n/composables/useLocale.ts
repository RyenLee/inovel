import { computed, ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { setLocale, getLocale, initializeLocale, getLanguageList } from '../index'
import type { AppLocale } from '../index'
import type { LanguageOption } from '../index'

export function useLocale() {
  const { t, locale } = useI18n()

  const isZhCN = computed(() => locale.value === 'zh-CN')
  const isEnUS = computed(() => locale.value === 'en-US')
  const isZHTW = computed(() => locale.value === 'zh-TW')

  const availableLocales = ref<LanguageOption[]>([])

  const loadLanguageList = async () => {
    try {
      const list = await getLanguageList()
      availableLocales.value = list
    } catch (error) {
      console.warn('[useLocale] Failed to load language list:', error)
    }
  }

  const switchLocale = async (newLocale: AppLocale) => {
    await setLocale(newLocale)
  }

  const currentLocale = computed(() => getLocale())

  const initialize = async () => {
    await initializeLocale()
    await loadLanguageList()
  }

  onMounted(() => {
    initialize()
  })

  return {
    t,
    locale,
    isZhCN,
    isEnUS,
    isZHTW,
    availableLocales,
    switchLocale,
    currentLocale,
    loadLanguageList,
  }
}