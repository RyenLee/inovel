<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
    NCard,
    NGrid,
    NGi,
    NButton,
    NSpin,
    NIcon,
} from "naive-ui";
import { ArrowLeft, TrendingUp, Clock, Calendar, BarChart3, PieChart } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "../stores/project";

interface WritingRecord {
    date: string;
    total_words: number;
    duration: number;
}

interface ChapterStatusCount {
    status: string;
    count: number;
}

const route = useRoute();
const router = useRouter();
const projectStore = useProjectStore();

const projectId = computed(() => Number(route.params.projectId));
const isLoading = ref(true);
const writingRecords = ref<WritingRecord[]>([]);
const chapterStatusCounts = ref<ChapterStatusCount[]>([]);

const projectName = computed(() => {
    return projectStore.currentProject?.name || "项目";
});

onMounted(async () => {
    if (!projectStore.currentProject || projectStore.currentProject.id !== projectId.value) {
        await projectStore.openProject(projectId.value);
    }
    await loadStats();
});

const loadStats = async () => {
    isLoading.value = true;
    try {
        const [records, statusCounts] = await Promise.all([
            invoke<WritingRecord[]>("get_writing_stats", {
                projectId: projectId.value,
                days: 30,
            }),
            invoke<ChapterStatusCount[]>("get_chapter_status_counts", {
                projectId: projectId.value,
            }),
        ]);
        writingRecords.value = records;
        chapterStatusCounts.value = statusCounts;
    } catch (error) {
        console.error("Failed to load stats:", error);
    } finally {
        isLoading.value = false;
    }
};

// Chapter status configuration
const STATUS_CONFIG = {
    outline: { label: '大纲', color: '#9CA3AF' },
    draft: { label: '草稿', color: '#F59E0B' },
    revised: { label: '修订', color: '#3B82F6' },
    final: { label: '定稿', color: '#10B981' },
    abandoned: { label: '废弃', color: '#EF4444' },
} as const;

const totalChapters = computed(() => {
    return chapterStatusCounts.value.reduce((sum, item) => sum + item.count, 0);
});

// Generate pie chart data
const pieChartData = computed(() => {
    const allStatuses = ['outline', 'draft', 'revised', 'final', 'abandoned'];
    const total = totalChapters.value || 1; // Avoid division by zero

    return allStatuses.map(status => {
        const countItem = chapterStatusCounts.value.find(c => c.status === status);
        const count = countItem?.count || 0;
        const percentage = Math.round((count / total) * 100);

        return {
            status,
            label: STATUS_CONFIG[status as keyof typeof STATUS_CONFIG].label,
            color: STATUS_CONFIG[status as keyof typeof STATUS_CONFIG].color,
            count,
            percentage,
        };
    });
});

// SVG pie chart calculations
const pieChartSize = 120;
const pieChartRadius = 40;
const pieChartCenter = pieChartSize / 2;

const pieChartPaths = computed(() => {
    const total = totalChapters.value || 1;
    let currentAngle = -90; // Start from top

    return pieChartData.value.map(item => {
        const angle = (item.count / total) * 360;
        const startAngle = currentAngle;
        const endAngle = currentAngle + angle;
        currentAngle = endAngle;

        if (item.count === 0) {
            return null;
        }

        const startRad = (startAngle * Math.PI) / 180;
        const endRad = (endAngle * Math.PI) / 180;

        const x1 = pieChartCenter + pieChartRadius * Math.cos(startRad);
        const y1 = pieChartCenter + pieChartRadius * Math.sin(startRad);
        const x2 = pieChartCenter + pieChartRadius * Math.cos(endRad);
        const y2 = pieChartCenter + pieChartRadius * Math.sin(endRad);

        const largeArc = angle > 180 ? 1 : 0;

        if (angle >= 360) {
            // Full circle
            return {
                d: `M ${pieChartCenter} ${pieChartCenter - pieChartRadius}
                    A ${pieChartRadius} ${pieChartRadius} 0 1 1 ${pieChartCenter - 0.01} ${pieChartCenter - pieChartRadius}
                    A ${pieChartRadius} ${pieChartRadius} 0 1 1 ${pieChartCenter} ${pieChartCenter - pieChartRadius}`,
                fill: item.color,
            };
        }

        return {
            d: `M ${pieChartCenter} ${pieChartCenter}
                L ${x1} ${y1}
                A ${pieChartRadius} ${pieChartRadius} 0 ${largeArc} 1 ${x2} ${y2}
                Z`,
            fill: item.color,
        };
    }).filter(Boolean);
});

const totalWordsThisMonth = computed(() => {
    return writingRecords.value.reduce((sum, r) => sum + r.total_words, 0);
});

const totalDays = computed(() => writingRecords.value.length);

const averageWordsPerDay = computed(() => {
    if (totalDays.value === 0) return 0;
    return Math.round(totalWordsThisMonth.value / totalDays.value);
});

const totalDuration = computed(() => {
    return writingRecords.value.reduce((sum, r) => sum + r.duration, 0);
});

const maxWordsInDay = computed(() => {
    if (writingRecords.value.length === 0) return 0;
    return Math.max(...writingRecords.value.map(r => r.total_words));
});

const heatmapData = computed(() => {
    const data: { date: string; value: number; level: number }[] = [];
    const today = new Date();

    for (let i = 83; i >= 0; i--) {
        const date = new Date(today);
        date.setDate(date.getDate() - i);
        const dateStr = date.toISOString().split('T')[0];

        const record = writingRecords.value.find(r => r.date === dateStr);
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
        .map(r => ({
            date: r.date.slice(5),
            words: r.total_words,
        }));
});

const lineChartPath = computed(() => {
    if (lineChartData.value.length < 2) return { pathD: "", areaD: "", width: 500, height: 150, padding: 30, maxValue: 1, points: [] };

    const width = 500;
    const height = 150;
    const padding = 30;
    const maxValue = Math.max(...lineChartData.value.map(d => d.words), 1);

    const points = lineChartData.value.map((d, i) => {
        const x = padding + (i / (lineChartData.value.length - 1)) * (width - padding * 2);
        const y = height - padding - (d.words / maxValue) * (height - padding * 2);
        return { x, y };
    });

    const pathD = points.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');
    const areaD = pathD + ` L ${points[points.length - 1].x} ${height - padding} L ${points[0].x} ${height - padding} Z`;

    return { pathD, areaD, width, height, padding, maxValue, points };
});

const goBack = () => {
    router.push(`/editor/${projectId.value}`);
};

const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return `${date.getMonth() + 1}/${date.getDate()}`;
};
</script>

<template>
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300">
        <header class="border-b bg-white dark:bg-gray-800 dark:border-gray-700 transition-colors duration-300">
            <div class="max-w-5xl mx-auto px-4 py-4 flex items-center gap-4">
                <n-button quaternary circle @click="goBack">
                    <template #icon>
                        <NIcon>
                            <ArrowLeft />
                        </NIcon>
                    </template>
                </n-button>
                <BarChart3 class="w-6 h-6 text-blue-600" />
                <h1 class="text-xl font-bold text-gray-900 dark:text-white">项目统计</h1>
                <span class="text-sm text-gray-500 dark:text-gray-400">{{ projectName }}</span>
            </div>
        </header>

        <main class="max-w-5xl mx-auto px-4 py-8">
            <div v-if="isLoading" class="flex justify-center py-12">
                <n-spin size="large" />
            </div>

            <n-grid v-else :cols="4" :x-gap="16" :y-gap="16" responsive="screen" :item-responsive="true">
                <!-- Total Words -->
                <n-gi span="0:24 640:12 1024:6">
                    <n-card hoverable>
                        <div class="flex items-center gap-3 whitespace-nowrap">
                            <div class="p-3 rounded-full bg-blue-100 dark:bg-blue-900/30 flex-shrink-0">
                                <TrendingUp class="w-6 h-6 text-blue-600" />
                            </div>
                            <div class="overflow-hidden">
                                <p class="text-sm text-gray-500 dark:text-gray-400">本月字数</p>
                                <p class="text-2xl font-bold text-gray-900 dark:text-white truncate">{{ totalWordsThisMonth.toLocaleString() }}</p>
                            </div>
                        </div>
                    </n-card>
                </n-gi>

                <!-- Average Words Per Day -->
                <n-gi span="0:24 640:12 1024:6">
                    <n-card hoverable>
                        <div class="flex items-center gap-3 whitespace-nowrap">
                            <div class="p-3 rounded-full bg-green-100 dark:bg-green-900/30 flex-shrink-0">
                                <BarChart3 class="w-6 h-6 text-green-600" />
                            </div>
                            <div class="overflow-hidden">
                                <p class="text-sm text-gray-500 dark:text-gray-400">日均字数</p>
                                <p class="text-2xl font-bold text-gray-900 dark:text-white truncate">{{ averageWordsPerDay.toLocaleString() }}</p>
                            </div>
                        </div>
                    </n-card>
                </n-gi>

                <!-- Total Duration -->
                <n-gi span="0:24 640:12 1024:6">
                    <n-card hoverable>
                        <div class="flex items-center gap-3 whitespace-nowrap">
                            <div class="p-3 rounded-full bg-purple-100 dark:bg-purple-900/30 flex-shrink-0">
                                <Clock class="w-6 h-6 text-purple-600" />
                            </div>
                            <div class="overflow-hidden">
                                <p class="text-sm text-gray-500 dark:text-gray-400">写作时长</p>
                                <p class="text-2xl font-bold text-gray-900 dark:text-white truncate">{{ totalDuration }} 分钟</p>
                            </div>
                        </div>
                    </n-card>
                </n-gi>

                <!-- Writing Days -->
                <n-gi span="0:24 640:12 1024:6">
                    <n-card hoverable>
                        <div class="flex items-center gap-3 whitespace-nowrap">
                            <div class="p-3 rounded-full bg-orange-100 dark:bg-orange-900/30 flex-shrink-0">
                                <Calendar class="w-6 h-6 text-orange-600" />
                            </div>
                            <div class="overflow-hidden">
                                <p class="text-sm text-gray-500 dark:text-gray-400">写作天数</p>
                                <p class="text-2xl font-bold text-gray-900 dark:text-white truncate">{{ totalDays }} 天</p>
                            </div>
                        </div>
                    </n-card>
                </n-gi>

                <!-- Line Chart -->
                <n-gi span="0:24 640:24 1024:24">
                    <n-card title="近30天码字趋势" hoverable>
                        <div v-if="lineChartData.length < 2" class="h-48 flex items-center justify-center text-gray-400">
                            暂无数据
                        </div>
                        <svg v-else :viewBox="`0 0 ${lineChartPath.width} ${lineChartPath.height}`" class="w-full h-48">
                            <line
                                v-for="i in 4"
                                :key="'grid-' + i"
                                :x1="lineChartPath.padding"
                                :y1="lineChartPath.padding + (i - 1) * (lineChartPath.height - lineChartPath.padding * 2) / 3"
                                :x2="lineChartPath.width - lineChartPath.padding"
                                :y2="lineChartPath.padding + (i - 1) * (lineChartPath.height - lineChartPath.padding * 2) / 3"
                                stroke="currentColor"
                                stroke-opacity="0.1"
                            />
                            <text
                                v-for="i in 4"
                                :key="'label-' + i"
                                :x="lineChartPath.padding - 5"
                                :y="lineChartPath.padding + (i - 1) * (lineChartPath.height - lineChartPath.padding * 2) / 3 + 4"
                                text-anchor="end"
                                class="fill-gray-400 text-xs"
                            >
                                {{ Math.round(lineChartPath.maxValue * (4 - i) / 3) }}
                            </text>
                            <path :d="lineChartPath.areaD" fill="url(#gradient)" opacity="0.3" />
                            <path :d="lineChartPath.pathD" fill="none" stroke="#3b82f6" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
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
                                :x="lineChartPath.padding + (i / (lineChartData.length - 1)) * (lineChartPath.width - lineChartPath.padding * 2)"
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
                    <n-card title="写作热力图" hoverable>
                        <div class="overflow-x-auto">
                            <div class="flex gap-1 mb-2">
                                <div class="text-xs text-gray-400 w-12">周一</div>
                                <div class="text-xs text-gray-400 w-12">周二</div>
                                <div class="text-xs text-gray-400 w-12">周三</div>
                                <div class="text-xs text-gray-400 w-12">周四</div>
                                <div class="text-xs text-gray-400 w-12">周五</div>
                                <div class="text-xs text-gray-400 w-12">周六</div>
                                <div class="text-xs text-gray-400 w-12">周日</div>
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
                                    :title="`${day.date}: ${day.value} 字`"
                                />
                            </div>
                            <div class="flex items-center gap-2 mt-4 text-xs text-gray-500">
                                <span>少</span>
                                <div class="flex gap-1">
                                    <div class="w-3 h-3 rounded-sm bg-gray-100 dark:bg-gray-700" />
                                    <div class="w-3 h-3 rounded-sm bg-green-200 dark:bg-green-900" />
                                    <div class="w-3 h-3 rounded-sm bg-green-400 dark:bg-green-700" />
                                    <div class="w-3 h-3 rounded-sm bg-green-500 dark:bg-green-600" />
                                    <div class="w-3 h-3 rounded-sm bg-green-600 dark:bg-green-500" />
                                </div>
                                <span>多</span>
                            </div>
                        </div>
                    </n-card>
                </n-gi>

                <!-- Recent Records -->
                <n-gi span="0:24 640:24 1024:24">
                    <n-card title="最近记录" hoverable>
                        <div v-if="writingRecords.length === 0" class="py-8 text-center text-gray-400">
                            暂无写作记录，开始写作后会自动记录
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
                                    {{ record.total_words.toLocaleString() }} 字
                                </span>
                                <span class="text-sm text-gray-400">
                                    {{ record.duration }} 分钟
                                </span>
                            </div>
                        </div>
                    </n-card>
                </n-gi>

                <!-- Chapter Status Distribution -->
                <n-gi span="0:24 640:24 1024:12">
                    <n-card title="章节状态分布" hoverable>
                        <div class="flex flex-col items-center">
                            <div v-if="totalChapters === 0" class="py-8 text-center text-gray-400">
                                暂无章节
                            </div>
                            <template v-else>
                                <!-- Pie Chart -->
                                <svg :width="pieChartSize" :height="pieChartSize" class="mb-4">
                                    <g v-for="(path, index) in pieChartPaths" :key="index">
                                        <path :d="path?.d || ''" :fill="path?.fill" stroke="white" stroke-width="2"/>
                                    </g>
                                    <circle :cx="pieChartCenter" :cy="pieChartCenter" r="20" fill="white" />
                                    <text :x="pieChartCenter" :y="pieChartCenter + 4" text-anchor="middle" class="text-xs font-medium fill-gray-600 dark:fill-gray-300">
                                        {{ totalChapters }}
                                    </text>
                                </svg>

                                <!-- Legend -->
                                <div class="w-full space-y-2">
                                    <div
                                        v-for="item in pieChartData"
                                        :key="item.status"
                                        class="flex items-center justify-between py-1"
                                    >
                                        <div class="flex items-center gap-2">
                                            <span
                                                class="w-3 h-3 rounded-full"
                                                :style="{ backgroundColor: item.color }"
                                            ></span>
                                            <span class="text-sm text-gray-600 dark:text-gray-400">
                                                {{ item.label }}
                                            </span>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span class="text-sm font-medium text-gray-900 dark:text-white">
                                                {{ item.count }} 章
                                            </span>
                                            <span class="text-xs text-gray-400 w-10 text-right">
                                                {{ item.percentage }}%
                                            </span>
                                        </div>
                                    </div>
                                </div>
                            </template>
                        </div>
                    </n-card>
                </n-gi>
            </n-grid>
        </main>
    </div>
</template>
