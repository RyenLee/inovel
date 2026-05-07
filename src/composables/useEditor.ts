import { ref, watch, onMounted, onUnmounted, nextTick, toRef, type Ref } from "vue";
import { useEditor, EditorContent } from "@tiptap/vue-3";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { invoke } from "@tauri-apps/api/core";
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
                    "prose prose-sm sm:prose lg:prose-lg dark:prose-invert max-w-none focus:outline-none min-h-full p-4",
            },
        },
        onUpdate: ({ editor: editorInstance }: { editor: any }) => {
            const html = editorInstance.getHTML();
            onContentChange?.(html);

            // 字数统计
            const text = editorInstance.getText();
            const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
            const englishLetters = (text.match(/[a-zA-Z]/g) || []).length;
            const digits = (text.match(/[0-9]/g) || []).length;
            wordCount.value = chineseChars + englishLetters + digits;
            onWordCountUpdate?.(wordCount.value);

            // 更新段落样式（特殊模式）
            if (editorModeRef.value === "typewriter" || editorModeRef.value === "focus") {
                nextTick(() => updateParagraphStyles(editorInstance));
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

    // 防抖扫描敏感词
    const debouncedScanSensitive = async (editorInstance: any, text: string) => {
        if (scanTimer.value) {
            clearTimeout(scanTimer.value);
        }
        scanTimer.value = setTimeout(async () => {
            if (!projectIdRef.value || !editorInstance) return;
            try {
                const matches: SensitiveMatch[] = await invoke("scan_sensitive_words", {
                    projectId: projectIdRef.value,
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
                editorInstance.state.tr.setMeta(sensitiveKey, (window as any).DecorationSet.empty)
            );
        }
    };

    // 更新段落样式
    const updateParagraphStyles = (editorInstance: any) => {
        const { view } = editorInstance;
        const { $anchor } = view.state.selection;

        let paragraphIndex = 0;
        const anchorPos = $anchor.pos;
        view.state.doc.descendants((node: any, pos: number) => {
            if (node.isBlock && node.isTextblock) {
                if (pos <= anchorPos) {
                    paragraphIndex++;
                }
                return false;
            }
            return true;
        });

        currentParagraphIndex.value = paragraphIndex;

        if (editorModeRef.value === "typewriter" || editorModeRef.value === "focus") {
            const editorContent = view.dom.querySelector(".tiptap");
            if (!editorContent) return;

            const paragraphs = editorContent.querySelectorAll("p, h1, h2, h3, li, blockquote");
            let currentIndex = 0;

            paragraphs.forEach((p: HTMLElement) => {
                p.classList.remove("typewriter-dim", "focus-dim", "focus-active");

                if (currentIndex === paragraphIndex - 1) {
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
                currentIndex++;
            });
        }
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
    let isUpdatingFromExternal = false;

    watch(
        modelValueRef,
        (newValue) => {
            const content = (newValue as string) || "";
            contentRef.value = content;

            isUpdatingFromExternal = true;

            nextTick(() => {
                if (!editor.value) return;

                const currentText = editor.value.getText().trim();
                const newText = content.replace(/<[^>]*>/g, '').trim();

                if (currentText !== newText || !lastSyncedContent) {
                    lastSyncedContent = content;
                    editor.value.commands.setContent(content, { emitUpdate: false });

                    nextTick(() => {
                        if (editor.value) {
                            const text = editor.value.getText();
                            const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;
                            const englishLetters = (text.match(/[a-zA-Z]/g) || []).length;
                            const digits = (text.match(/[0-9]/g) || []).length;
                            wordCount.value = chineseChars + englishLetters + digits;
                            onWordCountUpdate?.(wordCount.value);
                        }
                    });
                }

                // 重置外部更新标记
                isUpdatingFromExternal = false;
            });
        },
        { immediate: true }
    );

    // 监听模式变化
    watch(
        editorModeRef,
        (newMode) => {
            if (!editor.value) return;

            if (newMode === "normal") {
                const editorContent = editor.value.view.dom.querySelector(".tiptap");
                if (editorContent) {
                    editorContent.querySelectorAll("p, h1, h2, h3, li, blockquote").forEach((p: Element) => {
                        (p as HTMLElement).classList.remove("typewriter-dim", "focus-dim", "focus-active");
                    });
                }
            } else {
                nextTick(() => updateParagraphStyles(editor.value));
            }
        }
    );

    // 打字机滚动
    const scrollToCursor = () => {
        if (!editor.value) return;

        const { view } = editor.value;
        const { from } = view.state.selection;
        const container = view.dom.parentElement;
        if (!container) return;

        const coords = view.coordsAtPos(from);
        const containerRect = container.getBoundingClientRect();
        const relativeTop = coords.top - containerRect.top + container.scrollTop;
        const containerHeight = container.clientHeight;
        const targetScrollTop = relativeTop - containerHeight / 2;

        container.scrollTo({
            top: Math.max(0, targetScrollTop),
            behavior: "smooth",
        });
    };

    // 监听选择更新
    onMounted(() => {
        if (editor.value) {
            editor.value.on("selectionUpdate", () => {
                if (editorModeRef.value === "typewriter") {
                    nextTick(() => scrollToCursor());
                }
                if (editorModeRef.value === "typewriter" || editorModeRef.value === "focus") {
                    nextTick(() => updateParagraphStyles(editor.value!));
                }
            });

            (editor.value as any).on("mention-click", (id: string) => {
                onMentionClick?.(id);
            });
        }
    });

    // 清理
    onUnmounted(() => {
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