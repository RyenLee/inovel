import { createApp } from "vue";
import { createPinia } from "pinia";
import "./style.css";
import App from "./App.vue";
import router from "./router";

// =====================
// 禁用浏览器前进/后退功能
// 
// 拦截层次:
// 1. Rust 端 (tauri-plugin-prevent-default): 禁用所有键盘快捷键
//    - Alt+方向键 (浏览器导航)
//    - F5 (刷新)
//    - Ctrl+Shift+I (开发者工具)
//    - 其他浏览器键盘快捷键
//
// 2. JavaScript 层: 禁用鼠标侧键
//    - button 3 (鼠标前进键/XButton1)
//    - button 4 (鼠标后退键/XButton2)
// =====================

// 禁用鼠标前进/后退按钮（JavaScript 层辅助拦截）
// Windows: button 3 = XButton1(前进), button 4 = XButton2(后退)
document.addEventListener('auxclick', (e: MouseEvent) => {
  if (e.button === 3 || e.button === 4) {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }
}, { capture: true, passive: false });

document.addEventListener('mousedown', (e: MouseEvent) => {
  if (e.button === 3 || e.button === 4) {
    e.preventDefault();
    e.stopPropagation();
    return false;
  }
}, { capture: true, passive: false });

// 禁用浏览器默认的前进/后退键盘快捷键
// 注: Alt+方向键等已由 Rust 插件处理，这里作为备用
document.addEventListener('keydown', (e: KeyboardEvent) => {
  if (e.altKey && (e.key === 'ArrowRight' || e.key === 'ArrowLeft' || e.key === 'Backspace')) {
    e.preventDefault();
    return false;
  }
  // 禁用 F5 刷新
  if (e.key === 'F5') {
    e.preventDefault();
    return false;
  }
}, { capture: true, passive: false });

// Global error handler - 捕获所有未处理的错误
window.addEventListener("error", (event) => {
  console.error("[Global Error]", event.error);
  showErrorToUser(`发生错误: ${event.error?.message || event.message}`);
});

window.addEventListener("unhandledrejection", (event) => {
  console.error("[Unhandled Rejection]", event.reason);
  showErrorToUser(`异步错误: ${event.reason}`);
});

// 显示错误信息到页面的降级方案
function showErrorToUser(message: string) {
  const app = document.getElementById("app");
  if (app && app.innerHTML === "") {
    app.innerHTML = `
      <div style="
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        height: 100vh;
        font-family: system-ui, sans-serif;
        color: #333;
        background: #f5f5f5;
        padding: 20px;
        text-align: center;
      ">
        <h2 style="color: #e53e3e; margin-bottom: 16px;">加载失败</h2>
        <p style="color: #666; margin-bottom: 20px;">${message}</p>
        <button onclick="location.reload()" style="
          padding: 10px 20px;
          background: #18a058;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        ">重新加载</button>
        <details style="margin-top: 30px; text-align: left; max-width: 600px;">
          <summary style="cursor: pointer; color: #666;">查看技术信息</summary>
          <pre style="background: #f0f0f0; padding: 10px; overflow: auto; font-size: 12px;"></pre>
        </details>
      </div>
    `;
  }
}

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);

app.config.errorHandler = (err, instance, info) => {
  console.error("[Vue Error]", err);
  console.error("[Component]", instance);
  console.error("[Info]", info);

  // 显示友好错误信息
  const errorMessage = err instanceof Error ? err.message : String(err);
  showErrorToUser(`Vue 组件错误: ${errorMessage}`);
};

// 组件挂载前添加 loading 状态
const appElement = document.getElementById("app");
if (appElement) {
  appElement.innerHTML = `
    <div style="
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100vh;
      font-family: system-ui, sans-serif;
      color: #666;
    ">
      <div style="
        width: 40px;
        height: 40px;
        border: 3px solid #e0e0e0;
        border-top-color: #18a058;
        border-radius: 50%;
        animation: spin 1s linear infinite;
      "></div>
      <p style="margin-top: 16px;">加载中...</p>
      <style>@keyframes spin { to { transform: rotate(360deg); } }</style>
    </div>
  `;
}

app.mount("#app");
