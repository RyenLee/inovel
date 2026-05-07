import { ref, computed } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";

const MAX_FILE_SIZE = 10 * 1024 * 1024;
const CHUNK_SIZE = 50000;
const PREVIEW_LENGTH = 2000;

export interface ImportFileInfo {
  name: string;
  path: string;
  size: number;
  encoding: string;
}

export interface ImportState {
  file: ImportFileInfo | null;
  content: string;
  preview: string;
  isReading: boolean;
  error: string | null;
}

export function useTextImport() {
  const state = ref<ImportState>({
    file: null,
    content: "",
    preview: "",
    isReading: false,
    error: null,
  });

  const hasContent = computed(() => state.value.content.length > 0);
  const isLargeFile = computed(() => state.value.content.length > CHUNK_SIZE);
  const chunkCount = computed(() =>
    Math.ceil(state.value.content.length / CHUNK_SIZE)
  );

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  const detectEncoding = (content: string): string => {
    // 简单检测编码
    if (content.charCodeAt(0) === 0xFEFF || content.charCodeAt(0) === 0xFFFE) {
      return "UTF-16";
    }
    // 检查是否包含中文
    const hasChinese = /[\u4e00-\u9fa5]/.test(content);
    // 检查是否包含特殊 Unicode 字符
    const hasSpecialUnicode = /[\u0080-\uFFFF]/.test(content);
    
    if (hasChinese || hasSpecialUnicode) {
      return "UTF-8";
    }
    return "ASCII/UTF-8";
  };

  const selectAndReadFile = async (): Promise<boolean> => {
    state.value = { file: null, content: "", preview: "", isReading: false, error: null };

    try {
      const selected = await open({
        filters: [{ name: "文本文件", extensions: ["txt"] }],
        multiple: false,
      });

      if (!selected || Array.isArray(selected)) {
        return false;
      }

      const fileName = selected.split("\\").pop() || selected.split("/").pop() || "unknown.txt";
      const ext = fileName.split(".").pop()?.toLowerCase();

      if (ext !== "txt") {
        state.value.error = "仅支持 .txt 格式的文本文件";
        return false;
      }

      state.value.isReading = true;

      // 使用自定义命令读取文件
      const content = await invoke<string>("read_text_file", { filePath: selected });

      if (!content || content.trim().length === 0) {
        state.value.error = "文件内容为空（仅包含空白字符）";
        state.value.isReading = false;
        return false;
      }

      // 获取文件大小
      const fileSize = await invoke<number>("get_file_size", { filePath: selected });

      const encoding = detectEncoding(content);

      state.value = {
        file: {
          name: fileName,
          path: selected,
          size: fileSize,
          encoding,
        },
        content,
        preview: content.slice(0, PREVIEW_LENGTH),
        isReading: false,
        error: null,
      };

      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : 
        (typeof err === 'string' ? err : "未知错误");
      state.value = {
        file: null,
        content: "",
        preview: "",
        isReading: false,
        error: `读取文件失败：${message}`,
      };
      return false;
    }
  };

  const getChunks = (): string[] => {
    const { content } = state.value;
    if (content.length <= CHUNK_SIZE) {
      return [content];
    }

    const chunks: string[] = [];
    let offset = 0;

    while (offset < content.length) {
      let end = offset + CHUNK_SIZE;
      if (end < content.length) {
        const searchStart = Math.max(offset, end - 200);
        const slice = content.slice(searchStart, end);
        const paraBreak = slice.lastIndexOf("\n\n");
        const lineBreak = slice.lastIndexOf("\n");
        const sentenceBreak = Math.max(
          slice.lastIndexOf("。"),
          slice.lastIndexOf("！"),
          slice.lastIndexOf("？"),
          slice.lastIndexOf("；"),
          slice.lastIndexOf(".")
        );

        if (paraBreak !== -1) {
          end = searchStart + paraBreak + 2;
        } else if (lineBreak !== -1) {
          end = searchStart + lineBreak + 1;
        } else if (sentenceBreak !== -1) {
          end = searchStart + sentenceBreak + 1;
        }
      }

      chunks.push(content.slice(offset, end));
      offset = end;
    }

    return chunks;
  };

  const reset = () => {
    state.value = { file: null, content: "", preview: "", isReading: false, error: null };
  };

  return {
    state,
    hasContent,
    isLargeFile,
    chunkCount,
    formatFileSize,
    selectAndReadFile,
    getChunks,
    reset,
  };
}
