import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vueJsx from "@vitejs/plugin-vue-jsx";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

// naive-ui 按需导入插件
const naiveUiImportPlugin = {
  name: 'naive-ui-import',
  transform(code: string, id: string) {
    if (!id.includes('node_modules') && (id.endsWith('.ts') || id.endsWith('.vue'))) {
      // 将 import { NButton, NIcon } from 'naive-ui' 转换为按需导入
      // naive-ui 的目录结构使用小写组件名（如 button 而不是 NButton）
      // 组件使用命名导出，所以需要使用 import { NButton } from 'naive-ui/es/button'
      code = code.replace(
        /import\s+\{([^}]+)\}\s+from\s+['"]naive-ui['"]/g,
        (match, imports) => {
          const importStatements: string[] = [];
          const fallbackImports: string[] = [];

          const knownComponents = [
            'button', 'icon', 'config-provider', 'input', 'select', 'modal', 'card',
            'progress', 'tooltip', 'tag', 'radio', 'input-number', 'empty', 'popover',
            'tree', 'dropdown', 'spin', 'alert', 'switch', 'slider', 'space',
            'upload', 'date-picker', 'button-group', 'confirm', 'divider',
            'timeline', 'statistic', 'grid', 'layout', 'tabs', 'badge', 'breadcrumb',
            'avatar', 'drawer', 'rate', 'steps', 'table', 'pagination', 'form',
            'transfer', 'cascader', 'checkbox', 'time-picker', 'color-picker',
            'dynamic-input', 'drawer-content'
          ];

          const componentDirOverrides: Record<string, string> = {
            'drawer-content': 'drawer',
          };

          imports.split(',').forEach(item => {
            const name = item.trim();
            if (name.startsWith('N')) {
              let dirName = name.slice(1)
                .replace(/([A-Z])/g, '-$1')
                .toLowerCase()
                .replace(/^-/, '');
              dirName = componentDirOverrides[dirName] || dirName;
              if (knownComponents.includes(dirName)) {
                importStatements.push(`import { ${name} } from 'naive-ui/es/${dirName}'`);
              } else {
                fallbackImports.push(name);
              }
            } else {
              fallbackImports.push(name);
            }
          });

          // 处理需要回退到主包的导入
          if (fallbackImports.length > 0) {
            importStatements.push(`import { ${fallbackImports.join(', ')} } from 'naive-ui'`);
          }

          return importStatements.join('\n');
        }
      );
    }
    return code;
  }
};

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue(), vueJsx(), naiveUiImportPlugin],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },

  build: {
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          // 将 Tauri API 相关模块单独打包
          if (id.includes('@tauri-apps/api')) {
            return 'tauri';
          }
          // 将 naive-ui 组件库单独打包
          if (id.includes('naive-ui')) {
            return 'naive';
          }
          // 将 lucide 图标库单独打包
          if (id.includes('lucide-vue-next')) {
            return 'lucide';
          }
          // 将编辑器相关代码单独打包
          if (id.includes('@tiptap')) {
            return 'editor';
          }
        },
      },
    },
    // 增大 chunk 大小警告阈值
    chunkSizeWarningLimit: 1000,
  },
}));
