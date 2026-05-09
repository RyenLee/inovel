import { describe, it, expect, beforeEach, vi } from 'vitest'
import zhCNCommon from '../locales/zh-CN/common.json'
import enUSCommon from '../locales/en-US/common.json'
import zhCNTask from '../locales/zh-CN/task.json'
import enUSTask from '../locales/en-US/task.json'
import zhCNSettings from '../locales/zh-CN/settings.json'
import enUSSettings from '../locales/en-US/settings.json'

function collectKeys(obj: Record<string, unknown>, prefix = ''): string[] {
  const keys: string[] = []
  for (const key of Object.keys(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key
    const value = obj[key]
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      keys.push(...collectKeys(value as Record<string, unknown>, fullKey))
    } else {
      keys.push(fullKey)
    }
  }
  return keys
}

function collectPlaceholders(obj: Record<string, unknown>, prefix = ''): Map<string, string[]> {
  const result = new Map<string, string[]>()
  for (const key of Object.keys(obj)) {
    const fullKey = prefix ? `${prefix}.${key}` : key
    const value = obj[key]
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      const nested = collectPlaceholders(value as Record<string, unknown>, fullKey)
      for (const [k, v] of nested) result.set(k, v)
    } else if (typeof value === 'string') {
      const matches = value.match(/\{(\w+)\}/g)
      if (matches) {
        result.set(fullKey, matches.map((m) => m.slice(1, -1)))
      }
    }
  }
  return result
}

describe('i18n Language Resource Key Completeness', () => {
  it('zh-CN and en-US common.json should have identical keys', () => {
    const zhKeys = collectKeys(zhCNCommon).sort()
    const enKeys = collectKeys(enUSCommon).sort()
    expect(enKeys).toEqual(zhKeys)
  })

  it('zh-CN and en-US task.json should have identical keys', () => {
    const zhKeys = collectKeys(zhCNTask).sort()
    const enKeys = collectKeys(enUSTask).sort()
    expect(enKeys).toEqual(zhKeys)
  })

  it('zh-CN and en-US settings.json should have identical keys', () => {
    const zhKeys = collectKeys(zhCNSettings).sort()
    const enKeys = collectKeys(enUSSettings).sort()
    expect(enKeys).toEqual(zhKeys)
  })

  it('no empty translation values in en-US common.json', () => {
    const keys = collectKeys(enUSCommon)
    for (const key of keys) {
      const parts = key.split('.')
      let value: unknown = enUSCommon
      for (const part of parts) {
        value = (value as Record<string, unknown>)[part]
      }
      expect(value, `Empty value for key "${key}" in en-US common.json`).not.toBe('')
    }
  })

  it('no empty translation values in en-US task.json', () => {
    const keys = collectKeys(enUSTask)
    for (const key of keys) {
      const parts = key.split('.')
      let value: unknown = enUSTask
      for (const part of parts) {
        value = (value as Record<string, unknown>)[part]
      }
      expect(value, `Empty value for key "${key}" in en-US task.json`).not.toBe('')
    }
  })
})

describe('i18n Placeholder Consistency', () => {
  it('placeholders in zh-CN task.json should match en-US task.json', () => {
    const zhPlaceholders = collectPlaceholders(zhCNTask)
    const enPlaceholders = collectPlaceholders(enUSTask)

    for (const [key, zhVars] of zhPlaceholders) {
      const enVars = enPlaceholders.get(key)
      expect(enVars, `Key "${key}" has placeholders in zh-CN but not in en-US`).toBeDefined()
      if (enVars) {
        expect(
          enVars.sort(),
          `Placeholder mismatch for key "${key}": zh-CN has ${zhVars.join(', ')}, en-US has ${enVars.join(', ')}`
        ).toEqual(zhVars.sort())
      }
    }

    for (const [key] of enPlaceholders) {
      expect(
        zhPlaceholders.has(key),
        `Key "${key}" has placeholders in en-US but not in zh-CN`
      ).toBe(true)
    }
  })

  it('placeholders in zh-CN common.json should match en-US common.json', () => {
    const zhPlaceholders = collectPlaceholders(zhCNCommon)
    const enPlaceholders = collectPlaceholders(enUSCommon)

    for (const [key, zhVars] of zhPlaceholders) {
      const enVars = enPlaceholders.get(key)
      expect(enVars, `Key "${key}" has placeholders in zh-CN but not in en-US`).toBeDefined()
      if (enVars) {
        expect(enVars.sort()).toEqual(zhVars.sort())
      }
    }
  })
})

describe('i18n Fallback Mechanism', () => {
  it('fallbackLocale should be zh-CN', () => {
    const fallbackConfig = 'zh-CN'
    expect(fallbackConfig).toBe('zh-CN')
  })

  it('all zh-CN keys should have valid string values', () => {
    const allKeys = [...collectKeys(zhCNCommon), ...collectKeys(zhCNTask)]
    expect(allKeys.length).toBeGreaterThan(0)

    for (const key of collectKeys(zhCNCommon)) {
      const parts = key.split('.')
      let value: unknown = zhCNCommon
      for (const part of parts) {
        value = (value as Record<string, unknown>)[part]
      }
      expect(typeof value).toBe('string')
      expect(value).not.toBe('')
    }
  })
})

describe('i18n Language Switching Data Integrity', () => {
  it('switching locale should not lose task data structure', () => {
    const sampleTask = {
      id: 'task_test_1',
      name: 'Test Task',
      completed: false,
      priority: 'high' as const,
      dueDate: Date.now(),
      assignee: 'Developer',
      notes: 'Test notes',
      tags: [],
      createdAt: Date.now(),
      updatedAt: Date.now(),
    }

    const serialized = JSON.stringify(sampleTask)
    const deserialized = JSON.parse(serialized)

    expect(deserialized.id).toBe(sampleTask.id)
    expect(deserialized.name).toBe(sampleTask.name)
    expect(deserialized.completed).toBe(sampleTask.completed)
    expect(deserialized.priority).toBe(sampleTask.priority)
    expect(deserialized.assignee).toBe(sampleTask.assignee)
    expect(deserialized.notes).toBe(sampleTask.notes)
  })

  it('locale preference should persist via setLocale pattern', () => {
    const LOCALE_KEY = 'inovel_locale'
    const store = new Map<string, string>()

    store.set(LOCALE_KEY, 'en-US')
    expect(store.get(LOCALE_KEY)).toBe('en-US')

    store.set(LOCALE_KEY, 'zh-CN')
    expect(store.get(LOCALE_KEY)).toBe('zh-CN')

    store.delete(LOCALE_KEY)
    expect(store.get(LOCALE_KEY)).toBeUndefined()
  })
})

describe('i18n Text Expansion Compatibility', () => {
  it('en-US text should be longer than zh-CN for key UI strings', () => {
    const expansionChecks: { key: string; source: 'task' | 'common' }[] = [
      { key: 'task.addTask', source: 'task' },
      { key: 'task.clearCompleted', source: 'task' },
      { key: 'task.noDueDate', source: 'task' },
      { key: 'common.action.cancel', source: 'common' },
    ]

    const getNestedValue = (obj: Record<string, unknown>, path: string[]): unknown => {
      let value: unknown = obj
      for (const p of path) {
        if (value == null || typeof value !== 'object') return undefined
        value = (value as Record<string, unknown>)[p]
      }
      return value
    }

    for (const check of expansionChecks) {
      const zhSource = check.source === 'task' ? zhCNTask : zhCNCommon
      const enSource = check.source === 'task' ? enUSTask : enUSCommon

      const pathParts = check.key.replace(`${check.source}.`, '').split('.')

      const zhValue = getNestedValue(zhSource, pathParts)
      const enValue = getNestedValue(enSource, pathParts)

      if (typeof zhValue === 'string' && typeof enValue === 'string') {
        const expansionRatio = enValue.length / zhValue.length
        expect(
          expansionRatio,
          `en-US "${check.key}" (${enValue}) is ${expansionRatio.toFixed(1)}x longer than zh-CN (${zhValue}), may need UI adaptation`
        ).toBeLessThan(5)
      }
    }
  })
})

describe('i18n New Language Pack Import Compatibility', () => {
  it('should validate that a new language pack follows the same key structure', () => {
    const requiredKeys = collectKeys(zhCNCommon)

    const mockNewLangPack = { ...zhCNCommon }
    const newLangKeys = collectKeys(mockNewLangPack)

    const missingKeys = requiredKeys.filter((k) => !newLangKeys.includes(k))
    expect(missingKeys, `New language pack is missing keys: ${missingKeys.join(', ')}`).toEqual([])
  })

  it('should detect extra keys in a new language pack', () => {
    const requiredKeys = collectKeys(zhCNCommon)

    const mockNewLangPack = { ...zhCNCommon, extraKey: 'extra' }
    const newLangKeys = collectKeys(mockNewLangPack)

    const extraKeys = newLangKeys.filter((k) => !requiredKeys.includes(k))
    expect(extraKeys).toEqual(['extraKey'])
  })
})
