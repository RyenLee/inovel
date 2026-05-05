<script setup lang="ts">
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NLoadingBarProvider,
} from "naive-ui";
import { RouterView } from "vue-router";
import { useTheme } from "./composables/useTheme";
import { useGlobalShortcuts } from "./composables/useGlobalShortcuts";

const { isDark, toggleDark, theme } = useTheme();

// Initialize global shortcuts
useGlobalShortcuts();

defineExpose({ toggleDark, isDark });
</script>

<template>
  <div :class="{ dark: isDark }">
    <n-config-provider :theme="theme" :theme-overrides="isDark ? {
      common: {
        primaryColor: '#60a5fa',
        primaryColorHover: '#93c5fd',
        primaryColorPressed: '#3b82f6',
      }
    } : undefined">
      <n-loading-bar-provider>
        <n-dialog-provider>
          <n-message-provider>
            <RouterView />
          </n-message-provider>
        </n-dialog-provider>
      </n-loading-bar-provider>
    </n-config-provider>
  </div>
</template>

<style>
/* Global CSS Variables for theme */
:root {
  --bg-primary: #ffffff;
  --bg-secondary: #f9fafb;
  --bg-tertiary: #f3f4f6;
  --text-primary: #111827;
  --text-secondary: #6b7280;
  --border-color: #e5e7eb;
  --hover-bg: #f3f4f6;
}

.dark {
  --bg-primary: #1f2937;
  --bg-secondary: #111827;
  --bg-tertiary: #374151;
  --text-primary: #f9fafb;
  --text-secondary: #9ca3af;
  --border-color: #4b5563;
  --hover-bg: #374151;
}
</style>
