<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useMessage } from "naive-ui";
import {
    NCard,
    NGrid,
    NGi,
    NButton,
    NForm,
    NFormItem,
    NInput,
    NSpace,
    NSpin,
    NIcon,
    NInputNumber,
    NDivider,
    NResult,
    NSelect,
} from "naive-ui";
import { ArrowLeft, Target, Save } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "../stores/project";

const router = useRouter();
const message = useMessage();
const projectStore = useProjectStore();

const isLoading = ref(false);
const isSaving = ref(false);

const dailyGoal = ref(3000);
const autoSaveInterval = ref(1);

const windowSize = ref<string>("default");
const hasProject = ref(false);
const currentProjectId = ref<number | null>(null);

const windowSizeOptions = [
    // 横屏模式
    { label: "默认横屏 (1200×800)", value: "1200x800" },
    { label: "紧凑横屏 (1024×768)", value: "1024x768" },
    { label: "标准横屏 (1280×800)", value: "1280x800" },
    { label: "宽屏 (1440×900)", value: "1440x900" },
    { label: "全高清横屏 (1920×1080)", value: "1920x1080" },
    // 竖屏模式
    { label: "默认竖屏 (800×1200)", value: "800x1200" },
    { label: "紧凑竖屏 (600×900)", value: "600x900" },
    { label: "标准竖屏 (720×1280)", value: "720x1280" },
    { label: "大屏竖屏 (800×1400)", value: "800x1400" },
    { label: "全高清竖屏 (1080×1920)", value: "1080x1920" },
];

const autoSaveIntervalOptions = [
    { label: "30 秒", value: 0.5 },
    { label: "1 分钟", value: 1 },
    { label: "2 分钟", value: 2 },
    { label: "5 分钟", value: 5 },
    { label: "10 分钟", value: 10 },
];

const LOCAL_STORAGE_KEY = "inovel_settings";

onMounted(async () => {
    await loadSettings();
    await loadProjectInfo();
});

const loadProjectInfo = async () => {
    try {
        await projectStore.fetchRecentProjects();
        if (projectStore.recentProjects.length > 0) {
            hasProject.value = true;
            const lastProject = projectStore.recentProjects[0];
            currentProjectId.value = lastProject.id;

            const windowSizeResult = await invoke<[number, number] | null>("get_window_size", {
                projectId: lastProject.id,
            });
            if (windowSizeResult) {
                const [width, height] = windowSizeResult;
                windowSize.value = `${width}x${height}`;
            } else {
                windowSize.value = "default";
            }
        }
    } catch (error) {
        console.error("Failed to load project info:", error);
    }
};

const loadSettings = async () => {
    isLoading.value = true;
    try {
        const stored = localStorage.getItem(LOCAL_STORAGE_KEY);
        if (stored) {
            const savedSettings = JSON.parse(stored);
            if (savedSettings.dailyGoal !== undefined) {
                dailyGoal.value = savedSettings.dailyGoal;
            }
            if (savedSettings.autoSaveInterval !== undefined) {
                autoSaveInterval.value = savedSettings.autoSaveInterval;
            }
        }
    } catch (error) {
        console.error("Failed to load settings:", error);
    } finally {
        isLoading.value = false;
    }
};

const saveAllSettings = async () => {
    isSaving.value = true;
    try {
        const settings = {
            dailyGoal: dailyGoal.value,
            autoSaveInterval: autoSaveInterval.value,
        };
        localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(settings));

        if (hasProject.value && currentProjectId.value && windowSize.value !== "default") {
            const [width, height] = windowSize.value.split("x").map(Number);
            await invoke("set_window_size", {
                projectId: currentProjectId.value,
                width,
                height,
            });
        }

        message.success("设置已保存");
    } catch (error) {
        console.error("Failed to save settings:", error);
        message.warning("保存设置失败");
    } finally {
        isSaving.value = false;
    }
};

const goBack = () => {
    router.push("/");
};
</script>

<template>
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300">
        <header class="border-b bg-white dark:bg-gray-800 dark:border-gray-700 transition-colors duration-300">
            <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center gap-4">
                <n-button quaternary circle @click="goBack">
                    <template #icon>
                        <NIcon>
                            <ArrowLeft />
                        </NIcon>
                    </template>
                </n-button>
                <h1 class="text-xl font-bold text-gray-900 dark:text-white whitespace-nowrap">应用设置</h1>
            </div>
        </header>

        <main class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            <div v-if="isLoading" class="flex justify-center py-12">
                <n-spin size="large" />
            </div>

            <n-grid v-else :cols="1" :x-gap="16" :y-gap="16" class="settings-grid">
                <!-- Writing Goals -->
                <n-gi>
                    <n-card title="写作目标" hoverable class="settings-card">
                        <n-form label-placement="top" class="settings-form">
                            <n-form-item label="每日字数目标" class="settings-form-item">
                                <div class="flex flex-wrap items-center gap-3">
                                    <n-input-number v-model:value="dailyGoal" :min="0" :max="100000" :step="100"
                                        class="min-w-[140px] w-48" />
                                    <span class="text-gray-500 dark:text-gray-400 whitespace-nowrap">字/天</span>
                                </div>
                                <template #feedback>
                                    <span class="text-sm text-gray-500 dark:text-gray-400">
                                        设置每日写作目标，系统会在编辑器状态栏显示进度
                                    </span>
                                </template>
                            </n-form-item>
                        </n-form>
                    </n-card>
                </n-gi>

                <!-- Editor Settings -->
                <n-gi>
                    <n-card title="编辑器设置" hoverable class="settings-card">
                        <n-form label-placement="top" class="settings-form">
                            <n-form-item label="自动保存间隔" class="settings-form-item">
                                <div class="flex flex-nowrap items-center gap-3">
                                    <n-select v-model:value="autoSaveInterval" :options="autoSaveIntervalOptions"
                                        class="min-w-[140px] w-48" :teleport="'body'" />
                                    <span class="text-gray-500 dark:text-gray-400 whitespace-nowrap">分钟</span>
                                </div>
                                <template #feedback>
                                    <span class="text-sm text-gray-500 dark:text-gray-400">
                                        编辑器每隔设定时间自动保存章节内容到后端并创建快照
                                    </span>
                                </template>
                            </n-form-item>
                        </n-form>

                        <template #footer>
                            <n-space justify="end" class="w-full">
                                <n-button type="primary" @click="saveAllSettings" :loading="isSaving">
                                    <template #icon>
                                        <NIcon>
                                            <Save />
                                        </NIcon>
                                    </template>
                                    保存设置
                                </n-button>
                            </n-space>
                        </template>
                    </n-card>
                </n-gi>

                <!-- Window Size Settings -->
                <n-gi v-if="hasProject">
                    <n-card title="窗口大小" hoverable class="settings-card">
                        <n-form label-placement="top" class="settings-form">
                            <n-form-item label="窗口尺寸" class="settings-form-item">
                                <div class="flex flex-nowrap items-center gap-3">
                                    <n-select v-model:value="windowSize" :options="windowSizeOptions"
                                        class="min-w-[180px] w-56" :teleport="'body'" />
                                </div>
                                <template #feedback>
                                    <span class="text-sm text-gray-500 dark:text-gray-400">
                                        设置应用窗口大小，下次启动时生效
                                    </span>
                                </template>
                            </n-form-item>
                        </n-form>

                        <template #footer>
                            <n-space justify="end" class="w-full">
                                <n-button type="primary" @click="saveAllSettings" :loading="isSaving">
                                    <template #icon>
                                        <NIcon>
                                            <Save />
                                        </NIcon>
                                    </template>
                                    保存设置
                                </n-button>
                            </n-space>
                        </template>
                    </n-card>
                </n-gi>

                <!-- Stats Link -->
                <n-gi>
                    <n-card title="写作统计" hoverable @click="router.push('/stats')" class="settings-card cursor-pointer">
                        <div class="text-center py-4">
                            <p class="text-gray-500 dark:text-gray-400 whitespace-normal">
                                查看详细统计数据，请点击此处
                            </p>
                        </div>
                    </n-card>
                </n-gi>


            </n-grid>
        </main>
    </div>
</template>

<style scoped>
.settings-grid {
    display: grid;
    gap: 1rem;
}

.settings-card {
    width: 100%;
    box-sizing: border-box;
}

.settings-form {
    width: 100%;
}

.settings-form-item {
    width: 100%;
}

@media (max-width: 640px) {
    .settings-grid {
        gap: 0.75rem;
    }

    .settings-card {
        padding: 0.75rem;
    }
}

@media (min-width: 640px) {
    .settings-grid {
        gap: 1rem;
    }
}

@media (min-width: 1024px) {
    .settings-grid {
        gap: 1.5rem;
    }
}
</style>
