import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { EntryConfigSection, EntryLocation, UserRole } from "../config/types";
import { configService } from "../services/configService";

export const useEntryConfigStore = defineStore("entryConfig", () => {
  const config = ref<EntryConfigSection>({
    enabled: true,
    display_name: "配置管理",
    icon: "settings",
    tooltip: "打开配置管理页面",
    locations: ["menu_bar", "toolbar"],
    allowed_roles: ["admin", "advanced"],
    shortcut_key: "C",
    shortcut_modifiers: ["Ctrl", "Shift"],
  });

  const isLoading = ref(false);

  const currentUserRole = ref<UserRole>("admin");

  const isEntryEnabled = computed(() => {
    if (!config.value.enabled) return false;
    if (config.value.allowed_roles.length === 0) return false;
    return config.value.allowed_roles.includes(currentUserRole.value);
  });

  const hasLocation = (location: EntryLocation): boolean => {
    return config.value.locations.includes(location);
  };

  const shortcutString = computed(() => {
    if (!config.value.shortcut_key) return "";
    return [...config.value.shortcut_modifiers, config.value.shortcut_key.toUpperCase()].join(" + ");
  });

  const loadConfig = async () => {
    isLoading.value = true;
    try {
      const tomlConfig = await configService.loadConfig();
      if (tomlConfig.entry_config) {
        config.value = { ...config.value, ...tomlConfig.entry_config };
      }
    } catch (e) {
      console.error("Failed to load entry config:", e);
    } finally {
      isLoading.value = false;
    }
  };

  const saveConfig = async (newConfig: Partial<EntryConfigSection>) => {
    isLoading.value = true;
    try {
      Object.assign(config.value, newConfig);
      await configService.saveConfig({ entry_config: config.value });
      window.dispatchEvent(new CustomEvent("entryConfigUpdated", { detail: { ...config.value } }));
    } catch (e) {
      console.error("Failed to save entry config:", e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  };

  const resetToDefault = () => {
    config.value = {
      enabled: true,
      display_name: "配置管理",
      icon: "settings",
      tooltip: "打开配置管理页面",
      locations: ["menu_bar", "toolbar"],
      allowed_roles: ["admin", "advanced"],
      shortcut_key: "C",
      shortcut_modifiers: ["Ctrl", "Shift"],
    };
  };

  const updateUserRole = (role: UserRole) => {
    currentUserRole.value = role;
  };

  return {
    config,
    isLoading,
    currentUserRole,
    isEntryEnabled,
    hasLocation,
    shortcutString,
    loadConfig,
    saveConfig,
    resetToDefault,
    updateUserRole,
  };
});