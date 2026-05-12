import { ref, computed, watch, reactive } from "vue";
import { useLocale } from "../i18n/composables/useLocale";

export type PaperStyle = "none" | "lined" | "lined-margin" | "grid" | "dots";

const INDENT_CHAR = "　　";

export interface UseTextBeautifyOptions {
  editor: any;
  onContentChange?: (html: string) => void;
}

export function useTextBeautify(options: UseTextBeautifyOptions) {
  const { editor, onContentChange } = options;
  const { t } = useLocale();

  const paperStyle = ref<PaperStyle>("none");
  const lineHeight = ref(20);
  const showLineHeightControl = ref(false);
  const showSplitDialog = ref(false);
  const splitThreshold = ref(200);
  const splitPreview = ref<{ original: string; split: string[] }[]>([]);
  const smartSymbolsEnabled = ref(true);
  const paragraphIndentEnabled = ref(false);
  let indentOperationInProgress = false;

  const lineHeightPresets = computed(() => [
    { label: t('textBeautify.lineHeight.compact'), value: 16 },
    { label: t('textBeautify.lineHeight.standard'), value: 20 },
    { label: t('textBeautify.lineHeight.loose'), value: 24 },
    { label: t('textBeautify.lineHeight.extraLoose'), value: 36 },
  ]);

  const togglePaperStyle = () => {
    paperStyle.value = paperStyle.value === "none" ? "lined" : "none";
  };

  const toggleLineHeightControl = () => {
    showLineHeightControl.value = !showLineHeightControl.value;
  };

  const setLineHeight = (value: number) => {
    lineHeight.value = value;
  };

  const beautifyDropdownOptions = computed(() => [
    {
      label: `${t('textBeautify.menu.paragraphIndent')} ${paragraphIndentEnabled.value ? "✓" : ""}`,
      key: "indent",
    },
    { type: "divider", key: "d1" },
    {
      label: `${t('textBeautify.menu.paperEffect')} ${paperStyle.value !== "none" ? "✓" : ""}`,
      key: "paper",
    },
    {
      label: t('textBeautify.menu.lineHeightSettings'),
      key: "lineHeight",
    },
    {
      label: t('textBeautify.menu.paragraphSplit'),
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

    if (from === to) {
      doc.descendants((node: any, pos: number) => {
        if (node.type.name === "paragraph" && node.textContent.length > 0) {
          targetParagraphs.push({ pos, node });
        }
      });
    } else {
      doc.nodesBetween(from, to, (node: any, pos: number) => {
        if (node.type.name === "paragraph" && node.textContent.length > 0) {
          targetParagraphs.push({ pos, node });
        }
      });
    }

    if (targetParagraphs.length === 0) return;

    const allIndented = targetParagraphs.every(({ node }) => isParagraphIndented(node));

    const tr = state.tr;
    for (let i = targetParagraphs.length - 1; i >= 0; i--) {
      const { pos, node } = targetParagraphs[i];

      if (allIndented) {
        let offset = 0;
        let found = false;
        for (let ci = 0; ci < node.childCount; ci++) {
          const child = node.child(ci);
          if (child.isText && child.text?.startsWith(INDENT_CHAR)) {
            tr.delete(pos + 1 + offset, pos + 1 + offset + INDENT_CHAR.length);
            found = true;
            break;
          }
          offset += child.nodeSize;
        }
        if (!found) {
          const textContent = node.textContent;
          if (textContent.startsWith(INDENT_CHAR)) {
            tr.delete(pos + 1, pos + 1 + INDENT_CHAR.length);
          }
        }
      } else {
        if (!isParagraphIndented(node)) {
          const schema = state.schema;
          const textNode = schema.text(INDENT_CHAR);
          tr.insert(pos + 1, textNode);
        }
      }
    }

    if (tr.docChanged) {
      indentOperationInProgress = true;
      tr.scrollIntoView = false;
      view.dispatch(tr);
      paragraphIndentEnabled.value = !allIndented;
      triggerSave();
      setTimeout(() => {
        indentOperationInProgress = false;
      }, 100);
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
      if (editor.value && !indentOperationInProgress) {
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