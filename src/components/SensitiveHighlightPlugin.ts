import { Plugin, PluginKey } from "prosemirror-state";
import { Decoration, DecorationSet } from "prosemirror-view";

export interface SensitiveMatch {
  word: string;
  start: number;
  end: number;
}

export const sensitiveKey = new PluginKey("sensitive-highlight");

export function createSensitivePlugin() {
  return new Plugin<DecorationSet>({
    key: sensitiveKey,
    state: {
      init() {
        return DecorationSet.empty;
      },
      apply(tr, set) {
        const meta = tr.getMeta(sensitiveKey);
        if (meta !== undefined) {
          return meta;
        }
        return set.map(tr.mapping, tr.doc);
      },
    },
    props: {
      decorations(state) {
        return this.getState(state);
      },
    },
  });
}

/** 从 ProseMirror doc 中提取纯文本（仅文本节点拼接，不含块分隔符） */
export function getDocPlainText(doc: any): string {
  let text = "";
  doc.descendants((node: any) => {
    if (node.isText) {
      text += node.text;
    }
  });
  return text;
}

/** 将纯文本位置映射为 ProseMirror 文档位置，再生成 DecorationSet */
export function buildDecorations(
  doc: any,
  matches: SensitiveMatch[]
): DecorationSet {
  if (matches.length === 0) return DecorationSet.empty;

  // 遍历文档收集所有文本节点及其位置
  const textSegments: { docStart: number; docEnd: number; text: string }[] = [];
  doc.descendants((node: any, pos: number) => {
    if (node.isText) {
      textSegments.push({
        docStart: pos,
        docEnd: pos + node.nodeSize,
        text: node.text,
      });
    }
  });

  // 构建从纯文本位置到文档位置的换算表
  const decorations: Decoration[] = [];
  let plainTextOffset = 0;

  for (const seg of textSegments) {
    const segLen = seg.text.length;
    const segPlainStart = plainTextOffset;
    const segPlainEnd = plainTextOffset + segLen;

    for (const m of matches) {
      // 检查匹配是否与此文本片段重叠
      const overlapStart = Math.max(m.start, segPlainStart);
      const overlapEnd = Math.min(m.end, segPlainEnd);
      if (overlapStart < overlapEnd) {
        const docFrom = seg.docStart + (overlapStart - segPlainStart);
        const docTo = seg.docStart + (overlapEnd - segPlainStart);
        decorations.push(
          Decoration.inline(docFrom, docTo, {
            class: "sensitive-highlight",
            style:
              "text-decoration:underline;text-decoration-color:#ef4444;text-decoration-style:wavy;text-underline-offset:3px;",
          })
        );
      }
    }
    plainTextOffset += segLen;
  }

  return DecorationSet.create(doc, decorations);
}
