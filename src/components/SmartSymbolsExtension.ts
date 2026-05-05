/**
 * 智能符号自动补全插件
 * 功能：输入左标点自动补全右标点，删除时同时删除配对符号
 */

import { Extension } from '@tiptap/core';
import { Plugin, PluginKey } from 'prosemirror-state';
import { TextSelection } from 'prosemirror-state';

// 中文标点符号配对映射
const LEFT_SYMBOLS = ['"', "'", '\u300a', '\u3008', '\u300c', '\u300e', '\u300f', '\u201c', '\u2018'];
const RIGHT_SYMBOLS: Record<number, number> = {
  0x201c: 0x201d,  // "" -> ""
  0x2018: 0x2019,  // '' -> ''
  0x300a: 0x300b,  // 《》 -> 《》
  0x3008: 0x3009,  // 〈〉 -> 〈〉
  0x300c: 0x300d,  // 「」 -> 「」
  0x300e: 0x300f,  // 『』 -> 『』
  0xff02: 0xff02,  // "" -> ""
  0xff07: 0xff07,  // '' -> ''
};

// 获取右标点对应的左标点
const LEFT_FROM_RIGHT: Record<number, number> = {
  0x201d: 0x201c,
  0x2019: 0x2018,
  0x300b: 0x300a,
  0x3009: 0x3008,
  0x300d: 0x300c,
  0x300f: 0x300e,
  0xff02: 0xff02,
  0xff07: 0xff07,
};

// 检测是否是左标点
const isLeftPair = (char: string): boolean => {
  const code = char.charCodeAt(0);
  return code in RIGHT_SYMBOLS;
};

// 检测是否是右标点
const isRightPair = (char: string): boolean => {
  const code = char.charCodeAt(0);
  return code in LEFT_FROM_RIGHT;
};

// 获取对应的右标点
const getRightPair = (left: string): string | undefined => {
  const code = left.charCodeAt(0);
  const rightCode = RIGHT_SYMBOLS[code];
  return rightCode ? String.fromCharCode(rightCode) : undefined;
};

// 获取对应的左标点
const getLeftPair = (right: string): string | undefined => {
  const code = right.charCodeAt(0);
  const leftCode = LEFT_FROM_RIGHT[code];
  return leftCode ? String.fromCharCode(leftCode) : undefined;
};

export interface SmartSymbolsOptions {
  enabled: boolean;
}

export const SmartSymbolsPluginKey = new PluginKey('smartSymbols');

export const createSmartSymbolsExtension = (options: SmartSymbolsOptions = { enabled: true }) => {
  return Extension.create<SmartSymbolsOptions>({
    name: 'smartSymbols',
    
    addOptions() {
      return {
        enabled: options.enabled,
      };
    },
    
    addProseMirrorPlugins() {
      const { enabled } = this.options;
      
      if (!enabled) {
        return [];
      }
      
      return [
        new Plugin<{ isComposing: boolean }>({
          key: SmartSymbolsPluginKey,
          
          props: {
            handleTextInput(view, from, to, text) {
              const pair = getRightPair(text);
              if (pair) {
                const tr = view.state.tr;
                // 插入配对符号，将光标置于中间
                tr.insertText(text + pair, from, to);
                // 将光标移动到中间位置（在两个字符之间）
                // 插入后的位置是 from + 2，光标应该在 from + 1 处
                const cursorPos = from + 1;
                tr.setSelection(TextSelection.create(tr.doc, cursorPos));
                view.dispatch(tr);
                return true;
              }
              return false;
            },
            
            handleKeyDown(view, event) {
              if (event.key === 'Backspace') {
                const { state } = view;
                const { selection } = state;
                const { $head } = selection;
                
                // 获取光标前后的字符
                const before = $head.nodeBefore?.textContent || '';
                const after = $head.nodeAfter?.textContent || '';
                
                const beforeLast = before.slice(-1);
                const afterFirst = after[0];
                
                // 检查是否是配对符号（左符号 + 右符号）
                if (isLeftPair(beforeLast) && afterFirst === getRightPair(beforeLast)) {
                  const tr = state.tr;
                  // 删除配对符号和左符号（光标前一个字符）
                  tr.delete($head.pos - 1, $head.pos + 1);
                  view.dispatch(tr);
                  return true;
                }
                
                // 检查是否是右符号配合左符号
                if (isRightPair(beforeLast)) {
                  const leftPair = getLeftPair(beforeLast);
                  if (leftPair && before.length >= 1 && before[before.length - 2] === leftPair) {
                    const tr = state.tr;
                    // 删除右符号和左符号（光标前两个字符）
                    tr.delete($head.pos - 2, $head.pos + 1);
                    view.dispatch(tr);
                    return true;
                  }
                }
              }
              return false;
            },
          },
        }),
      ];
    },
  });
};

export default createSmartSymbolsExtension;
