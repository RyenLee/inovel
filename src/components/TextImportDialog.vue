<script setup lang="ts">
import { ref, watch } from "vue";
import {
  NModal,
  NButton,
  NIcon,
  NSpace,
  NText,
  NSpin,
  NAlert,
  NRadioGroup,
  NRadioButton,
  NTooltip,
  NProgress,
  useMessage,
} from "naive-ui";
import { FileText, Upload, AlertTriangle, CheckCircle } from "lucide-vue-next";
import { useTextImport } from "../composables/useTextImport";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();

const props = withDefaults(
  defineProps<{
    show: boolean;
    isDark?: boolean;
  }>(),
  {
    isDark: false,
  }
);

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
  (e: "import", payload: { content: string; mode: "insert" | "replace" }): void;
}>();

const message = useMessage();
const {
  state,
  hasContent,
  isLargeFile,
  chunkCount,
  formatFileSize,
  selectAndReadFile,
  getChunks,
  reset,
} = useTextImport();

const importMode = ref<"insert" | "replace">("insert");
const isImporting = ref(false);
const importProgress = ref(0);

watch(
  () => props.show,
  async (val) => {
    if (val) {
      reset();
      importMode.value = "insert";
      isImporting.value = false;
      importProgress.value = 0;
      await selectAndReadFile();
    }
  }
);

const handleRetry = async () => {
  await selectAndReadFile();
};

const handleImport = async () => {
  if (!hasContent.value) return;

  isImporting.value = true;
  importProgress.value = 0;

  try {
    const chunks = getChunks();
    const totalChunks = chunks.length;

    for (let i = 0; i < totalChunks; i++) {
      await new Promise<void>((resolve) => {
        requestAnimationFrame(() => {
          emit("import", {
            content: chunks[i],
            mode: i === 0 ? importMode.value : "insert",
          });
          importProgress.value = Math.round(((i + 1) / totalChunks) * 100);
          resolve();
        });
      });
    }

    message.success(
      t("textImport.messages.importSuccess", {
        size: formatFileSize(state.value.file?.size || 0),
      })
    );
    emit("update:show", false);
  } catch (err) {
    message.error(t("textImport.messages.importError"));
  } finally {
    isImporting.value = false;
  }
};

const handleClose = () => {
  if (!isImporting.value) {
    emit("update:show", false);
  }
};
</script>

<template>
  <n-modal
    :show="show"
    :mask-closable="!isImporting"
    :close-on-esc="!isImporting"
    @update:show="(val: boolean) => !isImporting && emit('update:show', val)"
  >
    <div
      class="import-dialog-card"
      :class="isDark ? 'import-dialog-dark' : 'import-dialog-light'"
    >
      <div class="import-dialog-header">
        <div class="flex items-center gap-3">
          <span class="import-dialog-icon">
            <FileText class="w-5 h-5" />
          </span>
          <div>
            <h3 class="import-dialog-title">{{ t("textImport.title") }}</h3>
            <p class="import-dialog-subtitle">{{ t("textImport.subtitle") }}</p>
          </div>
        </div>
      </div>

      <div class="import-dialog-body">
        <!-- 加载状态 -->
        <div
          v-if="state.isReading"
          class="flex flex-col items-center justify-center py-16"
        >
          <n-spin size="large" />
          <n-text depth="3" class="mt-4">{{ t("textImport.reading") }}</n-text>
        </div>

        <!-- 错误状态 -->
        <div
          v-else-if="state.error"
          class="flex flex-col items-center gap-4 py-8"
        >
          <div class="import-error-icon">
            <AlertTriangle class="w-10 h-10" />
          </div>
          <n-text class="import-error-text">{{ state.error }}</n-text>
          <div class="flex gap-3">
            <n-button @click="handleRetry" type="primary" size="small">
              <template #icon>
                <n-icon><Upload /></n-icon>
              </template>
              {{ t("textImport.reselect") }}
            </n-button>
            <n-button @click="handleClose" size="small">{{
              t("textImport.cancel")
            }}</n-button>
          </div>
        </div>

        <!-- 文件信息 + 预览 -->
        <div v-else-if="state.file" class="space-y-4">
          <!-- 文件信息卡片 -->
          <div class="import-file-info">
            <div class="flex items-center gap-2 mb-3">
              <CheckCircle class="w-4 h-4 text-green-500" />
              <n-text strong class="text-sm">{{
                t("textImport.readSuccess")
              }}</n-text>
            </div>
            <div class="import-file-details">
              <div class="import-file-detail">
                <span class="import-detail-label">{{
                  t("textImport.fileName")
                }}</span>
                <span class="import-detail-value">{{ state.file.name }}</span>
              </div>
              <div class="import-file-detail">
                <span class="import-detail-label">{{
                  t("textImport.fileSize")
                }}</span>
                <span class="import-detail-value">{{
                  formatFileSize(state.file.size)
                }}</span>
              </div>
              <div class="import-file-detail">
                <span class="import-detail-label">{{
                  t("textImport.encoding")
                }}</span>
                <span class="import-detail-value">{{
                  state.file.encoding
                }}</span>
              </div>
              <div class="import-file-detail">
                <span class="import-detail-label">{{
                  t("textImport.charCount")
                }}</span>
                <span class="import-detail-value">{{
                  state.content.length.toLocaleString()
                }}</span>
              </div>
            </div>

            <div v-if="isLargeFile" class="mt-3">
              <n-alert type="info" :bordered="false" class="text-xs!">
                {{ t("textImport.largeFileHint", { chunkCount }) }}
              </n-alert>
            </div>
          </div>

          <!-- 导入模式选择 -->
          <div class="import-mode-selector">
            <span class="import-detail-label mb-2 block">{{
              t("textImport.importMode")
            }}</span>
            <n-radio-group
              v-model:value="importMode"
              name="importMode"
              size="small"
            >
              <n-radio-button value="insert">
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <span>{{ t("textImport.insertAtCursor") }}</span>
                  </template>
                  {{ t("textImport.insertTooltip") }}
                </n-tooltip>
              </n-radio-button>
              <n-radio-button value="replace">
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <span>{{ t("textImport.replaceAll") }}</span>
                  </template>
                  {{ t("textImport.replaceTooltip") }}
                </n-tooltip>
              </n-radio-button>
            </n-radio-group>
          </div>

          <!-- 内容预览 -->
          <div class="import-preview-section">
            <div class="flex items-center justify-between mb-2">
              <span class="import-detail-label">{{
                t("textImport.contentPreview")
              }}</span>
              <span
                v-if="state.content.length > 2000"
                class="text-xs text-gray-400"
              >
                {{ t("textImport.previewHint") }}
              </span>
            </div>
            <div class="import-preview-box">
              <pre class="import-preview-text">{{ state.preview }}</pre>
              <div
                v-if="state.content.length > 2000"
                class="import-preview-fade"
              ></div>
            </div>
          </div>

          <!-- 导入进度 -->
          <div v-if="isImporting" class="space-y-2">
            <div class="flex items-center justify-between text-xs">
              <span class="text-gray-500">{{
                t("textImport.importProgress")
              }}</span>
              <span class="font-medium text-blue-500"
                >{{ importProgress }}%</span
              >
            </div>
            <n-progress
              :percentage="importProgress"
              :height="6"
              :border-radius="3"
              :show-indicator="false"
              processing
            />
          </div>
        </div>
      </div>

      <div class="import-dialog-footer">
        <n-space justify="end">
          <n-button @click="handleClose" :disabled="isImporting">{{
            t("textImport.cancel")
          }}</n-button>
          <n-button
            type="primary"
            :disabled="!hasContent || isImporting"
            :loading="isImporting"
            @click="handleImport"
          >
            <template #icon>
              <n-icon><Upload /></n-icon>
            </template>
            {{
              isImporting
                ? t("textImport.importing")
                : t("textImport.confirmImport")
            }}
          </n-button>
        </n-space>
      </div>
    </div>
  </n-modal>
</template>

<style scoped>
.import-dialog-card {
  width: 620px;
  max-width: 92vw;
  max-height: 88vh;
  border-radius: 14px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.18);
}

.import-dialog-light {
  background: #fff;
  border: 1px solid #e5e7eb;
}

.import-dialog-dark {
  background: #1f2937;
  border: 1px solid #374151;
}

.import-dialog-header {
  padding: 20px 24px 16px;
  border-bottom: 1px solid #e5e7eb;
}

.import-dialog-dark .import-dialog-header {
  border-bottom-color: #374151;
}

.import-dialog-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: linear-gradient(135deg, #3b82f6, #6366f1);
  color: #fff;
  flex-shrink: 0;
}

.import-dialog-title {
  font-size: 17px;
  font-weight: 600;
  margin: 0;
  color: #111827;
}

.import-dialog-dark .import-dialog-title {
  color: #f9fafb;
}

.import-dialog-subtitle {
  font-size: 13px;
  color: #6b7280;
  margin: 2px 0 0;
}

.import-dialog-dark .import-dialog-subtitle {
  color: #9ca3af;
}

.import-dialog-body {
  padding: 20px 24px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}

.import-dialog-footer {
  padding: 14px 24px;
  border-top: 1px solid #e5e7eb;
}

.import-dialog-dark .import-dialog-footer {
  border-top-color: #374151;
}

.import-error-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: #fef2f2;
  color: #ef4444;
}

.import-dialog-dark .import-error-icon {
  background: #450a0a;
  color: #fca5a5;
}

.import-error-text {
  font-size: 14px;
  color: #dc2626;
  text-align: center;
  max-width: 400px;
}

.import-dialog-dark .import-error-text {
  color: #fca5a5;
}

.import-file-info {
  background: #f9fafb;
  border: 1px solid #e5e7eb;
  border-radius: 10px;
  padding: 16px;
}

.import-dialog-dark .import-file-info {
  background: #111827;
  border-color: #374151;
}

.import-file-details {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px 16px;
}

.import-file-detail {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.import-detail-label {
  font-size: 11px;
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.import-detail-value {
  font-size: 13px;
  color: #111827;
  font-weight: 500;
  word-break: break-all;
}

.import-dialog-dark .import-detail-value {
  color: #e5e7eb;
}

.import-mode-selector {
  padding: 12px 16px;
  background: #f9fafb;
  border-radius: 8px;
  border: 1px solid #e5e7eb;
}

.import-dialog-dark .import-mode-selector {
  background: #111827;
  border-color: #374151;
}

.import-preview-section {
  margin-top: 4px;
}

.import-preview-box {
  position: relative;
  max-height: 220px;
  overflow: hidden;
  background: #1a1a2e;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.import-dialog-dark .import-preview-box {
  background: #0f0f1a;
}

.import-preview-text {
  font-size: 12px;
  line-height: 1.7;
  color: #d4d4d8;
  padding: 14px;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: "Consolas", "Monaco", "Courier New", monospace;
}

.import-preview-fade {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 50px;
  background: linear-gradient(transparent, #1a1a2e);
  pointer-events: none;
}

.import-dialog-dark .import-preview-fade {
  background: linear-gradient(transparent, #0f0f1a);
}

/* 响应式 */
@media screen and (max-width: 640px) {
  .import-dialog-card {
    width: 96vw;
    max-height: 92vh;
  }

  .import-file-details {
    grid-template-columns: 1fr;
  }

  .import-preview-box {
    max-height: 150px;
  }
}
</style>
