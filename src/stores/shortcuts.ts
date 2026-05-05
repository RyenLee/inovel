import { defineStore } from "pinia";
import { ref, computed } from "vue";

export interface Shortcut {
  id: string;
  name: string;
  category: string;
  defaultKeys: string[];
  keys: string[];
  description?: string;
}

export interface ShortcutConfig {
  shortcuts: Record<string, string[]>;
}

const STORAGE_KEY = "inovel_shortcuts";

// Default shortcuts configuration
const defaultShortcuts: Shortcut[] = [
  // File operations
  { id: "save", name: "保存", category: "文件", defaultKeys: ["Ctrl", "S"], keys: ["Ctrl", "S"], description: "保存当前章节" },
  { id: "new_chapter", name: "新建章节", category: "文件", defaultKeys: ["Ctrl", "N"], keys: ["Ctrl", "N"], description: "创建新章节" },
  { id: "export", name: "导出", category: "文件", defaultKeys: ["Ctrl", "E"], keys: ["Ctrl", "E"], description: "导出项目" },
  { id: "backup", name: "备份", category: "文件", defaultKeys: ["Ctrl", "Shift", "B"], keys: ["Ctrl", "Shift", "B"], description: "备份项目" },

  // Editor modes
  { id: "typewriter", name: "打字机模式", category: "编辑模式", defaultKeys: ["Ctrl", "Shift", "T"], keys: ["Ctrl", "Shift", "T"], description: "开启/关闭打字机模式" },
  { id: "focus", name: "聚焦模式", category: "编辑模式", defaultKeys: ["Ctrl", "Shift", "F"], keys: ["Ctrl", "Shift", "F"], description: "开启/关闭聚焦模式" },
  { id: "zen", name: "禅模式", category: "编辑模式", defaultKeys: ["Ctrl", "Shift", "Z"], keys: ["Ctrl", "Shift", "Z"], description: "开启/关闭禅模式" },
  { id: "fullscreen", name: "全屏", category: "编辑模式", defaultKeys: ["F11"], keys: ["F11"], description: "切换全屏" },

  // Navigation
  { id: "prev_chapter", name: "上一章", category: "导航", defaultKeys: ["Ctrl", "PageUp"], keys: ["Ctrl", "PageUp"], description: "切换到上一章节" },
  { id: "next_chapter", name: "下一章", category: "导航", defaultKeys: ["Ctrl", "PageDown"], keys: ["Ctrl", "PageDown"], description: "切换到下一章节" },
  { id: "toggle_sidebar", name: "切换侧边栏", category: "导航", defaultKeys: ["Ctrl", "B"], keys: ["Ctrl", "B"], description: "显示/隐藏侧边栏" },
  { id: "toggle_worldbuilding", name: "切换世界观面板", category: "导航", defaultKeys: ["Ctrl", "W"], keys: ["Ctrl", "W"], description: "显示/隐藏世界观面板" },

  // View
  { id: "toggle_theme", name: "切换主题", category: "视图", defaultKeys: ["Ctrl", "Shift", "D"], keys: ["Ctrl", "Shift", "D"], description: "切换深色/浅色主题" },
  { id: "show_stats", name: "项目统计", category: "视图", defaultKeys: ["Ctrl", "Shift", "S"], keys: ["Ctrl", "Shift", "S"], description: "打开项目统计" },
  { id: "show_settings", name: "项目设置", category: "视图", defaultKeys: ["Ctrl", ","], keys: ["Ctrl", ","], description: "打开项目设置" },

  // Tools
  { id: "snapshot", name: "创建快照", category: "工具", defaultKeys: ["Ctrl", "Shift", "G"], keys: ["Ctrl", "Shift", "G"], description: "创建Git快照" },
  { id: "name_generator", name: "名称生成器", category: "工具", defaultKeys: ["Ctrl", "Shift", "N"], keys: ["Ctrl", "Shift", "N"], description: "打开名称生成器" },
  { id: "sensitive_words", name: "敏感词管理", category: "工具", defaultKeys: ["Ctrl", "Shift", "W"], keys: ["Ctrl", "Shift", "W"], description: "打开敏感词管理" },
];

// Load shortcuts from localStorage
function loadShortcuts(): Record<string, string[]> {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return JSON.parse(stored);
    }
  } catch {
    // ignore
  }
  // Return defaults
  const config: Record<string, string[]> = {};
  defaultShortcuts.forEach((s) => {
    config[s.id] = [...s.keys];
  });
  return config;
}

// Save shortcuts to localStorage
function saveShortcuts(config: Record<string, string[]>) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
}

export const useShortcutStore = defineStore("shortcuts", () => {
  const shortcuts = ref<Shortcut[]>(defaultShortcuts.map((s) => ({
    ...s,
    keys: loadShortcuts()[s.id] || [...s.keys],
  })));

  const groupedShortcuts = computed(() => {
    const groups: Record<string, Shortcut[]> = {};
    shortcuts.value.forEach((s) => {
      if (!groups[s.category]) {
        groups[s.category] = [];
      }
      groups[s.category].push(s);
    });
    return groups;
  });

  // Get shortcut by ID
  function getShortcut(id: string): Shortcut | undefined {
    return shortcuts.value.find((s) => s.id === id);
  }

  // Update shortcut keys
  function updateShortcut(id: string, keys: string[]) {
    const shortcut = shortcuts.value.find((s) => s.id === id);
    if (shortcut) {
      shortcut.keys = keys;
      saveConfig();
    }
  }

  // Reset to default
  function resetToDefault(id?: string) {
    if (id) {
      const shortcut = shortcuts.value.find((s) => s.id === id);
      if (shortcut) {
        shortcut.keys = [...shortcut.defaultKeys];
        saveConfig();
      }
    } else {
      // Reset all
      shortcuts.value.forEach((s) => {
        s.keys = [...s.defaultKeys];
      });
      saveConfig();
    }
  }

  // Reset all to defaults
  function resetAll() {
    resetToDefault();
  }

  // Save current config to localStorage
  function saveConfig() {
    const config: Record<string, string[]> = {};
    shortcuts.value.forEach((s) => {
      config[s.id] = [...s.keys];
    });
    saveShortcuts(config);
  }

  // Export config as JSON string
  function exportConfig(): string {
    const config: Record<string, string[]> = {};
    shortcuts.value.forEach((s) => {
      config[s.id] = [...s.keys];
    });
    return JSON.stringify(config, null, 2);
  }

  // Import config from JSON string
  function importConfig(json: string): boolean {
    try {
      const config: Record<string, string[]> = JSON.parse(json);
      shortcuts.value.forEach((s) => {
        if (config[s.id] && Array.isArray(config[s.id])) {
          s.keys = [...config[s.id]];
        }
      });
      saveConfig();
      return true;
    } catch {
      return false;
    }
  }

  // Check if a key combination matches a shortcut
  function matchShortcut(shortcut: Shortcut, event: KeyboardEvent): boolean {
    const pressed: string[] = [];
    if (event.ctrlKey) pressed.push("Ctrl");
    if (event.altKey) pressed.push("Alt");
    if (event.shiftKey) pressed.push("Shift");
    if (event.metaKey) pressed.push("Meta");

    const key = event.key;
    if (key !== "Control" && key !== "Alt" && key !== "Shift" && key !== "Meta") {
      // Normalize key names
      let normalizedKey = key;
      if (key === " ") normalizedKey = "Space";
      if (key === "PageUp") normalizedKey = "PageUp";
      if (key === "PageDown") normalizedKey = "PageDown";
      if (key.length === 1) normalizedKey = key.toUpperCase();
      pressed.push(normalizedKey);
    }

    if (pressed.length !== shortcut.keys.length) return false;

    return shortcut.keys.every((k) => pressed.includes(k));
  }

  return {
    shortcuts,
    groupedShortcuts,
    getShortcut,
    updateShortcut,
    resetToDefault,
    resetAll,
    saveConfig,
    exportConfig,
    importConfig,
    matchShortcut,
  };
});
