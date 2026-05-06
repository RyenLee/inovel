<script setup lang="ts">import { ref, toRef, toRefs, computed } from "vue";
import { NButton, NIcon, NSpace, NTooltip, NDropdown, DropdownOption, NModal, NCard, NSlider, NText, NSpace as NSpaceVertical } from "naive-ui";
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
  X,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import type { EditorMode } from "../stores/editor";
import TemplateSelector from "./TemplateSelector.vue";
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
}>();
const editorRootRef = ref<HTMLElement | null>(null);
const editorContainerRef = ref<HTMLElement | null>(null);
const modelValueRef = toRef(props, 'modelValue');
const projectIdRef = toRef(props, 'projectId');
const editorModeRef = toRef(props, 'editorMode');

const { editor, wordCount: editorWordCount, EditorContent, toggleBold, toggleItalic, setHeading, toggleBulletList, toggleOrderedList, toggleBlockquote, toggleHorizontalRule, isActive, } = useEditorComposable({
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
const chapterIdRef = toRef(props, 'chapterId');

const { wordCount, updateWordCount, cleanup: cleanupWordCount } = useWordCount({
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

const lineHeightPx = computed(() => `${lineHeight.value}px`);
const lineHeightNum = computed(() => lineHeight.value);
useEditorLayout({
  editorRootRef: () => editorRootRef.value,
  editor,
});
const showTemplateSelector = ref(false);
const templateInsertMode = ref<'replace' | 'insert'>('replace');
const isApplyingTemplate = ref(false);
const markdownToHtml = (markdown: string): string => {
  if (!markdown)
    return '';
  let html = markdown;
  const tableRegex = /^\|(.+)\|\s*\n\|[-:\s|]+\|\s*\n((?:\|.+\|\s*\n?)+)/gm;
  html = html.replace(tableRegex, (match, headerRow, bodyRows) => {
    const headers = headerRow.split('|').map((h: string) => h.trim()).filter(Boolean);
    const rows = bodyRows.trim().split('\n').map((row: string) => row.split('|').map((cell: string) => cell.trim()).filter(Boolean));
    let tableHtml = '<table><thead><tr>';
    headers.forEach((h: string) => { tableHtml += `<th>${h}</th>`; });
    tableHtml += '</tr></thead><tbody>';
    rows.forEach((row: string[]) => {
      tableHtml += '<tr>';
      row.forEach((cell: string) => { tableHtml += `<td>${cell}</td>`; });
      tableHtml += '</tr>';
    });
    tableHtml += '</tbody></table>';
    return tableHtml;
  });
  html = html.replace(/^---$/gim, '<hr>');
  html = html.replace(/^### (.*$)/gim, '<h3>$1</h3>');
  html = html.replace(/^## (.*$)/gim, '<h2>$1</h2>');
  html = html.replace(/^# (.*$)/gim, '<h1>$1</h1>');
  html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/\*(.*?)\*/g, '<em>$1</em>');
  html = html.replace(/!\[(.*?)\]\((.*?)\)/g, '<img alt="$1" src="$2">');
  html = html.replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2">$1</a>');
  const quoteBlocks = html.split(/(?:<table>[\s\S]*?<\/table>)/);
  html = quoteBlocks.map(block => {
    if (block.includes('<table>'))
      return block;
    const lines = block.split('\n');
    let result = '';
    let inQuote = false;
    let quoteContent: string[] = [];
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed.startsWith('>')) {
        if (!inQuote) {
          inQuote = true;
          quoteContent = [];
        }
        const quoteText = trimmed.replace(/^>\s*/g, '');
        if (quoteText) {
          quoteContent.push(quoteText);
        }
      }
      else {
        if (inQuote) {
          result += `<blockquote><p>${quoteContent.join('</p><p>')}</p></blockquote>`;
          inQuote = false;
          quoteContent = [];
        }
        result += (result ? '\n' : '') + line;
      }
    }
    if (inQuote && quoteContent.length > 0) {
      result += `<blockquote><p>${quoteContent.join('</p><p>')}</p></blockquote>`;
    }
    return result;
  }).join('');
  const ulParts = html.split(/(<blockquote>[\s\S]*?<\/blockquote>)/);
  html = ulParts.map(part => {
    if (part.includes('<blockquote>'))
      return part;
    const lines = part.split('\n');
    let inList = false;
    let listItems: string[] = [];
    let result = '';
    for (const line of lines) {
      const trimmed = line.trim();
      if (/^-\s/.test(trimmed)) {
        if (!inList) {
          inList = true;
          listItems = [];
        }
        listItems.push(`<li>${trimmed.replace(/^-\s/, '')}</li>`);
      }
      else {
        if (inList) {
          result += `<ul>${listItems.join('')}</ul>`;
          inList = false;
          listItems = [];
        }
        result += (result ? '\n' : '') + line;
      }
    }
    if (inList) {
      result += `<ul>${listItems.join('')}</ul>`;
    }
    return result;
  }).join('');
  const olParts = html.split(/(<ul>[\s\S]*?<\/ul>)/);
  html = olParts.map(part => {
    if (part.includes('<ul>'))
      return part;
    const lines = part.split('\n');
    let inList = false;
    let listItems: string[] = [];
    let result = '';
    for (const line of lines) {
      const trimmed = line.trim();
      if (/^\d+\.\s/.test(trimmed)) {
        if (!inList) {
          inList = true;
          listItems = [];
        }
        listItems.push(`<li>${trimmed.replace(/^\d+\.\s/, '')}</li>`);
      }
      else {
        if (inList) {
          result += `<ol>${listItems.join('')}</ol>`;
          inList = false;
          listItems = [];
        }
        result += (result ? '\n' : '') + line;
      }
    }
    if (inList) {
      result += `<ol>${listItems.join('')}</ol>`;
    }
    return result;
  }).join('');
  const blockParts = html.split(/(<h[123]>|<\/h[123]>|<ul>[\s\S]*?<\/ul>|<ol>[\s\S]*?<\/ol>|<blockquote>[\s\S]*?<\/blockquote>|<hr>|<table>[\s\S]*?<\/table>)/);
  html = blockParts.map(part => {
    if (/^<(h[123]>|<\/h[123]>|<ul>|<ol>|<li>|<blockquote>|<\/blockquote>|<hr>|<table>|<\/table>|<thead>|<tbody>|<tr>|<th>|<td>)/.test(part)) {
      return part;
    }
    const trimmed = part.trim();
    if (!trimmed)
      return '';
    return `<p>${trimmed.replace(/\n/g, '<br>')}</p>`;
  }).join('');
  html = html.replace(/<p><\/p>/g, '');
  html = html.replace(/<p>\s*<\/p>/g, '');
  html = html.replace(/<p>(<br>)+<\/p>/g, '<br>');
  return html;
};
const handleTemplateSelect = async (payload: string | {
  content: string;
  mode: 'replace' | 'insert';
}) => {
  if (!editor.value) {
    console.error('编辑器实例未初始化');
    return;
  }
  let content: string;
  let mode: 'replace' | 'insert';
  if (typeof payload === 'string') {
    content = payload;
    mode = templateInsertMode.value;
  }
  else {
    content = payload.content;
    mode = payload.mode;
    templateInsertMode.value = mode;
  }
  try {
    isApplyingTemplate.value = true;
    const htmlContent = markdownToHtml(content);
    if (mode === 'replace') {
      editor.value.commands.setContent(htmlContent || content, { emitUpdate: false });
    }
    else {
      if (htmlContent) {
        editor.value.commands.insertContent(htmlContent);
      }
      else {
        editor.value.commands.insertContent(content);
      }
    }
    await new Promise(resolve => setTimeout(resolve, 0));
    const newHtml = editor.value.getHTML();
    emit('update:modelValue', newHtml);
    const text = editor.value.getText();
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishLetters = (text.match(/[a-zA-Z]/g) || []).length;
    const digits = (text.match(/[0-9]/g) || []).length;
    wordCount.value = chineseChars + englishLetters + digits;
    showTemplateSelector.value = false;
    await new Promise(resolve => setTimeout(resolve, 0));
    editor.value?.commands.focus();
  }
  catch (error) {
    console.error('应用模板失败:', error);
    if (editor.value) {
      editor.value.commands.insertContent(content);
    }
  }
  finally {
    isApplyingTemplate.value = false;
  }
};
const handleTemplateSelectorClose = () => {
  showTemplateSelector.value = false;
};
const showTemplate = () => {
  showTemplateSelector.value = true;
};
const toggleTemplateMode = () => {
  templateInsertMode.value = templateInsertMode.value === 'replace' ? 'insert' : 'replace';
};
const handleEditorClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement;
  if (target.tagName === 'IMG') {
    const img = target as HTMLImageElement;
    if (!img.src || img.src.includes('在此插入图片') || img.src === window.location.href || img.src.endsWith('()')) {
      selectImageForImg(img);
    }
  }
};
const selectImageForImg = async (img: HTMLImageElement) => {
  try {
    const selected = await open({
      filters: [{
        name: 'Image',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp']
      }]
    });
    if (selected && !Array.isArray(selected)) {
      const fileData = await readFile(selected);
      let binary = '';
      for (let i = 0; i < fileData.length; i++) {
        binary += String.fromCharCode(fileData[i]);
      }
      const base64Data = btoa(binary);
      const fileName = selected.split('/').pop() || 'image.png';
      const newPath = await invoke<string>('save_image', {
        projectId: props.projectId,
        fileName,
        fileData: base64Data
      });
      img.src = newPath;
      if (editor.value) {
        const html = editor.value.getHTML();
        emit('update:modelValue', html);
        emit('requestSave');
      }
    }
  }
  catch (error) {
    console.error('选择图片失败:', error);
  }
};
defineExpose({
  getWordCount: () => editorWordCount.value,
  getHTML: () => editor.value?.getHTML() || '',
  editor,
});
</script>

<template>
  <div ref="editorRootRef" class="flex flex-col h-full rounded-lg border transition-colors duration-300 overflow-hidden"
    :class="editor ? (isDark ? 'bg-gray-800 border-gray-700' : 'bg-white border-gray-200') : ''">
    <div v-if="editor" class="relative flex items-center gap-1 px-3 py-2 border-b flex-wrap"
      :class="isDark ? 'border-gray-700' : 'border-gray-200'">
      <NSpace :size="4">
        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" :type="isActive('bold') ? 'primary' : 'default'" :tertiary="!isActive('bold')"
              @click="toggleBold">
              <template #icon>
                <NIcon>
                  <Bold />
                </NIcon>
              </template>
            </NButton>
          </template>
          加粗 (Ctrl+B)
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" :type="isActive('italic') ? 'primary' : 'default'" :tertiary="!isActive('italic')"
              @click="toggleItalic">
              <template #icon>
                <NIcon>
                  <Italic />
                </NIcon>
              </template>
            </NButton>
          </template>
          斜体 (Ctrl+I)
        </NTooltip>

        <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" :type="isActive('heading', { level: 1 }) ? 'primary' : 'default'"
              :tertiary="!isActive('heading', { level: 1 })" @click="setHeading(1)">
              <template #icon>
                <NIcon>
                  <Heading1 />
                </NIcon>
              </template>
            </NButton>
          </template>
          一级标题
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" :type="isActive('heading', { level: 2 }) ? 'primary' : 'default'"
              :tertiary="!isActive('heading', { level: 2 })" @click="setHeading(2)">
              <template #icon>
                <NIcon>
                  <Heading2 />
                </NIcon>
              </template>
            </NButton>
          </template>
          二级标题
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" :type="isActive('heading', { level: 3 }) ? 'primary' : 'default'"
              :tertiary="!isActive('heading', { level: 3 })" @click="setHeading(3)">
              <template #icon>
                <NIcon>
                  <Heading3 />
                </NIcon>
              </template>
            </NButton>
          </template>
          三级标题
        </NTooltip>

        <div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" :type="isActive('bulletList') ? 'primary' : 'default'"
              :tertiary="!isActive('bulletList')" @click="toggleBulletList">
              <template #icon>
                <NIcon>
                  <List />
                </NIcon>
              </template>
            </NButton>
          </template>
          无序列表
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" :type="isActive('orderedList') ? 'primary' : 'default'"
              :tertiary="!isActive('orderedList')" @click="toggleOrderedList">
              <template #icon>
                <NIcon>
                  <ListOrdered />
                </NIcon>
              </template>
            </NButton>
          </template>
          有序列表
        </NTooltip>

        <NTooltip trigger="hover">
          <template #trigger>
            <NButton size="small" :type="isActive('blockquote') ? 'primary' : 'default'"
              :tertiary="!isActive('blockquote')" @click="toggleBlockquote">
              <template #icon>
                <NIcon>
                  <Quote />
                </NIcon>
              </template>
            </NButton>
          </template>
          引用
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
          分割线
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
          版本历史
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
          创建快照
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
          打开模板库
        </NTooltip>
        <NTooltip trigger="hover">
          <template #trigger>
            <div class="flex items-center">
              <NButton size="tiny" quaternary class="px-1!" @click="toggleTemplateMode">
                <template #icon>
                  <NIcon size="12">
                    <component :is="templateInsertMode === 'replace' ? Replace : Plus" />
                  </NIcon>
                </template>
              </NButton>
            </div>
          </template>
          {{ templateInsertMode === 'replace' ? '替换模式' : '插入模式' }}（点击切换）
        </NTooltip>

        <NDropdown trigger="hover" :options="beautifyDropdownOptions" @select="beautify.handleBeautifyDropdown">
          <NButton size="small" tertiary>
            <template #icon>
              <NIcon>
                <Wand2 />
              </NIcon>
            </template>
          </NButton>
        </NDropdown>

        <div v-if="showLineHeightControl"
          class="absolute top-full right-0 mt-2 z-50 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 p-4 min-w-[240px]">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">行间距</span>
            <button @click="beautify.toggleLineHeightControl"
              class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
              <X class="w-4 h-4" />
            </button>
          </div>

          <div class="flex gap-1 mb-3">
            <button v-for="preset in beautify.lineHeightPresets" :key="preset.value"
              @click="beautify.setLineHeight(preset.value)" class="flex-1 px-2 py-1.5 text-xs rounded transition-colors"
              :class="lineHeight === preset.value
                ? 'bg-blue-500 text-white'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'">
              {{ preset.label }}
            </button>
          </div>

          <div class="space-y-2">
            <NSlider v-model:value="lineHeight" :min="18" :max="50" :step="1" :tooltip="false" />
            <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400">
              <span>紧凑</span>
              <span class="font-medium text-blue-500">{{ lineHeight }}px</span>
              <span>宽松</span>
            </div>
          </div>

          <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
            <p class="text-xs text-gray-500 dark:text-gray-400">
              当前行高：{{ lineHeight }}px
            </p>
          </div>
        </div>
      </NSpace>
    </div>

    <div ref="editorContainerRef" class="flex-1 min-h-0">
      <div class="h-full min-h-0" :class="`paper-${paperStyle || 'none'}`">
        <EditorContent :editor="editor" class="h-full min-h-0 editor-content-wrapper" />
      </div>
    </div>

    <div class="flex items-center justify-between px-4 py-2 text-sm border-t"
      :class="isDark ? 'border-gray-700 text-gray-400 bg-gray-800' : 'border-gray-200 text-gray-500 bg-gray-50'">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-1">
          <Type class="w-4 h-4" />
          <span>本章字数：</span>
          <span class="font-medium text-blue-600 dark:text-blue-400">{{ editorWordCount }}</span>
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

    <TemplateSelector v-model:show="showTemplateSelector" :project-id="projectId || 0" :insert-mode="templateInsertMode"
      @select="handleTemplateSelect" @update:show="handleTemplateSelectorClose" />

    <NModal v-model:show="showSplitDialog" preset="card" title="段落拆分" style="width: 600px; max-width: 90vw"
      :segmented="{ content: true, footer: true }">
      <div class="space-y-4">
        <div class="flex items-center gap-4">
          <NText>拆分阈值：</NText>
          <NSlider v-model:value="splitThreshold" :min="100" :max="500" :step="10" style="width: 200px" />
          <NText>{{ splitThreshold }} 字符</NText>
        </div>

        <NButton @click="beautify.previewSplitParagraphs" secondary>
          <template #icon>
            <NIcon>
              <Eye />
            </NIcon>
          </template>
          刷新预览
        </NButton>

        <div v-if="splitPreview && splitPreview.length > 0"
          class="max-h-80 overflow-auto border rounded-lg p-4 space-y-4">
          <div v-for="(item, index) in splitPreview" :key="index">
            <NText depth="3" class="text-xs mb-1 block">原文（{{ item.original.length }} 字）：</NText>
            <div class="bg-gray-100 dark:bg-gray-800 rounded p-2 mb-2 text-sm">
              {{ item.original.slice(0, 100) }}{{ item.original.length > 100 ? '...' : '' }}
            </div>

            <NText depth="3" class="text-xs mb-1 block">拆分后（{{ item.split.length }} 段）：</NText>
            <div class="bg-blue-50 dark:bg-blue-900/30 rounded p-2 space-y-1">
              <div v-for="(split, si) in item.split" :key="si" class="text-sm">
                <span class="text-blue-500 font-medium">[段{{ si + 1 }}]</span>
                {{ split.slice(0, 50) }}{{ split.length > 50 ? '...' : '' }}
              </div>
            </div>
          </div>
        </div>

        <div v-else class="flex items-center justify-center py-8">
          <NText depth="3">当前没有超过阈值的段落</NText>
        </div>
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showSplitDialog = false">取消</NButton>
          <NButton type="primary" :disabled="!splitPreview || splitPreview.length === 0"
            @click="beautify.applySplitParagraphs">
            确认拆分
          </NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

<style>
.tiptap {
  min-height: 100%;
  max-height: 100%;
  height: 100%;
  overflow-y: auto;
  scroll-behavior: smooth;
  box-sizing: border-box;
}

.tiptap .ProseMirror {
  min-height: calc(100% - 16px);
  height: 100%;
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

.tiptap p.is-editor-empty:first-child::before {
  content: attr(data-placeholder);
  float: left;
  color: #9ca3af;
  pointer-events: none;
  height: 0;
}

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
  color: #f3f4f6;
}

.dark .tiptap pre {
  background-color: #1f2937;
}

.dark .tiptap a {
  color: #60a5fa;
}

.typewriter-dim {
  opacity: 0.3;
  transition: opacity 0.3s ease;
}

.focus-dim {
  opacity: 0.2;
  transition: opacity 0.3s ease;
}

.focus-active {
  background-color: rgba(59, 130, 246, 0.1);
  border-radius: 4px;
}

.tiptap p {
  margin-bottom: 1.25em;
  line-height: 1.75;
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

.editor-content-wrapper .tiptap {
  line-height: v-bind(lineHeightPx) !important;
}

.editor-content-wrapper .tiptap p {
  line-height: v-bind(lineHeightPx) !important;
  margin-bottom: v-bind(lineHeightPx) !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}

.editor-content-wrapper .tiptap li {
  line-height: v-bind(lineHeightPx) !important;
}

.paper-lined,
.paper-lined-margin,
.paper-grid,
.paper-dots {
  line-height: v-bind(lineHeightPx) !important;
}

.paper-lined .tiptap,
.paper-lined-margin .tiptap,
.paper-grid .tiptap,
.paper-dots .tiptap {
  line-height: v-bind(lineHeightPx) !important;
}

.paper-lined .ProseMirror {
  background-image: repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeightNum) * 1px - 1px),
      #e5e7eb calc(v-bind(lineHeightNum) * 1px - 1px),
      #e5e7eb calc(v-bind(lineHeightNum) * 1px));
  background-position: 0 calc(v-bind(lineHeightNum) * 1px - 9px);
  background-attachment: local;
}

.paper-lined-margin .ProseMirror {
  background-image:
    repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeightNum) * 1px - 9px),
      #e5e7eb calc(v-bind(lineHeightNum) * 1px - 9px),
      #e5e7eb calc(v-bind(lineHeightNum) * 1px));
  background-position: 0 calc(v-bind(lineHeightNum) * 1px - 9px);
  background-attachment: local;
}

.paper-lined-margin .tiptap::before {
  content: '';
  position: absolute;
  left: 40px;
  top: 0;
  bottom: 0;
  width: 1px;
  background: linear-gradient(to bottom, #fca5a5 0%, #fca5a5 100%);
  pointer-events: none;
  z-index: 1;
}

.paper-lined-margin {
  position: relative;
  padding-left: 50px !important;
}

.paper-grid .ProseMirror {
  background-image:
    repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeightNum) * 1px - 9px),
      #d1d5db calc(v-bind(lineHeightNum) * 1px - 9px),
      #d1d5db calc(v-bind(lineHeightNum) * 1px)),
    repeating-linear-gradient(to right,
      #f3f4f6 1px,
      transparent 1px);
  background-position: 0 calc(v-bind(lineHeightNum) * 1px - 9px), 0 0;
  background-size: 100% v-bind(lineHeightPx), 24px 100%;
  background-attachment: local, local;
}

.paper-dots .ProseMirror {
  background-image: radial-gradient(circle,
      #d1d5db 1px,
      transparent 1px);
  background-size: 24px v-bind(lineHeightPx);
  background-position: 0 calc(v-bind(lineHeightNum) * 1px - 9px);
  background-attachment: local;
}

.dark .paper-lined .ProseMirror {
  background-image: repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeightNum) * 1px - 9px),
      #374151 calc(v-bind(lineHeightNum) * 1px - 9px),
      #374151 calc(v-bind(lineHeightNum) * 1px));
}

.dark .paper-lined-margin .ProseMirror {
  background-image:
    repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeightNum) * 1px - 9px),
      #374151 calc(v-bind(lineHeightNum) * 1px - 9px),
      #374151 calc(v-bind(lineHeightNum) * 1px));
}

.dark .paper-lined-margin .tiptap::before {
  background: linear-gradient(to bottom, #7f1d1d 0%, #7f1d1d 100%);
}

.dark .paper-grid .ProseMirror {
  background-image:
    repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeightNum) * 1px - 9px),
      #374151 calc(v-bind(lineHeightNum) * 1px - 9px),
      #374151 calc(v-bind(lineHeightNum) * 1px)),
    repeating-linear-gradient(to right,
      #374151 1px,
      transparent 1px);
}

.dark .paper-dots .ProseMirror {
  background-image: radial-gradient(circle,
      #4b5563 1px,
      transparent 1px);
}

.paper-lined .tiptap p,
.paper-lined-margin .tiptap p,
.paper-grid .tiptap p,
.paper-dots .tiptap p {
  line-height: v-bind(lineHeightPx) !important;
  margin-bottom: v-bind(lineHeightPx) !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}

.paper-lined .tiptap li,
.paper-lined-margin .tiptap li,
.paper-grid .tiptap li,
.paper-dots .tiptap li {
  line-height: v-bind(lineHeightPx) !important;
}

.paper-lined .tiptap h1,
.paper-lined-margin .tiptap h1,
.paper-grid .tiptap h1,
.paper-dots .tiptap h1 {
  margin-bottom: calc(v-bind(lineHeightNum) * 0.5px) !important;
}

.paper-lined .tiptap h2,
.paper-lined-margin .tiptap h2,
.paper-grid .tiptap h2,
.paper-dots .tiptap h2 {
  margin-bottom: calc(v-bind(lineHeightNum) * 0.5px) !important;
}

.paper-lined .tiptap h3,
.paper-lined-margin .tiptap h3,
.paper-grid .tiptap h3,
.paper-dots .tiptap h3 {
  margin-bottom: calc(v-bind(lineHeightNum) * 0.5px) !important;
}

.paper-lined .tiptap blockquote,
.paper-lined-margin .tiptap blockquote,
.paper-grid .tiptap blockquote,
.paper-dots .tiptap blockquote {
  margin-top: v-bind(lineHeightPx) !important;
  margin-bottom: v-bind(lineHeightPx) !important;
}

.paper-lined .tiptap hr,
.paper-lined-margin .tiptap hr,
.paper-grid .tiptap hr,
.paper-dots .tiptap hr {
  margin-top: v-bind(lineHeightPx) !important;
  margin-bottom: v-bind(lineHeightPx) !important;
}
</style>