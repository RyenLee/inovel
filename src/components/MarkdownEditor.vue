<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { useEditor, EditorContent } from "@tiptap/vue-3";
import { DecorationSet } from "prosemirror-view";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { NButton, NIcon, NSpace, NTooltip } from "naive-ui";
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
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import type { EditorMode } from "../stores/editor";
import { createMentionExtension } from "./MentionExtension";
import {
  createSensitivePlugin,
  sensitiveKey,
  buildDecorations,
  getDocPlainText,
} from "./SensitiveHighlightPlugin";
import type { SensitiveMatch } from "./SensitiveHighlightPlugin";

const props = defineProps<{
  modelValue: string;
  chapterId: number | null;
  projectId?: number | null; // 用于敏感词扫描
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
}>();

const wordCount = ref(0);
const editorContainerRef = ref<HTMLElement | null>(null);
const currentParagraphIndex = ref(-1);
const scanTimer = ref<ReturnType<typeof setTimeout> | null>(null);
const wordCountSaveTimer = ref<ReturnType<typeof setTimeout> | null>(null);

// Update word count cache to backend (with debounce)
const updateWordCountCache = async () => {
  if (wordCountSaveTimer.value) {
    clearTimeout(wordCountSaveTimer.value);
  }
  
  // 防抖：停止输入 2 秒后才更新数据库
  wordCountSaveTimer.value = setTimeout(async () => {
    if (props.chapterId && wordCount.value >= 0) {
      try {
        await invoke("update_chapter_word_count", {
          chapterId: props.chapterId,
          wordCount: wordCount.value,
        });
        // 通知父组件字数已更新
        emit("word-count-updated", wordCount.value);
      } catch (error) {
        console.error("更新字数失败:", error);
      }
    }
  }, 2000);
};

// 防抖扫描敏感词
const debouncedScanSensitive = (text: string) => {
  if (scanTimer.value) {
    clearTimeout(scanTimer.value);
  }
  scanTimer.value = setTimeout(async () => {
    if (!props.projectId || !editor.value) return;
    try {
      const matches: SensitiveMatch[] = await invoke(
        "scan_sensitive_words",
        {
          projectId: props.projectId,
          content: text,
        }
      );
      const decorations = buildDecorations(
        editor.value.state.doc,
        matches
      );
      editor.value.view.dispatch(
        editor.value.state.tr.setMeta(sensitiveKey, decorations)
      );
    } catch (error) {
      console.error("扫描敏感词失败:", error);
    }
  }, 1000); // 1秒防抖
};

// 清空敏感词高亮
const clearSensitiveDecorations = () => {
  if (editor.value) {
    editor.value.view.dispatch(
      editor.value.state.tr.setMeta(sensitiveKey, DecorationSet.empty as any)
    );
  }
};

// Typewriter scrolling: keep current line centered
const scrollToCursor = () => {
  if (!editor.value || !editorContainerRef.value) return;
  
  const { view } = editor.value;
  const { from } = view.state.selection;
  
  // Get the DOM position of the cursor
  const coords = view.coordsAtPos(from);
  const container = editorContainerRef.value;
  const containerRect = container.getBoundingClientRect();
  
  // Calculate the relative position
  const relativeTop = coords.top - containerRect.top + container.scrollTop;
  const containerHeight = container.clientHeight;
  
  // Center the cursor line in the container
  const targetScrollTop = relativeTop - containerHeight / 2;
  container.scrollTo({
    top: Math.max(0, targetScrollTop),
    behavior: "smooth",
  });
};

// Update paragraph styles based on mode
const updateParagraphStyles = () => {
  if (!editor.value) return;
  
  const { view } = editor.value;
  const { $anchor } = view.state.selection;
  
  // Get the paragraph's position in document
  let paragraphIndex = 0;
  const anchorPos = $anchor.pos;
  view.state.doc.descendants((node, pos) => {
    if (node.isBlock && node.isTextblock) {
      if (pos <= anchorPos) {
        paragraphIndex++;
      }
      return false; // Don't descend into blocks
    }
    return true;
  });
  
  currentParagraphIndex.value = paragraphIndex;
  
  // Apply styles based on mode
  if (props.editorMode === "typewriter" || props.editorMode === "focus") {
    const editorContent = view.dom.querySelector(".tiptap");
    if (!editorContent) return;
    
    const paragraphs = editorContent.querySelectorAll("p, h1, h2, h3, li, blockquote");
    let currentIndex = 0;
    
    paragraphs.forEach((p) => {
      p.classList.remove("typewriter-dim", "focus-dim", "focus-active");
      
      if (currentIndex === paragraphIndex - 1) {
        // Current paragraph - highlight it
        if (props.editorMode === "focus") {
          p.classList.add("focus-active");
        }
        // In typewriter mode, current paragraph has no dim class (normal visibility)
      } else {
        // Other paragraphs - dim them
        if (props.editorMode === "typewriter") {
          p.classList.add("typewriter-dim");
        } else {
          p.classList.add("focus-dim");
        }
      }
      currentIndex++;
    });
  }
};

// Create editor
const editor = useEditor({
  content: props.modelValue,
  extensions: [
    StarterKit.configure({
      heading: {
        levels: [1, 2, 3],
      },
    }),
    Placeholder.configure({
      placeholder: "开始编写您的小说...",
    }),
    createMentionExtension(),
    createSensitivePlugin() as any,
  ],
  editorProps: {
    attributes: {
      class:
        "prose prose-sm sm:prose lg:prose-lg dark:prose-invert max-w-none focus:outline-none min-h-full p-4",
    },
  },
  onUpdate: ({ editor }) => {
    const html = editor.getHTML();
    emit("update:modelValue", html);

    // Update word count
    const text = editor.getText();
    // Count Chinese characters + English words
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = text
      .replace(/[\u4e00-\u9fa5]/g, " ")
      .split(/\s+/)
      .filter((w) => w.length > 0).length;
    wordCount.value = chineseChars + englishWords;

    // Update word count cache immediately (no debounce)
    updateWordCountCache();

    // Update paragraph styles in special modes
    if (props.editorMode === "typewriter" || props.editorMode === "focus") {
      nextTick(() => updateParagraphStyles());
    }

    // Debounced sensitive word scan
    // 使用 getDocPlainText 确保位置与 buildDecorations 一致
    if (props.projectId) {
      const scanText = getDocPlainText(editor.state.doc);
      if (scanText.length > 0) {
        debouncedScanSensitive(scanText);
      } else {
        clearSensitiveDecorations();
      }
    }
  },
});

// Listen for selection updates
onMounted(() => {
  if (editor.value) {
    editor.value.on("selectionUpdate", () => {
      if (props.editorMode === "typewriter") {
        nextTick(() => scrollToCursor());
      }
      if (props.editorMode === "typewriter" || props.editorMode === "focus") {
        nextTick(() => updateParagraphStyles());
      }
    });

    // Listen for mention click events from NodeView
    (editor.value as any).on('mention-click', (id: string) => {
      emit('mention-click', id)
    })
  }
});

// Sync external content changes (chapter switching)
watch(
  () => props.modelValue,
  (newValue) => {
    if (editor.value && editor.value.getHTML() !== newValue) {
      // 切换章节时，重新计算字数
      editor.value.commands.setContent(newValue, { emitUpdate: false });
      // 立即更新字数统计
      nextTick(() => {
        if (editor.value) {
          const text = editor.value.getText();
          const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
          const englishWords = text
            .replace(/[\u4e00-\u9fa5]/g, " ")
            .split(/\s+/)
            .filter((w) => w.length > 0).length;
          wordCount.value = chineseChars + englishWords;
        }
      });
    }
  }
);

// Watch for mode changes
watch(
  () => props.editorMode,
  (newMode) => {
    if (newMode === "normal") {
      // Remove all special classes
      if (editor.value) {
        const editorContent = editor.value.view.dom.querySelector(".tiptap");
        if (editorContent) {
          editorContent.querySelectorAll("p, h1, h2, h3, li, blockquote").forEach((p) => {
            p.classList.remove("typewriter-dim", "focus-dim", "focus-active");
          });
        }
      }
    } else {
      nextTick(() => updateParagraphStyles());
      if (newMode === "typewriter") {
        nextTick(() => scrollToCursor());
      }
    }
  }
);

// Expose method for parent to get current word count and editor instance
defineExpose({
  getWordCount: () => wordCount.value,
  editor,
});

// Toolbar actions
const toggleBold = () => editor.value?.chain().focus().toggleBold().run();
const toggleItalic = () => editor.value?.chain().focus().toggleItalic().run();
const setHeading = (level: 1 | 2 | 3) =>
  editor.value?.chain().focus().toggleHeading({ level }).run();
const toggleBulletList = () =>
  editor.value?.chain().focus().toggleBulletList().run();
const toggleOrderedList = () =>
  editor.value?.chain().focus().toggleOrderedList().run();
const toggleBlockquote = () =>
  editor.value?.chain().focus().toggleBlockquote().run();
const toggleHorizontalRule = () =>
  editor.value?.chain().focus().setHorizontalRule().run();

// Check if format is active
const isActive = (type: string, attrs?: Record<string, unknown>) => {
  return editor.value?.isActive(type, attrs) ?? false;
};

onUnmounted(() => {
  // 清理所有定时器
  if (wordCountSaveTimer.value) {
    clearTimeout(wordCountSaveTimer.value);
  }
  if (scanTimer.value) {
    clearTimeout(scanTimer.value);
  }
  editor.value?.destroy();
});
</script>

<template>
  <div class="flex flex-col h-full rounded-lg border transition-colors duration-300"
    :class="editor ? (isDark ? 'bg-gray-800 border-gray-700' : 'bg-white border-gray-200') : ''">
    <!-- Toolbar -->
    <div v-if="editor"
      class="flex items-center gap-1 px-3 py-2 border-b flex-wrap"
      :class="isDark ? 'border-gray-700' : 'border-gray-200'">
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
                <NIcon><Bold /></NIcon>
              </template>
            </NButton>
          </template>
          加粗 (Ctrl+B)
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
                <NIcon><Italic /></NIcon>
              </template>
            </NButton>
          </template>
          斜体 (Ctrl+I)
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
                <NIcon><Heading1 /></NIcon>
              </template>
            </NButton>
          </template>
          一级标题
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
                <NIcon><Heading2 /></NIcon>
              </template>
            </NButton>
          </template>
          二级标题
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
                <NIcon><Heading3 /></NIcon>
              </template>
            </NButton>
          </template>
          三级标题
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
                <NIcon><List /></NIcon>
              </template>
            </NButton>
          </template>
          无序列表
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
                <NIcon><ListOrdered /></NIcon>
              </template>
            </NButton>
          </template>
          有序列表
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
                <NIcon><Quote /></NIcon>
              </template>
            </NButton>
          </template>
          引用
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="toggleHorizontalRule">
              <template #icon>
                <NIcon><Minus /></NIcon>
              </template>
            </NButton>
          </template>
          分割线
        </NTooltip>

        <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="$emit('show-history')">
              <template #icon>
                <NIcon><History /></NIcon>
              </template>
            </NButton>
          </template>
          版本历史
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" tertiary @click="$emit('create-snapshot')">
              <template #icon>
                <NIcon><Camera /></NIcon>
              </template>
            </NButton>
          </template>
          创建快照
        </NTooltip>
      </NSpace>
    </div>

    <!-- Editor Content -->
    <div ref="editorContainerRef" class="flex-1 overflow-auto">
      <EditorContent :editor="editor"
        class="h-full" />
    </div>

    <!-- Status Bar -->
    <div class="flex items-center justify-between px-4 py-2 text-sm border-t"
      :class="isDark ? 'border-gray-700 text-gray-400 bg-gray-800' : 'border-gray-200 text-gray-500 bg-gray-50'">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-1">
          <Type class="w-4 h-4" />
          <span>本章字数：</span>
          <span class="font-medium text-blue-600 dark:text-blue-400">{{ wordCount }}</span>
        </div>
        <div v-if="volumeWordCount !== undefined" class="flex items-center gap-1">
          <BookOpen class="w-4 h-4" />
          <span>本卷字数：</span>
          <span class="font-medium">{{ volumeWordCount.toLocaleString() }}</span>
        </div>
        <div v-if="totalWordCount !== undefined" class="flex items-center gap-1">
          <BookOpen class="w-4 h-4" />
          <span>全书字数：</span>
          <span class="font-medium">{{ totalWordCount.toLocaleString() }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
/* Tiptap Editor Styles */
.tiptap {
  height: 100%;
}

.tiptap p.is-editor-empty:first-child::before {
  content: attr(data-placeholder);
  float: left;
  color: #9ca3af;
  pointer-events: none;
  height: 0;
}

/* Prose styles for dark mode */
.dark .tiptap {
  color: #e5e7eb;
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
}

/* Prose styles */
.tiptap {
  color: #1f2937;
}

.tiptap h1 {
  font-size: 1.875rem;
  font-weight: 700;
  margin-top: 1.5rem;
  margin-bottom: 0.75rem;
  line-height: 1.2;
}

.tiptap h2 {
  font-size: 1.5rem;
  font-weight: 600;
  margin-top: 1.25rem;
  margin-bottom: 0.625rem;
  line-height: 1.3;
}

.tiptap h3 {
  font-size: 1.25rem;
  font-weight: 600;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
  line-height: 1.4;
}

.tiptap p {
  margin-bottom: 0.75rem;
  line-height: 1.75;
}

.tiptap ul,
.tiptap ol {
  padding-left: 1.5rem;
  margin-bottom: 0.75rem;
}

.tiptap ul {
  list-style-type: disc;
}

.tiptap ol {
  list-style-type: decimal;
}

.tiptap li {
  margin-bottom: 0.25rem;
}

.tiptap blockquote {
  border-left: 4px solid #d1d5db;
  padding-left: 1rem;
  margin-left: 0;
  margin-bottom: 0.75rem;
  color: #6b7280;
  font-style: italic;
}

.tiptap hr {
  border: none;
  border-top: 2px solid #e5e7eb;
  margin: 1.5rem 0;
}

.tiptap code {
  background-color: #f3f4f6;
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-size: 0.875em;
  font-family: ui-monospace, monospace;
}

.tiptap pre {
  background-color: #1f2937;
  color: #e5e7eb;
  padding: 1rem;
  border-radius: 0.5rem;
  margin-bottom: 0.75rem;
  overflow-x: auto;
}

.tiptap pre code {
  background-color: transparent;
  padding: 0;
  color: inherit;
}

.tiptap strong {
  font-weight: 600;
}

.tiptap em {
  font-style: italic;
}

/* Typewriter Mode Styles */
.typewriter-dim {
  opacity: 0.4;
  transition: opacity 0.2s ease;
}

.tiptap :global(.typewriter-dim) {
  opacity: 0.4;
}

/* Focus Mode Styles */
.focus-dim {
  opacity: 0.2;
  filter: blur(0.5px);
  transition: opacity 0.2s ease, filter 0.2s ease;
}

.focus-active {
  background-color: rgba(59, 130, 246, 0.08);
  border-radius: 4px;
  transition: background-color 0.2s ease;
}

.dark .tiptap :global(.focus-active) {
  background-color: rgba(59, 130, 246, 0.15);
}

/* Mention Styles */
.mention {
  display: inline-flex;
  align-items: center;
  padding: 0.125rem 0.375rem;
  background-color: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 0.25rem;
  color: #3b82f6;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  vertical-align: baseline;
}

.mention:hover {
  background-color: rgba(59, 130, 246, 0.2);
  border-color: rgba(59, 130, 246, 0.5);
}

/* Mention Suggestion List */
.mention-suggestion-container {
  position: fixed;
  z-index: 9999;
}

.mention-list {
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.15);
  max-height: 300px;
  overflow-y: auto;
  min-width: 200px;
  padding: 4px;
}

.mention-list-empty {
  padding: 12px 16px;
  color: #9ca3af;
  font-size: 14px;
  text-align: center;
}

.mention-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.1s ease;
}

.mention-item:hover,
.mention-item.selected {
  background-color: #f3f4f6;
}

.mention-item.selected {
  background-color: #e0e7ff;
}

.mention-icon {
  font-size: 16px;
}

.mention-label {
  flex: 1;
  font-weight: 500;
  color: #1f2937;
}

.mention-type-tag {
  flex-shrink: 0;
}

/* Dark mode styles */
.dark .mention {
  background-color: rgba(59, 130, 246, 0.2);
  border-color: rgba(59, 130, 246, 0.4);
  color: #60a5fa;
}

.dark .mention:hover {
  background-color: rgba(59, 130, 246, 0.3);
}

.dark .mention-list {
  background: #1f2937;
  border-color: #374151;
}

.dark .mention-item:hover,
.dark .mention-item.selected {
  background-color: #374151;
}

.dark .mention-item.selected {
  background-color: rgba(59, 130, 246, 0.3);
}

.dark .mention-label {
  color: #f3f4f6;
}

.dark .mention-list-empty {
  color: #6b7280;
}

/* Sensitive Word Highlight */
.sensitive-highlight {
  text-decoration: underline;
  text-decoration-color: #ef4444;
  text-decoration-style: wavy;
  text-underline-offset: 3px;
}
</style>
