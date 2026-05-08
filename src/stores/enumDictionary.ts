/**
 * 枚举字典服务层
 * 
 * 提供前端与后端枚举字典API的交互能力，支持动态获取枚举值的显示名称。
 * 采用单例模式管理全局枚举字典数据，提供渐进式迁移支持（字典API优先，硬编码fallback）。
 * 
 * 主要功能：
 * 1. 从后端加载枚举字典数据
 * 2. 提供按类别查询枚举的方法
 * 3. 提供获取枚举显示名称的便捷方法
 * 4. 支持响应式数据更新
 * 
 * 使用方式：
 * ```typescript
 * import { useEnumDictionary } from '@/stores/enumDictionary'
 * 
 * const { loadDictionary, getChapterStatusName, isLoaded } = useEnumDictionary()
 * 
 * // 在应用启动时加载字典
 * await loadDictionary()
 * 
 * // 获取枚举名称
 * const statusName = getChapterStatusName('draft') // 返回 '草稿'
 * ```
 */

import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/**
 * 枚举定义接口
 */
export interface EnumDefinition {
  /** 枚举类别 */
  category: string
  /** 枚举代码（唯一标识） */
  code: string
  /** 枚举显示名称 */
  name: string
  /** 枚举描述 */
  description: string | null
  /** 排序序号 */
  sort_order: number
}

/**
 * 枚举类别映射类型
 * key: 类别名称, value: 该类别的所有枚举定义
 */
export type EnumCategoryMap = Record<string, EnumDefinition[]>

/**
 * 默认字典数据（作为fallback）
 * 当后端API无法获取字典数据时使用
 */
const defaultDictionary: EnumCategoryMap = {
  // 性别
  gender: [
    { category: 'gender', code: 'male', name: '男', description: '男性', sort_order: 1 },
    { category: 'gender', code: 'female', name: '女', description: '女性', sort_order: 2 },
    { category: 'gender', code: 'other', name: '其他', description: '其他', sort_order: 3 },
    { category: 'gender', code: 'unknown', name: '未知', description: '未知', sort_order: 4 },
  ],
  // 地点类型
  location_type: [
    { category: 'location_type', code: 'city', name: '城市', description: '城市', sort_order: 1 },
    { category: 'location_type', code: 'town', name: '城镇', description: '城镇', sort_order: 2 },
    { category: 'location_type', code: 'village', name: '村庄', description: '村庄', sort_order: 3 },
    { category: 'location_type', code: 'building', name: '建筑', description: '建筑', sort_order: 4 },
    { category: 'location_type', code: 'region', name: '区域', description: '区域', sort_order: 5 },
    { category: 'location_type', code: 'country', name: '国家', description: '国家', sort_order: 6 },
    { category: 'location_type', code: 'kingdom', name: '王国', description: '王国', sort_order: 7 },
    { category: 'location_type', code: 'mountain', name: '山脉', description: '山脉', sort_order: 8 },
    { category: 'location_type', code: 'forest', name: '森林', description: '森林', sort_order: 9 },
    { category: 'location_type', code: 'ocean', name: '海洋', description: '海洋', sort_order: 10 },
    { category: 'location_type', code: 'other', name: '其他', description: '其他类型地点', sort_order: 99 },
  ],
  // 组织类型
  organization_type: [
    { category: 'organization_type', code: 'kingdom', name: '王国', description: '王国', sort_order: 1 },
    { category: 'organization_type', code: 'guild', name: '公会', description: '公会', sort_order: 2 },
    { category: 'organization_type', code: 'gang', name: '帮派', description: '帮派', sort_order: 3 },
    { category: 'organization_type', code: 'cult', name: '教派', description: '教派', sort_order: 4 },
    { category: 'organization_type', code: 'company', name: '商会', description: '商会', sort_order: 5 },
    { category: 'organization_type', code: 'military', name: '军队', description: '军队', sort_order: 6 },
    { category: 'organization_type', code: 'secret_society', name: '秘密组织', description: '秘密组织', sort_order: 7 },
    { category: 'organization_type', code: 'family', name: '家族', description: '家族', sort_order: 8 },
    { category: 'organization_type', code: 'church', name: '教会', description: '教会', sort_order: 9 },
    { category: 'organization_type', code: 'other', name: '其他', description: '其他类型组织', sort_order: 99 },
  ],
  // 章节状态
  chapter_status: [
    { category: 'chapter_status', code: 'draft', name: '草稿', description: '草稿状态', sort_order: 1 },
    { category: 'chapter_status', code: 'review', name: '审核中', description: '审核中', sort_order: 2 },
    { category: 'chapter_status', code: 'published', name: '已发布', description: '已发布', sort_order: 3 },
  ],
  // 专注会话类型
  session_type: [
    { category: 'session_type', code: 'pomodoro', name: '番茄专注', description: '番茄钟专注会话', sort_order: 1 },
    { category: 'session_type', code: 'free', name: '自由写作', description: '自由写作会话', sort_order: 2 },
  ],
  // 备份状态
  backup_status: [
    { category: 'backup_status', code: 'pending', name: '待备份', description: '待备份', sort_order: 1 },
    { category: 'backup_status', code: 'success', name: '成功', description: '备份成功', sort_order: 2 },
    { category: 'backup_status', code: 'failed', name: '失败', description: '备份失败', sort_order: 3 },
  ],
  // 操作结果
  operation_result: [
    { category: 'operation_result', code: 'success', name: '成功', description: '操作成功', sort_order: 1 },
    { category: 'operation_result', code: 'failed', name: '失败', description: '操作失败', sort_order: 2 },
  ],
  // 操作类别
  operation_category: [
    { category: 'operation_category', code: 'create', name: '创建', description: '创建操作', sort_order: 1 },
    { category: 'operation_category', code: 'update', name: '更新', description: '更新操作', sort_order: 2 },
    { category: 'operation_category', code: 'delete', name: '删除', description: '删除操作', sort_order: 3 },
    { category: 'operation_category', code: 'import', name: '导入', description: '导入操作', sort_order: 4 },
    { category: 'operation_category', code: 'export', name: '导出', description: '导出操作', sort_order: 5 },
  ],
}

/**
 * 全局枚举字典数据（响应式）
 * 初始化为默认字典数据作为fallback
 */
const enumDictionary = ref<EnumCategoryMap>({ ...defaultDictionary })

/**
 * 字典是否已加载完成
 */
const isLoaded = ref(false)

/**
 * 字典是否正在加载中
 */
const isLoading = ref(false)

/**
 * 枚举字典组合式函数
 * 
 * @returns 枚举字典服务实例
 */
export function useEnumDictionary() {
  /**
   * 加载枚举字典
   * 
   * @param category - 可选，指定加载的类别，不传则加载所有类别
   */
  async function loadDictionary(category?: string) {
    if (isLoading.value) return
    isLoading.value = true

    // console.log('[EnumDictionary] 开始加载字典数据', category ? `类别: ${category}` : '全部类别')

    try {
      // console.log('[EnumDictionary] 调用后端API: get_enum_dictionary', { category: category || null })
      const result = await invoke<EnumCategoryMap>('get_enum_dictionary', {
        category: category || null,
      })
      // console.log('[EnumDictionary] API返回结果:', JSON.stringify(result, null, 2))

      // 合并已有的字典数据（增量加载）
      enumDictionary.value = {
        ...enumDictionary.value,
        ...result,
      }
      // console.log('[EnumDictionary] 合并后字典数据:', JSON.stringify(enumDictionary.value, null, 2))
      isLoaded.value = true
    } catch (error) {
      console.error('[EnumDictionary] 加载字典失败:', error)
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 根据类别获取所有枚举定义
   * 
   * @param category - 类别名称
   * @returns 该类别的枚举定义数组，不存在则返回空数组
   */
  function getEnumsByCategory(category: string): EnumDefinition[] {
    return enumDictionary.value[category] || []
  }

  /**
   * 获取指定枚举的显示名称
   * 
   * @param category - 类别名称
   * @param code - 枚举代码
   * @returns 枚举显示名称，不存在则返回原始code
   */
  function getEnumName(category: string, code: string): string {
    const enums = enumDictionary.value[category] || []
    const found = enums.find(e => e.code === code)
    return found?.name || code
  }

  /**
   * 获取指定枚举的完整定义
   * 
   * @param category - 类别名称
   * @param code - 枚举代码
   * @returns 枚举定义对象，不存在则返回undefined
   */
  function getEnumDefinition(category: string, code: string): EnumDefinition | undefined {
    const enums = enumDictionary.value[category] || []
    return enums.find(e => e.code === code)
  }

  /**
   * 章节状态枚举（响应式）
   */
  const chapterStatuses = computed(() => getEnumsByCategory('chapter_status'))

  /**
   * 专注会话类型枚举（响应式）
   */
  const sessionTypes = computed(() => getEnumsByCategory('session_type'))

  /**
   * 备份状态枚举（响应式）
   */
  const backupStatuses = computed(() => getEnumsByCategory('backup_status'))

  /**
   * 操作结果枚举（响应式）
   */
  const operationResults = computed(() => getEnumsByCategory('operation_result'))

  /**
   * 性别枚举（响应式）
   */
  const genders = computed(() => getEnumsByCategory('gender'))

  /**
   * 操作类别枚举（响应式）
   */
  const operationCategories = computed(() => getEnumsByCategory('operation_category'))

  /**
   * 地点类型枚举（响应式）
   */
  const locationTypes = computed(() => getEnumsByCategory('location_type'))

  /**
   * 组织类型枚举（响应式）
   */
  const organizationTypes = computed(() => getEnumsByCategory('organization_type'))

  /**
   * 章节状态选项（用于表单选择器）
   */
  const chapterStatusOptions = computed(() =>
    chapterStatuses.value.map(e => ({
      label: e.name,
      value: e.code,
    }))
  )

  /**
   * 会话类型选项（用于表单选择器）
   */
  const sessionTypeOptions = computed(() =>
    sessionTypes.value.map(e => ({
      label: e.name,
      value: e.code,
    }))
  )

  /**
   * 性别选项（用于表单选择器）
   */
  const genderOptions = computed(() =>
    genders.value.map(e => ({
      label: e.name,
      value: e.code,
    }))
  )

  /**
   * 地点类型选项（用于表单选择器）
   */
  const locationTypeOptions = computed(() =>
    locationTypes.value.map(e => ({
      label: e.name,
      value: e.code,
    }))
  )

  /**
   * 组织类型选项（用于表单选择器）
   */
  const organizationTypeOptions = computed(() =>
    organizationTypes.value.map(e => ({
      label: e.name,
      value: e.code,
    }))
  )

  /**
   * 获取章节状态的显示名称
   * 
   * @param code - 状态代码
   * @returns 状态显示名称
   */
  function getChapterStatusName(code: string): string {
    return getEnumName('chapter_status', code)
  }

  /**
   * 获取会话类型的显示名称
   * 
   * @param code - 类型代码
   * @returns 类型显示名称
   */
  function getSessionTypeName(code: string): string {
    return getEnumName('session_type', code)
  }

  /**
   * 获取性别的显示名称
   * 
   * @param code - 性别代码
   * @returns 性别显示名称
   */
  function getGenderName(code: string): string {
    return getEnumName('gender', code)
  }

  /**
   * 获取地点类型的显示名称
   * 
   * @param code - 地点类型代码
   * @returns 地点类型显示名称
   */
  function getLocationTypeName(code: string): string {
    return getEnumName('location_type', code)
  }

  /**
   * 获取组织类型的显示名称
   * 
   * @param code - 组织类型代码
   * @returns 组织类型显示名称
   */
  function getOrganizationTypeName(code: string): string {
    return getEnumName('organization_type', code)
  }

  /**
   * 获取操作结果的显示名称
   * 
   * @param code - 结果代码
   * @returns 结果显示名称
   */
  function getOperationResultName(code: string): string {
    return getEnumName('operation_result', code)
  }

  /**
   * 获取备份状态的显示名称
   * 
   * @param code - 状态代码
   * @returns 状态显示名称
   */
  function getBackupStatusName(code: string): string {
    return getEnumName('backup_status', code)
  }

  return {
    /** 全局枚举字典数据 */
    enumDictionary,
    /** 字典是否已加载完成 */
    isLoaded,
    /** 字典是否正在加载中 */
    isLoading,
    /** 加载枚举字典 */
    loadDictionary,
    /** 根据类别获取所有枚举定义 */
    getEnumsByCategory,
    /** 获取指定枚举的显示名称 */
    getEnumName,
    /** 获取指定枚举的完整定义 */
    getEnumDefinition,
    /** 章节状态枚举 */
    chapterStatuses,
    /** 专注会话类型枚举 */
    sessionTypes,
    /** 备份状态枚举 */
    backupStatuses,
    /** 操作结果枚举 */
    operationResults,
    /** 性别枚举 */
    genders,
    /** 操作类别枚举 */
    operationCategories,
    /** 地点类型枚举 */
    locationTypes,
    /** 组织类型枚举 */
    organizationTypes,
    /** 章节状态选项 */
    chapterStatusOptions,
    /** 会话类型选项 */
    sessionTypeOptions,
    /** 性别选项 */
    genderOptions,
    /** 地点类型选项 */
    locationTypeOptions,
    /** 组织类型选项 */
    organizationTypeOptions,
    /** 获取章节状态名称 */
    getChapterStatusName,
    /** 获取会话类型名称 */
    getSessionTypeName,
    /** 获取性别名称 */
    getGenderName,
    /** 获取地点类型名称 */
    getLocationTypeName,
    /** 获取组织类型名称 */
    getOrganizationTypeName,
    /** 获取操作结果名称 */
    getOperationResultName,
    /** 获取备份状态名称 */
    getBackupStatusName,
  }
}
