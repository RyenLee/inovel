<script setup lang="ts">
import { ref } from "vue";
import {
  NModal,
  NButton,
  NIcon,
  NCheckboxGroup,
  NCheckbox,
  NSpace,
  useMessage,
} from "naive-ui";
import { Download, FolderOpen } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";

const props = defineProps<{
  show: boolean;
  projectId: number;
}>();

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
}>();

const message = useMessage();

const formats = ref<string[]>(["txt"]);
const isExporting = ref(false);
const exportResult = ref<{ folderPath: string; formats: string[] } | null>(null);

const formatOptions = [
  { label: "纯文本 (.txt)", value: "txt" },
  { label: "Markdown (.md)", value: "markdown" },
  { label: "EPUB 电子书 (.epub)", value: "epub" },
  { label: "PDF 文档 (.pdf)", value: "pdf" },
];



const doExport = async () => {
  if (formats.value.length === 0) {
    message.warning("请至少选择一种导出格式");
    return;
  }

  isExporting.value = true;
  exportResult.value = null;

  let successCount = 0;
  const succeededFormats: string[] = [];
  let lastFolderPath = "";

  for (const fmt of formats.value) {
    try {
      if (fmt === "txt") {
        const path = await invoke<string>("export_txt", { projectId: props.projectId });
        lastFolderPath = path.substring(0, path.lastIndexOf("\\"));
        succeededFormats.push("txt");
        successCount++;
      } else if (fmt === "markdown") {
        const path = await invoke<string>("export_markdown", { projectId: props.projectId });
        lastFolderPath = path.substring(0, path.lastIndexOf("\\"));
        succeededFormats.push("md");
        successCount++;
      } else if (fmt === "epub") {
        const path = await invoke<string>("export_epub", { projectId: props.projectId });
        lastFolderPath = path.substring(0, path.lastIndexOf("\\"));
        succeededFormats.push("epub");
        successCount++;
      } else if (fmt === "pdf") {
        // PDF 导出：Rust 写 HTML 文件，前端用 file:// URL 打开并触发打印
        const htmlPath = await invoke<string>("export_html_for_print", { projectId: props.projectId });
        lastFolderPath = htmlPath.substring(0, htmlPath.lastIndexOf("\\"));
        // 转换为 file:// URL 在浏览器中打开
        const fileUrl = "file:///" + htmlPath.replace(/\\/g, "/");
        const printWindow = window.open(fileUrl, "_blank");
        if (printWindow) {
          printWindow.onload = () => {
            printWindow.print();
          };
        }
        succeededFormats.push("pdf");
        successCount++;
      }
    } catch (error) {
      console.error(`导出 ${fmt} 失败:`, error);
      message.error(`导出 ${fmt.toUpperCase()} 失败`);
    }
  }

  if (succeededFormats.length > 0) {
    // 获取 exports 目录：优先用已知文件路径推导，否则调用 Rust 命令
    if (!lastFolderPath) {
      try {
        lastFolderPath = await invoke<string>("get_exports_dir", { projectId: props.projectId });
      } catch {
        lastFolderPath = "";
      }
    }
    exportResult.value = { folderPath: lastFolderPath, formats: succeededFormats };
  }

  if (successCount > 0) {
    message.success(`导出完成，成功 ${successCount}/${formats.value.length} 个文件`);
  }

  isExporting.value = false;
};

const openExportFolder = async () => {
  if (!exportResult.value) return;
  try {
    await invoke("open_folder_in_explorer", { projectId: props.projectId });
  } catch (error) {
    console.error("打开文件夹失败:", error);
    message.error("无法打开文件夹");
  }
};
</script>

<template>
  <n-modal
    :show="show"
    @update:show="(v: boolean) => emit('update:show', v)"
    preset="card"
    title="导出文档"
    :style="{ width: '460px', maxWidth: '90vw' }"
    :mask-closable="true"
  >
    <div class="space-y-5">
      <!-- 格式选择 -->
      <div>
        <label class="block text-sm font-medium mb-2 text-gray-700 dark:text-gray-300">选择导出格式</label>
        <n-checkbox-group v-model:value="formats">
          <n-space vertical :size="10">
            <n-checkbox
              v-for="opt in formatOptions"
              :key="opt.value"
              :value="opt.value"
              :label="opt.label"
            />
          </n-space>
        </n-checkbox-group>
      </div>

      <!-- 导出按钮 -->
      <n-button
        type="primary"
        block
        size="large"
        :loading="isExporting"
        :disabled="formats.length === 0"
        @click="doExport"
      >
        <template #icon>
          <n-icon><Download /></n-icon>
        </template>
        {{ isExporting ? "导出中..." : "开始导出" }}
      </n-button>

      <!-- 导出结果 -->
      <div v-if="exportResult" class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-3">
        <p class="text-sm text-green-700 dark:text-green-300 mb-2">
          导出完成，文件保存在：
        </p>
        <p class="text-xs text-gray-500 dark:text-gray-400 break-all mb-3 font-mono">
          {{ exportResult.folderPath }}
        </p>
        <n-button size="small" @click="openExportFolder">
          <template #icon>
            <n-icon><FolderOpen /></n-icon>
          </template>
          打开文件夹
        </n-button>
      </div>
    </div>
  </n-modal>
</template>
