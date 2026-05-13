import { ref, watch, onUnmounted, nextTick, toRef, type Ref } from "vue";
import { useEditor, EditorContent } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { invoke } from "@tauri-apps/api/core";
import { PluginKey } from "prosemirror-state";
import { DecorationSet } from "prosemirror-view";
import { marked } from 'marked';
import { createSmartSymbolsExtension } from "../components/SmartSymbolsExtension";
import { createMentionExtension } from "../components/MentionExtension";
import {
    createSensitivePlugin,
    sensitiveKey,
    buildDecorations,
    getDocPlainText,
} from "../components/SensitiveHighlightPlugin";
import type { SensitiveMatch } from "../components/SensitiveHighlightPlugin";
import type { EditorMode } from "../stores/editor";

// Configure marked
marked.setOptions({
    breaks: true,
    gfm: true,
});

// Detect if content is Markdown
function isMarkdownContent(content: string): boolean {
    if (!content) return false;
    const markdownPatterns = [
        /^#+\s/m,                    // Headers (# ## ###)
        /^\s*[-*+]\s/m,             // Unordered lists (- * +)
        /^\s*\d+\.\s/m,             // Ordered lists (1. 2. 3.)
        /^\s*>/m,                   // Blockquotes (>)
        /```[\s\S]*?```/g,          // Code blocks
        /`[^`]+`/g,                 // Inline code
        /\[.+?\]\(.+?\)/g,          // Links [text](url)
        /!\[.+?\]\(.+?\)/g,         // Images ![alt](url)
        /\*\*[^*]+\*\*/g,           // Bold **text**
        /\*[^*]+\*/g,               // Italic *text*
        /^---$/m,                   // Horizontal rules ---
        /\|.+\|.+\|/m,              // Tables
    ];
    const matchCount = markdownPatterns.filter(pattern => pattern.test(content)).length;
    return matchCount >= 2;
}

// Convert Markdown to HTML
function markdownToHtml(markdown: string): string {
    if (!markdown) return "";
    try {
        return marked.parse(markdown) as string;
    } catch (error) {
        console.error("Markdown parsing failed:", error);
        return markdown;
    }
}

export interface UseEditorOptions {
    modelValue: string | Ref<string>;
    projectId?: number | null | Ref<number | null | undefined>;
    editorMode?: EditorMode | Ref<EditorMode | undefined>;
    smartSymbolsEnabled?: boolean;
    onContentChange?: (html: string) => void;
    onWordCountUpdate?: (count: number) => void;
    onMentionClick?: (id: string) => void;
}

export function useEditorComposable(options: UseEditorOptions) {
    const {
        smartSymbolsEnabled: initialSmartSymbolsEnabled = true,
        onContentChange,
        onWordCountUpdate,
        onMentionClick,
    } = options;

    const modelValueRef = toRef(options, 'modelValue');
    const projectIdRef = toRef(options, 'projectId');
    const editorModeRef = toRef(options, 'editorMode');
    const contentRef = ref(modelValueRef.value);

    const wordCount = ref(0);
    const currentParagraphIndex = ref(-1);
    const scanTimer = ref<ReturnType<typeof setTimeout> | null>(null);
    const smartSymbolsEnabled = ref(initialSmartSymbolsEnabled);
    const isInternalUpdate = ref(false);

    // 创建编辑器配置
    const createEditorConfig = (symbolsEnabled: boolean) => ({
        content: contentRef.value,
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
                enabled: symbolsEnabled,
            }),
        ],
        editorProps: {
            attributes: {
                class:
                    "prose dark:prose-invert max-w-none focus:outline-none min-h-full p-4",
            },
        },
        onUpdate: ({ editor: editorInstance }: { editor: any }) => {
            const html = editorInstance.getHTML();
            isInternalUpdate.value = true;
            onContentChange?.(html);
            requestAnimationFrame(() => {
                isInternalUpdate.value = false;
            });

            const text = editorInstance.getText();
            const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
            const englishLetters = (text.match(/[a-zA-Z]/g) || []).length;
            const digits = (text.match(/[0-9]/g) || []).length;
            wordCount.value = chineseChars + englishLetters + digits;
            onWordCountUpdate?.(wordCount.value);

            if (editorModeRef.value === "typewriter" || editorModeRef.value === "focus") {
                requestAnimationFrame(() => updateParagraphStyles(editorInstance));
            }

            // 敏感词扫描
            if (projectIdRef.value) {
                const scanText = getDocPlainText(editorInstance.state.doc);
                if (scanText.length > 0) {
                    debouncedScanSensitive(editorInstance, scanText);
                } else {
                    clearSensitiveDecorations(editorInstance);
                }
            }
        },
    });

    // 立即扫描（无防抖，用于章节切换后）
    const rescanSensitive = async (editorInstance: any, text: string) => {
        if (!projectIdRef.value || !editorInstance) return;
        try {
            const matches: SensitiveMatch[] = await invoke("scan_sensitive_words", {
                project_id: projectIdRef.value,
                content: text,
            });
            const decorations = buildDecorations(editorInstance.state.doc, matches);
            editorInstance.view.dispatch(
                editorInstance.state.tr.setMeta(sensitiveKey, decorations)
            );
        } catch (error) {
            console.error("扫描敏感词失败:", error);
        }
    };

    // 防抖扫描敏感词
    const debouncedScanSensitive = async (editorInstance: any, text: string) => {
        if (scanTimer.value) {
            clearTimeout(scanTimer.value);
        }
        scanTimer.value = setTimeout(async () => {
            if (!projectIdRef.value || !editorInstance) return;
            try {
                const matches: SensitiveMatch[] = await invoke("scan_sensitive_words", {
                    project_id: projectIdRef.value,
                    content: text,
                });
                const decorations = buildDecorations(editorInstance.state.doc, matches);
                editorInstance.view.dispatch(
                    editorInstance.state.tr.setMeta(sensitiveKey, decorations)
                );
            } catch (error) {
                console.error("扫描敏感词失败:", error);
            }
        }, 1000);
    };

    // 清空敏感词高亮
    const clearSensitiveDecorations = (editorInstance: any) => {
        if (editorInstance) {
            editorInstance.view.dispatch(
                editorInstance.state.tr.setMeta(sensitiveKey, DecorationSet.empty)
            );
        }
    };

    const updateParagraphStyles = (editorInstance: any) => {
        if (!editorInstance) return;
        const { view } = editorInstance;

        if (editorModeRef.value !== "typewriter" && editorModeRef.value !== "focus") {
            const paragraphs = view.dom.querySelectorAll("p, h1, h2, h3, li, blockquote");
            paragraphs.forEach((p: Element) => {
                (p as HTMLElement).classList.remove("typewriter-dim", "focus-dim", "focus-active");
            });
            return;
        }

        const { from } = view.state.selection;
        const domAtPos = view.domAtPos.bind(view);

        const paragraphs = view.dom.querySelectorAll("p, h1, h2, h3, li, blockquote");

        paragraphs.forEach((p: HTMLElement) => {
            p.classList.remove("typewriter-dim", "focus-dim", "focus-active");
        });

        let activeElement: HTMLElement | null = null;
        try {
            const $pos = view.state.doc.resolve(from);
            for (let d = $pos.depth; d > 0; d--) {
                const node = $pos.node(d);
                if (node.isBlock && node.isTextblock) {
                    const domNode = view.nodeDOM($pos.before(d));
                    if (domNode && domNode instanceof HTMLElement) {
                        activeElement = domNode;
                        break;
                    }
                }
            }
            if (!activeElement) {
                const parent = domAtPos(from).node;
                if (parent instanceof HTMLElement) {
                    activeElement = parent.closest("p, h1, h2, h3, li, blockquote") as HTMLElement;
                }
            }
        } catch (e) {
            try {
                const parent = domAtPos(from).node;
                if (parent instanceof HTMLElement) {
                    activeElement = parent.closest("p, h1, h2, h3, li, blockquote") as HTMLElement;
                }
            } catch (e2) {
                console.warn("Failed to find active paragraph:", e2);
            }
        }

        paragraphs.forEach((p: HTMLElement) => {
            if (p === activeElement) {
                if (editorModeRef.value === "focus") {
                    p.classList.add("focus-active");
                }
            } else {
                if (editorModeRef.value === "typewriter") {
                    p.classList.add("typewriter-dim");
                } else {
                    p.classList.add("focus-dim");
                }
            }
        });

        currentParagraphIndex.value = activeElement
            ? Array.from(paragraphs).indexOf(activeElement) + 1
            : -1;
    };

    // 使用 Tiptap 的 useEditor
    const editor = useEditor(createEditorConfig(smartSymbolsEnabled.value));

    // 监听smartSymbols变化
    watch(
        () => smartSymbolsEnabled.value,
        () => {
            if (editor.value) {
                const content = editor.value.getHTML();
                editor.value.destroy();
                const newEditor = useEditor({
                    ...createEditorConfig(smartSymbolsEnabled.value),
                    content,
                });
                editor.value = newEditor.value;
            }
        }
    );

    // 监听外部内容变化（章节切换）
    let lastSyncedContent = "";

    watch(
        modelValueRef,
        (newValue) => {
            const content = (newValue as string) || "";
            contentRef.value = content;

            if (isInternalUpdate.value) {
                isInternalUpdate.value = false;
                return;
            }

            nextTick(() => {
                if (!editor.value) return;

                // Check if content is Markdown and convert if needed
                let contentToSet = content;
                if (isMarkdownContent(content)) {
                    console.log("Detected Markdown content, converting to HTML...");
                    contentToSet = markdownToHtml(content);
                }

                const currentText = editor.value.getText().trim();
                const newText = contentToSet.replace(/<[^>]*>/g, '').trim();

                if (currentText !== newText || !lastSyncedContent) {
                    lastSyncedContent = contentToSet;
                    editor.value.commands.setContent(contentToSet, { emitUpdate: false });

                    nextTick(() => {
                        if (editor.value) {
                            const text = editor.value.getText();
                            const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
                            const englishLetters = (text.match(/[a-zA-Z]/g) || []).length;
                            const digits = (text.match(/[0-9]/g) || []).length;
                            wordCount.value = chineseChars + englishLetters + digits;
                            onWordCountUpdate?.(wordCount.value);

                            if (projectIdRef.value) {
                                const plainText = getDocPlainText(editor.value.state.doc);
                                if (plainText.length > 0) {
                                    rescanSensitive(editor.value, plainText);
                                } else {
                                    clearSensitiveDecorations(editor.value);
                                }
                            }
                        }
                    });
                }
            });
        },
        { immediate: true }
    );

    // 监听模式变化
    watch(
        editorModeRef,
        (newMode) => {
            if (!editor.value) return;
            requestAnimationFrame(() => updateParagraphStyles(editor.value));
            if (newMode === "typewriter") {
                requestAnimationFrame(() => scrollToCursor());
            }
        }
    );

    // 打字机滚动 - 只在打字机模式下滚动，且只有当光标接近视口边缘时才滚动
    const scrollToCursor = () => {
        if (!editor.value || editorModeRef.value !== "typewriter") return;

        const { view } = editor.value;
        const { from } = view.state.selection;
        const scrollContainer = view.dom;
        if (!scrollContainer) return;

        const coords = view.coordsAtPos(from);
        const containerRect = scrollContainer.getBoundingClientRect();
        const cursorTop = coords.top - containerRect.top;
        const containerHeight = containerRect.height;

        const threshold = containerHeight / 4;
        if (cursorTop < threshold || cursorTop > containerHeight - threshold) {
            const targetScrollTop = scrollContainer.scrollTop + cursorTop - containerHeight / 2;
            scrollContainer.scrollTo({
                top: Math.max(0, targetScrollTop),
                behavior: "smooth",
            });
        }
    };

    let selectionUpdateHandler: (() => void) | null = null;
    let mentionClickHandler: ((id: string) => void) | null = null;

    const registerEditorEvents = (editorInstance: any) => {
        if (!editorInstance) return;

        if (selectionUpdateHandler) {
            editorInstance.off("selectionUpdate", selectionUpdateHandler);
        }
        if (mentionClickHandler) {
            (editorInstance as any).off("mention-click", mentionClickHandler);
        }

        selectionUpdateHandler = () => {
            if (editorModeRef.value === "typewriter") {
                requestAnimationFrame(() => scrollToCursor());
            }
            if (editorModeRef.value === "typewriter" || editorModeRef.value === "focus") {
                requestAnimationFrame(() => updateParagraphStyles(editor.value!));
            }
        };

        mentionClickHandler = (id: string) => {
            onMentionClick?.(id);
        };

        editorInstance.on("selectionUpdate", selectionUpdateHandler);
        (editorInstance as any).on("mention-click", mentionClickHandler);

        if (editorModeRef.value === "typewriter" || editorModeRef.value === "focus") {
            requestAnimationFrame(() => updateParagraphStyles(editorInstance));
        }
    };

    watch(
        () => editor.value,
        (newEditor) => {
            if (newEditor) {
                nextTick(() => registerEditorEvents(newEditor));
            }
        },
        { immediate: true }
    );

    onUnmounted(() => {
        if (editor.value) {
            if (selectionUpdateHandler) {
                editor.value.off("selectionUpdate", selectionUpdateHandler);
            }
            if (mentionClickHandler) {
                (editor.value as any).off("mention-click", mentionClickHandler);
            }
        }
        if (scanTimer.value) {
            clearTimeout(scanTimer.value);
        }
        editor.value?.destroy();
    });

    // 格式化操作
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

    const isActive = (type: string, attrs?: Record<string, unknown>) => {
        return editor.value?.isActive(type, attrs) ?? false;
    };

    return {
        editor,
        wordCount,
        currentParagraphIndex,
        smartSymbolsEnabled,
        EditorContent,
        toggleBold,
        toggleItalic,
        setHeading,
        toggleBulletList,
        toggleOrderedList,
        toggleBlockquote,
        toggleHorizontalRule,
        isActive,
        scrollToCursor,
    };
}