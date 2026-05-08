import { open, save } from "@tauri-apps/plugin-dialog";

export interface FolderSelectOptions {
  title?: string;
  defaultPath?: string;
}

export interface FileSelectOptions {
  title?: string;
  filters?: { name: string; extensions: string[] }[];
  multiple?: boolean;
  defaultPath?: string;
}

export interface SavePathOptions {
  title?: string;
  defaultPath?: string;
  filters?: { name: string; extensions: string[] }[];
}

export interface DialogResult {
  path: string | null;
  error: string | null;
}

export function useFolderDialog() {
  async function selectFolder(options: FolderSelectOptions = {}): Promise<DialogResult> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: options.title || "选择文件夹",
        defaultPath: options.defaultPath,
      });
      if (!selected) {
        return { path: null, error: null };
      }
      return { path: selected as string, error: null };
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      return { path: null, error: `选择文件夹失败: ${message}` };
    }
  }

  async function selectFile(options: FileSelectOptions = {}): Promise<DialogResult> {
    try {
      const selected = await open({
        multiple: options.multiple || false,
        title: options.title || "选择文件",
        filters: options.filters,
        defaultPath: options.defaultPath,
      });
      if (!selected) {
        return { path: null, error: null };
      }
      return { path: selected as string, error: null };
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      return { path: null, error: `选择文件失败: ${message}` };
    }
  }

  async function selectSavePath(options: SavePathOptions = {}): Promise<DialogResult> {
    try {
      const selected = await save({
        title: options.title || "保存文件",
        filters: options.filters,
        defaultPath: options.defaultPath,
      });
      if (!selected) {
        return { path: null, error: null };
      }
      return { path: selected, error: null };
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      return { path: null, error: `选择保存路径失败: ${message}` };
    }
  }

  return {
    selectFolder,
    selectFile,
    selectSavePath,
  };
}
