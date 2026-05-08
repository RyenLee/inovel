import { ref, computed, watch, reactive } from "vue";

// 信纸背景效果类型
export type PaperStyle = "none" | "lined" | "lined-margin" | "grid" | "dots";

// 行高预设选项
export const lineHeightPresets = [
  { label: "紧凑", value: 16 },
  { label: "标准", value: 20 },
  { label: "宽松", value: 24 },
  { label: "超大", value: 36 },
];

// 全角空格缩进字符
const INDENT_CHAR = "　　"; // 两个全角空格

export interface UseTextBeautifyOptions {
  editor: any;
  onContentChange?: (html: string) => void;
}

export function useTextBeautify(options: UseTextBeautifyOptions) {
  const { editor, onContentChange } = options;

  // 状态
  const paperStyle = ref<PaperStyle>("none");
  // 行高
  const lineHeight = ref(20);
  const showLineHeightControl = ref(false);
  const showSplitDialog = ref(false);
  const splitThreshold = ref(200);
  const splitPreview = ref<{ original: string; split: string[] }[]>([]);
  const smartSymbolsEnabled = ref(true);
  const paragraphIndentEnabled = ref(false);

  // 切换信纸效果（仅在开/关状态间切换）
  const togglePaperStyle = () => {
    paperStyle.value = paperStyle.value === "none" ? "lined" : "none";
  };

  // 切换行间距控制面板
  const toggleLineHeightControl = () => {
    showLineHeightControl.value = !showLineHeightControl.value;
  };

  const setLineHeight = (value: number) => {
    lineHeight.value = value;
  };

  // 美化菜单选项
  const beautifyDropdownOptions = computed(() => [
    {
      label: `首行缩进 ${paragraphIndentEnabled.value ? "✓" : ""}`,
      key: "indent",
    },
    { type: "divider", key: "d1" },
    {
      label: `信纸效果 ${paperStyle.value !== "none" ? "✓" : ""}`,
      key: "paper",
    },
    {
      label: "行间距设置...",
      key: "lineHeight",
    },
    {
      label: "段落拆分...",
      key: "split",
    },
  ]);

  // 处理美化菜单选择
  const handleBeautifyDropdown = (key: string) => {
    switch (key) {
      case "indent":
        toggleParagraphIndent();
        break;
      case "symbols":
        smartSymbolsEnabled.value = !smartSymbolsEnabled.value;
        break;
      case "paper":
        togglePaperStyle();
        break;
      case "lineHeight":
        toggleLineHeightControl();
        break;
      case "split":
        openSplitDialog();
        break;
    }
  };

  const isParagraphIndented = (node: any): boolean => {
    const text = node.textContent;
    return text.startsWith(INDENT_CHAR);
  };

  const toggleParagraphIndent = () => {
    if (!editor.value?.state?.doc) return;

    const { state, view } = editor.value;
    const { from, to } = state.selection;
    const { doc } = state;

    const targetParagraphs: { pos: number; node: any }[] = [];
    doc.nodesBetween(from, to, (node: any, pos: number) => {
      if (node.type.name === "paragraph") {
        targetParagraphs.push({ pos, node });
      }
    });

    if (targetParagraphs.length === 0) return;

    const allIndented = targetParagraphs.every(({ node }) => isParagraphIndented(node));

    const tr = state.tr;
    for (let i = targetParagraphs.length - 1; i >= 0; i--) {
      const { pos, node } = targetParagraphs[i];
      const textContent = node.textContent;

      if (allIndented) {
        if (textContent.startsWith(INDENT_CHAR)) {
          tr.delete(pos + 1, pos + 1 + INDENT_CHAR.length);
        }
      } else {
        if (!textContent.startsWith(INDENT_CHAR)) {
          const schema = state.schema;
          const textNode = schema.text(INDENT_CHAR);
          tr.insert(pos + 1, textNode);
        }
      }
    }

    if (tr.docChanged) {
      tr.scrollIntoView = false;
      view.dispatch(tr);
      paragraphIndentEnabled.value = !allIndented;
      triggerSave();
    }
  };

  const checkParagraphIndent = (): boolean => {
    if (!editor.value?.state?.doc) return false;

    let hasIndented = false;
    let hasNonIndented = false;

    editor.value.state.doc.descendants((node: any) => {
      if (node.type.name === "paragraph") {
        const text = node.textContent;
        if (text.length > 0) {
          if (text.startsWith(INDENT_CHAR)) {
            hasIndented = true;
          } else {
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
        end: match.index + 1,
      });
      lastEnd = match.index + 1;
    }

    if (lastEnd < text.length) {
      sentences.push({
        text: text.slice(lastEnd),
        end: text.length,
      });
    }

    const result: string[] = [];
    let current = "";

    for (const sentence of sentences) {
      if (current.length + sentence.text.length <= maxLen) {
        current += sentence.text;
      } else {
        if (current) {
          result.push(current);
        }
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
    if (!editor.value?.state?.doc) return;

    const { from, to } = editor.value.state.selection;

    splitPreview.value = [];

    editor.value.state.doc.nodesBetween(from, to, (node: any, pos: number) => {
      if (
        node.type.name === "paragraph" &&
        node.textContent.length > splitThreshold.value
      ) {
        const text = node.textContent;
        const split = smartSplitText(text, splitThreshold.value);

        if (split.length > 1) {
          splitPreview.value.push({
            original: text,
            split,
          });
        }
      }
    });

    if (splitPreview.value.length === 0) {
      editor.value.state.doc.descendants((node: any, pos: number) => {
        if (
          node.type.name === "paragraph" &&
          node.textContent.length > splitThreshold.value
        ) {
          const text = node.textContent;
          const split = smartSplitText(text, splitThreshold.value);

          if (
            split.length > 1 &&
            !splitPreview.value.some((p) => p.original === text)
          ) {
            splitPreview.value.push({
              original: text,
              split,
            });
          }
        }
      });
    }
  };

  // 执行段落拆分
  const applySplitParagraphs = () => {
    if (!editor.value?.state?.doc || splitPreview.value.length === 0) return;

    const paragraphsToSplit: { pos: number; splits: string[] }[] = [];

    editor.value.state.doc.descendants((node: any, pos: number) => {
      if (node.type.name === "paragraph") {
        const text = node.textContent;
        const preview = splitPreview.value.find((p) => p.original === text);
        if (preview && preview.split.length > 1) {
          paragraphsToSplit.push({
            pos,
            splits: preview.split,
          });
        }
      }
    });

    paragraphsToSplit.reverse().forEach(({ pos, splits }) => {
      const node = editor.value!.state.doc.nodeAt(pos);
      if (!node) return;

      const nodeEnd = pos + node.nodeSize;

      editor.value!.chain()
        .focus()
        .deleteRange({ from: pos, to: nodeEnd })
        .run();

      const tr = editor.value!.state.tr;
      const frag = splits.map((text) =>
        editor.value!.state.schema.nodes.paragraph.create(
          null,
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
      onContentChange?.(html);
    }
  };

  watch(
    () => editor.value?.state?.doc?.content?.size,
    () => {
      if (editor.value) {
        paragraphIndentEnabled.value = checkParagraphIndent();
      }
    }
  );

  return reactive({
    // 状态
    paperStyle,
    lineHeight,
    showLineHeightControl,
    showSplitDialog,
    splitThreshold,
    splitPreview,
    smartSymbolsEnabled,
    paragraphIndentEnabled,
    lineHeightPresets,
    beautifyDropdownOptions,

    // 方法
    togglePaperStyle,
    toggleLineHeightControl,
    setLineHeight,
    handleBeautifyDropdown,
    toggleParagraphIndent,
    checkParagraphIndent,
    previewSplitParagraphs,
    applySplitParagraphs,
    openSplitDialog,
    triggerSave,
  });
}