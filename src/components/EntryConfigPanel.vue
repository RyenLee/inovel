<template>
  <div class="entry-config-panel">
    <div class="panel-header">
      <h3 class="panel-title">配置页面入口设置</h3>
      <div class="header-actions">
        <button class="btn-reset" @click="handleReset">
          重置为默认
        </button>
      </div>
    </div>

    <div class="config-section">
      <div class="section-title">基础设置</div>
      
      <div class="config-item">
        <label class="config-label">
          <span class="label-text">启用入口</span>
          <span class="label-desc">控制配置页面入口是否可用</span>
        </label>
        <div class="config-value">
          <div class="checkbox-wrapper">
            <input
              type="checkbox"
              class="value-checkbox"
              v-model="localConfig.enabled"
            />
            <span class="checkbox-track"></span>
          </div>
        </div>
      </div>

      <div class="config-item">
        <label class="config-label">
          <span class="label-text">显示名称</span>
          <span class="label-desc">入口在界面上显示的名称</span>
        </label>
        <div class="config-value">
          <input
            v-model="localConfig.display_name"
            type="text"
            class="value-input"
            placeholder="配置管理"
          />
        </div>
      </div>

      <div class="config-item">
        <label class="config-label">
          <span class="label-text">图标样式</span>
          <span class="label-desc">选择入口图标</span>
        </label>
        <div class="config-value">
          <select v-model="localConfig.icon" class="value-select">
            <option v-for="icon in iconOptions" :key="icon.value" :value="icon.value">
              {{ icon.label }}
            </option>
          </select>
        </div>
      </div>

      <div class="config-item">
        <label class="config-label">
          <span class="label-text">提示文本</span>
          <span class="label-desc">鼠标悬停时显示的提示信息</span>
        </label>
        <div class="config-value">
          <input
            v-model="localConfig.tooltip"
            type="text"
            class="value-input"
            placeholder="打开配置管理页面"
          />
        </div>
      </div>
    </div>

    <div class="config-section">
      <div class="section-title">入口位置</div>
      <p class="section-desc">选择配置页面入口的显示位置（可多选）</p>
      
      <div class="location-grid">
        <label
          v-for="location in locationOptions"
          :key="location.value"
          class="location-item"
          :class="{ selected: localConfig.locations.includes(location.value) }"
        >
          <input
            type="checkbox"
            :value="location.value"
            v-model="localConfig.locations"
            :disabled="!localConfig.enabled"
          />
          <span class="location-icon">{{ location.icon }}</span>
          <span class="location-label">{{ location.label }}</span>
          <span class="location-desc">{{ location.description }}</span>
        </label>
      </div>
    </div>

    <div class="config-section">
      <div class="section-title">可见性控制</div>
      <p class="section-desc">设置哪些角色可以看到配置页面入口</p>
      
      <div class="role-grid">
        <label
          v-for="role in roleOptions"
          :key="role.value"
          class="role-item"
          :class="{ selected: localConfig.allowed_roles.includes(role.value) }"
        >
          <input
            type="checkbox"
            :value="role.value"
            v-model="localConfig.allowed_roles"
            :disabled="!localConfig.enabled"
          />
          <span class="role-badge" :class="role.value">{{ role.label }}</span>
          <span class="role-desc">{{ role.description }}</span>
        </label>
      </div>
    </div>

    <div class="config-section" v-if="localConfig.locations.includes('keyboard')">
      <div class="section-title">快捷键设置</div>
      
      <div class="shortcut-config">
        <div class="config-item">
          <label class="config-label">
            <span class="label-text">修饰键</span>
            <span class="label-desc">选择快捷键的修饰键（可多选）</span>
          </label>
          <div class="config-value">
            <div class="modifier-buttons">
              <button
                v-for="modifier in modifierOptions"
                :key="modifier"
                class="modifier-btn"
                :class="{ active: localConfig.shortcut_modifiers.includes(modifier) }"
                @click="toggleModifier(modifier)"
              >
                {{ modifier }}
              </button>
            </div>
          </div>
        </div>

        <div class="config-item">
          <label class="config-label">
            <span class="label-text">快捷键</span>
            <span class="label-desc">设置触发配置页面的按键</span>
          </label>
          <div class="config-value">
            <input
              v-model="localConfig.shortcut_key"
              type="text"
              class="value-input shortcut-input"
              maxlength="1"
              @keyup="validateShortcut"
              placeholder="C"
            />
          </div>
        </div>

        <div class="shortcut-preview">
          <span class="preview-label">快捷键预览:</span>
          <div class="preview-keys">
            <span
              v-for="modifier in localConfig.shortcut_modifiers"
              :key="modifier"
              class="key-badge"
            >
              {{ modifier }}
            </span>
            <span class="key-badge" v-if="localConfig.shortcut_key">
              {{ localConfig.shortcut_key.toUpperCase() }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div class="config-section">
      <div class="section-title">配置预览</div>
      
      <div class="preview-container">
        <div class="preview-header">
          <span class="preview-title">入口预览</span>
          <span class="preview-status" :class="localConfig.enabled ? 'enabled' : 'disabled'">
            {{ localConfig.enabled ? '已启用' : '已禁用' }}
          </span>
        </div>
        
        <div class="preview-content">
          <div v-if="localConfig.enabled" class="preview-entry">
            <span class="preview-icon">{{ getIconEmoji(localConfig.icon) }}</span>
            <span class="preview-name">{{ localConfig.display_name }}</span>
            <span class="preview-tooltip-hint">{{ localConfig.tooltip }}</span>
          </div>
          <div v-else class="preview-disabled">
            入口已禁用，启用后将根据配置显示
          </div>
        </div>

        <div class="preview-locations">
          <span class="preview-label">显示位置:</span>
          <span class="location-tags">
            <span
              v-for="loc in localConfig.locations"
              :key="loc"
              class="location-tag"
            >
              {{ getLocationLabel(loc) }}
            </span>
            <span v-if="localConfig.locations.length === 0" class="empty-tag">
              未选择
            </span>
          </span>
        </div>

        <div class="preview-roles">
          <span class="preview-label">可见角色:</span>
          <span class="role-tags">
            <span
              v-for="role in localConfig.allowed_roles"
              :key="role"
              class="role-tag"
              :class="role"
            >
              {{ getRoleLabel(role) }}
            </span>
            <span v-if="localConfig.allowed_roles.length === 0" class="empty-tag">
              无
            </span>
          </span>
        </div>
      </div>
    </div>

    <div class="panel-footer">
      <button class="btn-save" @click="handleSave">
        保存配置
      </button>
      <button class="btn-apply" @click="handleApply">
        应用并刷新入口
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from "vue";
import { useMessage } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import type { EntryConfigSection } from "../config/types";

const message = useMessage();

const defaultConfig: EntryConfigSection = {
  enabled: true,
  display_name: "配置管理",
  icon: "settings",
  tooltip: "打开配置管理页面",
  locations: ["menu_bar", "toolbar"],
  allowed_roles: ["admin", "advanced"],
  shortcut_key: "C",
  shortcut_modifiers: ["Ctrl", "Shift"],
};

const localConfig = reactive<EntryConfigSection>({ ...defaultConfig });

const iconOptions = [
  { value: "settings", label: "⚙️ 设置" },
  { value: "gear", label: "⚙️ 齿轮" },
  { value: "wrench", label: "🔧 扳手" },
  { value: "sliders", label: "🎚️ 滑块" },
  { value: "cog", label: "⚙️ 齿轮" },
  { value: "configuration", label: "📋 配置" },
];

const locationOptions = [
  { value: "menu_bar", label: "菜单栏", description: "应用顶部菜单栏", icon: "📋" },
  { value: "toolbar", label: "工具栏", description: "编辑器工具栏", icon: "🛠️" },
  { value: "system_tray", label: "系统托盘", description: "系统托盘菜单", icon: "🖥️" },
  { value: "keyboard", label: "快捷键", description: "键盘快捷键", icon: "⌨️" },
];

const roleOptions = [
  { value: "admin", label: "管理员", description: "具有全部权限" },
  { value: "advanced", label: "高级用户", description: "具有高级功能访问权限" },
  { value: "standard", label: "普通用户", description: "基础功能访问权限" },
  { value: "guest", label: "访客", description: "受限访问权限" },
];

const modifierOptions = ["Ctrl", "Shift", "Alt", "Meta"];

const isSaving = ref(false);

const loadConfig = async () => {
  try {
    const config = await invoke<TomlConfig>("read_toml_config");
    if (config.entry_config) {
      Object.assign(localConfig, config.entry_config);
    }
  } catch (e) {
    console.error("加载入口配置失败:", e);
  }
};

const handleSave = async () => {
  if (isSaving.value) return;
  
  isSaving.value = true;
  try {
    await invoke("write_toml_config", {
      newConfig: {
        entry_config: localConfig,
      },
    });
    message.success("入口配置保存成功");
  } catch (e) {
    message.error("保存失败: " + (e as Error).message);
  } finally {
    isSaving.value = false;
  }
};

const handleApply = async () => {
  await handleSave();
  window.dispatchEvent(new CustomEvent("entryConfigUpdated", { detail: { ...localConfig } }));
  message.info("入口配置已更新，将在下次操作时生效");
};

const handleReset = () => {
  if (confirm("确定要重置所有入口配置吗？")) {
    Object.assign(localConfig, defaultConfig);
    message.info("已重置为默认配置");
  }
};

const toggleModifier = (modifier: string) => {
  const index = localConfig.shortcut_modifiers.indexOf(modifier);
  if (index === -1) {
    localConfig.shortcut_modifiers.push(modifier);
  } else {
    localConfig.shortcut_modifiers.splice(index, 1);
  }
};

const validateShortcut = (event: Event) => {
  const target = event.target as HTMLInputElement;
  target.value = target.value.toUpperCase().replace(/[^A-Z0-9]/g, "");
};

const getIconEmoji = (icon: string): string => {
  const icons: Record<string, string> = {
    settings: "⚙️",
    gear: "⚙️",
    wrench: "🔧",
    sliders: "🎚️",
    cog: "⚙️",
    configuration: "📋",
  };
  return icons[icon] || "⚙️";
};

const getLocationLabel = (location: string): string => {
  const option = locationOptions.find((o) => o.value === location);
  return option?.label || location;
};

const getRoleLabel = (role: string): string => {
  const option = roleOptions.find((o) => o.value === role);
  return option?.label || role;
};

watch(
  () => localConfig.locations,
  (newLocations) => {
    if (!newLocations.includes("keyboard")) {
      localConfig.shortcut_key = "";
      localConfig.shortcut_modifiers = [];
    }
  },
  { deep: true }
);

loadConfig();
</script>

<style scoped>
.entry-config-panel {
  background: #ffffff;
  border-radius: 16px;
  padding: 24px;
  max-width: 800px;
  margin: 0 auto;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid #e5e7eb;
}

.panel-title {
  font-size: 20px;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.btn-reset {
  padding: 8px 16px;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 14px;
  color: #6b7280;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-reset:hover {
  background: #e5e7eb;
  color: #374151;
}

.config-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 8px;
}

.section-desc {
  font-size: 13px;
  color: #6b7280;
  margin-bottom: 16px;
}

.config-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: #f9fafb;
  border-radius: 10px;
  margin-bottom: 12px;
}

.config-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 200px;
}

.label-text {
  font-weight: 500;
  color: #374151;
}

.label-desc {
  font-size: 13px;
  color: #9ca3af;
}

.config-value {
  flex-shrink: 0;
}

.value-input {
  padding: 10px 14px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 14px;
  min-width: 200px;
  transition: all 0.2s;
}

.value-input:focus {
  outline: none;
  border-color: #4f46e5;
}

.value-select {
  padding: 10px 14px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 14px;
  min-width: 200px;
  background: white;
  cursor: pointer;
}

.checkbox-wrapper {
  position: relative;
  width: 44px;
  height: 24px;
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
  cursor: pointer;
  transition: all 0.3s;
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
  transition: all 0.3s;
}

.value-checkbox:checked + .checkbox-track {
  background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%);
}

.value-checkbox:checked + .checkbox-track::before {
  transform: translateX(20px);
}

.location-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.location-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 16px;
  background: #f9fafb;
  border: 2px solid #e5e7eb;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.location-item:hover {
  border-color: #4f46e5;
  background: rgba(79, 70, 229, 0.05);
}

.location-item.selected {
  border-color: #4f46e5;
  background: rgba(79, 70, 229, 0.08);
}

.location-item input {
  display: none;
}

.location-icon {
  font-size: 24px;
}

.location-label {
  font-weight: 500;
  color: #1f2937;
}

.location-desc {
  font-size: 13px;
  color: #6b7280;
}

.role-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.role-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  background: #f9fafb;
  border: 2px solid #e5e7eb;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.role-item:hover {
  border-color: #4f46e5;
}

.role-item.selected {
  border-color: #4f46e5;
  background: rgba(79, 70, 229, 0.08);
}

.role-item input {
  display: none;
}

.role-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
  width: fit-content;
}

.role-badge.admin {
  background: #fee2e2;
  color: #dc2626;
}

.role-badge.advanced {
  background: #dbeafe;
  color: #2563eb;
}

.role-badge.standard {
  background: #dcfce7;
  color: #16a34a;
}

.role-badge.guest {
  background: #fef3c7;
  color: #d97706;
}

.role-desc {
  font-size: 13px;
  color: #6b7280;
}

.shortcut-config {
  background: #f9fafb;
  border-radius: 12px;
  padding: 16px;
}

.modifier-buttons {
  display: flex;
  gap: 8px;
}

.modifier-btn {
  padding: 10px 16px;
  border: 2px solid #e5e7eb;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.modifier-btn:hover {
  border-color: #4f46e5;
}

.modifier-btn.active {
  background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%);
  border-color: #4f46e5;
  color: white;
}

.shortcut-input {
  text-transform: uppercase;
  text-align: center;
}

.shortcut-preview {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #e5e7eb;
}

.preview-label {
  font-size: 14px;
  color: #6b7280;
}

.preview-keys {
  display: flex;
  gap: 6px;
}

.key-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 36px;
  height: 36px;
  padding: 0 10px;
  background: #374151;
  color: white;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
}

.preview-container {
  background: #f9fafb;
  border-radius: 12px;
  padding: 16px;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.preview-title {
  font-weight: 600;
  color: #1f2937;
}

.preview-status {
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 500;
}

.preview-status.enabled {
  background: #dcfce7;
  color: #16a34a;
}

.preview-status.disabled {
  background: #fee2e2;
  color: #dc2626;
}

.preview-content {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: white;
  border-radius: 10px;
  margin-bottom: 16px;
}

.preview-entry {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.preview-icon {
  font-size: 48px;
}

.preview-name {
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
}

.preview-tooltip-hint {
  font-size: 14px;
  color: #6b7280;
}

.preview-disabled {
  font-size: 14px;
  color: #9ca3af;
}

.preview-locations,
.preview-roles {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.location-tags,
.role-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.location-tag {
  padding: 4px 10px;
  background: #dbeafe;
  color: #2563eb;
  border-radius: 20px;
  font-size: 13px;
}

.role-tag {
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 13px;
}

.role-tag.admin {
  background: #fee2e2;
  color: #dc2626;
}

.role-tag.advanced {
  background: #dbeafe;
  color: #2563eb;
}

.role-tag.standard {
  background: #dcfce7;
  color: #16a34a;
}

.role-tag.guest {
  background: #fef3c7;
  color: #d97706;
}

.empty-tag {
  padding: 4px 10px;
  background: #f3f4f6;
  color: #9ca3af;
  border-radius: 20px;
  font-size: 13px;
}

.panel-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid #e5e7eb;
}

.btn-save,
.btn-apply {
  padding: 12px 24px;
  border: none;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-save {
  background: #f3f4f6;
  color: #374151;
}

.btn-save:hover {
  background: #e5e7eb;
}

.btn-apply {
  background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%);
  color: white;
}

.btn-apply:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(79, 70, 229, 0.4);
}

@media (max-width: 640px) {
  .entry-config-panel {
    padding: 16px;
  }

  .config-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .config-label {
    min-width: 100%;
  }

  .value-input,
  .value-select {
    width: 100%;
  }

  .location-grid,
  .role-grid {
    grid-template-columns: 1fr;
  }

  .panel-footer {
    flex-direction: column;
  }

  .btn-save,
  .btn-apply {
    width: 100%;
  }
}
</style>