<template>
  <div class="config-manager">
    <div class="config-header">
      <div class="header-left">
        <h2 class="config-title">配置管理</h2>
        <div class="config-version">
          当前版本: {{ configData?.app.version || "1.0.0" }}
        </div>
      </div>
    </div>

    <div class="config-actions">
      <button
        class="btn btn-primary"
        @click="handleSaveAll"
        :disabled="isLoading"
      >
        <span class="btn-icon">💾</span>
        保存所有配置
      </button>
      <button
        class="btn btn-secondary"
        @click="handleExport"
        :disabled="isLoading"
      >
        <span class="btn-icon">📥</span>
        导出配置
      </button>
      <button
        class="btn btn-warning"
        @click="handleReset"
        :disabled="isLoading"
      >
        <span class="btn-icon">🔄</span>
        重置配置
      </button>
      <button class="btn btn-info" @click="handleReload" :disabled="isLoading">
        <span class="btn-icon">🔃</span>
        重新加载
      </button>
      <button class="btn btn-success" @click="testButtonClick">
        <span class="btn-icon">🔍</span>
        测试点击
      </button>
    </div>

    <div class="config-content">
      <CategoryTabs
        :categories="categories"
        v-model:active-category="activeCategory"
      />

      <div class="config-items">
        <div
          v-for="item in currentCategoryItems"
          :key="item.key"
          class="config-item"
        >
          <div class="config-item-label">
            <span class="item-key">{{ item.key }}</span>
            <span v-if="item.description" class="item-description">
              {{ item.description }}
            </span>
          </div>
          <div class="config-item-value">
            <div
              class="checkbox-wrapper"
              :class="{ active: !!editingValues[item.key] }"
              v-if="item.type === 'boolean'"
              @click="toggleBoolean(item.key)"
            >
              <input
                type="checkbox"
                class="value-checkbox"
                :checked="!!editingValues[item.key]"
                disabled
              />
              <span class="checkbox-track"></span>
            </div>
            <input
              v-else-if="item.type === 'number'"
              type="number"
              class="value-input"
              v-model.number="editingValues[item.key]"
              :placeholder="String(item.default)"
            />
            <input
              v-else-if="item.type === 'array'"
              type="text"
              class="value-input"
              :value="(editingValues[item.key] as string[])?.join(', ') || ''"
              @input="handleArrayInput(item.key, $event)"
              placeholder="多个值用逗号分隔"
            />
            <input
              v-else
              v-model="editingValues[item.key]"
              type="text"
              class="value-input"
              :placeholder="item.default?.toString()"
            />
          </div>
        </div>
      </div>
    </div>

    <div v-if="isLoading" class="loading-overlay">
      <div class="loading-container">
        <div class="loading-spinner"></div>
        <span class="loading-text">加载中...</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * 配置管理组件 - ConfigManager.vue
 *
 * 功能概述：
 * - 提供应用配置的可视化管理界面
 * - 支持配置的加载、编辑、保存、导出和重置
 * - 通过 Tauri IPC 与后端进行数据通信
 * - 采用分类标签页形式展示不同类型的配置项
 *
 * 技术实现：
 * - Vue 3 + TypeScript 组合式API
 * - Tauri invoke 进行跨进程通信
 * - Naive UI 组件库提供 UI 组件
 * - TOML 格式配置文件支持
 */
import { ref, computed, onMounted, watch, nextTick } from "vue";
import { useMessage } from "naive-ui";
import type { TomlConfig } from "../config/types";
import { configService } from "../services/configService";
import CategoryTabs from "./CategoryTabs.vue";

/**
 * 配置字段接口定义
 * 用于描述每个配置项的元数据
 */
interface ConfigField {
  key: string; // 配置项唯一标识
  description: string; // 配置项描述（显示给用户）
  type: "string" | "number" | "boolean" | "array"; // 数据类型
  default: unknown; // 默认值
  path: string[]; // 在配置对象中的路径（用于嵌套对象访问）
}

// Naive UI 消息提示组件
const message = useMessage();

// 当前加载的配置数据（从后端获取）
const configData = ref<TomlConfig | null>(null);

// 当前激活的配置分类标签
const activeCategory = ref("app");

// 用户正在编辑的配置值（与 configData 分离，支持取消操作）
const editingValues = ref<Record<string, unknown>>({});

// 加载状态标识（用于显示加载遮罩）
const isLoading = ref(false);

/**
 * 配置分类列表
 * 定义了配置管理页面中显示的各个分类标签
 */
const categories = computed(() => [
  { name: "app", label: "应用配置", description: "应用基本信息" },
  { name: "api", label: "API配置", description: "接口相关配置" },
  { name: "window", label: "窗口配置", description: "窗口尺寸设置" },
  { name: "editor", label: "编辑器配置", description: "编辑器相关设置" },
  { name: "features", label: "功能开关", description: "功能特性配置" },
  { name: "performance", label: "性能配置", description: "性能监控设置" },
  { name: "cache", label: "缓存配置", description: "缓存相关设置" },
  { name: "gzip", label: "Gzip配置", description: "压缩相关设置" },
  { name: "pagination", label: "分页配置", description: "分页参数设置" },
  { name: "requestMerging", label: "请求合并", description: "请求合并配置" },
  { name: "security", label: "安全配置", description: "敏感配置项" },
  { name: "entryConfig", label: "入口配置", description: "配置页面入口设置" },
]);

const configFields: Record<string, ConfigField[]> = {
  app: [
    {
      key: "name",
      description: "应用名称",
      type: "string",
      default: "iNovel",
      path: ["app", "name"],
    },
    {
      key: "version",
      description: "应用版本号",
      type: "string",
      default: "1.1.2",
      path: ["app", "version"],
    },
    {
      key: "environment",
      description: "运行环境",
      type: "string",
      default: "development",
      path: ["app", "environment"],
    },
    {
      key: "description",
      description: "应用描述",
      type: "string",
      default: "一款现代化的小说创作工具",
      path: ["app", "description"],
    },
  ],
  api: [
    {
      key: "base_url",
      description: "API基础地址",
      type: "string",
      default: "http://localhost:8080",
      path: ["api", "base_url"],
    },
    {
      key: "timeout_ms",
      description: "API超时时间(毫秒)",
      type: "number",
      default: 30000,
      path: ["api", "timeout_ms"],
    },
    {
      key: "max_retries",
      description: "最大重试次数",
      type: "number",
      default: 3,
      path: ["api", "max_retries"],
    },
  ],
  window: [
    {
      key: "default_width",
      description: "默认窗口宽度",
      type: "number",
      default: 1200,
      path: ["window", "default_width"],
    },
    {
      key: "default_height",
      description: "默认窗口高度",
      type: "number",
      default: 800,
      path: ["window", "default_height"],
    },
    {
      key: "min_width",
      description: "最小窗口宽度",
      type: "number",
      default: 600,
      path: ["window", "min_width"],
    },
    {
      key: "min_height",
      description: "最小窗口高度",
      type: "number",
      default: 800,
      path: ["window", "min_height"],
    },
    {
      key: "max_width",
      description: "最大窗口宽度",
      type: "number",
      default: 1920,
      path: ["window", "max_width"],
    },
    {
      key: "max_height",
      description: "最大窗口高度",
      type: "number",
      default: 1200,
      path: ["window", "max_height"],
    },
    {
      key: "resizable",
      description: "允许调整大小",
      type: "boolean",
      default: false,
      path: ["window", "resizable"],
    },
  ],
  editor: [
    {
      key: "default_font_size",
      description: "默认字体大小",
      type: "number",
      default: 16,
      path: ["editor", "default_font_size"],
    },
    {
      key: "default_font",
      description: "默认字体",
      type: "string",
      default: "微软雅黑",
      path: ["editor", "default_font"],
    },
    {
      key: "line_spacing",
      description: "行间距",
      type: "number",
      default: 1.5,
      path: ["editor", "line_spacing"],
    },
    {
      key: "show_line_numbers",
      description: "显示行号",
      type: "boolean",
      default: true,
      path: ["editor", "show_line_numbers"],
    },
    {
      key: "spell_check_enabled",
      description: "拼写检查",
      type: "boolean",
      default: true,
      path: ["editor", "spell_check_enabled"],
    },
  ],
  features: [
    {
      key: "auto_save_enabled",
      description: "自动保存开关",
      type: "boolean",
      default: true,
      path: ["features", "auto_save_enabled"],
    },
    {
      key: "sync_enabled",
      description: "云同步开关",
      type: "boolean",
      default: false,
      path: ["features", "sync_enabled"],
    },
    {
      key: "writing_stats_enabled",
      description: "写作统计开关",
      type: "boolean",
      default: true,
      path: ["features", "writing_stats_enabled"],
    },
    {
      key: "inspiration_board_enabled",
      description: "灵感面板开关",
      type: "boolean",
      default: true,
      path: ["features", "inspiration_board_enabled"],
    },
  ],
  performance: [
    {
      key: "monitoring_enabled",
      description: "性能监控",
      type: "boolean",
      default: true,
      path: ["performance", "monitoring_enabled"],
    },
    {
      key: "slow_request_threshold_ms",
      description: "慢请求阈值(毫秒)",
      type: "number",
      default: 1000,
      path: ["performance", "slow_request_threshold_ms"],
    },
    {
      key: "log_payload_size",
      description: "记录请求大小",
      type: "boolean",
      default: true,
      path: ["performance", "log_payload_size"],
    },
  ],
  cache: [
    {
      key: "enabled",
      description: "启用缓存",
      type: "boolean",
      default: true,
      path: ["cache", "enabled"],
    },
    {
      key: "max_entries",
      description: "最大缓存条目",
      type: "number",
      default: 1000,
      path: ["cache", "max_entries"],
    },
    {
      key: "ttl_seconds",
      description: "缓存过期时间(秒)",
      type: "number",
      default: 300,
      path: ["cache", "ttl_seconds"],
    },
    {
      key: "cached_commands",
      description: "缓存命令列表",
      type: "array",
      default: [],
      path: ["cache", "cached_commands"],
    },
  ],
  gzip: [
    {
      key: "enabled",
      description: "启用Gzip压缩",
      type: "boolean",
      default: true,
      path: ["gzip", "enabled"],
    },
    {
      key: "level",
      description: "压缩级别(0-9)",
      type: "number",
      default: 6,
      path: ["gzip", "level"],
    },
    {
      key: "min_size",
      description: "最小压缩阈值(字节)",
      type: "number",
      default: 1024,
      path: ["gzip", "min_size"],
    },
    {
      key: "compress_types",
      description: "压缩MIME类型",
      type: "array",
      default: [],
      path: ["gzip", "compress_types"],
    },
  ],
  pagination: [
    {
      key: "default_page_size",
      description: "默认每页条数",
      type: "number",
      default: 20,
      path: ["pagination", "default_page_size"],
    },
    {
      key: "max_page_size",
      description: "最大每页条数",
      type: "number",
      default: 100,
      path: ["pagination", "max_page_size"],
    },
  ],
  requestMerging: [
    {
      key: "enabled",
      description: "启用请求合并",
      type: "boolean",
      default: true,
      path: ["request_merging", "enabled"],
    },
    {
      key: "window_ms",
      description: "合并窗口时间(毫秒)",
      type: "number",
      default: 300,
      path: ["request_merging", "window_ms"],
    },
    {
      key: "max_batch_size",
      description: "最大批量大小",
      type: "number",
      default: 50,
      path: ["request_merging", "max_batch_size"],
    },
  ],
  security: [
    {
      key: "api_key",
      description: "API密钥",
      type: "string",
      default: "",
      path: ["security", "api_key"],
    },
    {
      key: "secret_token",
      description: "安全令牌",
      type: "string",
      default: "",
      path: ["security", "secret_token"],
    },
  ],
  entryConfig: [
    {
      key: "entry_enabled",
      description: "启用配置页面入口",
      type: "boolean",
      default: true,
      path: ["entry_config", "enabled"],
    },
    {
      key: "entry_display_name",
      description: "入口显示名称",
      type: "string",
      default: "配置管理",
      path: ["entry_config", "display_name"],
    },
    {
      key: "entry_icon",
      description: "入口图标",
      type: "string",
      default: "settings",
      path: ["entry_config", "icon"],
    },
    {
      key: "entry_tooltip",
      description: "入口提示文本",
      type: "string",
      default: "打开配置管理页面",
      path: ["entry_config", "tooltip"],
    },
    {
      key: "entry_locations",
      description:
        "入口位置(逗号分隔: menu_bar, toolbar, system_tray, keyboard)",
      type: "array",
      default: ["menu_bar", "toolbar"],
      path: ["entry_config", "locations"],
    },
    {
      key: "entry_roles",
      description: "允许访问的角色(逗号分隔: admin, advanced, standard, guest)",
      type: "array",
      default: ["admin", "advanced"],
      path: ["entry_config", "allowed_roles"],
    },
    {
      key: "entry_shortcut_key",
      description: "快捷键按键",
      type: "string",
      default: "C",
      path: ["entry_config", "shortcut_key"],
    },
    {
      key: "entry_shortcut_modifiers",
      description: "快捷键修饰键(逗号分隔: Ctrl, Shift, Alt, Meta)",
      type: "array",
      default: ["Ctrl", "Shift"],
      path: ["entry_config", "shortcut_modifiers"],
    },
  ],
};

const currentCategoryItems = computed(() => {
  return configFields[activeCategory.value] || [];
});

/**
 * 从嵌套对象中按路径获取值
 * @param obj - 目标对象
 * @param path - 路径数组，如 ['app', 'name']
 * @returns 路径指向的值，若路径不存在返回 undefined
 */
function getValueFromPath(
  obj: Record<string, unknown>,
  path: string[]
): unknown {
  let value: unknown = obj;
  for (const key of path) {
    if (value && typeof value === "object") {
      value = (value as Record<string, unknown>)[key];
    } else {
      return undefined;
    }
  }
  return value;
}

/**
 * 按路径设置嵌套对象的值
 * @param obj - 目标对象
 * @param path - 路径数组，如 ['app', 'name']
 * @param value - 要设置的值
 */
function setValueToPath(
  obj: Record<string, unknown>,
  path: string[],
  value: unknown
): void {
  let current = obj;
  for (let i = 0; i < path.length; i++) {
    const key = path[i];
    if (i === path.length - 1) {
      current[key] = value;
    } else {
      if (!current[key] || typeof current[key] !== "object") {
        current[key] = {};
      }
      current = current[key] as Record<string, unknown>;
    }
  }
}

/**
 * 处理数组类型输入框的变化
 * 将逗号分隔的字符串转换为数组
 * @param key - 配置项标识
 * @param event - 输入事件
 */
function handleArrayInput(key: string, event: Event) {
  const target = event.target as HTMLInputElement;
  const value = target.value;
  const arr = value
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s);
  editingValues.value[key] = arr;
}

/**
 * 从后端加载配置数据
 * 通过 configService 动态获取配置，支持回退到默认配置
 */
async function loadConfig() {
  console.log("loadConfig called");
  isLoading.value = true;
  try {
    console.log("Loading config via configService...");
    const fetchedConfig = await configService.loadConfig();
    console.log("Config loaded successfully:", fetchedConfig);

    configData.value = fetchedConfig;
    console.log("configData updated:", configData.value);

    initEditingValues();
    console.log("Editing values initialized:", editingValues.value);

    const loadError = configService.getLoadError();
    if (loadError) {
      console.warn("Config loaded with fallback:", loadError.message);
      message.warning("配置文件不存在或读取失败，已使用默认配置");
    } else {
      message.success("配置加载成功");
    }
  } catch (e) {
    console.error("Error loading config:", e);
    const errorMessage = e instanceof Error ? e.message : String(e);
    message.error("加载配置失败: " + (errorMessage || "未知错误"));
  } finally {
    isLoading.value = false;
    console.log("loadConfig finished, isLoading:", isLoading.value);
  }
}

/**
 * 切换布尔值配置
 * @param key 配置项的 key
 */
function toggleBoolean(key: string) {
  console.log(
    "toggleBoolean called for:",
    key,
    "current value:",
    editingValues.value[key]
  );
  editingValues.value[key] = !editingValues.value[key];
  console.log("new value:", editingValues.value[key]);
}

/**
 * 初始化编辑值
 * 将 configData 中的配置值同步到 editingValues，供用户编辑
 */
function initEditingValues() {
  if (!configData.value) return;

  for (const category of Object.values(configFields)) {
    for (const field of category) {
      const value = getValueFromPath(
        configData.value as Record<string, unknown>,
        field.path
      );
      editingValues.value[field.key] = value ?? field.default;
    }
  }
}

/**
 * 保存所有配置
 * 将用户编辑的值提交到后端并保存到配置文件
 */
async function handleSaveAll() {
  console.log("handleSaveAll called");
  if (!configData.value) {
    console.log("No config data to save");
    message.error("没有可保存的配置");
    return;
  }

  isLoading.value = true;
  try {
    const newConfig = { ...configData.value };

    for (const category of Object.values(configFields)) {
      for (const field of category) {
        const value = editingValues.value[field.key];
        setValueToPath(newConfig as Record<string, unknown>, field.path, value);
      }
    }

    console.log("Saving config via configService:", newConfig);
    await configService.saveConfig(newConfig);
    configData.value = configService.getConfig();
    message.success("所有配置保存成功");

    emit("configUpdated", configData.value);
  } catch (e) {
    console.error("Error saving config:", e);
    const errorMessage = e instanceof Error ? e.message : String(e);
    message.error("保存配置失败: " + (errorMessage || "未知错误"));
  } finally {
    isLoading.value = false;
  }
}

/**
 * 导出配置
 * 将当前配置数据导出为文件下载
 */
async function handleExport() {
  console.log("handleExport called");
  if (!configData.value) return;

  try {
    if (!window.showSaveFilePicker) {
      const content = JSON.stringify(configData.value, null, 2);
      const blob = new Blob([content], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `config_export_${Date.now()}.toml`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } else {
      const path = await window.showSaveFilePicker({
        suggestedName: `config_export_${Date.now()}.toml`,
        types: [
          { description: "TOML文件", accept: { "text/plain": [".toml"] } },
        ],
      });

      const content = JSON.stringify(configData.value, null, 2);
      const blob = new Blob([content], { type: "text/plain" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = path.name;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    }

    message.success("配置导出成功");
  } catch (e: unknown) {
    if ((e as Error).name !== "AbortError") {
      message.error("导出失败: " + (e as Error).message);
    }
  }
}

/**
 * 重置配置到默认值
 * 从资源目录的 default_config.toml 文件加载原始配置
 */
async function handleReset() {
  console.log("handleReset called");
  if (!confirm("确定要重置所有配置吗？此操作不可撤销！")) return;

  isLoading.value = true;
  try {
    console.log("Resetting to default config from resources...");
    const defaultConfig = await invoke<TomlConfig>("reset_to_default_config");
    console.log("Default config loaded:", defaultConfig);

    configData.value = defaultConfig;
    initEditingValues();
    message.success("配置已重置为默认值");

    emit("configUpdated", defaultConfig);
  } catch (e) {
    const errorMessage = e instanceof Error ? e.message : String(e);
    message.error("重置失败: " + (errorMessage || "未知错误"));
  } finally {
    isLoading.value = false;
  }
}

/**
 * 重新加载配置
 * 从后端重新读取配置文件
 */
async function handleReload() {
  console.log("handleReload called");
  isLoading.value = true;
  try {
    await loadConfig();
    message.success("配置重新加载成功");
  } catch (e) {
    const errorMessage = e instanceof Error ? e.message : String(e);
    message.error("重新加载失败: " + (errorMessage || "未知错误"));
  } finally {
    isLoading.value = false;
  }
}

const emit = defineEmits<{
  configUpdated: [config: TomlConfig];
}>();

/**
 * 测试按钮点击函数
 */
function testButtonClick() {
  console.log("Button click handler works!");
  message.success("按钮点击测试成功");
}

/**
 * 监听分类切换
 */
watch(activeCategory, () => {
  initEditingValues();
});

/**
 * 组件挂载时执行
 */
onMounted(async () => {
  console.log("ConfigManager mounted, loading config...");

  window.addEventListener("error", (event) => {
    console.error(
      "Global error:",
      event.error,
      event.message,
      event.filename,
      event.lineno
    );
  });

  window.addEventListener("unhandledrejection", (event) => {
    console.error("Unhandled rejection:", event.reason);
  });

  await loadConfig();
  console.log("Config loaded:", configData.value);
});
</script>

<style scoped>
.config-manager {
  max-width: 100%;
  margin: 0 auto;
  padding: 24px;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
    "Helvetica Neue", Arial, sans-serif;
  background: linear-gradient(135deg, #f5f7fa 0%, #e4e8ec 100%);
  min-height: 100vh;
}

.config-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding: 20px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 16px;
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.3);
}

.header-left {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.config-title {
  margin: 0;
  font-size: 28px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: 0.5px;
}

.config-version {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.85);
}

.config-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 24px;
  border: none;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: hidden;
}

.btn::before {
  content: "";
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.2),
    transparent
  );
  transition: left 0.5s;
}

.btn:hover::before {
  left: 100%;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none !important;
}

.btn:disabled::before {
  display: none;
}

.btn-primary {
  background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(79, 70, 229, 0.4);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(79, 70, 229, 0.5);
}

.btn-secondary {
  background: #ffffff;
  color: #4b5563;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  border: 1px solid #e5e7eb;
}

.btn-secondary:hover:not(:disabled) {
  background: #f9fafb;
  border-color: #d1d5db;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.btn-warning {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(245, 158, 11, 0.4);
}

.btn-warning:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(245, 158, 11, 0.5);
}

.btn-info {
  background: linear-gradient(135deg, #06b6d4 0%, #0891b2 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(6, 182, 212, 0.4);
}

.btn-info:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(6, 182, 212, 0.5);
}

.btn-success {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
  box-shadow: 0 4px 15px rgba(16, 185, 129, 0.4);
}

.btn-success:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(16, 185, 129, 0.5);
}

.btn-icon {
  font-size: 18px;
}

.config-content {
  background: #ffffff;
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
}

.config-items {
  padding: 24px;
}

.config-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  margin-bottom: 12px;
  background: linear-gradient(135deg, #ffffff 0%, #f9fafb 100%);
  border-radius: 12px;
  border: 1px solid #e5e7eb;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.config-item:hover {
  border-color: #4f46e5;
  box-shadow: 0 4px 16px rgba(79, 70, 229, 0.12);
  transform: translateX(4px);
}

.config-item-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 200px;
}

.item-key {
  font-weight: 600;
  color: #1f2937;
  font-size: 15px;
}

.item-description {
  font-size: 13px;
  color: #9ca3af;
}

.config-item-value {
  display: flex;
  align-items: center;
}

.checkbox-wrapper {
  position: relative;
  width: 44px;
  height: 24px;
  cursor: pointer;
  user-select: none;
  -webkit-user-select: none;
}

.value-checkbox {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.checkbox-track {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: #d1d5db;
  border-radius: 12px;
  transition: all 0.3s ease;
}

.checkbox-track::before {
  content: "";
  position: absolute;
  top: 3px;
  left: 3px;
  width: 18px;
  height: 18px;
  background: white;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  transition: all 0.3s ease;
}

.checkbox-wrapper.active .checkbox-track {
  background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%);
}

.checkbox-wrapper.active .checkbox-track::before {
  transform: translateX(20px);
}

.checkbox-wrapper:hover .checkbox-track {
  background: #9ca3af;
}

.checkbox-wrapper.active:hover .checkbox-track {
  background: linear-gradient(135deg, #4338ca 0%, #6d28d9 100%);
}

.checkbox-wrapper:active .checkbox-track::before {
  transform: scale(0.95);
}

.checkbox-wrapper.active:active .checkbox-track::before {
  transform: translateX(20px) scale(0.95);
}

.value-input {
  padding: 12px 16px;
  border: 2px solid #e5e7eb;
  border-radius: 10px;
  font-size: 14px;
  width: 220px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  color: #374151;
  background: #ffffff;
}

.value-input:focus {
  outline: none;
  border-color: #4f46e5;
  box-shadow: 0 0 0 3px rgba(79, 70, 229, 0.1);
  transform: translateY(-1px);
}

.value-input::placeholder {
  color: #9ca3af;
}

:deep(.dark) .config-manager {
  background: linear-gradient(135deg, #1f2937 0%, #111827 100%);
}

:deep(.dark) .config-content {
  background: #1f2937;
}

:deep(.dark) .config-item {
  background: #374151;
  border-color: #4b5563;
}

:deep(.dark) .config-item:hover {
  border-color: #4f46e5;
}

:deep(.dark) .item-key {
  color: #f9fafb;
}

:deep(.dark) .item-description {
  color: #9ca3af;
}

:deep(.dark) .value-input {
  color: #f9fafb;
  background: #1f2937;
  border-color: #4b5563;
}

:deep(.dark) .value-input::placeholder {
  color: #6b7280;
}

:deep(.dark) .checkbox-track {
  background: #4b5563;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  background: white;
  padding: 40px 50px;
  border-radius: 16px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
}

.loading-spinner {
  width: 48px;
  height: 48px;
  border: 4px solid #e5e7eb;
  border-top-color: #4f46e5;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.loading-text {
  font-size: 15px;
  font-weight: 500;
  color: #6b7280;
}

@media (max-width: 1024px) {
  .config-manager {
    padding: 16px;
  }

  .config-header {
    padding: 16px;
  }

  .config-title {
    font-size: 24px;
  }

  .config-actions {
    gap: 8px;
  }

  .btn {
    padding: 10px 18px;
    font-size: 13px;
  }
}

@media (max-width: 768px) {
  .config-manager {
    padding: 12px;
  }

  .config-header {
    padding: 14px;
    border-radius: 12px;
  }

  .config-title {
    font-size: 22px;
  }

  .config-version {
    font-size: 12px;
  }

  .config-actions {
    gap: 6px;
  }

  .btn {
    padding: 10px 16px;
    font-size: 12px;
  }

  .btn-icon {
    font-size: 14px;
  }

  .config-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
    padding: 14px;
  }

  .config-item-label {
    min-width: 100%;
  }

  .config-item-value {
    width: 100%;
  }

  .value-input {
    width: 100%;
  }
}

@media (max-width: 480px) {
  .config-manager {
    padding: 8px;
  }

  .config-header {
    padding: 12px;
  }

  .config-title {
    font-size: 20px;
  }

  .config-actions {
    justify-content: center;
  }

  .btn {
    padding: 8px 14px;
    font-size: 11px;
  }

  .config-items {
    padding: 12px;
  }

  .config-item {
    padding: 12px;
    border-radius: 8px;
  }
}
</style>