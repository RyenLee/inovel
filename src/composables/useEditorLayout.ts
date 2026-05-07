import { ref, onMounted, onUnmounted, nextTick } from "vue";

export interface UseEditorLayoutOptions {
  editorRootRef: () => HTMLElement | null;
  editor: any;
}

export function useEditorLayout(options: UseEditorLayoutOptions) {
  const { editorRootRef, editor } = options;

  let resizeObserver: ResizeObserver | null = null;
  let windowResizeHandler: (() => void) | null = null;
  let resizeTimeout: ReturnType<typeof setTimeout> | null = null;
  let isUpdating = false;

  const updateEditorHeight = () => {
    if (isUpdating) return;
    const root = editorRootRef();
    if (!root || !editor.value) return;

    isUpdating = true;

    requestAnimationFrame(() => {
      const rootRect = root.getBoundingClientRect();

      const toolbar = root.querySelector(
        '[data-editor-toolbar]'
      ) as HTMLElement;
      const toolbarHeight = toolbar ? toolbar.offsetHeight : 0;

      const statusBar = root.querySelector(
        '[data-editor-statusbar]'
      ) as HTMLElement;
      const statusBarHeight = statusBar ? statusBar.offsetHeight : 0;

      const availableHeight = rootRect.height - toolbarHeight - statusBarHeight;

      const tiptapElement = root.querySelector(".tiptap") as HTMLElement;
      if (tiptapElement) {
        tiptapElement.style.maxHeight = `${Math.max(availableHeight, 100)}px`;
        tiptapElement.style.height = `${Math.max(availableHeight, 100)}px`;
        tiptapElement.style.overflowY = "auto";
      }

      isUpdating = false;
    });
  };

  // 初始化布局
  const initLayout = () => {
    const root = editorRootRef();
    if (!root) return;

    // 设置ResizeObserver
    resizeObserver = new ResizeObserver(() => {
      nextTick(() => updateEditorHeight());
    });
    resizeObserver.observe(root);

    // 初始高度计算
    nextTick(() => updateEditorHeight());

    // 添加窗口resize监听
    windowResizeHandler = () => {
      if (resizeTimeout) clearTimeout(resizeTimeout);
      resizeTimeout = setTimeout(() => {
        nextTick(() => updateEditorHeight());
      }, 16);
    };
    window.addEventListener("resize", windowResizeHandler);
  };

  // 清理
  const cleanup = () => {
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    if (windowResizeHandler) {
      window.removeEventListener("resize", windowResizeHandler);
      windowResizeHandler = null;
    }
    if (resizeTimeout) {
      clearTimeout(resizeTimeout);
    }
  };

  // 手动触发高度更新
  const triggerResize = () => {
    nextTick(() => updateEditorHeight());
  };

  onMounted(() => {
    initLayout();
  });

  onUnmounted(() => {
    cleanup();
  });

  return {
    updateEditorHeight,
    triggerResize,
    cleanup,
  };
}