// 番茄钟专注会话相关类型定义

/** 专注会话类型 */
export type SessionType = 'work' | 'short_break' | 'long_break';

/** 专注会话记录 */
export interface FocusSession {
  id: number;
  project_id: number;
  session_type: SessionType;
  duration_minutes: number;
  started_at: string;
  completed: boolean;
  created_at: string;
}

/** 专注统计数据 */
export interface FocusStats {
  total_sessions: number;
  total_minutes: number;
  completed_sessions: number;
  work_sessions: number;
  short_break_sessions: number;
  long_break_sessions: number;
}

/** 番茄钟状态 */
export type PomodoroStatus = 'idle' | 'running' | 'paused';

/** 番茄钟设置 */
export interface PomodoroSettings {
  workDuration: number;      // 工作时长（分钟），默认25
  shortBreakDuration: number; // 短休息时长（分钟），默认5
  longBreakDuration: number;  // 长休息时长（分钟），默认15
  longBreakInterval: number;  // 长休息间隔（工作次数），默认4
  autoStartBreaks: boolean;   // 自动开始休息
  autoStartWork: boolean;     // 自动开始工作
  soundEnabled: boolean;      // 声音提示
  zenModeEnabled: boolean;    // 专注模式（隐藏侧边栏）
}

/** 番茄钟状态信息 */
export interface PomodoroState {
  status: PomodoroStatus;
  currentSessionType: SessionType;
  timeRemaining: number;      // 剩余秒数
  totalTime: number;         // 总时间（秒）
  completedPomodoros: number; // 已完成的番茄数
  currentStreak: number;      // 当前连续专注次数
  todayTotalMinutes: number;  // 今日总专注时长（分钟）
  todaySessions: number;      // 今日专注次数
}

/** 默认设置 */
export const DEFAULT_POMODORO_SETTINGS: PomodoroSettings = {
  workDuration: 25,
  shortBreakDuration: 5,
  longBreakDuration: 15,
  longBreakInterval: 4,
  autoStartBreaks: false,
  autoStartWork: false,
  soundEnabled: true,
  zenModeEnabled: false,
};

/** 获取会话类型的中文名称 */
export function getSessionTypeName(type: SessionType): string {
  switch (type) {
    case 'work':
      return '专注';
    case 'short_break':
      return '短休息';
    case 'long_break':
      return '长休息';
  }
}

/** 获取会话类型对应的颜色 */
export function getSessionTypeColor(type: SessionType, isDark: boolean): string {
  switch (type) {
    case 'work':
      return '#ef4444'; // 红色
    case 'short_break':
      return '#22c55e'; // 绿色
    case 'long_break':
      return '#3b82f6'; // 蓝色
  }
}

/** 格式化时间为 MM:SS */
export function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

/** 格式化分钟为小时:分钟 */
export function formatMinutes(minutes: number): string {
  if (minutes < 60) {
    return `${minutes}分钟`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return mins > 0 ? `${hours}小时${mins}分钟` : `${hours}小时`;
}
