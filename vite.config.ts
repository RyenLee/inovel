import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vueJsx from "@vitejs/plugin-vue-jsx";
import path from "path";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [vue(), vueJsx()],

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
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
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
