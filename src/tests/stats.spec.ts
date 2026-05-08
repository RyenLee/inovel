/**
 * 写作统计功能单元测试
 * 
 * 测试覆盖：
 * 1. 专注时长计算准确性
 * 2. 完成番茄数统计规则
 * 3. 最近专注记录排序和过滤
 * 4. 数据一致性校验机制
 * 5. 边界条件和异常数据处理
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { ref, computed } from 'vue'

// Mock数据
const mockFocusStats = {
  total_sessions: 10,
  total_minutes: 300,
  completed_sessions: 8,
  work_sessions: 6,
  short_break_sessions: 3,
  long_break_sessions: 1,
  completed_work_sessions: 5,
  work_duration_minutes: 150,
}

const mockFocusSessions = [
  { id: 1, project_id: 1, session_type: 'work', duration_minutes: 25, started_at: '2024-01-15T10:00:00', completed: true, created_at: '2024-01-15T10:25:00' },
  { id: 2, project_id: 1, session_type: 'short_break', duration_minutes: 5, started_at: '2024-01-15T10:25:00', completed: true, created_at: '2024-01-15T10:30:00' },
  { id: 3, project_id: 1, session_type: 'work', duration_minutes: 20, started_at: '2024-01-15T10:30:00', completed: false, created_at: '2024-01-15T10:50:00' },
  { id: 4, project_id: 1, session_type: 'long_break', duration_minutes: 15, started_at: '2024-01-15T11:00:00', completed: true, created_at: '2024-01-15T11:15:00' },
  { id: 5, project_id: 1, session_type: 'work', duration_minutes: 25, started_at: '2024-01-14T09:00:00', completed: true, created_at: '2024-01-14T09:25:00' },
]

const mockInvalidSessions = [
  { id: -1, project_id: 1, session_type: 'work', duration_minutes: 25, started_at: '', completed: true, created_at: '2024-01-15T10:25:00' },
  { id: 0, project_id: 1, session_type: 'work', duration_minutes: -5, started_at: '2024-01-15T10:00:00', completed: true, created_at: '2024-01-15T10:25:00' },
  { id: 6, project_id: 1, session_type: 'work', duration_minutes: 25, started_at: '2024-01-15T10:00:00', completed: true, created_at: '2024-01-15T10:25:00' },
]

describe('写作统计功能测试', () => {
  describe('专注时长计算', () => {
    it('应该正确计算专注时长（仅工作会话）', () => {
      const focusStats = ref({ ...mockFocusStats })
      const totalFocusMinutes = computed(() => focusStats.value.work_duration_minutes || 0)
      
      expect(totalFocusMinutes.value).toBe(150)
      expect(totalFocusMinutes.value).toBeTypeOf('number')
    })

    it('应该正确格式化专注时长（分钟）', () => {
      const formatFocusTime = (minutes: number): string => {
        if (minutes < 60) {
          return `${minutes}分钟`
        }
        const hours = Math.floor(minutes / 60)
        const mins = minutes % 60
        return mins > 0 ? `${hours}小时${mins}分钟` : `${hours}小时`
      }

      expect(formatFocusTime(45)).toBe('45分钟')
      expect(formatFocusTime(60)).toBe('1小时')
      expect(formatFocusTime(90)).toBe('1小时30分钟')
      expect(formatFocusTime(150)).toBe('2小时30分钟')
      expect(formatFocusTime(0)).toBe('0分钟')
    })

    it('应该处理零和负数时长', () => {
      const focusStats = ref({ ...mockFocusStats, work_duration_minutes: 0 })
      const totalFocusMinutes = computed(() => Math.max(focusStats.value.work_duration_minutes || 0, 0))
      
      expect(totalFocusMinutes.value).toBe(0)

      focusStats.value.work_duration_minutes = -10
      expect(totalFocusMinutes.value).toBe(0)
    })
  })

  describe('完成番茄数统计', () => {
    it('应该正确统计完成的工作会话数', () => {
      const focusStats = ref({ ...mockFocusStats })
      const completedPomodoros = computed(() => focusStats.value.completed_work_sessions || 0)
      
      expect(completedPomodoros.value).toBe(5)
      expect(completedPomodoros.value).toBeTypeOf('number')
    })

    it('应该排除未完成的工作会话', () => {
      const sessions = mockFocusSessions.filter(s => s.session_type === 'work' && s.completed)
      expect(sessions.length).toBe(2)
      expect(sessions.every(s => s.completed)).toBe(true)
    })

    it('应该区分工作会话和休息会话', () => {
      const workSessions = mockFocusSessions.filter(s => s.session_type === 'work')
      const breakSessions = mockFocusSessions.filter(s => s.session_type !== 'work')
      
      expect(workSessions.length).toBe(3)
      expect(breakSessions.length).toBe(2)
      expect(workSessions.every(s => s.session_type === 'work')).toBe(true)
    })
  })

  describe('最近专注记录排序', () => {
    it('应该按开始时间降序排序', () => {
      const sessions = ref([...mockFocusSessions])
      const sortedSessions = computed(() => {
        return sessions.value
          .sort((a, b) => new Date(b.started_at).getTime() - new Date(a.started_at).getTime())
      })

      expect(sortedSessions.value[0].started_at).toBe('2024-01-15T11:00:00')
      expect(sortedSessions.value[sortedSessions.value.length - 1].started_at).toBe('2024-01-14T09:00:00')
      
      // 验证排序正确性
      for (let i = 0; i < sortedSessions.value.length - 1; i++) {
        const current = new Date(sortedSessions.value[i].started_at).getTime()
        const next = new Date(sortedSessions.value[i + 1].started_at).getTime()
        expect(current).toBeGreaterThanOrEqual(next)
      }
    })

    it('应该限制最多显示10条记录', () => {
      const manySessions = ref([...Array(15)].map((_, i) => ({
        id: i + 1,
        project_id: 1,
        session_type: 'work',
        duration_minutes: 25,
        started_at: `2024-01-15T${String(i).padStart(2, '0')}:00:00`,
        completed: true,
        created_at: `2024-01-15T${String(i).padStart(2, '0')}:25:00`,
      })))

      const limitedSessions = computed(() => {
        return manySessions.value
          .sort((a, b) => new Date(b.started_at).getTime() - new Date(a.started_at).getTime())
          .slice(0, 10)
      })

      expect(limitedSessions.value.length).toBe(10)
      expect(limitedSessions.value[0].id).toBe(15)
    })

    it('应该处理时间格式不一致的情况', () => {
      const mixedSessions = ref([
        { id: 1, project_id: 1, session_type: 'work', duration_minutes: 25, started_at: '2024-01-15T10:00:00Z', completed: true, created_at: '2024-01-15T10:25:00' },
        { id: 2, project_id: 1, session_type: 'work', duration_minutes: 25, started_at: '2024-01-15 11:00:00', completed: true, created_at: '2024-01-15T11:25:00' },
        { id: 3, project_id: 1, session_type: 'work', duration_minutes: 25, started_at: '2024-01-15', completed: true, created_at: '2024-01-15T12:25:00' },
      ])

      const sortedSessions = computed(() => {
        return mixedSessions.value
          .sort((a, b) => {
            const timeA = new Date(a.started_at).getTime()
            const timeB = new Date(b.started_at).getTime()
            return timeB - timeA
          })
      })

      expect(sortedSessions.value.length).toBe(3)
    })
  })

  describe('数据一致性校验', () => {
    it('应该校验负数统计值', () => {
      const validateFocusStats = (stats: typeof mockFocusStats): boolean => {
        if (stats.total_sessions < 0 || stats.total_minutes < 0 || stats.completed_sessions < 0) {
          return false
        }
        return true
      }

      expect(validateFocusStats(mockFocusStats)).toBe(true)
      expect(validateFocusStats({ ...mockFocusStats, total_sessions: -1 })).toBe(false)
      expect(validateFocusStats({ ...mockFocusStats, total_minutes: -10 })).toBe(false)
    })

    it('应该校验完成数不超过总数', () => {
      const validateFocusStats = (stats: typeof mockFocusStats): boolean => {
        if (stats.completed_sessions > stats.total_sessions) {
          return false
        }
        if (stats.completed_work_sessions > stats.work_sessions) {
          return false
        }
        return true
      }

      expect(validateFocusStats(mockFocusStats)).toBe(true)
      expect(validateFocusStats({ ...mockFocusStats, completed_sessions: 11 })).toBe(false)
      expect(validateFocusStats({ ...mockFocusStats, completed_work_sessions: 7 })).toBe(false)
    })

    it('应该过滤无效的会话记录', () => {
      const sessions = ref([...mockFocusSessions, ...mockInvalidSessions])
      
      const validSessions = computed(() => {
        return sessions.value.filter(session => {
          if (!session.id || session.id <= 0) return false
          if (!session.started_at) return false
          if (session.duration_minutes < 0) return false
          return true
        })
      })

      expect(validSessions.value.length).toBe(6)
      expect(validSessions.value.every(s => s.id > 0)).toBe(true)
      expect(validSessions.value.every(s => s.started_at)).toBe(true)
      expect(validSessions.value.every(s => s.duration_minutes >= 0)).toBe(true)
    })

    it('应该处理空数据情况', () => {
      const emptyStats = ref({
        total_sessions: 0,
        total_minutes: 0,
        completed_sessions: 0,
        work_sessions: 0,
        short_break_sessions: 0,
        long_break_sessions: 0,
        completed_work_sessions: 0,
        work_duration_minutes: 0,
      })

      const totalFocusMinutes = computed(() => emptyStats.value.work_duration_minutes || 0)
      const completedPomodoros = computed(() => emptyStats.value.completed_work_sessions || 0)

      expect(totalFocusMinutes.value).toBe(0)
      expect(completedPomodoros.value).toBe(0)
    })
  })

  describe('会话类型统计', () => {
    it('应该正确分类统计不同类型的会话', () => {
      const sessions = ref([...mockFocusSessions])
      
      const workCount = computed(() => sessions.value.filter(s => s.session_type === 'work').length)
      const shortBreakCount = computed(() => sessions.value.filter(s => s.session_type === 'short_break').length)
      const longBreakCount = computed(() => sessions.value.filter(s => s.session_type === 'long_break').length)

      expect(workCount.value).toBe(3)
      expect(shortBreakCount.value).toBe(1)
      expect(longBreakCount.value).toBe(1)
      expect(workCount.value + shortBreakCount.value + longBreakCount.value).toBe(sessions.value.length)
    })

    it('应该正确计算各类型的总时长', () => {
      const sessions = ref([...mockFocusSessions])
      
      const workDuration = computed(() => 
        sessions.value
          .filter(s => s.session_type === 'work')
          .reduce((sum, s) => sum + s.duration_minutes, 0)
      )
      const breakDuration = computed(() => 
        sessions.value
          .filter(s => s.session_type !== 'work')
          .reduce((sum, s) => sum + s.duration_minutes, 0)
      )

      expect(workDuration.value).toBe(70)
      expect(breakDuration.value).toBe(20)
    })
  })

  describe('跨天时间处理', () => {
    it('应该正确处理跨天的专注会话', () => {
      const crossDaySessions = ref([
        { id: 1, project_id: 1, session_type: 'work', duration_minutes: 45, started_at: '2024-01-15T23:30:00', completed: true, created_at: '2024-01-16T00:15:00' },
        { id: 2, project_id: 1, session_type: 'work', duration_minutes: 25, started_at: '2024-01-16T00:30:00', completed: true, created_at: '2024-01-16T01:00:00' },
      ])

      const sortedSessions = computed(() => {
        return crossDaySessions.value
          .sort((a, b) => new Date(b.started_at).getTime() - new Date(a.started_at).getTime())
      })

      expect(sortedSessions.value[0].id).toBe(2)
      expect(sortedSessions.value[1].id).toBe(1)
    })

    it('应该正确计算跨天的日期分组', () => {
      const sessions = ref([
        { id: 1, project_id: 1, session_type: 'work', duration_minutes: 30, started_at: '2024-01-15T23:30:00', completed: true, created_at: '2024-01-16T00:00:00' },
        { id: 2, project_id: 1, session_type: 'work', duration_minutes: 30, started_at: '2024-01-16T00:30:00', completed: true, created_at: '2024-01-16T01:00:00' },
      ])

      // 按开始日期分组
      const groupedByDate = computed(() => {
        const groups: Record<string, typeof sessions.value> = {}
        sessions.value.forEach(session => {
          const date = session.started_at.split('T')[0]
          if (!groups[date]) groups[date] = []
          groups[date].push(session)
        })
        return groups
      })

      expect(Object.keys(groupedByDate.value)).toEqual(['2024-01-15', '2024-01-16'])
      expect(groupedByDate.value['2024-01-15'].length).toBe(1)
      expect(groupedByDate.value['2024-01-16'].length).toBe(1)
    })
  })
})
