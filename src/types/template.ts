export type TemplateCategory = '全部' | '章节' | '图文' | '对话' | '结构化' | '自定义';

export interface WritingTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  content: string;
  is_builtin: boolean;
}

export interface UserTemplate {
  id: number;
  project_id: number;
  name: string;
  description: string;
  category: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface CreateUserTemplateParams {
  project_id: number;
  name: string;
  description: string;
  category: string;
  content: string;
}

export interface UpdateUserTemplateParams {
  name?: string;
  description?: string;
  category?: string;
  content?: string;
}

export const TEMPLATE_CATEGORY_OPTIONS = [
  { label: '全部', value: '全部' },
  { label: '章节', value: '章节' },
  { label: '图文', value: '图文' },
  { label: '对话', value: '对话' },
  { label: '结构化', value: '结构化' },
  { label: '自定义', value: '自定义' },
] as const;

export const TEMPLATE_CATEGORY_TABS = [
  { label: '全部', value: '全部' },
  { label: '章节', value: '章节' },
  { label: '图文', value: '图文' },
  { label: '对话', value: '对话' },
  { label: '结构化', value: '结构化' },
] as const;
