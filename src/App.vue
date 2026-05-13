<script setup lang="ts">
import { computed, onMounted } from "vue";
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NLoadingBarProvider,
  zhCN,
  dateZhCN,
  enUS,
  dateEnUS,
} from "naive-ui";
import { RouterView } from "vue-router";
import { useTheme } from "./composables/useTheme";
import { useGlobalShortcuts } from "./composables/useGlobalShortcuts";
import { lightThemeConfig, darkThemeConfig } from "./composables/themeConfig";
import { useLocale } from "./i18n/composables/useLocale";
import { useI18n } from "vue-i18n";
import GlobalPasswordOverlay from "./components/GlobalPasswordOverlay.vue";

const { t } = useI18n();
const { isDark, toggleDark, theme } = useTheme();
const { isZhCN } = useLocale();

useGlobalShortcuts();

onMounted(() => {
  document.title = t("common.app.name");
});

const naiveLocale = computed(() => isZhCN.value ? zhCN : enUS);
const naiveDateLocale = computed(() => isZhCN.value ? dateZhCN : dateEnUS);

const themeOverrides = computed(() => isDark.value ? darkThemeConfig : lightThemeConfig);

defineExpose({ toggleDark, isDark });
</script>

<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides" :locale="naiveLocale" :date-locale="naiveDateLocale">
    <n-loading-bar-provider>
      <n-dialog-provider>
        <n-message-provider>
          <GlobalPasswordOverlay />
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
