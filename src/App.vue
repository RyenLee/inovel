<script setup lang="ts">
import { computed } from "vue";
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NLoadingBarProvider,
} from "naive-ui";
import { RouterView } from "vue-router";
import { useTheme } from "./composables/useTheme";
import { useGlobalShortcuts } from "./composables/useGlobalShortcuts";
import { lightThemeConfig, darkThemeConfig } from "./composables/themeConfig";

const { isDark, toggleDark, theme } = useTheme();

// Initialize global shortcuts
useGlobalShortcuts();

// 主题覆盖配置
const themeOverrides = computed(() => isDark.value ? darkThemeConfig : lightThemeConfig);

defineExpose({ toggleDark, isDark });
</script>

<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-loading-bar-provider>
      <n-dialog-provider>
        <n-message-provider>
          <RouterView />
        </n-message-provider>
      </n-dialog-provider>
    </n-loading-bar-provider>
  </n-config-provider>
</template>

<style>
/* 全局样式 - Tailwind v4 使用 @theme */
html, body, #app {
  height: 100%;
  margin: 0;
  padding: 0;
}
</style>
