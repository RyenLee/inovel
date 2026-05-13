<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import {
  NModal,
  NButton,
  NIcon,
  NSwitch,
  NSpace,
  NTabs,
  NTabPane,
  NTable,
  NTag,
  useMessage,
  NEmpty,
  NSpin,
  NInput,
} from "naive-ui";
import { Package, FolderOpen, Trash2, Upload, Clock, BarChart3 } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();

const props = defineProps<{
  show: boolean;
  projectId: number;
}>();

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
}>();

const message = useMessage();

const isLoading = ref(false);
const isBackingUp = ref(false);
const excludeExports = ref(true);
const isFullBackup = ref(true);
const backupDescription = ref("");
const activeTab = ref("create");

// Data
const backups = ref<BackupRecord[]>([]);
const logs = ref<BackupLogEntry[]>([]);
const stats = ref<BackupStats | null>(null);

interface BackupRecord {
  id: number;
  project_id: number;
  backup_type: string;
  file_path: string;
  file_size: number;
  git_commit: string | null;
  description: string;
  created_at: string;
  status: string;
}

interface BackupLogEntry {
  id: number;
  project_id: number;
  backup_id: number | null;
  operation: string;
  message: string;
  level: string;
  created_at: string;
}

interface BackupStats {
  total: number;
  full_count: number;
  incr_count: number;
  total_size: number;
  last_backup: string | null;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleString("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

// 日志默认显示数量
const LOG_DISPLAY_LIMIT = 5;
const showAllLogs = ref(false);

async function loadData() {
  isLoading.value = true;
  try {
    const [b, l, s] = await Promise.all([
      invoke<BackupRecord[]>("list_backups", { project_id: props.projectId }),
      invoke<BackupLogEntry[]>("get_backup_logs", { project_id: props.projectId, limit: 50 }),
      invoke<BackupStats>("get_backup_stats", { project_id: props.projectId }),
    ]);
    backups.value = b;
    // Sort logs by created_at descending (newest first)
    logs.value = l.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
    stats.value = s;
  } catch (e) {
    console.error(t('backup.messages.loadFailed') + ":", e);
  } finally {
    isLoading.value = false;
  }
}

// Get logs to display (limited by LOG_DISPLAY_LIMIT unless showAllLogs is true)
const displayedLogs = computed(() => {
  if (showAllLogs.value) {
    return logs.value;
  }
  return logs.value.slice(0, LOG_DISPLAY_LIMIT);
});

async function doBackup() {
  isBackingUp.value = true;

  try {
    let path: string;
    if (isFullBackup.value) {
      path = await invoke<string>("backup_project", {
        project_id: props.projectId,
        exclude_exports: excludeExports.value,
        description: backupDescription.value || null,
      });
    } else {
      path = await invoke<string>("create_incremental_backup", {
        project_id: props.projectId,
        exclude_exports: excludeExports.value,
        description: backupDescription.value || null,
      });
    }
    message.success(t('backup.messages.backupSuccess', { path }));
    await loadData();
    activeTab.value = "history";
  } catch (error) {
    message.error(String(error));
  } finally {
    isBackingUp.value = false;
  }
}

async function restoreBackup(backup: BackupRecord) {
  if (!confirm(t('backup.messages.restoreConfirm', { description: backup.description, time: formatDate(backup.created_at) }))) {
    return;
  }

  try {
    await invoke("restore_backup", {
      project_id: props.projectId,
      backup_id: backup.id,
    });
    message.success(t('backup.messages.restoreSuccess'));
    await loadData();
  } catch (error) {
    message.error(t('backup.messages.restoreFailed', { error }));
  }
}

async function deleteBackup(backup: BackupRecord) {
  if (!confirm(t('backup.messages.deleteConfirm'))) {
    return;
  }
  try {
    await invoke("delete_backup_record", {
      project_id: props.projectId,
      backup_id: backup.id,
    });
    message.success(t('backup.messages.deleteSuccess'));
    await loadData();
  } catch (error) {
    message.error(t('backup.messages.deleteFailed', { error }));
  }
}

async function openBackupFolder() {
  try {
    await invoke("open_folder_in_explorer", { project_id: props.projectId });
  } catch {
    message.error(t('backup.messages.openFolderFailed'));
  }
}

onMounted(loadData);
</script>

<template>
  <n-modal
    :show="show"
    @update:show="(v: boolean) => emit('update:show', v)"
    preset="card"
    :title="t('backup.title')"
    :style="{ width: '720px', maxWidth: '95vw' }"
    :mask-closable="false"
  >
    <n-tabs v-model:value="activeTab" type="line" animated>
      <!-- 创建备份 -->
      <n-tab-pane name="create" :tab="t('backup.tabs.create')">
        <div class="space-y-5">
          <!-- 备份说明 -->
          <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3 text-sm text-blue-700 dark:text-blue-300">
            <p class="mb-1">
              <strong>{{ t('backup.fullBackup') }}</strong>{{ t('backup.backupInfo.fullDesc') }}<br />
              <strong>{{ t('backup.incrementalBackup') }}</strong>{{ t('backup.backupInfo.incrDesc') }}
            </p>
          </div>

          <!-- 备份类型 -->
          <div class="flex gap-4">
            <n-button
              :type="isFullBackup ? 'primary' : 'default'"
              @click="isFullBackup = true"
              class="flex-1"
            >
              {{ t('backup.fullBackup') }}
            </n-button>
            <n-button
              :type="!isFullBackup ? 'primary' : 'default'"
              @click="isFullBackup = false"
              class="flex-1"
            >
              {{ t('backup.incrementalBackup') }}
            </n-button>
          </div>

          <!-- 选项 -->
          <div class="flex items-center justify-between">
            <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('backup.excludeExports') }}</span>
            <n-switch v-model:value="excludeExports" />
          </div>

          <!-- 描述 -->
          <div>
            <label class="block text-sm font-medium mb-1.5 text-gray-700 dark:text-gray-300">
              {{ t('backup.description') }}
            </label>
            <n-input
              v-model:value="backupDescription"
              :placeholder="t('backup.descriptionPlaceholder')"
              clearable
            />
          </div>

          <!-- 统计摘要 -->
          <div v-if="stats && stats.total > 0" class="grid grid-cols-3 gap-3">
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-3 text-center">
              <div class="text-xl font-bold text-blue-600">{{ stats.total }}</div>
              <div class="text-xs text-gray-500">{{ t('backup.stats.totalBackups') }}</div>
            </div>
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-3 text-center">
              <div class="text-xl font-bold text-green-600">{{ formatSize(stats.total_size) }}</div>
              <div class="text-xs text-gray-500">{{ t('backup.stats.totalSize') }}</div>
            </div>
            <div class="bg-gray-50 dark:bg-gray-800 rounded-lg p-3 text-center">
              <div class="text-xl font-bold text-orange-600">{{ stats.incr_count }}</div>
              <div class="text-xs text-gray-500">{{ t('backup.stats.incrementalCount') }}</div>
            </div>
          </div>

          <!-- 按钮组 -->
          <div class="flex gap-3">
            <n-button
              type="primary"
              block
              size="large"
              :loading="isBackingUp"
              @click="doBackup"
            >
              <template #icon>
                <n-icon><Package /></n-icon>
              </template>
              {{ isBackingUp ? t('backup.backingUp') : isFullBackup ? t('backup.createFullBackup') : t('backup.createIncrementalBackup') }}
            </n-button>
            <n-button @click="openBackupFolder">
              <template #icon>
                <n-icon><FolderOpen /></n-icon>
              </template>
              {{ t('backup.openFolder') }}
            </n-button>
          </div>
        </div>
      </n-tab-pane>

      <!-- 备份历史 -->
      <n-tab-pane name="history" :tab="t('backup.tabs.history')">
        <n-spin :show="isLoading">
          <div v-if="backups.length === 0" class="py-8">
            <n-empty :description="t('backup.noBackups')" />
          </div>
          <div v-else class="space-y-2">
            <div
              v-for="b in backups"
              :key="b.id"
              class="border border-gray-200 dark:border-gray-700 rounded-lg p-3 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1">
                    <n-tag
                      :type="b.backup_type === 'full' ? 'primary' : 'warning'"
                      size="small"
                      round
                    >
                      {{ b.backup_type === 'full' ? t('backup.full') : t('backup.incremental') }}
                    </n-tag>
                    <span class="text-sm font-medium text-gray-800 dark:text-gray-200 truncate">
                      {{ b.description || (b.backup_type === 'full' ? t('backup.fullBackup') : t('backup.incrementalBackup')) }}
                    </span>
                  </div>
                  <div class="flex items-center gap-3 text-xs text-gray-500">
                    <span class="flex items-center gap-1">
                      <Clock class="w-3 h-3" />
                      {{ formatDate(b.created_at) }}
                    </span>
                    <span>{{ formatSize(b.file_size) }}</span>
                    <span v-if="b.git_commit" class="font-mono text-xs">commit {{ b.git_commit.substring(0, 7) }}</span>
                  </div>
                  <p class="text-xs text-gray-400 mt-1 truncate font-mono">
                    {{ b.file_path }}
                  </p>
                </div>
                <div class="flex items-center gap-1 shrink-0">
                  <n-button
                    size="tiny"
                    type="warning"
                    quaternary
                    @click="restoreBackup(b)"
                  >
                    <template #icon><n-icon><Upload /></n-icon></template>
                    {{ t('backup.restore') }}
                  </n-button>
                  <n-button
                    size="tiny"
                    type="error"
                    quaternary
                    @click="deleteBackup(b)"
                  >
                    <template #icon><n-icon><Trash2 /></n-icon></template>
                  </n-button>
                </div>
              </div>
            </div>
          </div>
        </n-spin>
      </n-tab-pane>

      <!-- 操作日志 -->
      <n-tab-pane name="logs" :tab="t('backup.tabs.logs')">
        <n-spin :show="isLoading">
          <div v-if="logs.length === 0" class="py-8">
            <n-empty :description="t('backup.noLogs')" />
          </div>
          <div v-else>
            <div class="space-y-1 max-h-64 overflow-y-auto">
              <div
                v-for="log in displayedLogs"
                :key="log.id"
                class="flex flex-wrap items-start gap-2 py-1.5 text-sm"
              >
                <span
                  class="shrink-0 px-1.5 py-0.5 rounded text-xs font-medium"
                  :class="{
                    'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300': log.level === 'info',
                    'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300': log.level === 'warn',
                    'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300': log.level === 'error',
                  }"
                >
                  {{ log.level === 'warn' ? 'WARN' : log.level.toUpperCase() }}
                </span>
                <span class="text-gray-500 shrink-0 text-xs">
                  {{ formatDate(log.created_at) }}
                </span>
                <span
                  class="shrink-0 text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300"
                >
                  {{ log.operation }}
                </span>
                <span class="text-gray-700 dark:text-gray-300 text-xs break-all word-wrap max-w-[400px]">
                  {{ log.message }}
                </span>
              </div>
            </div>
            <!-- Show more button -->
            <div v-if="logs.length > LOG_DISPLAY_LIMIT" class="pt-3 mt-3 border-t border-gray-200 dark:border-gray-700 text-center">
              <n-button
                size="small"
                text
                @click="showAllLogs = !showAllLogs"
              >
                {{ showAllLogs ? t('backup.showLess', { limit: LOG_DISPLAY_LIMIT }) : t('backup.showAll', { total: logs.length }) }}
              </n-button>
            </div>
          </div>
        </n-spin>
      </n-tab-pane>
    </n-tabs>
  </n-modal>
</template>
