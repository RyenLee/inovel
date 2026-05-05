<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useDialog, useMessage } from "naive-ui";
import {
    NCard,
    NGrid,
    NGi,
    NButton,
    NModal,
    NForm,
    NFormItem,
    NInput,
    NEmpty,
    NSpin,
    NIcon,
    NTooltip,
    NProgress,
    NTag,
    NAlert,
    NTimeline,
    NTimelineItem,
} from "naive-ui";
import { Book, Plus, FolderOpen, FileText, AlertTriangle, Trash2, Edit3, Sun, Moon, Settings, BarChart3, TrendingUp, Calendar, Keyboard, Database, Play, RotateCcw, Eye, CheckCircle, XCircle } from "lucide-vue-next";
import { useRouter } from "vue-router";
import { useProjectStore, type ProjectMeta } from "../stores/project";
import { useTheme } from "../composables/useTheme";
import { invoke } from "@tauri-apps/api/core";
import DeleteConfirmModal from "../components/DeleteConfirmModal.vue";

const router = useRouter();
const { isDark, toggleDark } = useTheme();
const message = useMessage();
const dialog = useDialog();
const projectStore = useProjectStore();

const showModal = ref(false);
const showShortcuts = ref(false);
const isCreating = ref(false);
const isEditing = ref(false);
const editingProject = ref<ProjectMeta | null>(null);

// 删除确认弹窗状态
const showDeleteProjectModal = ref(false);
const projectToDelete = ref<ProjectMeta | null>(null);

// Form state
const formData = ref({
    name: "",
    author: "",
    description: "",
    path: "",
});

// Writing stats
const isLoadingStats = ref(false);
const writingRecords = ref<{ date: string; total_words: number; duration: number }[]>([]);

const totalWordsThisMonth = computed(() => {
    return writingRecords.value.reduce((sum, r) => sum + r.total_words, 0);
});

const totalDays = computed(() => writingRecords.value.length);

const averageWordsPerDay = computed(() => {
    if (totalDays.value === 0) return 0;
    return Math.round(totalWordsThisMonth.value / totalDays.value);
});

onMounted(async () => {
    await projectStore.fetchRecentProjects();
    await loadStats();
    await checkMigrationStatus();
});

const loadStats = async () => {
    isLoadingStats.value = true;
    try {
        const records = await invoke<{ date: string; total_words: number; duration: number }[]>("get_writing_stats", {
            projectId: 0,
            days: 30,
        });
        writingRecords.value = records;
    } catch (error) {
        console.error("Failed to load stats:", error);
    } finally {
        isLoadingStats.value = false;
    }
};

// === 数据迁移相关 ===
const showMigrationModal = ref(false);
const showMigrationResult = ref(false);
const pendingMigrationCount = ref(0);
const dryRunResult = ref<import("../stores/project").MigrateResult | null>(null);
const migrationProgress = ref(0);

const checkMigrationStatus = async () => {
    try {
        const count = await projectStore.checkMigrationNeeded();
        pendingMigrationCount.value = count;
    } catch (error) {
        console.error("检查迁移状态失败:", error);
    }
};

const openMigrationPreview = async () => {
    showMigrationModal.value = true;
    migrationProgress.value = 0;
    dryRunResult.value = await projectStore.migrateProjects(true);
};

const executeMigration = async () => {
    migrationProgress.value = 25;
    showMigrationResult.value = false;
    const result = await projectStore.migrateProjects(false);
    migrationProgress.value = 100;

    if (result) {
        showMigrationResult.value = true;
        showMigrationModal.value = false;

        // 刷新项目列表
        await projectStore.fetchRecentProjects();
        await checkMigrationStatus();
    }
};

const handleRollback = async () => {
    const dialog = useDialog();
    dialog.warning({
        title: "确认回滚迁移",
        content: "确定要将所有项目回滚到迁移前的状态吗？此操作会撤销之前的迁移。",
        positiveText: "确认回滚",
        negativeText: "取消",
        onPositiveClick: async () => {
            const result = await projectStore.rollbackMigration();
            if (result && result.success > 0) {
                message.success(`成功回滚 ${result.success} 个项目`);
                await projectStore.fetchRecentProjects();
                await checkMigrationStatus();
            } else if (result) {
                message.error(`回滚失败，请查看备份文件：${result.backup_path}`);
            }
        },
    });
};

const goToSettings = () => {
    router.push("/settings");
};

const goToStats = () => {
    router.push("/stats");
};

const openFolderDialog = async () => {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: "选择项目存储位置",
        });
        if (selected) {
            formData.value.path = selected as string;
        }
    } catch (error) {
        console.error("Failed to open folder dialog:", error);
        message.error("选择文件夹失败");
    }
};

const resetForm = () => {
    formData.value = {
        name: "",
        author: "",
        description: "",
        path: "",
    };
};

const openEditModal = (project: ProjectMeta, event: Event) => {
    event.stopPropagation();
    editingProject.value = project;
    formData.value = {
        name: project.name,
        author: project.author || "",
        description: project.description || "",
        path: project.path,
    };
    isEditing.value = true;
    showModal.value = true;
};

const handleEditProject = async () => {
    if (!editingProject.value) return;
    if (!formData.value.name.trim()) {
        message.warning("请输入书名");
        return;
    }

    isCreating.value = true;
    try {
        const project = await projectStore.updateProject(editingProject.value.id, {
            name: formData.value.name.trim(),
            author: formData.value.author.trim(),
            description: formData.value.description.trim(),
        });

        if (project) {
            message.success("项目修改成功！");
            closeModal();
        } else {
            message.error(projectStore.error || "修改项目失败");
        }
    } catch (error) {
        message.error(`修改失败: ${error}`);
    } finally {
        isCreating.value = false;
    }
};

const handleCreateProject = async () => {
    if (!formData.value.name.trim()) {
        message.warning("请输入书名");
        return;
    }
    if (!formData.value.path.trim()) {
        message.warning("请选择存储路径");
        return;
    }

    isCreating.value = true;
    try {
        const project = await projectStore.createProject({
            name: formData.value.name.trim(),
            author: formData.value.author.trim(),
            description: formData.value.description.trim(),
            path: formData.value.path,
        });

        if (project) {
            message.success("项目创建成功！");
            showModal.value = false;
            resetForm();
            router.push(`/editor/${project.id}`);
        } else {
            message.error(projectStore.error || "创建项目失败");
        }
    } catch (error) {
        message.error(`创建失败: ${error}`);
    } finally {
        isCreating.value = false;
    }
};

const handleOpenProject = async (project: ProjectMeta) => {
    if (!project.is_valid) {
        dialog.warning({
            title: "项目路径失效",
            content: `项目 "${project.name}" 的文件夹已被移动或删除。\n路径：${project.path}`,
            positiveText: "从列表中移除",
            negativeText: "取消",
            onPositiveClick: async () => {
                const success = await projectStore.removeProjectFromList(project.id);
                if (success) {
                    message.success("已从列表中移除");
                }
            },
        });
        return;
    }

    const result = await projectStore.openProject(project.id);
    if (result.success) {
        router.push(`/editor/${project.id}`);
    } else if (projectStore.error) {
        message.error(projectStore.error);
    }
};

// 打开删除项目确认弹窗
const openDeleteProjectModal = (project: ProjectMeta, event: Event) => {
    event.stopPropagation();
    projectToDelete.value = project;
    showDeleteProjectModal.value = true;
};

// 确认删除项目
const handleConfirmDeleteProject = async (keepFiles: boolean) => {
    if (!projectToDelete.value) return;

    const success = await projectStore.removeProjectFromList(
        projectToDelete.value.id,
        keepFiles
    );

    if (success) {
        if (keepFiles) {
            message.success("已从列表移除");
        } else {
            message.success("项目已删除");
        }
    } else {
        message.error("删除失败");
    }

    showDeleteProjectModal.value = false;
    projectToDelete.value = null;
};

// 取消删除
const handleCancelDeleteProject = () => {
    showDeleteProjectModal.value = false;
    projectToDelete.value = null;
};

const closeModal = () => {
    showModal.value = false;
    resetForm();
    isEditing.value = false;
    editingProject.value = null;
};
</script>

<template>
    <div class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300">
        <header class="border-b bg-white dark:bg-gray-800 dark:border-gray-700 transition-colors duration-300">
            <div class="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <Book class="w-8 h-8 text-blue-600" />
                    <h1 class="text-xl font-bold text-gray-900 dark:text-white">小说工坊</h1>
                </div>
                <div class="flex items-center gap-2">
                    <n-button quaternary circle @click="showShortcuts = true" title="快捷键">
                        <template #icon>
                            <NIcon><Keyboard /></NIcon>
                        </template>
                    </n-button>
                    <n-button quaternary circle @click="goToSettings" title="项目设置">
                        <template #icon>
                            <NIcon><Settings /></NIcon>
                        </template>
                    </n-button>
                    <n-button quaternary circle @click="goToStats" title="写作统计">
                        <template #icon>
                            <NIcon><BarChart3 /></NIcon>
                        </template>
                    </n-button>
                    <button @click="() => toggleDark()" class="p-2 rounded-lg transition-colors duration-300" :class="isDark
                        ? 'bg-gray-700 hover:bg-gray-600 text-yellow-400'
                        : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                        <Sun v-if="isDark" class="w-5 h-5" />
                        <Moon v-else class="w-5 h-5" />
                    </button>
                    <n-button type="primary" @click="showModal = true">
                        <template #icon>
                            <NIcon>
                                <Plus />
                            </NIcon>
                        </template>
                        新建项目
                    </n-button>
                </div>
            </div>
        </header>

        <main class="max-w-7xl mx-auto px-4 py-6 space-y-6">
            <!-- Stats Row -->
            <n-grid :cols="3" :x-gap="16" :y-gap="16" responsive="screen" :item-responsive="true">
                <!-- Monthly Words -->
                <n-gi span="0:24 640:12 1024:1">
                    <n-card hoverable>
                        <div class="flex items-center gap-3">
                            <div class="p-2 rounded-lg bg-green-100 dark:bg-green-900/30 flex-shrink-0">
                                <TrendingUp class="w-5 h-5 text-green-600" />
                            </div>
                            <div class="overflow-hidden">
                                <p class="text-xs text-gray-500 dark:text-gray-400">本月字数</p>
                                <p class="text-xl font-bold text-gray-900 dark:text-white truncate whitespace-nowrap">
                                    {{ totalWordsThisMonth.toLocaleString() }}
                                </p>
                            </div>
                        </div>
                    </n-card>
                </n-gi>

                <!-- Average Words -->
                <n-gi span="0:24 640:12 1024:1">
                    <n-card hoverable>
                        <div class="flex items-center gap-3">
                            <div class="p-2 rounded-lg bg-purple-100 dark:bg-purple-900/30 flex-shrink-0">
                                <BarChart3 class="w-5 h-5 text-purple-600" />
                            </div>
                            <div class="overflow-hidden">
                                <p class="text-xs text-gray-500 dark:text-gray-400">日均字数</p>
                                <p class="text-xl font-bold text-gray-900 dark:text-white truncate whitespace-nowrap">
                                    {{ averageWordsPerDay.toLocaleString() }}
                                </p>
                            </div>
                        </div>
                    </n-card>
                </n-gi>

                <!-- Writing Days -->
                <n-gi span="0:24 640:12 1024:1">
                    <n-card hoverable>
                        <div class="flex items-center gap-3">
                            <div class="p-2 rounded-lg bg-orange-100 dark:bg-orange-900/30 flex-shrink-0">
                                <Calendar class="w-5 h-5 text-orange-600" />
                            </div>
                            <div class="overflow-hidden">
                                <p class="text-xs text-gray-500 dark:text-gray-400">写作天数</p>
                                <p class="text-xl font-bold text-gray-900 dark:text-white truncate whitespace-nowrap">
                                    {{ totalDays }} 天
                                </p>
                            </div>
                        </div>
                    </n-card>
                </n-gi>
            </n-grid>

            <!-- Migration Banner -->
            <n-alert v-if="pendingMigrationCount > 0" type="info" closable
                class="mb-4">
                <template #header>
                    <div class="flex items-center gap-2">
                        <Database class="w-5 h-5" />
                        <span>发现 {{ pendingMigrationCount }} 个项目需要迁移到新的项目ID系统</span>
                    </div>
                </template>
                <div class="flex items-center gap-3 mt-1">
                    <span class="text-sm">
                        旧项目文件夹将使用项目ID重命名，数据不会丢失。建议在开始迁移前先预览。
                    </span>
                    <n-button size="small" @click="openMigrationPreview">
                        <template #icon>
                            <NIcon><Eye /></NIcon>
                        </template>
                        查看详情
                    </n-button>
                    <n-button size="small" type="primary" @click="executeMigration" :loading="projectStore.isMigrating">
                        <template #icon>
                            <NIcon><Play /></NIcon>
                        </template>
                        开始迁移
                    </n-button>
                    <n-button size="small" quaternary @click="handleRollback">
                        <template #icon>
                            <NIcon><RotateCcw /></NIcon>
                        </template>
                        回滚
                    </n-button>
                </div>
            </n-alert>

            <!-- Migration Result Alert -->
            <n-alert v-if="showMigrationResult && projectStore.migrationResult" 
                :type="(projectStore.migrationResult.failed === 0) ? 'success' : 'warning'"
                closable @close="showMigrationResult = false" class="mb-4">
                <template #header>
                    <div class="flex items-center gap-2">
                        <CheckCircle v-if="projectStore.migrationResult.failed === 0" class="w-5 h-5 text-green-500" />
                        <XCircle v-else class="w-5 h-5 text-orange-500" />
                        <span>迁移{{ projectStore.migrationResult.failed === 0 ? '完成' : '部分完成' }}</span>
                    </div>
                </template>
                <div class="text-sm space-y-1 mt-1">
                    <p>总计: {{ projectStore.migrationResult.total }} | 
                        成功: <span class="text-green-600 font-medium">{{ projectStore.migrationResult.success }}</span> | 
                        失败: <span class="text-orange-600 font-medium">{{ projectStore.migrationResult.failed }}</span></p>
                    <p class="text-gray-500">备份文件: {{ projectStore.migrationResult.backup_path }}</p>
                </div>
            </n-alert>

            <!-- Recent Projects Section -->
            <section>
                <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">最近项目</h2>

                <div v-if="projectStore.isLoading" class="flex justify-center py-12">
                    <n-spin size="large" />
                </div>

                <n-empty v-else-if="projectStore.recentProjects.length === 0" description="暂无项目，点击上方按钮创建新项目"
                    class="py-12">
                    <template #icon>
                        <FileText class="w-16 h-16 text-gray-300 dark:text-gray-600" />
                    </template>
                </n-empty>

                <n-grid v-else :cols="3" :x-gap="16" :y-gap="16" responsive="screen" :item-responsive="true">
                    <n-gi v-for="project in projectStore.recentProjects" :key="project.id" span="0:24 640:12 1024:8">
                        <n-card hoverable
                            class="cursor-pointer transition-all duration-200 hover:shadow-lg relative group" :class="project.is_valid
                                ? 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:border-blue-400 dark:hover:border-blue-500'
                                : 'bg-gray-100 dark:bg-gray-800 border-orange-300 dark:border-orange-700 opacity-75'
                                " @click="handleOpenProject(project)">
                            <!-- Invalid project indicator -->
                            <div v-if="!project.is_valid" class="absolute top-2 right-2">
                                <n-tooltip trigger="hover">
                                    <template #trigger>
                                        <AlertTriangle class="w-5 h-5 text-orange-500" />
                                    </template>
                                    项目路径已失效
                                </n-tooltip>
                            </div>

                            <!-- Remove button -->
                            <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1"
                                v-if="project.is_valid">
                                <n-button quaternary circle size="small" @click="openEditModal(project, $event)" class="hover:text-blue-500">
                                    <template #icon>
                                        <NIcon>
                                            <Edit3 />
                                        </NIcon>
                                    </template>
                                </n-button>
                                <n-button quaternary circle size="small" type="error" @click="openDeleteProjectModal(project, $event)">
                                    <template #icon>
                                        <NIcon>
                                            <Trash2 />
                                        </NIcon>
                                    </template>
                                </n-button>
                            </div>

                            <div class="flex flex-col gap-2">
                                <div class="flex items-center gap-2">
                                    <h3 class="font-semibold text-lg text-gray-900 dark:text-white truncate">
                                        {{ project.name }}
                                    </h3>
                                    <span class="px-2 py-0.5 text-xs font-mono bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded">
                                        {{ project.project_id }}
                                    </span>
                                </div>
                                <p v-if="project.author" class="text-sm text-gray-500 dark:text-gray-400">
                                    笔名: {{ project.author }}
                                </p>
                                <p v-if="project.description"
                                    class="text-sm text-gray-600 dark:text-gray-300 line-clamp-2">
                                    {{ project.description }}
                                </p>
                                <div class="flex items-center justify-between mt-2">
                                    <p class="text-xs text-gray-400 dark:text-gray-500">
                                        创建于: {{ new Date(project.created_at).toLocaleDateString("zh-CN") }}
                                    </p>
                                    <n-button type="primary" size="small" @click.stop="handleOpenProject(project)">
                                        <template #icon>
                                            <NIcon>
                                                <Edit3 />
                                            </NIcon>
                                        </template>
                                        打开
                                    </n-button>
                                </div>
                            </div>
                        </n-card>
                    </n-gi>
                </n-grid>
            </section>
        </main>

        <!-- Create/Edit Project Modal -->
        <n-modal v-model:show="showModal" preset="card" :title="isEditing ? '编辑项目' : '新建项目'" style="width: 520px" :mask-closable="false">
            <n-form :model="formData" label-placement="top">
                <n-form-item label="书名" path="name" :rule="{
                    required: true,
                    message: '请输入书名',
                    trigger: ['input', 'blur'],
                }">
                    <n-input v-model:value="formData.name" placeholder="请输入书名（必填）" maxlength="100" show-count />
                </n-form-item>

                <n-form-item label="笔名" path="author">
                    <n-input v-model:value="formData.author" placeholder="请输入作者笔名" maxlength="50" />
                </n-form-item>

                <n-form-item label="简介" path="description">
                    <n-input v-model:value="formData.description" type="textarea" placeholder="请输入小说简介" :rows="3"
                        maxlength="500" show-count />
                </n-form-item>

                <n-form-item v-if="!isEditing" label="存储路径" path="path">
                    <div class="flex gap-2 w-full">
                        <n-input v-model:value="formData.path" placeholder="请选择项目存储路径（必填）" readonly class="flex-1" />
                        <n-button @click="openFolderDialog">
                            <template #icon>
                                <NIcon>
                                    <FolderOpen />
                                </NIcon>
                            </template>
                            选择
                        </n-button>
                    </div>
                </n-form-item>

                <n-form-item v-else label="存储路径">
                    <n-input v-model:value="formData.path" readonly />
                </n-form-item>
            </n-form>

            <template #footer>
                <n-space justify="end">
                    <n-button @click="closeModal" :disabled="isCreating">取消</n-button>
                    <n-button type="primary" @click="isEditing ? handleEditProject() : handleCreateProject()" :loading="isCreating">
                        {{ isEditing ? '保存修改' : '创建项目' }}
                    </n-button>
                </n-space>
            </template>
        </n-modal>

        <!-- 迁移预览弹窗 -->
        <n-modal v-model:show="showMigrationModal" preset="card" title="数据迁移预览" style="width: 640px" :mask-closable="false">
            <div v-if="projectStore.isMigrating" class="flex flex-col items-center py-8 gap-4">
                <n-spin size="large" />
                <p class="text-gray-500">正在执行迁移...</p>
                <n-progress type="line" :percentage="migrationProgress" :height="8" :show-indicator="true" />
            </div>

            <div v-else class="space-y-4">
                <n-alert type="info" :bordered="false">
                    以下 {{ dryRunResult?.total || 0 }} 个项目将被迁移。文件夹将从旧书名重命名为项目ID。
                </n-alert>

                <div class="max-h-64 overflow-y-auto space-y-2">
                    <div v-for="detail in dryRunResult?.details || []" :key="detail.project_db_id"
                        class="p-3 rounded-lg bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700">
                        <div class="flex items-center justify-between">
                            <span class="font-medium text-gray-900 dark:text-white">{{ detail.old_name }}</span>
                            <n-tag size="small" type="info">待迁移</n-tag>
                        </div>
                        <div class="mt-1 text-sm text-gray-500">
                            <p class="font-mono text-xs">{{ detail.old_path }} → <span class="text-blue-500">PXXXXX</span></p>
                        </div>
                    </div>
                </div>
            </div>

            <template #footer>
                <n-space justify="end">
                    <n-button @click="showMigrationModal = false" :disabled="projectStore.isMigrating">取消</n-button>
                    <n-button type="primary" @click="executeMigration" :loading="projectStore.isMigrating">
                        <template #icon>
                            <NIcon><Play /></NIcon>
                        </template>
                        确认迁移
                    </n-button>
                </n-space>
            </template>
        </n-modal>

        <!-- 快捷键说明弹窗 -->
        <n-modal v-model:show="showShortcuts" preset="card" title="快捷键说明" style="width: 480px" :mask-closable="true">
            <div class="space-y-4">
                <div class="p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
                    <p class="text-sm text-blue-800 dark:text-blue-300">
                        在编辑器页面中使用以下快捷键：
                    </p>
                </div>
                
                <div class="space-y-3">
                    <div class="flex items-center justify-between">
                        <span class="text-gray-700 dark:text-gray-300">保存</span>
                        <kbd class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded border border-gray-300 dark:border-gray-600">Ctrl + S</kbd>
                    </div>
                    
                    <div class="flex items-center justify-between">
                        <div>
                            <span class="text-gray-700 dark:text-gray-300">打字机模式</span>
                            <p class="text-xs text-gray-500 dark:text-gray-400">光标行居中，其他行淡化</p>
                        </div>
                        <kbd class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded border border-gray-300 dark:border-gray-600">Ctrl + Shift + T</kbd>
                    </div>
                    
                    <div class="flex items-center justify-between">
                        <div>
                            <span class="text-gray-700 dark:text-gray-300">聚焦模式</span>
                            <p class="text-xs text-gray-500 dark:text-gray-400">当前段落高亮，其他内容淡化</p>
                        </div>
                        <kbd class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded border border-gray-300 dark:border-gray-600">Ctrl + Shift + F</kbd>
                    </div>
                    
                    <div class="flex items-center justify-between">
                        <span class="text-gray-700 dark:text-gray-300">退出特殊模式</span>
                        <kbd class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded border border-gray-300 dark:border-gray-600">Esc</kbd>
                    </div>
                </div>
            </div>
        </n-modal>

        <!-- 删除项目确认弹窗 -->
        <DeleteConfirmModal
            v-model:show="showDeleteProjectModal"
            title="确认删除项目"
            :message="`确定要删除项目 &quot;${projectToDelete?.name}&quot; 吗？`"
            confirm-text="删除"
            :show-keep-files="true"
            :default-keep-files="true"
            @confirm="handleConfirmDeleteProject"
            @cancel="handleCancelDeleteProject"
        />
    </div>
</template>

<style scoped>
.line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}
</style>
