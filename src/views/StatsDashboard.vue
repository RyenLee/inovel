<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import { useMessage } from "naive-ui";
import { NCard, NGrid, NGi, NButton, NSpace, NSpin, NIcon } from "naive-ui";
import {
  ArrowLeft,
  TrendingUp,
  Clock,
  Target,
  Calendar,
  BarChart3,
  Timer,
  Coffee,
  Brain,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { useLocale } from "../i18n/composables/useLocale";

interface WritingRecord {
  date: string;
  total_words: number;
  duration: number;
}

interface FocusSession {
  id: number;
  project_id: number;
  session_type: string;
  duration_minutes: number;
  started_at: string;
  completed: boolean;
  created_at: string;
}

interface FocusStats {
  total_sessions: number;
  total_minutes: number;
  completed_sessions: number;
  work_sessions: number;
  short_break_sessions: number;
  long_break_sessions: number;
  completed_work_sessions: number;
  work_duration_minutes: number;
}

const router = useRouter();
const { t } = useLocale();
const isLoading = ref(true);
const writingRecords = ref<WritingRecord[]>([]);
const focusSessions = ref<FocusSession[]>([]);
const focusStats = ref<FocusStats>({
  total_sessions: 0,
  total_minutes: 0,
  completed_sessions: 0,
  work_sessions: 0,
  short_break_sessions: 0,
  long_break_sessions: 0,
  completed_work_sessions: 0,
  work_duration_minutes: 0,
});

onMounted(async () => {
  await loadStats();
});

const loadStats = async () => {
  isLoading.value = true;
  try {
    const records = await invoke<WritingRecord[]>("get_writing_stats", {
      project_id: 0,
      days: 30,
    });
    writingRecords.value = validateWritingRecords(records);

    try {
      const stats = await invoke<FocusStats>("get_focus_stats", {
        project_id: 0,
        days: 30,
      });

      if (!validateFocusStats(stats)) {
        console.warn("Focus stats validation failed, using sanitized values");
        focusStats.value = sanitizeFocusStats(stats);
      } else {
        focusStats.value = stats;
      }

      const sessions = await invoke<FocusSession[]>("get_focus_sessions", {
        project_id: 0,
        days: 30,
      });
      focusSessions.value = validateFocusSessions(sessions);
    } catch (e) {
      console.error("Failed to load focus stats:", e);
    }
  } catch (error) {
    console.error("Failed to load stats:", error);
  } finally {
    isLoading.value = false;
  }
};

// 写作记录数据校验
const validateWritingRecords = (records: WritingRecord[]): WritingRecord[] => {
  return records.filter((record) => {
    if (!record.date) return false;
    if (!isValidDate(record.date)) {
      console.warn("Invalid writing record date:", record);
      return false;
    }
    if (record.total_words < 0) {
      console.warn("Negative word count:", record);
      record.total_words = 0;
    }
    if (record.duration < 0) {
      console.warn("Negative duration:", record);
      record.duration = 0;
    }
    return true;
  });
};

// 日期格式校验
const isValidDate = (dateStr: string): boolean => {
  const date = new Date(dateStr);
  return !isNaN(date.getTime());
};

// 专注会话数据校验
const validateFocusSessions = (sessions: FocusSession[]): FocusSession[] => {
  return sessions.filter((session) => {
    // 校验ID
    if (!session.id || session.id <= 0) {
      console.warn("Invalid session ID:", session);
      return false;
    }
    // 校验开始时间
    if (!session.started_at || !isValidDate(session.started_at)) {
      console.warn("Invalid session started_at:", session);
      return false;
    }
    // 校验收录时长（非负）
    if (session.duration_minutes < 0) {
      console.warn("Negative session duration:", session);
      session.duration_minutes = 0;
    }
    // 校验会话类型
    const validTypes = ["work", "short_break", "long_break"];
    if (!validTypes.includes(session.session_type)) {
      console.warn("Invalid session type:", session);
      return false;
    }
    return true;
  });
};

// 专注统计数据校验
const validateFocusStats = (stats: FocusStats): boolean => {
  // 检查负数
  if (
    stats.total_sessions < 0 ||
    stats.total_minutes < 0 ||
    stats.completed_sessions < 0
  ) {
    console.warn("Focus stats contains negative values", stats);
    return false;
  }
  if (
    stats.work_sessions < 0 ||
    stats.short_break_sessions < 0 ||
    stats.long_break_sessions < 0
  ) {
    console.warn("Focus stats contains negative session counts", stats);
    return false;
  }
  if (stats.completed_work_sessions < 0 || stats.work_duration_minutes < 0) {
    console.warn("Focus stats contains negative work values", stats);
    return false;
  }
  // 检查完成数不能超过总数
  if (stats.completed_sessions > stats.total_sessions) {
    console.warn("Completed sessions exceeds total sessions", stats);
    return false;
  }
  if (stats.completed_work_sessions > stats.work_sessions) {
    console.warn("Completed work sessions exceeds work sessions", stats);
    return false;
  }
  // 检查会话类型计数总和
  const totalTypeCount =
    stats.work_sessions +
    stats.short_break_sessions +
    stats.long_break_sessions;
  if (totalTypeCount !== stats.total_sessions) {
    console.warn("Session type count mismatch", stats);
    return false;
  }
  return true;
};

// 统计数据清理（将无效数据转为安全值）
const sanitizeFocusStats = (stats: FocusStats): FocusStats => {
  return {
    total_sessions: Math.max(stats.total_sessions, 0),
    total_minutes: Math.max(stats.total_minutes, 0),
    completed_sessions: Math.min(
      Math.max(stats.completed_sessions, 0),
      stats.total_sessions
    ),
    work_sessions: Math.max(stats.work_sessions, 0),
    short_break_sessions: Math.max(stats.short_break_sessions, 0),
    long_break_sessions: Math.max(stats.long_break_sessions, 0),
    completed_work_sessions: Math.min(
      Math.max(stats.completed_work_sessions, 0),
      stats.work_sessions
    ),
    work_duration_minutes: Math.max(stats.work_duration_minutes, 0),
  };
};

const totalWordsThisMonth = computed(() => {
  return writingRecords.value.reduce((sum, r) => sum + r.total_words, 0);
});

const totalDays = computed(() => {
  return writingRecords.value.length;
});

const averageWordsPerDay = computed(() => {
  if (totalDays.value === 0) return 0;
  return Math.round(totalWordsThisMonth.value / totalDays.value);
});

const totalDuration = computed(() => {
  return writingRecords.value.reduce((sum, r) => sum + r.duration, 0);
});

const maxWordsInDay = computed(() => {
  if (writingRecords.value.length === 0) return 0;
  return Math.max(...writingRecords.value.map((r) => r.total_words));
});

// Format date to YYYY-MM-DD using local time (not UTC)
const formatDateToYMD = (date: Date): string => {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
};

const heatmapData = computed(() => {
  const data: { date: string; value: number; level: number }[] = [];
  const today = new Date();

  for (let i = 83; i >= 0; i--) {
    const date = new Date(today);
    date.setDate(date.getDate() - i);
    const dateStr = formatDateToYMD(date);

    const record = writingRecords.value.find((r) => r.date === dateStr);
    const value = record?.total_words || 0;

    let level = 0;
    if (value > 0 && maxWordsInDay.value > 0) {
      const ratio = value / maxWordsInDay.value;
      if (ratio <= 0.25) level = 1;
      else if (ratio <= 0.5) level = 2;
      else if (ratio <= 0.75) level = 3;
      else level = 4;
    }

    data.push({ date: dateStr, value, level });
  }

  return data;
});

const lineChartData = computed(() => {
  return writingRecords.value
    .slice(0, 30)
    .reverse()
    .map((r) => ({
      date: r.date.slice(5),
      words: r.total_words,
    }));
});

const lineChartPath = computed(() => {
  if (lineChartData.value.length < 2)
    return {
      pathD: "",
      areaD: "",
      width: 500,
      height: 150,
      padding: 30,
      maxValue: 1,
      points: [],
    };

  const width = 500;
  const height = 150;
  const padding = 30;
  const maxValue = Math.max(...lineChartData.value.map((d) => d.words), 1);

  const points = lineChartData.value.map((d, i) => {
    const x =
      padding + (i / (lineChartData.value.length - 1)) * (width - padding * 2);
    const y = height - padding - (d.words / maxValue) * (height - padding * 2);
    return { x, y };
  });

  const pathD = points
    .map((p, i) => `${i === 0 ? "M" : "L"} ${p.x} ${p.y}`)
    .join(" ");
  const areaD =
    pathD +
    ` L ${points[points.length - 1].x} ${height - padding} L ${points[0].x} ${
      height - padding
    } Z`;

  return { pathD, areaD, width, height, padding, maxValue, points };
});

const goBack = () => {
  router.push("/");
};

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr);
  // 处理无效日期
  if (isNaN(date.getTime())) {
    return dateStr;
  }
  return `${date.getMonth() + 1}/${date.getDate()}`;
};

// Focus session computed properties
// 专注时长：仅统计工作会话的时长（精确到分钟）
const totalFocusMinutes = computed(() => {
  // 确保非负
  return Math.max(focusStats.value.work_duration_minutes || 0, 0);
});

// 完成番茄数：仅统计完成的工作会话
const completedPomodoros = computed(() => {
  // 确保非负且不超过工作会话总数
  const completed = focusStats.value.completed_work_sessions || 0;
  const totalWork = focusStats.value.work_sessions || 0;
  return Math.max(0, Math.min(completed, totalWork));
});

// 格式化专注时长显示
const formatFocusTime = (minutes: number): string => {
  const mins = Math.max(Math.round(minutes), 0);

  if (mins < 60) {
    return `${mins}${t("stats.minutes")}`;
  }
  const hours = Math.floor(mins / 60);
  const remainingMins = mins % 60;
  return remainingMins > 0
    ? `${hours}${t("stats.hours")}${remainingMins}${t("stats.minutes")}`
    : `${hours}${t("stats.hours")}`;
};

// 最近专注记录，已排序并添加数据校验
const recentFocusSessions = computed(() => {
  // 使用已校验过的数据
  const validSessions = focusSessions.value;

  // 按开始时间降序排序（最新的在前）
  // 使用时间戳确保跨天时间比较正确
  return validSessions
    .sort((a, b) => {
      const timeA = new Date(a.started_at).getTime();
      const timeB = new Date(b.started_at).getTime();
      return timeB - timeA;
    })
    .slice(0, 5);
});

// 专注热力图数据
const focusHeatmapData = computed(() => {
  const data: { date: string; value: number; level: number }[] = [];
  const today = new Date();

  // 预先计算所有日期的总分钟数
  const dateToMinutes: Record<string, number> = {};
  for (let j = 83; j >= 0; j--) {
    const d = new Date(today);
    d.setDate(d.getDate() - j);
    const dStr = formatDateToYMD(d);
    const daySessions = focusSessions.value.filter((s) => {
      // 使用字符串前缀匹配确保日期正确分组
      return s.started_at.startsWith(dStr);
    });
    dateToMinutes[dStr] = daySessions.reduce(
      (sum, s) => sum + s.duration_minutes,
      0
    );
  }

  const maxMinutes = Math.max(...Object.values(dateToMinutes), 60);

  for (let i = 83; i >= 0; i--) {
    const date = new Date(today);
    date.setDate(date.getDate() - i);
    const dateStr = formatDateToYMD(date);
    const value = dateToMinutes[dateStr] || 0;

    let level = 0;
    if (value > 0) {
      if (value <= 25) level = 1;
      else if (value <= 50) level = 2;
      else if (value <= 100) level = 3;
      else level = 4;
    }

    data.push({ date: dateStr, value, level });
  }

  return data;
});

const getSessionTypeLabel = (type: string) => {
  const nameMap: Record<string, string> = {
    work: t("stats.sessionTypes.work"),
    short_break: t("stats.sessionTypes.shortBreak"),
    long_break: t("stats.sessionTypes.longBreak"),
  };
  return nameMap[type] ?? type;
};

const getSessionTypeColor = (type: string) => {
  const colorMap: Record<string, string> = {
    work: "#ef4444",
    short_break: "#22c55e",
    long_break: "#3b82f6",
  };
  return colorMap[type] ?? "#6b7280";
};
</script>

<template>
  <div
    class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300"
  >
    <header
      class="border-b bg-white dark:bg-gray-800 dark:border-gray-700 transition-colors duration-300"
    >
      <div class="max-w-5xl mx-auto px-4 py-4 flex items-center gap-4">
        <n-button quaternary circle @click="goBack">
          <template #icon>
            <NIcon>
              <ArrowLeft />
            </NIcon>
          </template>
        </n-button>
        <BarChart3 class="w-6 h-6 text-blue-600" />
        <h1 class="text-xl font-bold text-gray-900 dark:text-white">
          {{ t("stats.title") }}
        </h1>
      </div>
    </header>

    <main class="max-w-5xl mx-auto px-4 py-8">
      <div v-if="isLoading" class="flex justify-center py-12">
        <n-spin size="large" />
      </div>

      <n-grid
        v-else
        :cols="4"
        :x-gap="16"
        :y-gap="16"
        responsive="screen"
        :item-responsive="true"
      >
        <!-- Total Words -->
        <n-gi span="0:24 640:12 1024:6">
          <n-card hoverable>
            <div class="flex items-center gap-3 whitespace-nowrap">
              <div
                class="p-3 rounded-full bg-blue-100 dark:bg-blue-900/30 shrink-0"
              >
                <TrendingUp class="w-6 h-6 text-blue-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {{ t("stats.monthlyWords") }}
                </p>
                <p
                  class="text-2xl font-bold text-gray-900 dark:text-white truncate"
                >
                  {{ totalWordsThisMonth.toLocaleString() }}
                </p>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Average Words Per Day -->
        <n-gi span="0:24 640:12 1024:6">
          <n-card hoverable>
            <div class="flex items-center gap-3 whitespace-nowrap">
              <div
                class="p-3 rounded-full bg-green-100 dark:bg-green-900/30 shrink-0"
              >
                <Target class="w-6 h-6 text-green-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {{ t("stats.avgDailyWords") }}
                </p>
                <p
                  class="text-2xl font-bold text-gray-900 dark:text-white truncate"
                >
                  {{ averageWordsPerDay.toLocaleString() }}
                </p>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Total Duration -->
        <n-gi span="0:24 640:12 1024:6">
          <n-card hoverable>
            <div class="flex items-center gap-3 whitespace-nowrap">
              <div
                class="p-3 rounded-full bg-purple-100 dark:bg-purple-900/30 shrink-0"
              >
                <Clock class="w-6 h-6 text-purple-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {{ t("stats.writingDuration") }}
                </p>
                <p
                  class="text-2xl font-bold text-gray-900 dark:text-white truncate"
                >
                  {{ formatFocusTime(totalDuration) }}
                </p>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Writing Days -->
        <n-gi span="0:24 640:12 1024:6">
          <n-card hoverable>
            <div class="flex items-center gap-3 whitespace-nowrap">
              <div
                class="p-3 rounded-full bg-orange-100 dark:bg-orange-900/30 shrink-0"
              >
                <Calendar class="w-6 h-6 text-orange-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {{ t("stats.writingDays") }}
                </p>
                <p
                  class="text-2xl font-bold text-gray-900 dark:text-white truncate"
                >
                  {{ totalDays }} {{ t("stats.days") }}
                </p>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Total Focus Minutes -->
        <n-gi span="0:24 640:12 1024:6">
          <n-card hoverable>
            <div class="flex items-center gap-3 whitespace-nowrap">
              <div
                class="p-3 rounded-full bg-red-100 dark:bg-red-900/30 shrink-0"
              >
                <Timer class="w-6 h-6 text-red-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {{ t("stats.focusDuration") }}
                </p>
                <p
                  class="text-2xl font-bold text-gray-900 dark:text-white truncate"
                >
                  {{ formatFocusTime(totalFocusMinutes) }}
                </p>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Completed Pomodoros -->
        <n-gi span="0:24 640:12 1024:6">
          <n-card hoverable>
            <div class="flex items-center gap-3 whitespace-nowrap">
              <div
                class="p-3 rounded-full bg-pink-100 dark:bg-pink-900/30 shrink-0"
              >
                <Brain class="w-6 h-6 text-pink-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-sm text-gray-500 dark:text-gray-400">
                  {{ t("stats.completedPomodoros") }}
                </p>
                <p
                  class="text-2xl font-bold text-gray-900 dark:text-white truncate"
                >
                  {{ completedPomodoros }}
                </p>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Line Chart -->
        <n-gi span="0:24 640:24 1024:24">
          <n-card :title="t('stats.trend30Days')" hoverable>
            <div
              v-if="lineChartData.length < 2"
              class="h-48 flex items-center justify-center text-gray-400"
            >
              {{ t("stats.noData") }}
            </div>
            <svg
              v-else
              :viewBox="`0 0 ${lineChartPath.width} ${lineChartPath.height}`"
              class="w-full h-48"
            >
              <line
                v-for="i in 4"
                :key="'grid-' + i"
                :x1="lineChartPath.padding"
                :y1="
                  lineChartPath.padding +
                  ((i - 1) *
                    (lineChartPath.height - lineChartPath.padding * 2)) /
                    3
                "
                :x2="lineChartPath.width - lineChartPath.padding"
                :y2="
                  lineChartPath.padding +
                  ((i - 1) *
                    (lineChartPath.height - lineChartPath.padding * 2)) /
                    3
                "
                stroke="currentColor"
                stroke-opacity="0.1"
              />
              <text
                v-for="i in 4"
                :key="'label-' + i"
                :x="lineChartPath.padding - 5"
                :y="
                  lineChartPath.padding +
                  ((i - 1) *
                    (lineChartPath.height - lineChartPath.padding * 2)) /
                    3 +
                  4
                "
                text-anchor="end"
                class="fill-gray-400 text-xs"
              >
                {{ Math.round((lineChartPath.maxValue * (4 - i)) / 3) }}
              </text>
              <path
                :d="lineChartPath.areaD"
                fill="url(#gradient)"
                opacity="0.3"
              />
              <path
                :d="lineChartPath.pathD"
                fill="none"
                stroke="#3b82f6"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <circle
                v-for="(point, i) in lineChartPath.points"
                :key="'point-' + i"
                :cx="point.x"
                :cy="point.y"
                r="3"
                fill="#3b82f6"
              />
              <text
                v-for="(d, i) in lineChartData"
                :key="'xlabel-' + i"
                v-show="i % 5 === 0"
                :x="
                  lineChartPath.padding +
                  (i / (lineChartData.length - 1)) *
                    (lineChartPath.width - lineChartPath.padding * 2)
                "
                :y="lineChartPath.height - 5"
                text-anchor="middle"
                class="fill-gray-400 text-xs"
              >
                {{ d.date }}
              </text>
              <defs>
                <linearGradient id="gradient" x1="0%" y1="0%" x2="0%" y2="100%">
                  <stop offset="0%" stop-color="#3b82f6" stop-opacity="0.4" />
                  <stop offset="100%" stop-color="#3b82f6" stop-opacity="0" />
                </linearGradient>
              </defs>
            </svg>
          </n-card>
        </n-gi>

        <!-- Calendar Heatmap -->
        <n-gi span="0:24 640:24 1024:24">
          <n-card :title="t('stats.writingHeatmap')" hoverable>
            <div class="overflow-x-auto">
              <div class="flex gap-1 mb-2">
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.mon") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.tue") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.wed") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.thu") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.fri") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.sat") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.sun") }}
                </div>
              </div>
              <div class="grid grid-rows-12 grid-flow-col gap-1">
                <div
                  v-for="(day, index) in heatmapData"
                  :key="index"
                  class="w-3 h-3 rounded-sm transition-colors"
                  :class="{
                    'bg-gray-100 dark:bg-gray-700': day.level === 0,
                    'bg-green-200 dark:bg-green-900': day.level === 1,
                    'bg-green-400 dark:bg-green-700': day.level === 2,
                    'bg-green-500 dark:bg-green-600': day.level === 3,
                    'bg-green-600 dark:bg-green-500': day.level === 4,
                  }"
                  :title="`${day.date}: ${day.value} ${t('stats.words')}`"
                />
              </div>
              <div class="flex items-center gap-2 mt-4 text-xs text-gray-500">
                <span>{{ t("stats.heatmap.less") }}</span>
                <div class="flex gap-1">
                  <div
                    class="w-3 h-3 rounded-sm bg-gray-100 dark:bg-gray-700"
                  />
                  <div
                    class="w-3 h-3 rounded-sm bg-green-200 dark:bg-green-900"
                  />
                  <div
                    class="w-3 h-3 rounded-sm bg-green-400 dark:bg-green-700"
                  />
                  <div
                    class="w-3 h-3 rounded-sm bg-green-500 dark:bg-green-600"
                  />
                  <div
                    class="w-3 h-3 rounded-sm bg-green-600 dark:bg-green-500"
                  />
                </div>
                <span>{{ t("stats.heatmap.more") }}</span>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Recent Records -->
        <n-gi span="0:24 640:24 1024:24">
          <n-card :title="t('stats.recentRecords')" hoverable>
            <div
              v-if="writingRecords.length === 0"
              class="py-8 text-center text-gray-400"
            >
              {{ t("stats.noWritingRecords") }}
            </div>
            <div v-else>
              <div
                v-for="record in writingRecords.slice(0, 10)"
                :key="record.date"
                class="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-700 last:border-0"
              >
                <span class="text-sm text-gray-600 dark:text-gray-400">
                  {{ formatDate(record.date) }}
                </span>
                <span class="font-medium text-gray-900 dark:text-white">
                  {{ record.total_words.toLocaleString() }}
                  {{ t("stats.words") }}
                </span>
                <span class="text-sm text-gray-400">
                  {{ record.duration }} {{ t("stats.minutes") }}
                </span>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Focus Session Heatmap -->
        <n-gi span="0:24 640:24 1024:24">
          <n-card :title="t('stats.focusHeatmap')" hoverable>
            <div
              v-if="focusSessions.length === 0"
              class="py-8 text-center text-gray-400"
            >
              {{ t("stats.noFocusRecords") }}
            </div>
            <div v-else class="overflow-x-auto">
              <div class="flex gap-1 mb-2">
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.mon") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.tue") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.wed") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.thu") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.fri") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.sat") }}
                </div>
                <div class="text-xs text-gray-400 w-12">
                  {{ t("stats.weekdays.sun") }}
                </div>
              </div>
              <div class="grid grid-rows-12 grid-flow-col gap-1">
                <div
                  v-for="(day, index) in focusHeatmapData"
                  :key="index"
                  class="w-3 h-3 rounded-sm transition-colors"
                  :class="{
                    'bg-gray-100 dark:bg-gray-700': day.level === 0,
                    'bg-red-200 dark:bg-red-900': day.level === 1,
                    'bg-red-400 dark:bg-red-700': day.level === 2,
                    'bg-red-500 dark:bg-red-600': day.level === 3,
                    'bg-red-600 dark:bg-red-500': day.level === 4,
                  }"
                  :title="`${day.date}: ${day.value} ${t('stats.minutes')}`"
                />
              </div>
              <div class="flex items-center gap-2 mt-4 text-xs text-gray-500">
                <span>{{ t("stats.heatmap.less") }}</span>
                <div class="flex gap-1">
                  <div
                    class="w-3 h-3 rounded-sm bg-gray-100 dark:bg-gray-700"
                  />
                  <div class="w-3 h-3 rounded-sm bg-red-200 dark:bg-red-900" />
                  <div class="w-3 h-3 rounded-sm bg-red-400 dark:bg-red-700" />
                  <div class="w-3 h-3 rounded-sm bg-red-500 dark:bg-red-600" />
                  <div class="w-3 h-3 rounded-sm bg-red-600 dark:bg-red-500" />
                </div>
                <span>{{ t("stats.heatmap.more") }}</span>
              </div>
            </div>
          </n-card>
        </n-gi>

        <!-- Recent Focus Sessions -->
        <n-gi span="0:24 640:24 1024:24">
          <n-card :title="t('stats.recentFocusRecords')" hoverable>
            <div
              v-if="recentFocusSessions.length === 0"
              class="py-8 text-center text-gray-400"
            >
              {{ t("stats.noFocusRecords") }}
            </div>
            <div v-else>
              <div
                v-for="session in recentFocusSessions"
                :key="session.id"
                class="flex items-center justify-between py-2 border-b border-gray-100 dark:border-gray-700 last:border-0"
              >
                <span class="text-sm text-gray-600 dark:text-gray-400">
                  {{ formatDate(session.started_at) }}
                </span>
                <span
                  class="px-2 py-0.5 rounded text-xs font-medium"
                  :style="{
                    backgroundColor:
                      getSessionTypeColor(session.session_type) + '20',
                    color: getSessionTypeColor(session.session_type),
                  }"
                >
                  {{ getSessionTypeLabel(session.session_type) }}
                </span>
                <span class="font-medium text-gray-900 dark:text-white">
                  {{ session.duration_minutes }} {{ t("stats.minutes") }}
                </span>
                <span v-if="session.completed" class="text-xs text-green-500"
                  >✓</span
                >
              </div>
            </div>
          </n-card>
        </n-gi>
      </n-grid>
    </main>
  </div>
</template>
