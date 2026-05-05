<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRouter } from "vue-router";
import { useMessage } from "naive-ui";
import {
    NCard,
    NGrid,
    NGi,
    NButton,
    NSpace,
    NSpin,
    NIcon,
} from "naive-ui";
import { ArrowLeft, TrendingUp, Clock, Target, Calendar, BarChart3 } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";

interface WritingRecord {
    date: string;
    total_words: number;
    duration: number;
}

const router = useRouter();
const isLoading = ref(true);
const writingRecords = ref<WritingRecord[]>([]);

onMounted(async () => {
    await loadStats();
});

const loadStats = async () => {
    isLoading.value = true;
    try {
        const records = await invoke<WritingRecord[]>("get_writing_stats", {
            projectId: 0,
            days: 30,
        });
        writingRecords.value = records;
    } catch (error) {
        console.error("Failed to load stats:", error);
    } finally {
        isLoading.value = false;
    }
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
    router.push("/");
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
                <h1 class="text-xl font-bold text-gray-900 dark:text-white">写作统计</h1>
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
                                <Target class="w-6 h-6 text-green-600" />
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
            </n-grid>
        </main>
    </div>
</template>
