<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useFolderDialog } from "../composables/useFolderDialog";
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
  NSpace,
  NPagination,
} from "naive-ui";
import {
  Book,
  Plus,
  FolderOpen,
  FileText,
  AlertTriangle,
  Trash2,
  Edit3,
  Sun,
  Moon,
  Settings,
  BarChart3,
  TrendingUp,
  Calendar,
  Keyboard,
  Database,
  Play,
  RotateCcw,
  Eye,
  CheckCircle,
  XCircle,
  ChevronLeft,
  ChevronRight,
} from "lucide-vue-next";
import { useRouter } from "vue-router";
import {
  useProjectStore,
  type ProjectMeta,
  type MigrateResult,
} from "../stores/project";
import { useTheme } from "../composables/useTheme";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import DeleteConfirmModal from "../components/DeleteConfirmModal.vue";
import { configService } from "../services/configService";
import { useLocale } from "../i18n/composables/useLocale";

const router = useRouter();
const { isDark, toggleDark } = useTheme();
const message = useMessage();
const dialog = useDialog();
const { selectFolder } = useFolderDialog();
const projectStore = useProjectStore();
const { t } = useLocale();

const showModal = ref(false);
const showShortcuts = ref(false);
const isCreating = ref(false);
const isEditing = ref(false);
const editingProject = ref<ProjectMeta | null>(null);

const goToTemplates = () => {
  router.push("/templates");
};

// 删除确认弹窗状态
const showDeleteProjectModal = ref(false);
const projectToDelete = ref<ProjectMeta | null>(null);

// 解密相关
const showDecryptModal = ref(false);
const decryptPassword = ref("");
const projectToDecrypt = ref<ProjectMeta | null>(null);

// Form state
const formData = ref({
  name: "",
  author: "",
  description: "",
  path: "",
});

// Writing stats
const isLoadingStats = ref(false);
const writingRecords = ref<
  { date: string; total_words: number; duration: number }[]
>([]);

const totalWordsThisMonth = computed(() => {
  return writingRecords.value.reduce((sum, r) => sum + r.total_words, 0);
});

const totalDays = computed(() => writingRecords.value.length);

const averageWordsPerDay = computed(() => {
  if (totalDays.value === 0) return 0;
  return Math.round(totalWordsThisMonth.value / totalDays.value);
});

onMounted(async () => {
  try {
    await projectStore.fetchRecentProjects();
  } catch (error) {
    console.error("加载项目列表失败:", error);
    message.error(t("welcome.dialog.loadFailed"));
  }

  try {
    await loadStats();
  } catch (error) {
    console.error("加载统计数据失败:", error);
  }

  try {
    await checkMigrationStatus();
  } catch (error) {
    console.error("检查迁移状态失败:", error);
  }
});

const loadStats = async () => {
  isLoadingStats.value = true;
  try {
    const records = await invoke<
      { date: string; total_words: number; duration: number }[]
    >("get_writing_stats", {
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
const dryRunResult = ref<MigrateResult | null>(null);
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
    title: t("welcome.dialog.rollbackTitle"),
    content: t("welcome.dialog.rollbackContent"),
    positiveText: t("welcome.dialog.rollbackConfirm"),
    negativeText: t("common.action.cancel"),
    onPositiveClick: async () => {
      const result = await projectStore.rollbackMigration();
      if (result && result.success > 0) {
        message.success(
          t("welcome.dialog.rollbackSuccess", { count: result.success })
        );
        await projectStore.fetchRecentProjects();
        await checkMigrationStatus();
      } else if (result) {
        message.error(
          t("welcome.dialog.rollbackFailed", { path: result.backup_path })
        );
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
  const { path, error } = await selectFolder({
    title: t("welcome.dialog.selectStoragePath"),
  });
  if (error) {
    message.error(error);
    return;
  }
  if (path) {
    formData.value.path = path;
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
    message.warning(t("welcome.dialog.enterBookName"));
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
      message.success(t("welcome.createProject.editSuccess"));
      closeModal();
    } else {
      message.error(projectStore.error || t("welcome.createProject.editError"));
    }
  } catch (error) {
    message.error(`${t("welcome.createProject.editError")}: ${error}`);
  } finally {
    isCreating.value = false;
  }
};

const handleCreateProject = async () => {
  if (!formData.value.name.trim()) {
    message.warning(t("welcome.createProject.placeholder.name"));
    return;
  }
  if (!formData.value.path.trim()) {
    message.warning(t("welcome.createProject.placeholder.path"));
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
      message.success(t("welcome.createProject.success"));
      showModal.value = false;
      resetForm();
      router.push(`/editor/${project.id}`);
    } else {
      message.error(projectStore.error || t("welcome.createProject.error"));
    }
  } catch (error) {
    message.error(`${t("welcome.createProject.error")}: ${error}`);
  } finally {
    isCreating.value = false;
  }
};

const handlePageChange = async (page: number) => {
  await projectStore.fetchRecentProjects(page);
  window.scrollTo({ top: 0, behavior: "smooth" });
};

const handleOpenProject = async (project: ProjectMeta) => {
  if (!project.is_valid) {
    dialog.warning({
      title: t("welcome.dialog.invalidPath"),
      content: t("welcome.dialog.invalidPathMessage", {
        name: project.name,
        path: project.path,
      }),
      positiveText: t("welcome.dialog.removeFromList"),
      negativeText: t("common.action.cancel"),
      onPositiveClick: async () => {
        const success = await projectStore.removeProjectFromList(project.id);
        if (success) {
          message.success(t("welcome.dialog.removed"));
        }
      },
    });
    return;
  }

  // 检查项目是否已加密
  if (project.encrypted) {
    projectToDecrypt.value = project;
    showDecryptModal.value = true;
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
      message.success(t("welcome.dialog.removed"));
    } else {
      message.success(t("welcome.dialog.deleteSuccess"));
    }
  } else {
    message.error(t("common.status.error"));
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
  <div
    class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300"
  >
    <header
      class="sticky top-0 z-50 border-b bg-white/95 dark:bg-gray-800/95 backdrop-blur-sm dark:border-gray-700 transition-colors duration-300"
    >
      <div
        class="max-w-7xl mx-auto px-4 py-4 flex items-center justify-between"
      >
        <div class="flex items-center gap-3">
          <Book class="w-8 h-8 text-blue-600" />
          <h1 class="text-xl font-bold text-gray-900 dark:text-white">
            {{ t("welcome.title") }}
          </h1>
        </div>
        <div class="flex items-center gap-2">
          <n-button
            quaternary
            circle
            @click="showShortcuts = true"
            :title="t('welcome.shortcutsButton')"
          >
            <template #icon>
              <NIcon><Keyboard /></NIcon>
            </template>
          </n-button>

          <n-button
            quaternary
            circle
            @click="goToTemplates"
            :title="t('userTemplates.open')"
          >
            <template #icon>
              <NIcon><FileText /></NIcon>
            </template>
          </n-button>

          <n-button
            quaternary
            circle
            @click="goToStats"
            :title="t('welcome.stats')"
          >
            <template #icon>
              <NIcon><BarChart3 /></NIcon>
            </template>
          </n-button>
          <button
            @click="() => toggleDark()"
            class="p-2 rounded-lg transition-colors duration-300"
            :class="
              isDark
                ? 'bg-gray-700 hover:bg-gray-600 text-yellow-400'
                : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
            "
          >
            <Sun v-if="isDark" class="w-5 h-5" />
            <Moon v-else class="w-5 h-5" />
          </button>
          <n-button
            quaternary
            circle
            @click="goToSettings"
            :title="t('welcome.settings')"
          >
            <template #icon>
              <NIcon><Settings /></NIcon>
            </template>
          </n-button>

          <n-button type="primary" @click="showModal = true">
            <template #icon>
              <NIcon>
                <Plus />
              </NIcon>
            </template>
            {{ t("welcome.newProject") }}
          </n-button>
        </div>
      </div>
    </header>

    <main class="max-w-7xl mx-auto px-4 py-6 space-y-6">
      <!-- Stats Row -->
      <n-grid
        :cols="3"
        :x-gap="16"
        :y-gap="16"
        responsive="screen"
        :item-responsive="true"
      >
        <!-- Monthly Words -->
        <n-gi span="0:24 640:12 1024:1">
          <n-card hoverable>
            <div class="flex items-center gap-3">
              <div
                class="p-2 rounded-lg bg-green-100 dark:bg-green-900/30 shrink-0"
              >
                <TrendingUp class="w-5 h-5 text-green-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-xs text-gray-500 dark:text-gray-400">
                  {{ t("welcome.monthlyWords") }}
                </p>
                <p
                  class="text-xl font-bold text-gray-900 dark:text-white truncate whitespace-nowrap"
                >
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
              <div
                class="p-2 rounded-lg bg-purple-100 dark:bg-purple-900/30 shrink-0"
              >
                <BarChart3 class="w-5 h-5 text-purple-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-xs text-gray-500 dark:text-gray-400">
                  {{ t("welcome.dailyAverage") }}
                </p>
                <p
                  class="text-xl font-bold text-gray-900 dark:text-white truncate whitespace-nowrap"
                >
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
              <div
                class="p-2 rounded-lg bg-orange-100 dark:bg-orange-900/30 shrink-0"
              >
                <Calendar class="w-5 h-5 text-orange-600" />
              </div>
              <div class="overflow-hidden">
                <p class="text-xs text-gray-500 dark:text-gray-400">
                  {{ t("welcome.writingDays") }}
                </p>
                <p
                  class="text-xl font-bold text-gray-900 dark:text-white truncate whitespace-nowrap"
                >
                  {{ totalDays }} {{ t("welcome.days") }}
                </p>
              </div>
            </div>
          </n-card>
        </n-gi>
      </n-grid>

      <!-- Migration Banner -->
      <n-alert
        v-if="pendingMigrationCount > 0"
        type="info"
        closable
        class="mb-4"
      >
        <template #header>
          <div class="flex items-center gap-2">
            <Database class="w-5 h-5" />
            <span>{{
              t("welcome.migrate.info", { count: pendingMigrationCount })
            }}</span>
          </div>
        </template>
        <div class="flex items-center gap-3 mt-1">
          <span class="text-sm">{{ t("welcome.migrate.description") }}</span>
          <n-button size="small" @click="openMigrationPreview">
            <template #icon>
              <NIcon><Eye /></NIcon>
            </template>
            {{ t("welcome.migrate.preview") }}
          </n-button>
          <n-button
            size="small"
            type="primary"
            @click="executeMigration"
            :loading="projectStore.isMigrating"
          >
            <template #icon>
              <NIcon><Play /></NIcon>
            </template>
            {{ t("welcome.migrate.start") }}
          </n-button>
          <n-button size="small" quaternary @click="handleRollback">
            <template #icon>
              <NIcon><RotateCcw /></NIcon>
            </template>
            {{ t("welcome.migrate.rollback") }}
          </n-button>
        </div>
      </n-alert>

      <!-- Migration Result Alert -->
      <n-alert
        v-if="showMigrationResult && projectStore.migrationResult"
        :type="
          projectStore.migrationResult.failed === 0 ? 'success' : 'warning'
        "
        closable
        @close="showMigrationResult = false"
        class="mb-4"
      >
        <template #header>
          <div class="flex items-center gap-2">
            <CheckCircle
              v-if="projectStore.migrationResult.failed === 0"
              class="w-5 h-5 text-green-500"
            />
            <XCircle v-else class="w-5 h-5 text-orange-500" />
            <span>{{
              t(
                `welcome.migrate.${
                  projectStore.migrationResult.failed === 0
                    ? "completed"
                    : "partialCompleted"
                }`
              )
            }}</span>
          </div>
        </template>
        <div class="text-sm space-y-1 mt-1">
          <p>
            {{ t("welcome.migrate.total") }}:
            {{ projectStore.migrationResult.total }} |
            {{ t("welcome.migrate.success") }}:
            <span class="text-green-600 font-medium">{{
              projectStore.migrationResult.success
            }}</span>
            | {{ t("welcome.migrate.failed") }}:
            <span class="text-orange-600 font-medium">{{
              projectStore.migrationResult.failed
            }}</span>
          </p>
          <p class="text-gray-500">
            {{
              t("welcome.migrate.backupPath", {
                path: projectStore.migrationResult.backup_path,
              })
            }}
          </p>
        </div>
      </n-alert>

      <!-- Recent Projects Section -->
      <section>
        <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">
          {{ t("welcome.recentProjects") }}
        </h2>

        <div v-if="projectStore.isLoading" class="flex justify-center py-12">
          <n-spin size="large" />
        </div>

        <n-empty
          v-else-if="projectStore.recentProjects.length === 0"
          :description="t('welcome.noProjects')"
          class="empty-no-project"
        >
          <template #icon>
            <FileText class="w-16 h-16 text-gray-300 dark:text-gray-600" />
          </template>
        </n-empty>

        <div v-else>
          <div
            class="max-h-[calc(100vh-370px)] overflow-y-auto scroll-smooth space-y-6 pr-1"
          >
            <div
              v-for="project in projectStore.recentProjects"
              :key="project.id"
            >
              <div
                class="relative bg-white dark:bg-gray-800 rounded-2xl shadow-sm hover:shadow-xl transition-all duration-300 cursor-pointer overflow-hidden group border border-gray-100 dark:border-gray-700"
                :class="project.is_valid ? '' : 'opacity-60'"
                @click="handleOpenProject(project)"
              >
                <!-- Invalid project indicator -->
                <div
                  v-if="!project.is_valid"
                  class="absolute top-4 right-4 z-10"
                >
                  <n-tooltip trigger="hover">
                    <template #trigger>
                      <AlertTriangle class="w-5 h-5 text-orange-500" />
                    </template>
                    {{ t("welcome.dialog.invalidPath") }}
                  </n-tooltip>
                </div>

                <!-- Action buttons -->
                <div
                  class="absolute top-4 right-4 flex gap-2 opacity-0 group-hover:opacity-100 transition-all duration-300 z-10"
                  v-if="project.is_valid"
                >
                  <n-button
                    quaternary
                    circle
                    size="small"
                    @click="openEditModal(project, $event)"
                    class="bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm hover:text-blue-500 hover:bg-white dark:hover:bg-gray-700"
                  >
                    <template #icon>
                      <NIcon>
                        <Edit3 class="w-4 h-4" />
                      </NIcon>
                    </template>
                  </n-button>
                  <n-button
                    quaternary
                    circle
                    size="small"
                    type="error"
                    @click="openDeleteProjectModal(project, $event)"
                    class="bg-white/80 dark:bg-gray-700/80 backdrop-blur-sm hover:bg-white dark:hover:bg-gray-700"
                  >
                    <template #icon>
                      <NIcon>
                        <Trash2 class="w-4 h-4" />
                      </NIcon>
                    </template>
                  </n-button>
                </div>

                <div class="flex p-6 gap-6">
                  <!-- Book Cover -->
                  <div class="relative flex-shrink-0">
                    <div
                      class="w-24 h-32 rounded-lg overflow-hidden shadow-lg transform group-hover:scale-105 group-hover:-translate-y-1 transition-all duration-300"
                      style="perspective: 1000px"
                    >
                      <template v-if="project.cover_path">
                        <img
                          :src="convertFileSrc(project.cover_path)"
                          :alt="project.name"
                          class="w-full h-full object-cover"
                          @error="(e: Event) => {
                          const target = e.target as HTMLImageElement;
                          target.style.display = 'none';
                          const placeholder = target.parentElement?.querySelector('.cover-placeholder');
                          if (placeholder) placeholder.classList.remove('hidden');
                        }"
                        />
                        <div
                          class="hidden absolute inset-0 flex items-center justify-center bg-gray-200 dark:bg-gray-600 cover-placeholder"
                        >
                          <Book class="w-8 h-8 text-gray-400" />
                        </div>
                      </template>
                      <template v-else>
                        <div
                          class="w-full h-full bg-gradient-to-br from-blue-100 to-indigo-100 dark:from-gray-600 dark:to-gray-700 flex items-center justify-center"
                        >
                          <Book
                            class="w-10 h-10 text-blue-400 dark:text-blue-300"
                          />
                        </div>
                      </template>
                    </div>
                    <!-- Book shadow effect -->
                    <div
                      class="absolute -bottom-2 -right-2 w-full h-full bg-black/10 rounded-lg blur-md -z-10"
                    ></div>
                  </div>

                  <!-- Project Info -->
                  <div class="flex-1 flex flex-col justify-between min-w-0">
                    <div>
                      <h3
                        class="text-xl font-bold text-gray-900 dark:text-white mb-2 truncate"
                      >
                        {{ project.name }}
                      </h3>
                      <div class="flex items-center gap-2 mb-3">
                        <span class="text-sm text-gray-500 dark:text-gray-400">
                          {{ project.author }}
                        </span>
                        <span
                          class="px-2 py-0.5 text-xs font-mono bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded"
                        >
                          {{ project.project_id }}
                        </span>
                      </div>
                      <p
                        v-if="project.description"
                        class="text-sm text-gray-600 dark:text-gray-300 line-clamp-2"
                      >
                        {{ project.description }}
                      </p>
                    </div>
                    <div class="flex items-center justify-between mt-4">
                      <p class="text-xs text-gray-400 dark:text-gray-500">
                        {{ t("welcome.createdAt") }}:
                        {{ new Date(project.created_at).toLocaleDateString() }}
                      </p>
                      <n-button
                        type="primary"
                        size="small"
                        @click.stop="handleOpenProject(project)"
                        class="group-hover:scale-105 transition-transform"
                      >
                        <template #icon>
                          <NIcon>
                            <Edit3 class="w-4 h-4" />
                          </NIcon>
                        </template>
                        {{ t("welcome.open") }}
                      </n-button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Pagination -->
          <div
            v-if="projectStore.totalPages > 1"
            class="flex justify-center mt-8"
          >
            <n-pagination
              v-model:page="projectStore.currentPage"
              :page-count="projectStore.totalPages"
              :page-slot="7"
              show-quick-jumper
              @update:page="handlePageChange"
            >
              <template #prev>
                <span class="flex items-center gap-1">
                  <ChevronLeft class="w-4 h-4" />
                  {{ t("welcome.pagination.prev") }}
                </span>
              </template>
              <template #next>
                <span class="flex items-center gap-1">
                  {{ t("welcome.pagination.next") }}
                  <ChevronRight class="w-4 h-4" />
                </span>
              </template>
            </n-pagination>
          </div>

          <!-- Project count info -->
          <div
            v-if="projectStore.totalProjects > 0"
            class="text-center mt-4 text-sm text-gray-500 dark:text-gray-400"
          >
            {{ t("welcome.pagination.showing") }}
            {{ (projectStore.currentPage - 1) * projectStore.pageSize + 1 }}
            -
            {{
              Math.min(
                projectStore.currentPage * projectStore.pageSize,
                projectStore.totalProjects
              )
            }}
            {{ t("welcome.pagination.of") }}
            {{ projectStore.totalProjects }}
            {{ t("welcome.pagination.items") }}
          </div>
        </div>
      </section>
    </main>

    <!-- Create/Edit Project Modal -->
    <n-modal
      v-model:show="showModal"
      preset="card"
      :title="
        isEditing
          ? t('welcome.createProject.editTitle')
          : t('welcome.createProject.title')
      "
      style="width: 520px"
      :mask-closable="false"
    >
      <n-form :model="formData" label-placement="top">
        <n-form-item
          :label="t('welcome.createProject.name')"
          path="name"
          :rule="{
            required: true,
            message: t('welcome.createProject.placeholder.name'),
            trigger: ['input', 'blur'],
          }"
        >
          <n-input
            v-model:value="formData.name"
            :placeholder="t('welcome.createProject.placeholder.name')"
            maxlength="100"
            show-count
          />
        </n-form-item>

        <n-form-item :label="t('welcome.createProject.author')" path="author">
          <n-input
            v-model:value="formData.author"
            :placeholder="t('welcome.createProject.placeholder.author')"
            maxlength="50"
          />
        </n-form-item>

        <n-form-item
          :label="t('welcome.createProject.description')"
          path="description"
        >
          <n-input
            v-model:value="formData.description"
            type="textarea"
            :placeholder="t('welcome.createProject.placeholder.description')"
            :rows="3"
            maxlength="500"
            show-count
          />
        </n-form-item>

        <n-form-item
          v-if="!isEditing"
          :label="t('welcome.createProject.path')"
          path="path"
        >
          <div class="flex gap-2 w-full">
            <n-input
              v-model:value="formData.path"
              :placeholder="t('welcome.createProject.placeholder.path')"
              readonly
              class="flex-1"
            />
            <n-button @click="openFolderDialog">
              <template #icon>
                <NIcon>
                  <FolderOpen />
                </NIcon>
              </template>
              {{ t("welcome.createProject.selectPath") }}
            </n-button>
          </div>
        </n-form-item>

        <n-form-item v-else :label="t('welcome.createProject.path')">
          <n-input v-model:value="formData.path" readonly />
        </n-form-item>
      </n-form>

      <template #footer>
        <n-space justify="end">
          <n-button @click="closeModal" :disabled="isCreating">{{
            t("common.action.cancel")
          }}</n-button>
          <n-button
            type="primary"
            @click="isEditing ? handleEditProject() : handleCreateProject()"
            :loading="isCreating"
          >
            {{
              isEditing
                ? t("common.action.save")
                : t("welcome.createProject.title")
            }}
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 迁移预览弹窗 -->
    <n-modal
      v-model:show="showMigrationModal"
      preset="card"
      :title="t('welcome.migrate.previewTitle')"
      style="width: 640px"
      :mask-closable="false"
    >
      <div
        v-if="projectStore.isMigrating"
        class="flex flex-col items-center py-8 gap-4"
      >
        <n-spin size="large" />
        <p class="text-gray-500">{{ t("welcome.migrate.migrating") }}</p>
        <n-progress
          type="line"
          :percentage="migrationProgress"
          :height="8"
          :show-indicator="true"
        />
      </div>

      <div v-else class="space-y-4">
        <n-alert type="info" :bordered="false">
          {{
            t("welcome.migrate.previewDescription", {
              count: dryRunResult?.total || 0,
            })
          }}
        </n-alert>

        <div class="max-h-64 overflow-y-auto space-y-2">
          <div
            v-for="detail in dryRunResult?.details || []"
            :key="detail.project_db_id"
            class="p-3 rounded-lg bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700"
          >
            <div class="flex items-center justify-between">
              <span class="font-medium text-gray-900 dark:text-white">{{
                detail.old_name
              }}</span>
              <n-tag size="small" type="info">{{
                t("welcome.migrate.pendingTag")
              }}</n-tag>
            </div>
            <div class="mt-1 text-sm text-gray-500">
              <p class="font-mono text-xs">
                {{ detail.old_path }} →
                <span class="text-blue-500">PXXXXX</span>
              </p>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <n-space justify="end">
          <n-button
            @click="showMigrationModal = false"
            :disabled="projectStore.isMigrating"
            >{{ t("welcome.migrate.cancel") }}</n-button
          >
          <n-button
            type="primary"
            @click="executeMigration"
            :loading="projectStore.isMigrating"
          >
            <template #icon>
              <NIcon><Play /></NIcon>
            </template>
            {{ t("welcome.migrate.confirm") }}
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 快捷键说明弹窗 -->
    <n-modal
      v-model:show="showShortcuts"
      preset="card"
      :title="t('welcome.shortcuts.title')"
      style="width: 480px"
      :mask-closable="true"
    >
      <div class="space-y-4">
        <div
          class="p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800"
        >
          <p class="text-sm text-blue-800 dark:text-blue-300">
            {{ t("welcome.shortcuts.description") }}
          </p>
        </div>

        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-gray-700 dark:text-gray-300">{{
              t("welcome.shortcuts.save")
            }}</span>
            <kbd
              class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded border border-gray-300 dark:border-gray-600"
              >Ctrl + S</kbd
            >
          </div>

          <div class="flex items-center justify-between">
            <div>
              <span class="text-gray-700 dark:text-gray-300">{{
                t("welcome.shortcuts.typewriterMode")
              }}</span>
              <p class="text-xs text-gray-500 dark:text-gray-400">
                {{ t("welcome.shortcuts.typewriterDesc") }}
              </p>
            </div>
            <kbd
              class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded border border-gray-300 dark:border-gray-600"
              >Ctrl + Shift + T</kbd
            >
          </div>

          <div class="flex items-center justify-between">
            <div>
              <span class="text-gray-700 dark:text-gray-300">{{
                t("welcome.shortcuts.focusMode")
              }}</span>
              <p class="text-xs text-gray-500 dark:text-gray-400">
                {{ t("welcome.shortcuts.focusDesc") }}
              </p>
            </div>
            <kbd
              class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded border border-gray-300 dark:border-gray-600"
              >Ctrl + Shift + F</kbd
            >
          </div>

          <div class="flex items-center justify-between">
            <span class="text-gray-700 dark:text-gray-300">{{
              t("welcome.shortcuts.exitSpecialMode")
            }}</span>
            <kbd
              class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-700 rounded border border-gray-300 dark:border-gray-600"
              >Esc</kbd
            >
          </div>
        </div>
      </div>
    </n-modal>

    <!-- 删除项目确认弹窗 -->
    <DeleteConfirmModal
      v-model:show="showDeleteProjectModal"
      :title="t('welcome.dialog.deleteConfirm')"
      :message="
        t('welcome.dialog.deleteMessage', { name: projectToDelete?.name })
      "
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

:deep(.empty-no-project) {
  padding: 48px 0;
}

:deep(.empty-no-project .n-empty__icon) {
  margin-bottom: 16px;
}

:deep(.empty-no-project .n-empty__description) {
  margin-top: 8px;
}
</style>
