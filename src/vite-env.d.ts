/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<{}, {}, any>;
  export default component;
}

interface FilePickerOptions {
  suggestedName?: string;
  types?: { description: string; accept: Record<string, string[]> }[];
}

interface FileSystemFileHandle {
  name: string;
}

interface Window {
  showSaveFilePicker?: (options?: FilePickerOptions) => Promise<FileSystemFileHandle>;
}
