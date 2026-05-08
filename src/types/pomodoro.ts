import { useEnumDictionary } from '../stores/enumDictionary'

export type SessionType = 'work' | 'short_break' | 'long_break';

export interface FocusSession {
  id: number;
  project_id: number;
  session_type: SessionType;
  duration_minutes: number;
  started_at: string;
  completed: boolean;
  created_at: string;
}

export interface FocusStats {
  total_sessions: number;
  total_minutes: number;
  completed_sessions: number;
  work_sessions: number;
  short_break_sessions: number;
  long_break_sessions: number;
  completed_work_sessions: number;
  work_duration_minutes: number;
}

export type PomodoroStatus = 'idle' | 'running' | 'paused';

export interface PomodoroSettings {
  workDuration: number;
  shortBreakDuration: number;
  longBreakDuration: number;
  longBreakInterval: number;
  autoStartBreaks: boolean;
  autoStartWork: boolean;
  soundEnabled: boolean;
  zenModeEnabled: boolean;
}

export interface PomodoroState {
  status: PomodoroStatus;
  currentSessionType: SessionType;
  timeRemaining: number;
  totalTime: number;
  completedPomodoros: number;
  currentStreak: number;
  todayTotalMinutes: number;
  todaySessions: number;
}

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

const SESSION_TYPE_NAME_MAP: Record<SessionType, string> = {
  work: '专注',
  short_break: '短休息',
  long_break: '长休息',
};

const SESSION_TYPE_COLOR_MAP: Record<SessionType, string> = {
  work: '#ef4444',
  short_break: '#22c55e',
  long_break: '#3b82f6',
};

export function getSessionTypeName(type: SessionType): string {
  try {
    const { getSessionTypeName: getDictName, isLoaded } = useEnumDictionary()
    if (isLoaded.value) {
      const name = getDictName(type)
      if (name !== type) return name
    }
  } catch {}
  return SESSION_TYPE_NAME_MAP[type] ?? type;
}

export function getSessionTypeColor(type: SessionType, _isDark?: boolean): string {
  return SESSION_TYPE_COLOR_MAP[type] ?? '#6b7280';
}

export function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

export function formatMinutes(minutes: number): string {
  if (minutes < 60) {
    return `${minutes}分钟`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return mins > 0 ? `${hours}小时${mins}分钟` : `${hours}小时`;
}
