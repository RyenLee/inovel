import { ref, toRef, type Ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface UseWordCountOptions {
  chapterId: number | null | Ref<number | null>;
  onWordCountUpdated?: (count: number) => void;
}

export function useWordCount(options: UseWordCountOptions) {
  const { onWordCountUpdated } = options;

  const chapterIdRef = toRef(options, 'chapterId');

  const wordCount = ref(0);
  const wordCountSaveTimer = ref<ReturnType<typeof setTimeout> | null>(null);

  // 计算字数（中文+英文）
  const calculateWordCount = (text: string): number => {
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishLetters = (text.match(/[a-zA-Z]/g) || []).length;
    const digits = (text.match(/[0-9]/g) || []).length;
    return chineseChars + englishLetters + digits;
  };

  // 更新字数
  const updateWordCount = (text: string) => {
    wordCount.value = calculateWordCount(text);
    updateWordCountCache();
  };

  // 更新字数缓存到后端（带防抖）
  const updateWordCountCache = async () => {
    if (wordCountSaveTimer.value) {
      clearTimeout(wordCountSaveTimer.value);
    }

    wordCountSaveTimer.value = setTimeout(async () => {
      if (chapterIdRef.value && wordCount.value >= 0) {
        try {
          await invoke("update_chapter_word_count", {
            chapterId: chapterIdRef.value,
            wordCount: wordCount.value,
          });
          onWordCountUpdated?.(wordCount.value);
        } catch (error) {
          console.error("更新字数失败:", error);
        }
      }
    }, 2000);
  };

  // 手动设置字数
  const setWordCount = (count: number) => {
    wordCount.value = count;
  };

  // 清理定时器
  const cleanup = () => {
    if (wordCountSaveTimer.value) {
      clearTimeout(wordCountSaveTimer.value);
    }
  };

  return {
    wordCount,
    calculateWordCount,
    updateWordCount,
    setWordCount,
    cleanup,
  };
}