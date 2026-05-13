<script setup lang="ts">
import { ref, toRef, toRefs, computed } from "vue";
import { marked } from "marked";
import {
  NButton,
  NIcon,
  NSpace,
  NTooltip,
  NDropdown,
  NModal,
  NSlider,
  NText,
} from "naive-ui";
import { useEditorComposable } from "../composables/useEditor";
import { useWordCount } from "../composables/useWordCount";
import { useTextBeautify } from "../composables/useTextBeautify";
import { useEditorLayout } from "../composables/useEditorLayout";
import {
  Bold,
  Italic,
  Heading1,
  Heading2,
  Heading3,
  List,
  ListOrdered,
  Quote,
  Minus,
  Type,
  BookOpen,
  History,
  Camera,
  FileText,
  Plus,
  Replace,
  Wand2,
  Eye,
  Upload,
  X,
  Sparkles,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { useFolderDialog } from "../composables/useFolderDialog";
import { readFile } from "@tauri-apps/plugin-fs";
import type { EditorMode } from "../stores/editor";
import TemplateSelector from "./TemplateSelector.vue";
import TextImportDialog from "./TextImportDialog.vue";
import { useLocale } from "../i18n/composables/useLocale";
import { useTemplateMerge } from "../composables/useTemplateMerge";

// Configure marked
marked.setOptions({
  breaks: true,
  gfm: true,
});

const { t } = useLocale();
const props = defineProps<{
  modelValue: string;
  chapterId: number | null;
  projectId?: number | null;
  volumeWordCount?: number;
  totalWordCount?: number;
  isDark?: boolean;
  editorMode?: EditorMode;
}>();
const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
  (e: "requestSave"): void;
  (e: "exitSpecialMode"): void;
  (e: "mention-click", id: string): void;
  (e: "show-history"): void;
  (e: "create-snapshot"): void;
  (e: "word-count-updated", count: number): void;
  (e: "open-name-generator"): void;
}>();
const editorRootRef = ref<HTMLElement | null>(null);
const editorContainerRef = ref<HTMLElement | null>(null);
const modelValueRef = toRef(props, "modelValue");
const projectIdRef = toRef(props, "projectId");
const editorModeRef = toRef(props, "editorMode");

const {
  editor,
  wordCount: editorWordCount,
  EditorContent,
  toggleBold,
  toggleItalic,
  setHeading,
  toggleBulletList,
  toggleOrderedList,
  toggleBlockquote,
  toggleHorizontalRule,
  isActive,
} = useEditorComposable({
  modelValue: modelValueRef,
  projectId: projectIdRef,
  editorMode: editorModeRef,
  onContentChange: (html) => {
    emit("update:modelValue", html);
  },
  onWordCountUpdate: (count) => {
    emit("word-count-updated", count);
  },
  onMentionClick: (id) => {
    emit("mention-click", id);
  },
});
const chapterIdRef = toRef(props, "chapterId");

const { wordCount } = useWordCount({
  chapterId: chapterIdRef,
  onWordCountUpdated: (count) => {
    emit("word-count-updated", count);
  },
});
const beautify = useTextBeautify({
  editor,
  onContentChange: (html) => {
    emit("update:modelValue", html);
    emit("requestSave");
  },
});
const {
  paperStyle,
  lineHeight,
  showLineHeightControl,
  showSplitDialog,
  splitThreshold,
  splitPreview,
  beautifyDropdownOptions,
} = toRefs(beautify);

// 行高比例
const lineHeightRatio = computed(() => {
  const fontSize = 14;
  return lineHeight.value / fontSize;
});
const lineHeightPx = computed(() => `${lineHeight.value}px`);
const lineHeightNum = computed(() => lineHeight.value);
useEditorLayout({
  editorRootRef: () => editorRootRef.value,
  editor,
});
const { selectFile } = useFolderDialog();
const showTemplateSelector = ref(false);
const showTextImportDialog = ref(false);
const templateInsertMode = ref<"replace" | "append" | "merge">("replace");
const isApplyingTemplate = ref(false);
const { merge } = useTemplateMerge();
const markdownToHtml = (markdown: string): string => {
  if (!markdown) return "";
  try {
    return marked.parse(markdown) as string;
  } catch (error) {
    console.error("Markdown parsing failed:", error);
    return markdown;
  }
};
const htmlToText = (html: string): string => {
  if (!html) return "";
  // 简单的 HTML 转文本函数
  return html
    .replace(/<br\s*\/?>/g, "\n")
    .replace(/<p\s*[^>]*>/g, "")
    .replace(/<\/p>/g, "\n\n")
    .replace(/<h[123]\s*[^>]*>/g, "")
    .replace(/<\/h[123]>/g, "\n\n")
    .replace(/<li\s*[^>]*>/g, "\n- ")
    .replace(/<\/li>/g, "")
    .replace(/<ul\s*[^>]*>/g, "")
    .replace(/<\/ul>/g, "\n")
    .replace(/<ol\s*[^>]*>/g, "")
    .replace(/<\/ol>/g, "\n")
    .replace(/<blockquote\s*[^>]*>/g, "")
    .replace(/<\/blockquote>/g, "\n\n")
    .replace(/<[^>]+>/g, "") // 移除所有标签
    .replace(/&nbsp;/g, " ")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&")
    .replace(/&quot;/g, '"')
    .replace(/&apos;/g, "'")
    .trim();
};

const handleTemplateSelect = async (
  payload:
    | string
    | {
        content: string;
        mode: "replace" | "append" | "merge";
      }
) => {
  if (!editor.value) {
    console.error(t("markdownEditor.messages.editorNotInitialized"));
    return;
  }
  let content: string;
  let mode: "replace" | "append" | "merge";
  if (typeof payload === "string") {
    content = payload;
    mode = templateInsertMode.value;
  } else {
    content = payload.content;
    mode = payload.mode;
    templateInsertMode.value = mode;
  }
  try {
    isApplyingTemplate.value = true;

    if (mode === "merge") {
      // 智能合并模式：需要获取当前文本内容并合并
      const currentHtml = editor.value.getHTML();
      const currentText = htmlToText(currentHtml);
      const mergedText = merge(content, currentText, {
        mode: "merge",
        dedupEnabled: true,
      });
      const mergedHtml = markdownToHtml(mergedText);
      editor.value.commands.setContent(mergedHtml || mergedText, {
        emitUpdate: false,
      });
    } else if (mode === "append") {
      // 追加模式：在当前内容后面添加
      const htmlContent = markdownToHtml(content);
      editor.value.commands.insertContent(htmlContent || content);
    } else {
      // 替换模式：直接替换
      const htmlContent = markdownToHtml(content);
      editor.value.commands.setContent(htmlContent || content, {
        emitUpdate: false,
      });
    }

    await new Promise((resolve) => setTimeout(resolve, 0));
    const newHtml = editor.value.getHTML();
    emit("update:modelValue", newHtml);
    const text = editor.value.getText();
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishLetters = (text.match(/[a-zA-Z]/g) || []).length;
    const digits = (text.match(/[0-9]/g) || []).length;
    wordCount.value = chineseChars + englishLetters + digits;
    showTemplateSelector.value = false;
    await new Promise((resolve) => setTimeout(resolve, 0));
    editor.value?.commands.focus();
  } catch (error) {
    console.error("应用模板失败:", error);
    if (editor.value) {
      editor.value.commands.insertContent(content);
    }
  } finally {
    isApplyingTemplate.value = false;
  }
};
const handleTemplateSelectorClose = () => {
  showTemplateSelector.value = false;
};
const handleTextImport = (payload: {
  content: string;
  mode: "insert" | "replace";
}) => {
  if (!editor.value) return;

  if (payload.mode === "replace") {
    editor.value.commands.setContent(payload.content, { emitUpdate: false });
  } else {
    editor.value.commands.insertContent(payload.content);
  }

  const newHtml = editor.value.getHTML();
  emit("update:modelValue", newHtml);
  emit("requestSave");
};
const toggleTemplateMode = () => {
  if (templateInsertMode.value === "replace") {
    templateInsertMode.value = "append";
  } else if (templateInsertMode.value === "append") {
    templateInsertMode.value = "merge";
  } else {
    templateInsertMode.value = "replace";
  }
};
const selectImageForImg = async (img: HTMLImageElement) => {
  const { path, error } = await selectFile({
    title: t("markdownEditor.imageDialog.title"),
    filters: [
      {
        name: "Image",
        extensions: ["png", "jpg", "jpeg", "gif", "bmp", "webp"],
      },
    ],
  });
  if (error) {
    console.error(error);
    return;
  }
  if (path) {
    const fileData = await readFile(path);
    let binary = "";
    for (let i = 0; i < fileData.length; i++) {
      binary += String.fromCharCode(fileData[i]);
    }
    const base64Data = btoa(binary);
    const fileName = path.replace(/\\/g, "/").split("/").pop() || "image.png";
    const newPath = await invoke<string>("save_image", {
      project_id: props.projectId,
      file_name: fileName,
      file_data: base64Data,
    });
    img.src = newPath;
    if (editor.value) {
      const html = editor.value.getHTML();
      emit("update:modelValue", html);
      emit("requestSave");
    }
  }
};
defineExpose({
  getWordCount: () => editorWordCount.value,
  getHTML: () => editor.value?.getHTML() || "",
  editor,
});
</script>

<template>
  <div
    ref="editorRootRef"
    class="flex flex-col h-full rounded-lg border transition-colors duration-300 overflow-hidden"
    :class="
      editor
        ? isDark
          ? 'bg-gray-800 border-gray-700'
          : 'bg-white border-gray-200'
        : ''
    "
  >
    <div
      v-if="editor"
      data-editor-toolbar
      class="relative flex items-center gap-1 px-3 py-2 border-b flex-wrap"
      :class="isDark ? 'border-gray-700' : 'border-gray-200'"
    >
      <NSpace :size="4">
        <NTooltip trigger="hover">
          <template #trigger>
            <NButton
              size="small"
              :type="isActive('bold') ? 'primary' : 'default'"
              :tertiary="!isActive('bold')"
              @click="toggleBold"
            >
              <template #icon>
                <NIcon>
                  <Bold />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.bold") }}
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton
              size="small"
              :type="isActive('italic') ? 'primary' : 'default'"
              :tertiary="!isActive('italic')"
              @click="toggleItalic"
            >
              <template #icon>
                <NIcon>
                  <Italic />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.italic") }}
        </NTooltip>

        <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton
              size="small"
              :type="isActive('heading', { level: 1 }) ? 'primary' : 'default'"
              :tertiary="!isActive('heading', { level: 1 })"
              @click="setHeading(1)"
            >
              <template #icon>
                <NIcon>
                  <Heading1 />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.heading1") }}
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton
              size="small"
              :type="isActive('heading', { level: 2 }) ? 'primary' : 'default'"
              :tertiary="!isActive('heading', { level: 2 })"
              @click="setHeading(2)"
            >
              <template #icon>
                <NIcon>
                  <Heading2 />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.heading2") }}
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton
              size="small"
              :type="isActive('heading', { level: 3 }) ? 'primary' : 'default'"
              :tertiary="!isActive('heading', { level: 3 })"
              @click="setHeading(3)"
            >
              <template #icon>
                <NIcon>
                  <Heading3 />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.heading3") }}
        </NTooltip>

        <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton
              size="small"
              :type="isActive('bulletList') ? 'primary' : 'default'"
              :tertiary="!isActive('bulletList')"
              @click="toggleBulletList"
            >
              <template #icon>
                <NIcon>
                  <List />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.bulletList") }}
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton
              size="small"
              :type="isActive('orderedList') ? 'primary' : 'default'"
              :tertiary="!isActive('orderedList')"
              @click="toggleOrderedList"
            >
              <template #icon>
                <NIcon>
                  <ListOrdered />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.orderedList") }}
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton
              size="small"
              :type="isActive('blockquote') ? 'primary' : 'default'"
              :tertiary="!isActive('blockquote')"
              @click="toggleBlockquote"
            >
              <template #icon>
                <NIcon>
                  <Quote />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.blockquote") }}
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="toggleHorizontalRule">
              <template #icon>
                <NIcon>
                  <Minus />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.horizontalRule") }}
        </NTooltip>

        <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="$emit('show-history')">
              <template #icon>
                <NIcon>
                  <History />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.history") }}
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="$emit('create-snapshot')">
              <template #icon>
                <NIcon>
                  <Camera />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.snapshot") }}
        </NTooltip>

        <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="showTemplateSelector = true">
              <template #icon>
                <NIcon>
                  <FileText />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.template") }}
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="emit('open-name-generator')">
              <template #icon>
                <NIcon>
                  <Type />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.nameGenerator") }}
        </NTooltip>

        <NDropdown
          trigger="hover"
          :options="beautifyDropdownOptions"
          @select="beautify.handleBeautifyDropdown"
        >
          <NButton size="small" tertiary>
            <template #icon>
              <NIcon>
                <Wand2 />
              </NIcon>
            </template>
          </NButton>
        </NDropdown>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="showTextImportDialog = true">
              <template #icon>
                <NIcon>
                  <Upload />
                </NIcon>
              </template>
            </NButton>
          </template>
          {{ t("markdownEditor.toolbar.importText") }}
        </NTooltip>

        <div
          v-if="showLineHeightControl"
          class="absolute top-full right-0 mt-2 z-50 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 p-4 min-w-[240px]"
        >
          <div class="flex items-center justify-between mb-3">
            <span
              class="text-sm font-medium text-gray-700 dark:text-gray-300"
              >{{ t("markdownEditor.lineHeight.title") }}</span
            >
            <button
              @click="beautify.toggleLineHeightControl"
              class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <div class="flex gap-1 mb-3">
            <button
              v-for="preset in beautify.lineHeightPresets"
              :key="preset.value"
              @click="beautify.setLineHeight(preset.value)"
              class="flex-1 px-2 py-1.5 text-xs rounded transition-colors"
              :class="
                lineHeight === preset.value
                  ? 'bg-blue-500 text-white'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
              "
            >
              {{ preset.label }}
            </button>
          </div>

          <div class="space-y-2">
            <NSlider
              v-model:value="lineHeight"
              :min="16"
              :max="50"
              :step="1"
              :tooltip="false"
            />
            <div
              class="flex justify-between text-xs text-gray-500 dark:text-gray-400"
            >
              <span>{{ t("markdownEditor.lineHeight.compact") }}</span>
              <span class="font-medium text-blue-500">{{ lineHeight }}px</span>
              <span>{{ t("markdownEditor.lineHeight.loose") }}</span>
            </div>
          </div>

          <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
            <p class="text-xs text-gray-500 dark:text-gray-400">
              {{
                t("markdownEditor.lineHeight.current", { height: lineHeight })
              }}
            </p>
          </div>
        </div>
      </NSpace>
    </div>

    <div ref="editorContainerRef" class="flex-1 min-h-0">
      <div class="h-full min-h-0" :class="`paper-${paperStyle || 'none'}`">
        <EditorContent
          :editor="editor"
          class="h-full min-h-0 editor-content-wrapper"
        />
      </div>
    </div>

    <div
      data-editor-statusbar
      class="flex items-center justify-between px-4 py-2 text-sm border-t"
      :class="
        isDark
          ? 'border-gray-700 text-gray-400 bg-gray-800'
          : 'border-gray-200 text-gray-500 bg-gray-50'
      "
    >
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-1">
          <Type class="w-4 h-4" />
          <span>{{ t("markdownEditor.statusBar.chapterWordCount") }}</span>
          <span class="font-medium text-blue-600 dark:text-blue-400">{{
            editorWordCount
          }}</span>
        </div>
        <div
          v-if="volumeWordCount !== undefined"
          class="flex items-center gap-1"
        >
          <BookOpen class="w-4 h-4" />
          <span>{{ t("markdownEditor.statusBar.volumeWordCount") }}</span>
          <span class="font-medium">{{
            volumeWordCount.toLocaleString()
          }}</span>
        </div>
        <div
          v-if="totalWordCount !== undefined"
          class="flex items-center gap-1"
        >
          <BookOpen class="w-4 h-4" />
          <span>{{ t("markdownEditor.statusBar.totalWordCount") }}</span>
          <span class="font-medium">{{ totalWordCount.toLocaleString() }}</span>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <span
          v-if="editorMode === 'typewriter'"
          class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium"
          :class="isDark ? 'bg-amber-900/40 text-amber-300' : 'bg-amber-100 text-amber-700'"
        >
          <Eye class="w-3 h-3" />
          {{ t("markdownEditor.statusBar.typewriterMode") }}
        </span>
        <span
          v-if="editorMode === 'focus'"
          class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium"
          :class="isDark ? 'bg-blue-900/40 text-blue-300' : 'bg-blue-100 text-blue-700'"
        >
          <Eye class="w-3 h-3" />
          {{ t("markdownEditor.statusBar.focusMode") }}
        </span>
      </div>
    </div>

    <TemplateSelector
      v-model:show="showTemplateSelector"
      :project-id="0"
      :insert-mode="templateInsertMode"
      @select="handleTemplateSelect"
      @update:show="handleTemplateSelectorClose"
    />

    <TextImportDialog
      v-model:show="showTextImportDialog"
      :is-dark="isDark"
      @import="handleTextImport"
    />

    <NModal
      v-model:show="showSplitDialog"
      preset="card"
      :title="t('markdownEditor.splitDialog.title')"
      style="width: 600px; max-width: 90vw"
      :segmented="{ content: true, footer: true }"
    >
      <div class="space-y-4">
        <div class="flex items-center gap-4">
          <NText>{{ t("markdownEditor.splitDialog.threshold") }}</NText>
          <NSlider
            v-model:value="splitThreshold"
            :min="100"
            :max="500"
            :step="10"
            style="width: 200px"
          />
          <NText
            >{{ splitThreshold }}
            {{ t("markdownEditor.splitDialog.characters") }}</NText
          >
        </div>

        <NButton @click="beautify.previewSplitParagraphs" secondary>
          <template #icon>
            <NIcon>
              <Eye />
            </NIcon>
          </template>
          {{ t("markdownEditor.splitDialog.refreshPreview") }}
        </NButton>

        <div
          v-if="splitPreview && splitPreview.length > 0"
          class="max-h-80 overflow-auto border rounded-lg p-4 space-y-4"
        >
          <div v-for="(item, index) in splitPreview" :key="index">
            <NText depth="3" class="text-xs mb-1 block">{{
              t("markdownEditor.splitDialog.original", {
                length: item.original.length,
              })
            }}</NText>
            <div class="bg-gray-100 dark:bg-gray-800 rounded p-2 mb-2 text-sm">
              {{ item.original.slice(0, 100)
              }}{{ item.original.length > 100 ? "..." : "" }}
            </div>

            <NText depth="3" class="text-xs mb-1 block">{{
              t("markdownEditor.splitDialog.splitResult", {
                count: item.split.length,
              })
            }}</NText>
            <div class="bg-blue-50 dark:bg-blue-900/30 rounded p-2 space-y-1">
              <div v-for="(split, si) in item.split" :key="si" class="text-sm">
                <span class="text-blue-500 font-medium"
                  >[{{
                    t("markdownEditor.splitDialog.segment", { index: si + 1 })
                  }}]</span
                >
                {{ split.slice(0, 50) }}{{ split.length > 50 ? "..." : "" }}
              </div>
            </div>
          </div>
        </div>

        <div v-else class="flex items-center justify-center py-8">
          <NText depth="3">{{
            t("markdownEditor.splitDialog.noParagraphs")
          }}</NText>
        </div>
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showSplitDialog = false">{{
            t("common.action.cancel")
          }}</NButton>
          <NButton
            type="primary"
            :disabled="!splitPreview || splitPreview.length === 0"
            @click="beautify.applySplitParagraphs"
          >
            {{ t("markdownEditor.splitDialog.confirmSplit") }}
          </NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

<style>
/* ===== Base Editor Layout ===== */
.tiptap {
  min-height: 100%;
  max-height: 100%;
  height: 100%;
  overflow-y: auto;
  scroll-behavior: smooth;
  scrollbar-gutter: stable;
  box-sizing: border-box;
  word-break: break-word;
  overflow-wrap: break-word;
}

.tiptap .ProseMirror {
  min-height: calc(100% - 16px);
  padding: 8px;
  outline: none;
  box-sizing: border-box;
}

.editor-content-wrapper {
  height: 100%;
  display: flex;
  flex-direction: column;
}

[data-v-editor-content] {
  flex: 1;
  min-height: 0;
  height: 100%;
}

/* ===== Scrollbar ===== */
.tiptap::-webkit-scrollbar {
  width: 8px;
}

.tiptap::-webkit-scrollbar-track {
  background: transparent;
}

.tiptap::-webkit-scrollbar-thumb {
  background-color: #d1d5db;
  border-radius: 4px;
  border: 2px solid transparent;
  background-clip: content-box;
}

.tiptap::-webkit-scrollbar-thumb:hover {
  background-color: #9ca3af;
}

/* ===== Placeholder ===== */
.tiptap p.is-editor-empty:first-child::before {
  content: attr(data-placeholder);
  float: left;
  color: #9ca3af;
  pointer-events: none;
  height: 0;
}

/* ===== Typography ===== */
.tiptap p {
  margin-bottom: 1em;
  line-height: 1.5;
}

.tiptap h1 {
  font-size: 2em;
  font-weight: 700;
  margin-bottom: 0.5em;
  line-height: 1.2;
}

.tiptap h2 {
  font-size: 1.5em;
  font-weight: 600;
  margin-bottom: 0.5em;
  line-height: 1.3;
}

.tiptap h3 {
  font-size: 1.25em;
  font-weight: 600;
  margin-bottom: 0.5em;
  line-height: 1.4;
}

.tiptap ul,
.tiptap ol {
  padding-left: 1.5em;
  margin-bottom: 1.25em;
}

.tiptap li {
  margin-bottom: 0.25em;
}

.tiptap blockquote {
  border-left: 4px solid #e5e7eb;
  padding-left: 1em;
  margin-left: 0;
  margin-bottom: 1.25em;
  color: #6b7280;
  font-style: italic;
}

.tiptap hr {
  border: none;
  border-top: 1px solid #e5e7eb;
  margin: 2em 0;
}

.tiptap code {
  background-color: #f3f4f6;
  padding: 0.2em 0.4em;
  border-radius: 0.25em;
  font-size: 0.875em;
}

.tiptap pre {
  background-color: #f9fafb;
  padding: 1em;
  border-radius: 0.5em;
  overflow-x: auto;
  margin-bottom: 1.25em;
}

.tiptap pre code {
  background-color: transparent;
  padding: 0;
}

.tiptap a {
  color: #3b82f6;
  text-decoration: underline;
}

.tiptap img {
  max-width: 100%;
  height: auto;
  border-radius: 0.5em;
}

.tiptap table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 1.25em;
}

.tiptap th,
.tiptap td {
  border: 1px solid #e5e7eb;
  padding: 0.5em;
  text-align: left;
}

.tiptap th {
  background-color: #f3f4f6;
  font-weight: 600;
}

/* ===== Editor Modes ===== */
.typewriter-dim {
  opacity: 0.35;
  transition: opacity 0.3s ease;
  filter: grayscale(0.3);
}

.focus-dim {
  opacity: 0.2;
  transition: opacity 0.3s ease;
  filter: grayscale(0.4);
}

.focus-active {
  background-color: rgba(59, 130, 246, 0.08);
  border-radius: 4px;
  box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.15);
}

.dark .focus-active {
  background-color: rgba(59, 130, 246, 0.12);
  box-shadow: inset 0 0 0 1px rgba(59, 130, 246, 0.25);
}

/* ===== Dynamic Line Height Override ===== */
.editor-content-wrapper .tiptap {
  line-height: v-bind(lineHeightRatio) !important;
}

.editor-content-wrapper .tiptap p {
  margin-bottom: 0.75em !important;
  line-height: v-bind(lineHeightRatio) !important;
}

.editor-content-wrapper .tiptap li {
  line-height: v-bind(lineHeightRatio) !important;
}

/* ===== Paper Styles ===== */
.paper-lined,
.paper-lined-margin,
.paper-grid,
.paper-dots {
  line-height: v-bind(lineHeightRatio) !important;
}

.paper-lined .ProseMirror {
  background-image: repeating-linear-gradient(
    transparent,
    transparent calc(v-bind(lineHeightNum) * 1px - 1px),
    #e5e7eb calc(v-bind(lineHeightNum) * 1px - 1px),
    #e5e7eb calc(v-bind(lineHeightNum) * 1px)
  );
  background-position: 0 calc(v-bind(lineHeightNum) * 1px - 9px);
  background-attachment: local;
}

.paper-lined-margin .ProseMirror {
  background-image: repeating-linear-gradient(
    transparent,
    transparent calc(v-bind(lineHeightNum) * 1px - 9px),
    #e5e7eb calc(v-bind(lineHeightNum) * 1px - 9px),
    #e5e7eb calc(v-bind(lineHeightNum) * 1px)
  );
  background-position: 0 calc(v-bind(lineHeightNum) * 1px - 9px);
  background-attachment: local;
}

.paper-lined-margin .tiptap::before {
  content: "";
  position: absolute;
  left: 40px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: #fca5a5;
  pointer-events: none;
  z-index: 1;
}

.paper-lined-margin {
  position: relative;
  padding-left: 50px !important;
}

.paper-grid .ProseMirror {
  background-image: repeating-linear-gradient(
      transparent,
      transparent calc(v-bind(lineHeightNum) * 1px - 9px),
      #d1d5db calc(v-bind(lineHeightNum) * 1px - 9px),
      #d1d5db calc(v-bind(lineHeightNum) * 1px)
    ),
    repeating-linear-gradient(to right, #f3f4f6 1px, transparent 1px);
  background-position: 0 calc(v-bind(lineHeightNum) * 1px - 9px), 0 0;
  background-size: 100% v-bind(lineHeightPx), 24px 100%;
  background-attachment: local, local;
}

.paper-dots .ProseMirror {
  background-image: radial-gradient(circle, #d1d5db 1px, transparent 1px);
  background-size: 24px v-bind(lineHeightPx);
  background-position: 0 calc(v-bind(lineHeightNum) * 1px - 9px);
  background-attachment: local;
}

/* ===== Paper Mode Element Spacing ===== */
.paper-lined .tiptap p,
.paper-lined-margin .tiptap p,
.paper-grid .tiptap p,
.paper-dots .tiptap p {
  line-height: v-bind(lineHeightRatio) !important;
  margin-bottom: 1.25em !important;
}

.paper-lined .tiptap li,
.paper-lined-margin .tiptap li,
.paper-grid .tiptap li,
.paper-dots .tiptap li {
  line-height: v-bind(lineHeightRatio) !important;
}

.paper-lined .tiptap h1,
.paper-lined-margin .tiptap h1,
.paper-grid .tiptap h1,
.paper-dots .tiptap h1,
.paper-lined .tiptap h2,
.paper-lined-margin .tiptap h2,
.paper-grid .tiptap h2,
.paper-dots .tiptap h2,
.paper-lined .tiptap h3,
.paper-lined-margin .tiptap h3,
.paper-grid .tiptap h3,
.paper-dots .tiptap h3 {
  margin-bottom: 0.5em !important;
}

.paper-lined .tiptap blockquote,
.paper-lined-margin .tiptap blockquote,
.paper-grid .tiptap blockquote,
.paper-dots .tiptap blockquote {
  margin-top: 0.75em !important;
  margin-bottom: 0.75em !important;
}

.paper-lined .tiptap hr,
.paper-lined-margin .tiptap hr,
.paper-grid .tiptap hr,
.paper-dots .tiptap hr {
  margin-top: 0.75em !important;
  margin-bottom: 0.75em !important;
}

/* ===== Dark Theme ===== */
.dark .tiptap {
  color: #e5e7eb;
  scrollbar-color: #4b5563 transparent;
  scrollbar-width: thin;
}

.dark .tiptap::-webkit-scrollbar-thumb {
  background-color: #4b5563;
}

.dark .tiptap::-webkit-scrollbar-thumb:hover {
  background-color: #6b7280;
}

.dark .tiptap p.is-editor-empty:first-child::before {
  color: #6b7280;
}

.dark .tiptap h1,
.dark .tiptap h2,
.dark .tiptap h3 {
  color: #f3f4f6;
}

.dark .tiptap blockquote {
  border-left-color: #4b5563;
  color: #9ca3af;
}

.dark .tiptap code {
  background-color: #374151;
  color: #f3f4f6;
}

.dark .tiptap pre {
  background-color: #1f2937;
}

.dark .tiptap a {
  color: #60a5fa;
}

.dark .paper-lined .ProseMirror {
  background-image: repeating-linear-gradient(
    transparent,
    transparent calc(v-bind(lineHeightNum) * 1px - 9px),
    #374151 calc(v-bind(lineHeightNum) * 1px - 9px),
    #374151 calc(v-bind(lineHeightNum) * 1px)
  );
}

.dark .paper-lined-margin .ProseMirror {
  background-image: repeating-linear-gradient(
    transparent,
    transparent calc(v-bind(lineHeightNum) * 1px - 9px),
    #374151 calc(v-bind(lineHeightNum) * 1px - 9px),
    #374151 calc(v-bind(lineHeightNum) * 1px)
  );
}

.dark .paper-lined-margin .tiptap::before {
  background: #7f1d1d;
}

.dark .paper-grid .ProseMirror {
  background-image: repeating-linear-gradient(
      transparent,
      transparent calc(v-bind(lineHeightNum) * 1px - 9px),
      #374151 calc(v-bind(lineHeightNum) * 1px - 9px),
      #374151 calc(v-bind(lineHeightNum) * 1px)
    ),
    repeating-linear-gradient(to right, #374151 1px, transparent 1px);
}

.dark .paper-dots .ProseMirror {
  background-image: radial-gradient(circle, #4b5563 1px, transparent 1px);
}
</style>