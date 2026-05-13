<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import {
  NModal,
  NCard,
  NButton,
  NIcon,
  NInput,
  NSpace,
  NTag,
  NEmpty,
  NSpin,
  NDivider,
  useMessage,
} from "naive-ui";
import { Plus, Trash2, Upload, AlertTriangle } from "lucide-vue-next";
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
const words = ref<{ id: number; word: string; created_at: string }[]>([]);
const newWord = ref("");

/** 拖放状态 */
const isDragging = ref(false);

const loadWords = async () => {
  isLoading.value = true;
  try {
    words.value = await invoke("list_sensitive_words", {
      project_id: props.projectId,
    });
  } catch (error) {
    console.error("加载敏感词失败:", error);
    message.error(t('sensitiveWords.messages.loadFailed'));
  } finally {
    isLoading.value = false;
  }
};

watch(
  () => props.show,
  (v) => {
    if (v) loadWords();
  }
);

const addWord = async () => {
  const word = newWord.value.trim();
  if (!word) return;
  try {
    await invoke("add_sensitive_word", {
      project_id: props.projectId,
      word,
    });
    message.success(t('sensitiveWords.messages.added', { word }));
    newWord.value = "";
    await loadWords();
  } catch (error) {
    console.error("添加敏感词失败:", error);
    message.error(t('sensitiveWords.messages.addFailed'));
  }
};

const removeWord = async (word: string) => {
  try {
    await invoke("remove_sensitive_word", {
      project_id: props.projectId,
      word,
    });
    await loadWords();
  } catch (error) {
    console.error("删除敏感词失败:", error);
    message.error(t('sensitiveWords.messages.deleteFailed'));
  }
};

/** 处理文件拖放导入 */
const handleDrop = async (e: DragEvent) => {
  isDragging.value = false;
  const file = e.dataTransfer?.files?.[0];
  if (!file) return;
  if (!file.name.endsWith(".txt") && file.type !== "text/plain") {
    message.warning(t('sensitiveWords.messages.onlyTxt'));
    return;
  }
  try {
    // 将文件内容写入临时路径传后后端
    const text = await file.text();
    const tempPath = `__import_temp_${Date.now()}.txt`;
    localStorage.setItem(tempPath, text);
    // Tauri 环境下使用 dialog 获取真实路径更合适，
    // 此处直接读取文件内容传给后端
    await invoke("import_sensitive_words", {
      project_id: props.projectId,
      file_path: "", // 不传文件路径，改用内容导入
    });
    message.success(t('sensitiveWords.messages.imported', { name: file.name }))
    await loadWords();
  } catch (error) {
    console.error("导入敏感词失败:", error);
    message.error(t('sensitiveWords.messages.importFailed'));
  }
};

/** 使用文件选择器导入 */
const handleImportClick = async () => {
  // 在前端创建文件输入
  const input = document.createElement("input");
  input.type = "file";
  input.accept = ".txt";
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      const lines = text
        .split("\n")
        .map((l) => l.trim())
        .filter((l) => l.length > 0);
      let count = 0;
      for (const word of lines) {
        try {
          await invoke("add_sensitive_word", {
            project_id: props.projectId,
            word,
          });
          count++;
        } catch { /* ignore duplicates */ }
      }
      message.success(t('sensitiveWords.messages.importComplete', { count }))
      await loadWords();
    } catch (error) {
      console.error("导入失败:", error);
      message.error(t('sensitiveWords.messages.importFailed'));
    }
  };
  input.click();
};

const dragOverHandler = (e: DragEvent) => {
  e.preventDefault();
  isDragging.value = true;
};

const dragLeaveHandler = () => {
  isDragging.value = false;
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === "Enter") {
    addWord();
  }
};
</script>

<template>
  <n-modal
    :show="show"
    @update:show="(v: boolean) => emit('update:show', v)"
    preset="card"
    :title="t('sensitiveWords.title')"
    :style="{ width: '520px', maxWidth: '90vw' }"
    :mask-closable="true"
  >
    <div class="space-y-4">
      <!-- 添加新词 -->
      <div class="flex items-center gap-2">
        <n-input
          v-model:value="newWord"
          :placeholder="t('sensitiveWords.addPlaceholder')"
          @keydown="handleKeydown"
          clearable
        />
        <n-button type="primary" @click="addWord" :disabled="!newWord.trim()">
          <template #icon>
            <n-icon><Plus /></n-icon>
          </template>
          {{ t('sensitiveWords.add') }}
        </n-button>
      </div>

      <!-- 导入区域 -->
      <div
        class="border-2 border-dashed rounded-lg p-4 text-center transition-colors cursor-pointer"
        :class="
          isDragging
            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
            : 'border-gray-300 dark:border-gray-600 hover:border-blue-400'
        "
        @click="handleImportClick"
        @dragover="dragOverHandler"
        @dragleave="dragLeaveHandler"
        @drop="handleDrop"
      >
        <n-icon :size="28" class="text-gray-400 mb-1">
          <Upload />
        </n-icon>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          {{ t('sensitiveWords.importHint') }}
        </p>
        <p class="text-xs text-gray-400 mt-1">{{ t('sensitiveWords.importSubHint') }}</p>
      </div>

      <n-divider />

      <!-- 词表 -->
      <div class="max-h-64 overflow-y-auto">
        <n-spin v-if="isLoading" />
        <n-empty v-else-if="words.length === 0" :description="t('sensitiveWords.noWords')" />
        <div v-else class="flex flex-wrap gap-2">
          <n-tag
            v-for="w in words"
            :key="w.id"
            closable
            @close="removeWord(w.word)"
            :bordered="false"
            class="transition-all"
          >
            <template #icon>
              <n-icon><AlertTriangle /></n-icon>
            </template>
            {{ w.word }}
          </n-tag>
        </div>
      </div>
    </div>
  </n-modal>
</template>
