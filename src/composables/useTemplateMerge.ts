/**
 * 模板内容整合工具
 * 提供替换、追加、智能合并三种模式
 */

export type MergeMode = 'replace' | 'append' | 'merge';

export interface MergeOptions {
  mode: MergeMode;
  dedupEnabled?: boolean; // 是否启用去重
  keepUserEdits?: boolean; // 是否优先保留用户编辑内容
}

/**
 * 智能合并策略
 * 1. 保留用户已有内容（优先）
 * 2. 补充模板中的新内容
 * 3. 去重（可选）
 */
export function useTemplateMerge() {
  /**
   * 判断内容是否为空（只包含空白字符）
   */
  const isContentEmpty = (content: string): boolean => {
    return !content || content.trim() === '';
  };

  /**
   * 将内容按段落拆分为数组
   */
  const splitToParagraphs = (content: string): string[] => {
    return content
      .split(/\r?\n/)
      .map(p => p.trim())
      .filter(p => p.length > 0);
  };

  /**
   * 将段落数组合并回字符串
   */
  const joinParagraphs = (paragraphs: string[]): string => {
    return paragraphs.join('\n\n');
  };

  /**
   * 去重：移除重复段落（保留第一次出现的）
   */
  const dedupParagraphs = (paragraphs: string[]): string[] => {
    const seen = new Set<string>();
    const result: string[] = [];
    
    for (const p of paragraphs) {
      const normalized = p.trim().toLowerCase();
      if (!seen.has(normalized)) {
        seen.add(normalized);
        result.push(p);
      }
    }
    
    return result;
  };

  /**
   * 计算两段内容的相似度（简单版本）
   * 返回 0-1 之间的值，1 表示完全相同
   */
  const calculateSimilarity = (str1: string, str2: string): number => {
    const s1 = str1.toLowerCase().trim();
    const s2 = str2.toLowerCase().trim();
    
    if (s1 === s2) return 1;
    
    const words1 = s1.split(/\s+/);
    const words2 = s2.split(/\s+/);
    
    if (words1.length === 0 || words2.length === 0) return 0;
    
    const set1 = new Set(words1);
    const set2 = new Set(words2);
    
    const intersection = [...set1].filter(x => set2.has(x)).length;
    const union = set1.size + set2.size - intersection;
    
    return intersection / union;
  };

  /**
   * 检查模板段落是否已经存在于用户内容中
   */
  const paragraphExistsIn = (templatePara: string, userParagraphs: string[]): boolean => {
    const similarityThreshold = 0.8;
    
    for (const userPara of userParagraphs) {
      const similarity = calculateSimilarity(templatePara, userPara);
      if (similarity >= similarityThreshold) {
        return true;
      }
    }
    
    return false;
  };

  /**
   * 替换模式：用模板内容替换全部现有内容
   */
  const replaceContent = (templateContent: string, _existingContent: string): string => {
    return templateContent;
  };

  /**
   * 追加模式：在现有内容后追加模板内容
   */
  const appendContent = (templateContent: string, existingContent: string): string => {
    if (isContentEmpty(existingContent)) return templateContent;
    if (isContentEmpty(templateContent)) return existingContent;
    
    return `${existingContent}\n\n${templateContent}`;
  };

  /**
   * 智能合并模式：优先保留用户编辑，智能补充模板新内容
   */
  const mergeContent = (
    templateContent: string,
    existingContent: string,
    options: Omit<MergeOptions, 'mode'> = {}
  ): string => {
    const { dedupEnabled = true } = options;
    
    // 如果任一内容为空，直接返回非空内容
    if (isContentEmpty(existingContent)) return templateContent;
    if (isContentEmpty(templateContent)) return existingContent;
    
    const existingParagraphs = splitToParagraphs(existingContent);
    const templateParagraphs = splitToParagraphs(templateContent);
    
    // 1. 保留所有用户已有的段落
    const mergedParagraphs = [...existingParagraphs];
    
    // 2. 添加模板中有但用户内容中没有的新段落
    for (const templatePara of templateParagraphs) {
      if (!paragraphExistsIn(templatePara, existingParagraphs)) {
        mergedParagraphs.push(templatePara);
      }
    }
    
    // 3. 可选去重
    const finalParagraphs = dedupEnabled ? dedupParagraphs(mergedParagraphs) : mergedParagraphs;
    
    return joinParagraphs(finalParagraphs);
  };

  /**
   * 主整合函数
   */
  const merge = (
    templateContent: string,
    existingContent: string,
    options: MergeOptions
  ): string => {
    switch (options.mode) {
      case 'replace':
        return replaceContent(templateContent, existingContent);
      case 'append':
        return appendContent(templateContent, existingContent);
      case 'merge':
      default:
        return mergeContent(templateContent, existingContent, options);
    }
  };

  return {
    merge,
    replaceContent,
    appendContent,
    mergeContent,
    splitToParagraphs,
    joinParagraphs,
    dedupParagraphs,
    isContentEmpty,
  };
}
