<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick, h } from "vue";
import { useEditor, EditorContent } from "@tiptap/vue-3";
import { DecorationSet } from "prosemirror-view";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { NButton, NIcon, NSpace, NTooltip, NDropdown, DropdownOption, NModal, NCard, NSlider, NSwitch, NText, NSpace as NSpaceVertical } from "naive-ui";

// Editor container ref for dynamic height calculation
const editorRootRef = ref<HTMLElement | null>(null);
let resizeObserver: ResizeObserver | null = null;
let windowResizeHandler: (() => void) | null = null;
import { createSmartSymbolsExtension } from "./SmartSymbolsExtension";
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
  Check,
  AlignLeft,
  Scissors,
  Wand2,
  Settings,
  Eye,
  X,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import type { EditorMode } from "../stores/editor";
import { createMentionExtension } from "./MentionExtension";
import {
  createSensitivePlugin,
  sensitiveKey,
  buildDecorations,
  getDocPlainText,
} from "./SensitiveHighlightPlugin";
import type { SensitiveMatch } from "./SensitiveHighlightPlugin";
import TemplateSelector from "./TemplateSelector.vue";

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

// 模板选择器状态
const showTemplateSelector = ref(false);
const templateInsertMode = ref<'replace' | 'insert'>('replace'); // 默认使用替换模式
const isApplyingTemplate = ref(false);

// 文本美化相关状态
const showSplitDialog = ref(false);  // 段落拆分对话框
const splitThreshold = ref(200);      // 拆分阈值（字符数）
const splitPreview = ref<{ original: string; split: string[] }[]>([]);  // 预览数据
const smartSymbolsEnabled = ref(true);  // 智能符号开关
const paragraphIndentEnabled = ref(false);  // 首行缩进状态

// 信纸背景效果类型
type PaperStyle = 'none' | 'lined' | 'lined-margin' | 'grid' | 'dots';
const paperStyle = ref<PaperStyle>('none');  // 当前信纸效果
const lineHeight = ref(28);  // 行高（与横线间距一致）
const showLineHeightControl = ref(false);  // 显示行高控制面板

// 行高预设选项
const lineHeightPresets = [
  { label: '紧凑', value: 22 },
  { label: '标准', value: 28 },
  { label: '宽松', value: 34 },
  { label: '超大', value: 42 },
];

// 切换信纸效果（仅在开/关状态间切换）
const togglePaperStyle = () => {
  // 默认激活状态使用 lined 样式
  paperStyle.value = paperStyle.value === 'none' ? 'lined' : 'none';
};

// 美化菜单选项
const beautifyDropdownOptions = computed(() => [
  {
    label: `首行缩进 ${paragraphIndentEnabled.value ? '✓' : ''}`,
    key: 'indent'
  },
  { type: 'divider', key: 'd1' },
  {
    label: `信纸效果 ${paperStyle.value !== 'none' ? '✓' : ''}`,
    key: 'paper'
  },
  {
    label: '行间距设置...',
    key: 'lineHeight'
  },
  {
    label: '段落拆分...',
    key: 'split'
  },
]);

// 处理美化菜单选择
const handleBeautifyDropdown = (key: string) => {
  switch (key) {
    case 'indent':
      toggleParagraphIndent();
      break;
    case 'symbols':
      smartSymbolsEnabled.value = !smartSymbolsEnabled.value;
      break;
    case 'paper':
      togglePaperStyle();
      break;
    case 'lineHeight':
      showLineHeightControl.value = !showLineHeightControl.value;
      break;
    case 'split':
      openSplitDialog();
      break;
  }
};

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
    createSmartSymbolsExtension({
      enabled: smartSymbolsEnabled.value
    }),
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

// Dynamic height calculation for editor
let resizeTimeout: ReturnType<typeof setTimeout> | null = null;
const updateEditorHeight = () => {
  if (!editorRootRef.value || !editor.value) return;

  const root = editorRootRef.value;
  const rootRect = root.getBoundingClientRect();

  // Get toolbar height
  const toolbar = root.querySelector('[class*="border-b"]') as HTMLElement;
  const toolbarHeight = toolbar ? toolbar.offsetHeight : 0;

  // Get status bar height
  const statusBar = root.querySelector('[class*="border-t"]') as HTMLElement;
  const statusBarHeight = statusBar ? statusBar.offsetHeight : 0;

  // Calculate available height for editor content
  const availableHeight = rootRect.height - toolbarHeight - statusBarHeight;

  // Update tiptap container height
  const tiptapElement = root.querySelector(".tiptap") as HTMLElement;
  if (tiptapElement) {
    tiptapElement.style.maxHeight = `${Math.max(availableHeight, 100)}px`;
    tiptapElement.style.height = `${Math.max(availableHeight, 100)}px`;
    tiptapElement.style.overflowY = "auto";
  }

  // Update EditorContent container
  const editorContent = root.querySelector('[data-v-editor-content]') as HTMLElement;
  if (editorContent) {
    editorContent.style.height = `${Math.max(availableHeight, 100)}px`;
    editorContent.style.maxHeight = `${Math.max(availableHeight, 100)}px`;
    editorContent.style.overflow = "hidden";
  }
};

// Listen for selection updates
onMounted(() => {
  // Setup resize observer for dynamic height calculation
  if (editorRootRef.value) {
    resizeObserver = new ResizeObserver(() => {
      nextTick(() => updateEditorHeight());
    });
    resizeObserver.observe(editorRootRef.value);

    // Initial height calculation
    nextTick(() => updateEditorHeight());
  }

  // 添加窗口 resize 事件监听，确保编辑器响应窗口大小变化
  windowResizeHandler = () => {
    if (resizeTimeout) clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
      nextTick(() => updateEditorHeight());
    }, 16); // 约 60fps 节流
  };
  window.addEventListener('resize', windowResizeHandler);

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
// 使用文本比较而非 HTML 比较，避免 Tiptap 规范化导致的差异
let lastSyncedContent = '';
watch(
  () => props.modelValue,
  (newValue) => {
    if (!editor.value) return;

    // 获取当前编辑器文本长度，避免 HTML 规范化差异
    const currentTextLength = editor.value.getText().length;
    const newTextLength = newValue ? newValue.length : 0;

    // 只有当内容实际发生变化时才更新
    if (currentTextLength !== newTextLength || (newValue && lastSyncedContent !== newValue)) {
      lastSyncedContent = newValue || '';
      editor.value.commands.setContent(newValue || '', { emitUpdate: false });

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

// Watch for smart symbols toggle
watch(
  () => smartSymbolsEnabled.value,
  (newValue) => {
    if (editor.value) {
      // 销毁旧编辑器并重新创建
      const content = editor.value.getHTML();
      editor.value.destroy();
      editor.value = useEditor({
        content: content,
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
          createSmartSymbolsExtension({
            enabled: newValue
          }),
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
          const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
          const englishWords = text
            .replace(/[\u4e00-\u9fa5]/g, " ")
            .split(/\s+/)
            .filter((w) => w.length > 0).length;
          wordCount.value = chineseChars + englishWords;

          updateWordCountCache();

          // Debounced sensitive word scan
          if (props.projectId) {
            const scanText = getDocPlainText(editor.state.doc);
            if (scanText.length > 0) {
              debouncedScanSensitive(scanText);
            } else {
              clearSensitiveDecorations();
            }
          }
        },
      }).value;
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

// 简单的 Markdown 转 HTML 转换
const markdownToHtml = (markdown: string): string => {
  if (!markdown) return '';

  let html = markdown;

  // 表格处理（需要先处理，因为表格包含 | 字符）
  const tableRegex = /^\|(.+)\|\s*\n\|[-:\s|]+\|\s*\n((?:\|.+\|\s*\n?)+)/gm;
  html = html.replace(tableRegex, (match, headerRow, bodyRows) => {
    const headers = headerRow.split('|').map((h: string) => h.trim()).filter(Boolean);
    const rows = bodyRows.trim().split('\n').map((row: string) =>
      row.split('|').map((cell: string) => cell.trim()).filter(Boolean)
    );

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

  // 分割线
  html = html.replace(/^---$/gim, '<hr>');

  // 标题（按顺序处理，从 h3 到 h1 避免嵌套问题）
  html = html.replace(/^### (.*$)/gim, '<h3>$1</h3>');
  html = html.replace(/^## (.*$)/gim, '<h2>$1</h2>');
  html = html.replace(/^# (.*$)/gim, '<h1>$1</h1>');

  // 粗体
  html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');

  // 斜体
  html = html.replace(/\*(.*?)\*/g, '<em>$1</em>');

  // 图片
  html = html.replace(/!\[(.*?)\]\((.*?)\)/g, '<img alt="$1" src="$2">');

  // 链接
  html = html.replace(/\[(.*?)\]\((.*?)\)/g, '<a href="$2">$1</a>');

  // 引用块处理（支持多行引用）
  const quoteBlocks = html.split(/(?:<table>[\s\S]*?<\/table>)/);
  html = quoteBlocks.map(block => {
    if (block.includes('<table>')) return block;

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
        // 移除 > 符号并添加内容
        const quoteText = trimmed.replace(/^>\s*/g, '');
        if (quoteText) {
          quoteContent.push(quoteText);
        }
      } else {
        if (inQuote) {
          // 输出引用块
          result += `<blockquote><p>${quoteContent.join('</p><p>')}</p></blockquote>`;
          inQuote = false;
          quoteContent = [];
        }
        result += (result ? '\n' : '') + line;
      }
    }

    // 处理末尾的引用块
    if (inQuote && quoteContent.length > 0) {
      result += `<blockquote><p>${quoteContent.join('</p><p>')}</p></blockquote>`;
    }

    return result;
  }).join('');

  // 无序列表处理
  const ulParts = html.split(/(<blockquote>[\s\S]*?<\/blockquote>)/);
  html = ulParts.map(part => {
    if (part.includes('<blockquote>')) return part;

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
      } else {
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

  // 有序列表处理
  const olParts = html.split(/(<ul>[\s\S]*?<\/ul>)/);
  html = olParts.map(part => {
    if (part.includes('<ul>')) return part;

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
      } else {
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

  // 段落处理
  const blockParts = html.split(/(<h[123]>|<\/h[123]>|<ul>[\s\S]*?<\/ul>|<ol>[\s\S]*?<\/ol>|<blockquote>[\s\S]*?<\/blockquote>|<hr>|<table>[\s\S]*?<\/table>)/);
  html = blockParts.map(part => {
    // 如果是已知的块级元素，直接返回
    if (/^<(h[123]>|<\/h[123]>|<ul>|<ol>|<li>|<blockquote>|<\/blockquote>|<hr>|<table>|<\/table>|<thead>|<tbody>|<tr>|<th>|<td>)/.test(part)) {
      return part;
    }
    // 否则作为段落处理
    const trimmed = part.trim();
    if (!trimmed) return '';
    // 保留换行但处理其他情况
    return `<p>${trimmed.replace(/\n/g, '<br>')}</p>`;
  }).join('');

  // 清理残留标签
  html = html.replace(/<p><\/p>/g, '');
  html = html.replace(/<p>\s*<\/p>/g, '');
  html = html.replace(/<p>(<br>)+<\/p>/g, '<br>');

  return html;
};

// ============================================
// 文本美化功能
// ============================================

// 全角空格缩进字符
const INDENT_CHAR = '　　';  // 两个全角空格

// 切换首行缩进
const toggleParagraphIndent = () => {
  if (!editor.value) return;

  const { from, to } = editor.value.state.selection;

  // 检查选区内的段落是否都已缩进
  let allIndented = true;
  editor.value.state.doc.nodesBetween(from, to, (node) => {
    if (node.type.name === 'paragraph') {
      const firstChild = node.firstChild;
      if (!firstChild || firstChild.type.name !== 'text' ||
        !firstChild.text?.startsWith(INDENT_CHAR)) {
        allIndented = false;
      }
    }
  });

  // 对选区内的段落添加/移除缩进
  editor.value.state.doc.nodesBetween(from, to, (node, pos) => {
    if (node.type.name === 'paragraph') {
      const firstChild = node.firstChild;
      const nodeStart = pos;
      const nodeEnd = pos + node.nodeSize;

      if (allIndented) {
        // 移除缩进
        if (firstChild && firstChild.type.name === 'text' &&
          firstChild.text?.startsWith(INDENT_CHAR)) {
          // 删除两个全角空格
          editor.value?.chain()
            .focus()
            .deleteRange({ from: nodeStart + 1, to: nodeStart + 1 + INDENT_CHAR.length })
            .run();
        }
      } else {
        // 添加缩进（在段落开头插入）
        if (!firstChild) {
          // 空段落，插入文本
          editor.value?.chain()
            .focus()
            .insertContentAt(nodeStart, INDENT_CHAR)
            .run();
        } else if (firstChild.type.name === 'text') {
          // 在文本前插入缩进字符
          editor.value?.chain()
            .focus()
            .insertContentAt(nodeStart + 1, INDENT_CHAR)
            .run();
        }
      }
    }
  });

  // 更新缩进状态
  paragraphIndentEnabled.value = !allIndented;

  // 触发保存
  triggerSave();
};

// 检查当前是否启用首行缩进
const checkParagraphIndent = () => {
  if (!editor.value) return false;

  let hasIndented = false;
  let hasNonIndented = false;

  editor.value.state.doc.descendants((node) => {
    if (node.type.name === 'paragraph') {
      const firstChild = node.firstChild;
      if (firstChild && firstChild.type.name === 'text') {
        if (firstChild.text?.startsWith(INDENT_CHAR)) {
          hasIndented = true;
        } else if (node.textContent.length > 0) {
          hasNonIndented = true;
        }
      }
    }
  });

  return hasIndented && !hasNonIndented;
};

// 智能拆分段落
const smartSplitText = (text: string, maxLen: number): string[] => {
  if (text.length <= maxLen) {
    return [text];
  }

  const sentences: { text: string; end: number }[] = [];
  const sentenceEndRegex = /[。！？！？；;]/g;
  let lastEnd = 0;
  let match;

  while ((match = sentenceEndRegex.exec(text)) !== null) {
    sentences.push({
      text: text.slice(lastEnd, match.index + 1),
      end: match.index + 1
    });
    lastEnd = match.index + 1;
  }

  // 处理最后一段
  if (lastEnd < text.length) {
    sentences.push({
      text: text.slice(lastEnd),
      end: text.length
    });
  }

  // 合并句子，确保每段不超过 maxLen
  const result: string[] = [];
  let current = '';

  for (const sentence of sentences) {
    if (current.length + sentence.text.length <= maxLen) {
      current += sentence.text;
    } else {
      if (current) {
        result.push(current);
      }
      // 如果单个句子超过阈值，硬拆分
      if (sentence.text.length > maxLen) {
        let remaining = sentence.text;
        while (remaining.length > maxLen) {
          result.push(remaining.slice(0, maxLen));
          remaining = remaining.slice(maxLen);
        }
        current = remaining;
      } else {
        current = sentence.text;
      }
    }
  }

  if (current) {
    result.push(current);
  }

  return result;
};

// 预览段落拆分
const previewSplitParagraphs = () => {
  if (!editor.value) return;

  const { from, to } = editor.value.state.selection;
  const isSelection = from !== to;

  splitPreview.value = [];

  editor.value.state.doc.nodesBetween(from, to, (node, pos) => {
    if (node.type.name === 'paragraph' && node.textContent.length > splitThreshold.value) {
      const text = node.textContent;
      const split = smartSplitText(text, splitThreshold.value);

      if (split.length > 1) {
        splitPreview.value.push({
          original: text,
          split: split
        });
      }
    }
  });

  if (splitPreview.value.length === 0) {
    // 如果没有找到需要拆分的段落，检查整个文档
    editor.value.state.doc.descendants((node, pos) => {
      if (node.type.name === 'paragraph' && node.textContent.length > splitThreshold.value) {
        const text = node.textContent;
        const split = smartSplitText(text, splitThreshold.value);

        if (split.length > 1 && !splitPreview.value.some(p => p.original === text)) {
          splitPreview.value.push({
            original: text,
            split: split
          });
        }
      }
    });
  }
};

// 执行段落拆分
const applySplitParagraphs = () => {
  if (!editor.value || splitPreview.value.length === 0) return;

  // 获取需要拆分的段落位置
  const paragraphsToSplit: { pos: number; splits: string[] }[] = [];

  editor.value.state.doc.descendants((node, pos) => {
    if (node.type.name === 'paragraph') {
      const text = node.textContent;
      const preview = splitPreview.value.find(p => p.original === text);
      if (preview && preview.split.length > 1) {
        paragraphsToSplit.push({
          pos: pos,
          splits: preview.split
        });
      }
    }
  });

  // 从后往前处理，避免位置偏移问题
  paragraphsToSplit.reverse().forEach(({ pos, splits }) => {
    const node = editor.value!.state.doc.nodeAt(pos);
    if (!node) return;

    const nodeEnd = pos + node.nodeSize;

    // 删除原段落
    editor.value!.chain()
      .focus()
      .deleteRange({ from: pos, to: nodeEnd })
      .run();

    // 在原位置插入拆分后的段落
    const tr = editor.value!.state.tr;
    const frag = splits.map(text =>
      editor.value!.state.schema.nodes.paragraph.create(null,
        editor.value!.state.schema.text(text)
      )
    );

    tr.replaceWith(pos, pos, frag);
    editor.value!.view.dispatch(tr);
  });

  showSplitDialog.value = false;
  triggerSave();
};

// 打开拆分对话框
const openSplitDialog = () => {
  previewSplitParagraphs();
  showSplitDialog.value = true;
};

// 触发保存
const triggerSave = () => {
  if (editor.value) {
    const html = editor.value.getHTML();
    emit('update:modelValue', html);
    emit('requestSave');
  }
};

// 处理模板选择
const handleTemplateSelect = async (payload: string | { content: string; mode: 'replace' | 'insert' }) => {
  if (!editor.value) {
    console.error('编辑器实例未初始化');
    return;
  }

  // 兼容新旧两种调用方式
  let content: string;
  let mode: 'replace' | 'insert';

  if (typeof payload === 'string') {
    // 旧方式：直接传递内容字符串
    content = payload;
    mode = templateInsertMode.value;
  } else {
    // 新方式：传递包含内容和模式的对象
    content = payload.content;
    mode = payload.mode;
    // 同步模式到编辑器
    templateInsertMode.value = mode;
  }

  try {
    isApplyingTemplate.value = true;

    // 将 Markdown 转换为 HTML
    const htmlContent = markdownToHtml(content);

    if (mode === 'replace') {
      // 替换模式：清空内容并插入新模板
      editor.value.commands.setContent(htmlContent || content, { emitUpdate: false });
    } else {
      // 插入模式：在当前光标位置插入模板内容
      if (htmlContent) {
        editor.value.commands.insertContent(htmlContent);
      } else {
        editor.value.commands.insertContent(content);
      }
    }

    // 强制触发内容更新，确保父组件状态同步
    await nextTick();
    const newHtml = editor.value.getHTML();
    emit('update:modelValue', newHtml);

    // 更新字数统计
    const text = editor.value.getText();
    const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
    const englishWords = text
      .replace(/[\u4e00-\u9fa5]/g, ' ')
      .split(/\s+/)
      .filter((w) => w.length > 0).length;
    wordCount.value = chineseChars + englishWords;

    // 触发字数保存
    updateWordCountCache();

    showTemplateSelector.value = false;

    nextTick(() => {
      editor.value?.commands.focus();
    });
  } catch (error) {
    console.error('应用模板失败:', error);
    // 即使出错，也尝试插入原始内容
    if (editor.value) {
      editor.value.commands.insertContent(content);
    }
  } finally {
    isApplyingTemplate.value = false;
  }
};

// 处理模板选择器关闭
const handleTemplateSelectorClose = () => {
  showTemplateSelector.value = false;
};

// 显示模板选择器
const showTemplate = () => {
  showTemplateSelector.value = true;
};

// 切换模板模式
const toggleTemplateMode = () => {
  templateInsertMode.value = templateInsertMode.value === 'replace' ? 'insert' : 'replace';
};

// 图片占位符处理
const handleEditorClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement;
  if (target.tagName === 'IMG') {
    const img = target as HTMLImageElement;
    // 检查是否是占位符图片
    if (!img.src || img.src.includes('在此插入图片') || img.src === window.location.href || img.src.endsWith('()')) {
      // 触发图片选择
      selectImageForImg(img);
    }
  }
};

// 选择图片
const selectImageForImg = async (img: HTMLImageElement) => {
  try {
    // 打开文件对话框
    const selected = await open({
      filters: [{
        name: 'Image',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp']
      }]
    });

    if (selected && !Array.isArray(selected)) {
      // 读取文件
      const fileData = await readFile(selected);

      // 将 Uint8Array 转换为 base64
      let binary = '';
      for (let i = 0; i < fileData.length; i++) {
        binary += String.fromCharCode(fileData[i]);
      }
      const base64Data = btoa(binary);

      // 获取文件名
      const fileName = selected.split('/').pop() || 'image.png';

      // 调用后端命令保存图片
      const newPath = await invoke<string>('save_image', {
        projectId: props.projectId,
        fileName: fileName,
        fileData: base64Data
      });

      // 更新图片 src
      img.src = newPath;

      // 触发更新
      if (editor.value) {
        const html = editor.value.getHTML();
        emit('update:modelValue', html);
        emit('requestSave');
      }
    }
  } catch (error) {
    console.error('选择图片失败:', error);
  }
};

onMounted(() => {
  // 添加编辑器点击事件监听
  if (editorContainerRef.value) {
    editorContainerRef.value.addEventListener('click', handleEditorClick);
  }
});

onUnmounted(() => {
  // 清理所有定时器
  if (wordCountSaveTimer.value) {
    clearTimeout(wordCountSaveTimer.value);
  }
  if (scanTimer.value) {
    clearTimeout(scanTimer.value);
  }
  if (resizeTimeout) {
    clearTimeout(resizeTimeout);
  }
  // 移除窗口 resize 事件监听
  if (windowResizeHandler) {
    window.removeEventListener('resize', windowResizeHandler);
    windowResizeHandler = null;
  }
  // 清理 ResizeObserver
  if (resizeObserver) {
    resizeObserver.disconnect();
    resizeObserver = null;
  }
  editor.value?.destroy();
});
</script>

<template>
  <div ref="editorRootRef" class="flex flex-col h-full rounded-lg border transition-colors duration-300 overflow-hidden"
    :class="editor ? (isDark ? 'bg-gray-800 border-gray-700' : 'bg-white border-gray-200') : ''">
    <!-- Toolbar -->
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

        <!-- 美化菜单 -->
        <NDropdown trigger="hover" :options="beautifyDropdownOptions" @select="handleBeautifyDropdown">
          <NButton size="small" tertiary>
            <template #icon>
              <NIcon>
                <Wand2 />
              </NIcon>
            </template>
          </NButton>
        </NDropdown>

        <!-- 行间距控制面板 -->
        <div v-if="showLineHeightControl"
          class="absolute top-full right-0 mt-2 z-50 bg-white dark:bg-gray-800 rounded-lg shadow-xl border border-gray-200 dark:border-gray-700 p-4 min-w-[240px]">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-medium text-gray-700 dark:text-gray-300">行间距</span>
            <button @click="showLineHeightControl = false"
              class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300">
              <X class="w-4 h-4" />
            </button>
          </div>

          <!-- 预设按钮 -->
          <div class="flex gap-1 mb-3">
            <button v-for="preset in lineHeightPresets" :key="preset.value" @click="lineHeight = preset.value"
              class="flex-1 px-2 py-1.5 text-xs rounded transition-colors"
              :class="lineHeight === preset.value
                ? 'bg-blue-500 text-white'
                : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'">
              {{ preset.label }}
            </button>
          </div>

          <!-- 滑块控制 -->
          <div class="space-y-2">
            <NSlider v-model:value="lineHeight" :min="18" :max="50" :step="1" :tooltip="false" />
            <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400">
              <span>紧凑</span>
              <span class="font-medium text-blue-500">{{ lineHeight }}px</span>
              <span>宽松</span>
            </div>
          </div>

          <!-- 行高提示 -->
          <div class="mt-3 pt-3 border-t border-gray-200 dark:border-gray-700">
            <p class="text-xs text-gray-500 dark:text-gray-400">
              当前行高：{{ lineHeight }}px
            </p>
          </div>
        </div>
      </NSpace>
    </div>

    <!-- Editor Content -->
    <div ref="editorContainerRef" class="flex-1 min-h-0">
      <EditorContent :editor="editor" class="h-full min-h-0 editor-content-wrapper" :class="`paper-${paperStyle}`" />
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

    <!-- 模板选择器 -->
    <TemplateSelector v-model:show="showTemplateSelector" :project-id="projectId || 0" :insert-mode="templateInsertMode"
      @select="handleTemplateSelect" @update:show="handleTemplateSelectorClose" />

    <!-- 段落拆分对话框 -->
    <NModal v-model:show="showSplitDialog" preset="card" title="段落拆分" style="width: 600px; max-width: 90vw"
      :segmented="{ content: true, footer: true }">
      <div class="space-y-4">
        <!-- 阈值设置 -->
        <div class="flex items-center gap-4">
          <NText>拆分阈值：</NText>
          <NSlider v-model:value="splitThreshold" :min="100" :max="500" :step="10" style="width: 200px" />
          <NText>{{ splitThreshold }} 字符</NText>
        </div>

        <!-- 预览按钮 -->
        <NButton @click="previewSplitParagraphs" secondary>
          <template #icon>
            <NIcon>
              <Eye />
            </NIcon>
          </template>
          刷新预览
        </NButton>

        <!-- 预览区域 -->
        <div v-if="splitPreview.length > 0" class="max-h-80 overflow-auto border rounded-lg p-4 space-y-4">
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

        <!-- 无需拆分提示 -->
        <NEmpty v-else description="当前没有超过阈值的段落" />
      </div>

      <template #footer>
        <NSpace justify="end">
          <NButton @click="showSplitDialog = false">取消</NButton>
          <NButton type="primary" :disabled="splitPreview.length === 0" @click="applySplitParagraphs">
            确认拆分
          </NButton>
        </NSpace>
      </template>
    </NModal>
  </div>
</template>

<style>
/* Tiptap Editor Styles */
.tiptap {
  min-height: 100%;
  max-height: 100%;
  height: 100%;
  overflow-y: auto;
  scroll-behavior: smooth;
  box-sizing: border-box;
  transition: height 0.1s ease-out;
}

/* ProseMirror 内部滚动容器 */
.tiptap .ProseMirror {
  min-height: calc(100% - 16px);
  height: 100%;
  padding: 8px;
  outline: none;
  box-sizing: border-box;
}

/* EditorContent 包装器样式，确保响应式高度 */
.editor-content-wrapper {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* 确保编辑器根容器正确响应窗口大小变化 */
[data-v-editor-content] {
  flex: 1;
  min-height: 0;
  height: 100%;
  transition: height 0.1s ease-out;
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

/* =====================
   行间距设置（始终生效）
   ===================== */

/* 基础行高设置 - 强制覆盖所有子元素 */
.editor-content-wrapper .tiptap {
  line-height: v-bind(lineHeight + 'px') !important;
}

.editor-content-wrapper .tiptap p {
  line-height: v-bind(lineHeight + 'px') !important;
  margin-bottom: v-bind(lineHeight + 'px') !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}

.editor-content-wrapper .tiptap li {
  line-height: v-bind(lineHeight + 'px') !important;
}

/* =====================
   信纸背景效果样式
   ===================== */

/* 信纸效果下的行高设置（与横线间距一致）- 强制覆盖所有子元素 */
.paper-lined,
.paper-lined-margin,
.paper-grid,
.paper-dots {
  line-height: v-bind(lineHeight + 'px') !important;
}

.paper-lined .tiptap,
.paper-lined-margin .tiptap,
.paper-grid .tiptap,
.paper-dots .tiptap {
  line-height: v-bind(lineHeight + 'px') !important;
}

/* 横线效果 - 浅灰色横线 */
.paper-lined .ProseMirror {
  background-image: repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeight) * 1px - 1px),
      #e5e7eb calc(v-bind(lineHeight) * 1px - 1px),
      #e5e7eb calc(v-bind(lineHeight) * 1px));
  /* 背景从 8px + 16px = 24px 开始，与第一行文字顶部对齐 */
  background-position: 0 calc(v-bind(lineHeight) * 1px - 9px);
  background-attachment: local;
}

/* 横线 + 红色边距线效果 */
.paper-lined-margin .ProseMirror {
  background-image:
    /* 横线 */
    repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeight) * 1px - 9px),
      #e5e7eb calc(v-bind(lineHeight) * 1px - 9px),
      #e5e7eb calc(v-bind(lineHeight) * 1px));
  /* 背景从 8px + 16px 开始，与第一行文字顶部对齐 */
  background-position: 0 calc(v-bind(lineHeight) * 1px - 9px);
  background-attachment: local;
}

/* 横线 + 边距线的竖线层（单独处理避免覆盖） */
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

/* 网格效果 - 横线和竖线 */
.paper-grid .ProseMirror {
  background-image:
    /* 横线 */
    repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeight) * 1px - 9px),
      #d1d5db calc(v-bind(lineHeight) * 1px - 9px),
      #d1d5db calc(v-bind(lineHeight) * 1px)),
    /* 竖线 */
    repeating-linear-gradient(to right,
      #f3f4f6 1px,
      transparent 1px);
  /* 横线从 8px + 16px 开始，与第一行文字顶部对齐 */
  background-position: 0 calc(v-bind(lineHeight) * 1px - 9px), 0 0;
  background-size: 100% v-bind(lineHeight)px, 24px 100%;
  background-attachment: local, local;
}

/* 点阵效果 */
.paper-dots .ProseMirror {
  background-image: radial-gradient(circle,
      #d1d5db 1px,
      transparent 1px);
  /* 点阵从 8px + 16px 开始，与第一行文字顶部对齐 */
  background-size: 24px v-bind(lineHeight)px;
  background-position: 0 calc(v-bind(lineHeight) * 1px - 9px);
  background-attachment: local;
}

/* 深色模式下的横线效果 */
.dark .paper-lined .ProseMirror {
  background-image: repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeight) * 1px - 9px),
      #374151 calc(v-bind(lineHeight) * 1px - 9px),
      #374151 calc(v-bind(lineHeight) * 1px));
}

/* 深色模式下的横线 + 边距线效果 */
.dark .paper-lined-margin .ProseMirror {
  background-image:
    repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeight) * 1px - 9px),
      #374151 calc(v-bind(lineHeight) * 1px - 9px),
      #374151 calc(v-bind(lineHeight) * 1px));
}

.dark .paper-lined-margin .tiptap::before {
  background: linear-gradient(to bottom, #7f1d1d 0%, #7f1d1d 100%);
}

/* 深色模式下的网格效果 */
.dark .paper-grid .ProseMirror {
  background-image:
    repeating-linear-gradient(transparent,
      transparent calc(v-bind(lineHeight) * 1px - 9px),
      #374151 calc(v-bind(lineHeight) * 1px - 9px),
      #374151 calc(v-bind(lineHeight) * 1px)),
    repeating-linear-gradient(to right,
      #374151 1px,
      transparent 1px);
}

/* 深色模式下的点阵效果 */
.dark .paper-dots .ProseMirror {
  background-image: radial-gradient(circle,
      #4b5563 1px,
      transparent 1px);
}

/* 强制覆盖段落样式，确保与横线对齐 */
.paper-lined .tiptap p,
.paper-lined-margin .tiptap p,
.paper-grid .tiptap p,
.paper-dots .tiptap p {
  line-height: v-bind(lineHeight + 'px') !important;
  margin-bottom: v-bind(lineHeight + 'px') !important;
  padding-top: 0 !important;
  padding-bottom: 0 !important;
}

/* 列表项也需要对齐 */
.paper-lined .tiptap li,
.paper-lined-margin .tiptap li,
.paper-grid .tiptap li,
.paper-dots .tiptap li {
  line-height: v-bind(lineHeight + 'px') !important;
}

/* 标题特殊处理 - 使用信纸行高作为基准 */
.paper-lined .tiptap h1,
.paper-lined-margin .tiptap h1,
.paper-grid .tiptap h1,
.paper-dots .tiptap h1 {
  line-height: v-bind(lineHeight + 'px') !important;
  margin-top: v-bind(lineHeight + 'px') !important;
  margin-bottom: calc(v-bind(lineHeight) * 0.5px) !important;
}

.paper-lined .tiptap h2,
.paper-lined-margin .tiptap h2,
.paper-grid .tiptap h2,
.paper-dots .tiptap h2 {
  line-height: v-bind(lineHeight + 'px') !important;
  margin-top: v-bind(lineHeight + 'px') !important;
  margin-bottom: calc(v-bind(lineHeight) * 0.5px) !important;
}

.paper-lined .tiptap h3,
.paper-lined-margin .tiptap h3,
.paper-grid .tiptap h3,
.paper-dots .tiptap h3 {
  line-height: v-bind(lineHeight + 'px') !important;
  margin-top: calc(v-bind(lineHeight) * 0.5px) !important;
  margin-bottom: calc(v-bind(lineHeight) * 0.5px) !important;
}

/* 引用块样式 */
.paper-lined .tiptap blockquote,
.paper-lined-margin .tiptap blockquote,
.paper-grid .tiptap blockquote,
.paper-dots .tiptap blockquote {
  margin-top: v-bind(lineHeight + 'px') !important;
  margin-bottom: v-bind(lineHeight + 'px') !important;
}

/* 分隔线样式 */
.paper-lined .tiptap hr,
.paper-lined-margin .tiptap hr,
.paper-grid .tiptap hr,
.paper-dots .tiptap hr {
  margin-top: v-bind(lineHeight + 'px') !important;
  margin-bottom: v-bind(lineHeight + 'px') !important;
}
</style>
