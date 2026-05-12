<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useMessage } from "naive-ui";
import {
  NCard,
  NGrid,
  NGi,
  NButton,
  NForm,
  NFormItem,
  NInput,
  NSpace,
  NSpin,
  NIcon,
  NDivider,
  NResult,
  NSelect,
} from "naive-ui";
import { ArrowLeft, Save, Languages } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { useProjectStore } from "../stores/project";
import { useLocale } from "../i18n/composables/useLocale";
import type { AppLocale } from "../i18n/index";

const router = useRouter();
const message = useMessage();
const projectStore = useProjectStore();
const { t, currentLocale, switchLocale, availableLocales } = useLocale();

const isLoading = ref(false);
const isSaving = ref(false);

const selectedLocale = computed({
  get: () => currentLocale.value,
  set: (val: AppLocale) => switchLocale(val),
});

const localeSelectOptions = availableLocales.map((l) => ({
  label: l.label,
  value: l.value,
}));

const autoSaveInterval = ref(1);

const windowSize = ref<string>("default");
const hasProject = ref(false);
const currentProjectId = ref<number | null>(null);

const windowSizeOptions = computed(() => [
  {
    label: t("settings.windowSizeOptions.defaultLandscape"),
    value: "1200x800",
  },
  {
    label: t("settings.windowSizeOptions.compactLandscape"),
    value: "1024x768",
  },
  {
    label: t("settings.windowSizeOptions.standardLandscape"),
    value: "1280x800",
  },
  { label: t("settings.windowSizeOptions.wideLandscape"), value: "1440x900" },
  {
    label: t("settings.windowSizeOptions.fullHDLandscape"),
    value: "1920x1080",
  },
  { label: t("settings.windowSizeOptions.defaultPortrait"), value: "800x1200" },
  { label: t("settings.windowSizeOptions.compactPortrait"), value: "600x900" },
  {
    label: t("settings.windowSizeOptions.standardPortrait"),
    value: "720x1280",
  },
  { label: t("settings.windowSizeOptions.largePortrait"), value: "800x1400" },
  { label: t("settings.windowSizeOptions.fullHDPortrait"), value: "1080x1920" },
]);

const autoSaveIntervalOptions = computed(() => [
  { label: t("settings.autoSaveOptions.30seconds"), value: 0.5 },
  { label: t("settings.autoSaveOptions.1minute"), value: 1 },
  { label: t("settings.autoSaveOptions.2minutes"), value: 2 },
  { label: t("settings.autoSaveOptions.5minutes"), value: 5 },
  { label: t("settings.autoSaveOptions.10minutes"), value: 10 },
]);

const LOCAL_STORAGE_KEY = "inovel_settings";

onMounted(async () => {
  await loadSettings();
  await loadProjectInfo();
});

const loadProjectInfo = async () => {
  try {
    await projectStore.fetchRecentProjects();
    if (projectStore.recentProjects.length > 0) {
      hasProject.value = true;
      const lastProject = projectStore.recentProjects[0];
      currentProjectId.value = lastProject.id;

      const windowSizeResult = await invoke<[number, number] | null>(
        "get_window_size",
        {
          projectId: lastProject.id,
        }
      );
      if (windowSizeResult) {
        const [width, height] = windowSizeResult;
        windowSize.value = `${width}x${height}`;
      } else {
        windowSize.value = "default";
      }
    }
  } catch (error) {
    console.error("Failed to load project info:", error);
  }
};

const loadSettings = async () => {
  isLoading.value = true;
  try {
    const stored = localStorage.getItem(LOCAL_STORAGE_KEY);
    if (stored) {
      const savedSettings = JSON.parse(stored);
      if (savedSettings.autoSaveInterval !== undefined) {
        autoSaveInterval.value = savedSettings.autoSaveInterval;
      }
    }
  } catch (error) {
    console.error("Failed to load settings:", error);
  } finally {
    isLoading.value = false;
  }
};

const saveAllSettings = async () => {
  isSaving.value = true;
  try {
    const settings = {
      autoSaveInterval: autoSaveInterval.value,
    };
    localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(settings));

    if (
      hasProject.value &&
      currentProjectId.value &&
      windowSize.value !== "default"
    ) {
      const [width, height] = windowSize.value.split("x").map(Number);
      await invoke("set_window_size", {
        projectId: currentProjectId.value,
        width,
        height,
      });
    }

    message.success(t("settings.settingsSaved"));
  } catch (error) {
    console.error("Failed to save settings:", error);
    message.warning(t("settings.settingsSaveFailed"));
  } finally {
    isSaving.value = false;
  }
};

const goBack = () => {
  router.push("/");
};
</script>

<template>
  <div
    class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300"
  >
    <header
      class="border-b bg-white dark:bg-gray-800 dark:border-gray-700 transition-colors duration-300"
    >
      <div
        class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center gap-4"
      >
        <n-button quaternary circle @click="goBack">
          <template #icon>
            <NIcon>
              <ArrowLeft />
            </NIcon>
          </template>
        </n-button>
        <h1
          class="text-xl font-bold text-gray-900 dark:text-white whitespace-nowrap"
        >
          {{ t("settings.pageTitle") }}
        </h1>
      </div>
    </header>

    <main class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div v-if="isLoading" class="flex justify-center py-12">
        <n-spin size="large" />
      </div>

      <n-grid v-else :cols="1" :x-gap="16" :y-gap="16" class="settings-grid">
        <!-- Language Settings -->
        <n-gi>
          <n-card hoverable class="settings-card">
            <template #header>
              <div class="flex items-center gap-2">
                <n-icon :size="18"><Languages /></n-icon>
                <span>{{ t("settings.language") }}</span>
              </div>
            </template>
            <n-form label-placement="top" class="settings-form">
              <n-form-item
                :label="t('settings.interfaceLanguage')"
                class="settings-form-item"
              >
                <n-select
                  v-model:value="selectedLocale"
                  :options="localeSelectOptions"
                  class="min-w-[180px] w-56"
                />
              </n-form-item>
            </n-form>
          </n-card>
        </n-gi>

        <!-- Editor Settings -->
        <n-gi>
          <n-card
            :title="t('settings.editorSettings')"
            hoverable
            class="settings-card"
          >
            <n-form label-placement="top" class="settings-form">
              <n-form-item
                :label="t('settings.autoSaveInterval')"
                class="settings-form-item"
              >
                <div class="flex flex-nowrap items-center gap-3">
                  <n-select
                    v-model:value="autoSaveInterval"
                    :options="autoSaveIntervalOptions"
                    class="min-w-[140px] w-48"
                    :teleport="'body'"
                  />
                  <span
                    class="text-gray-500 dark:text-gray-400 whitespace-nowrap"
                    >{{ t("settings.minutes") }}</span
                  >
                </div>
                <template #feedback>
                  <span class="text-sm text-gray-500 dark:text-gray-400">
                    {{ t("settings.autoSaveFeedback") }}
                  </span>
                </template>
              </n-form-item>
            </n-form>

            <template #footer>
              <n-space justify="end" class="w-full">
                <n-button
                  type="primary"
                  @click="saveAllSettings"
                  :loading="isSaving"
                >
                  <template #icon>
                    <NIcon>
                      <Save />
                    </NIcon>
                  </template>
                  {{ t("settings.saveSettings") }}
                </n-button>
              </n-space>
            </template>
          </n-card>
        </n-gi>

        <!-- Window Size Settings -->
        <n-gi v-if="hasProject">
          <n-card
            :title="t('settings.windowSize')"
            hoverable
            class="settings-card"
          >
            <n-form label-placement="top" class="settings-form">
              <n-form-item
                :label="t('settings.windowDimension')"
                class="settings-form-item"
              >
                <div class="flex flex-nowrap items-center gap-3">
                  <n-select
                    v-model:value="windowSize"
                    :options="windowSizeOptions"
                    class="min-w-[180px] w-56"
                    :teleport="'body'"
                  />
                </div>
                <template #feedback>
                  <span class="text-sm text-gray-500 dark:text-gray-400">
                    {{ t("settings.windowSizeFeedback") }}
                  </span>
                </template>
              </n-form-item>
            </n-form>

            <template #footer>
              <n-space justify="end" class="w-full">
                <n-button
                  type="primary"
                  @click="saveAllSettings"
                  :loading="isSaving"
                >
                  <template #icon>
                    <NIcon>
                      <Save />
                    </NIcon>
                  </template>
                  {{ t("settings.saveSettings") }}
                </n-button>
              </n-space>
            </template>
          </n-card>
        </n-gi>

        <!-- Stats Link -->
        <n-gi>
          <n-card
            :title="t('settings.writingStats')"
            hoverable
            @click="router.push('/stats')"
            class="settings-card cursor-pointer"
          >
            <div class="text-center py-4">
              <p class="text-gray-500 dark:text-gray-400 whitespace-normal">
                {{ t("settings.viewStatsDetail") }}
              </p>
            </div>
          </n-card>
        </n-gi>
      </n-grid>
    </main>
  </div>
</template>

<style scoped>
.settings-grid {
  display: grid;
  gap: 1rem;
}

.settings-card {
  width: 100%;
  box-sizing: border-box;
}

.settings-form {
  width: 100%;
}

.settings-form-item {
  width: 100%;
}

@media (max-width: 640px) {
  .settings-grid {
    gap: 0.75rem;
  }

  .settings-card {
    padding: 0.75rem;
  }
}

@media (min-width: 640px) {
  .settings-grid {
    gap: 1rem;
  }
}

@media (min-width: 1024px) {
  .settings-grid {
    gap: 1.5rem;
  }
}
</style>
