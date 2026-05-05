// 灵感看板相关类型定义

export interface InspirationItem {
  id: number;
  project_id: number;
  column_name: string;
  content: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface ColumnInfo {
  name: string;
  items: InspirationItem[];
}

export interface BoardData {
  columns: ColumnInfo[];
}

export interface CreateInspirationParams {
  project_id: number;
  column_name: string;
  content: string;
}

export interface UpdateInspirationParams {
  content: string;
}

export interface ReorderItem {
  id: number;
  column_name: string;
  sort_order: number;
}

// 默认列名
export const DEFAULT_COLUMNS = ["灵感", "对白", "场景"] as const;

export type DefaultColumnName = typeof DEFAULT_COLUMNS[number];
