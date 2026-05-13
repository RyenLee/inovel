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
  NSwitch,
  NModal,
} from "naive-ui";
import { ArrowLeft, Save, Languages, Shield } from "lucide-vue-next";
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

const localeSelectOptions = computed(() =>
  availableLocales.value.map((l: { value: string; label: string }) => ({
    label: l.label,
    value: l.value,
  }))
);

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

const globalEncryptionEnabled = ref(false);
const isEncryptionLoading = ref(false);

const showEnableDialog = ref(false);
const showDisableDialog = ref(false);
const showChangePasswordDialog = ref(false);

const enablePassword = ref("");
const enableConfirmPassword = ref("");
const disablePassword = ref("");
const changeOldPassword = ref("");
const changeNewPassword = ref("");
const changeConfirmPassword = ref("");

const isEnabling = ref(false);
const isDisabling = ref(false);
const isChangingPassword = ref(false);

onMounted(async () => {
  await loadSettings();
  await loadProjectInfo();
  await loadEncryptionStatus();
});

const loadEncryptionStatus = async () => {
  isEncryptionLoading.value = true;
  try {
    globalEncryptionEnabled.value = await invoke<boolean>(
      "get_global_encryption_status"
    );
  } catch (error) {
    console.error("Failed to load encryption status:", error);
  } finally {
    isEncryptionLoading.value = false;
  }
};

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
          project_id: lastProject.id,
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
        project_id: currentProjectId.value,
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

const handleEncryptionToggle = (value: boolean) => {
  if (value) {
    showEnableDialog.value = true;
  } else {
    showDisableDialog.value = true;
  }
};

const handleEnableEncryption = async () => {
  if (!enablePassword.value || !enableConfirmPassword.value) {
    message.warning(t("settings.encryption.passwordRequired"));
    return;
  }
  if (enablePassword.value.length < 8) {
    message.warning(t("settings.encryption.passwordTooShort"));
    return;
  }
  if (enablePassword.value !== enableConfirmPassword.value) {
    message.warning(t("settings.encryption.passwordMismatch"));
    return;
  }

  isEnabling.value = true;
  try {
    await invoke("enable_global_encryption", {
      params: {
        password: enablePassword.value,
        confirm_password: enableConfirmPassword.value,
      },
    });
    globalEncryptionEnabled.value = true;
    showEnableDialog.value = false;
    enablePassword.value = "";
    enableConfirmPassword.value = "";
    message.success(t("settings.encryption.enableSuccess"));
  } catch (error) {
    console.error("Failed to enable encryption:", error);
    message.error(String(error));
    globalEncryptionEnabled.value = false;
  } finally {
    isEnabling.value = false;
  }
};

const handleDisableEncryption = async () => {
  if (!disablePassword.value) {
    message.warning(t("settings.encryption.passwordRequired"));
    return;
  }

  isDisabling.value = true;
  try {
    await invoke("disable_global_encryption", {
      params: {
        password: disablePassword.value,
      },
    });
    globalEncryptionEnabled.value = false;
    showDisableDialog.value = false;
    disablePassword.value = "";
    message.success(t("settings.encryption.disableSuccess"));
  } catch (error) {
    console.error("Failed to disable encryption:", error);
    message.error(String(error));
  } finally {
    isDisabling.value = false;
  }
};

const handleChangePassword = async () => {
  if (!changeOldPassword.value || !changeNewPassword.value) {
    message.warning(t("settings.encryption.passwordRequired"));
    return;
  }
  if (changeNewPassword.value.length < 8) {
    message.warning(t("settings.encryption.passwordTooShort"));
    return;
  }
  if (changeNewPassword.value !== changeConfirmPassword.value) {
    message.warning(t("settings.encryption.passwordMismatch"));
    return;
  }

  isChangingPassword.value = true;
  try {
    await invoke("change_global_password", {
      params: {
        old_password: changeOldPassword.value,
        new_password: changeNewPassword.value,
        confirm_password: changeConfirmPassword.value,
      },
    });
    showChangePasswordDialog.value = false;
    changeOldPassword.value = "";
    changeNewPassword.value = "";
    changeConfirmPassword.value = "";
    message.success(t("settings.encryption.changePasswordSuccess"));
  } catch (error) {
    console.error("Failed to change password:", error);
    message.error(String(error));
  } finally {
    isChangingPassword.value = false;
  }
};

const cancelEnableDialog = () => {
  showEnableDialog.value = false;
  enablePassword.value = "";
  enableConfirmPassword.value = "";
  globalEncryptionEnabled.value = false;
};

const cancelDisableDialog = () => {
  showDisableDialog.value = false;
  disablePassword.value = "";
  globalEncryptionEnabled.value = true;
};

const cancelChangePasswordDialog = () => {
  showChangePasswordDialog.value = false;
  changeOldPassword.value = "";
  changeNewPassword.value = "";
  changeConfirmPassword.value = "";
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

        <!-- Global Encryption Settings -->
        <n-gi>
          <n-card hoverable class="settings-card">
            <template #header>
              <div class="flex items-center gap-2">
                <n-icon :size="18"><Shield /></n-icon>
                <span>{{ t("settings.encryption.title") }}</span>
              </div>
            </template>
            <div v-if="isEncryptionLoading" class="flex justify-center py-4">
              <n-spin size="small" />
            </div>
            <div v-else class="encryption-content">
              <div class="encryption-status">
                <div
                  class="status-indicator"
                  :class="{ active: globalEncryptionEnabled }"
                >
                  <div class="status-dot"></div>
                  <span class="status-text">
                    {{
                      globalEncryptionEnabled
                        ? t("settings.encryption.enabled")
                        : t("settings.encryption.disabled")
                    }}
                  </span>
                </div>
                <p class="encryption-description">
                  {{ t("settings.encryption.description") }}
                </p>
              </div>
              <div class="encryption-actions">
                <n-switch
                  :value="globalEncryptionEnabled"
                  @update:value="handleEncryptionToggle"
                  :loading="isEncryptionLoading"
                />
              </div>
              <div v-if="globalEncryptionEnabled" class="encryption-extra">
                <n-divider />
                <div class="password-change-section">
                  <span class="password-hint">{{
                    t("settings.encryption.changePassword")
                  }}</span>
                  <n-button
                    type="warning"
                    ghost
                    size="small"
                    @click="showChangePasswordDialog = true"
                  >
                    {{ t("settings.encryption.changePasswordButton") }}
                  </n-button>
                </div>
              </div>
            </div>
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

    <!-- Enable Encryption Dialog -->
    <n-modal
      v-model:show="showEnableDialog"
      preset="card"
      :title="t('settings.encryption.enableTitle')"
      style="max-width: 420px"
    >
      <n-form label-placement="top">
        <n-form-item :label="t('settings.encryption.newPassword')">
          <n-input
            v-model:value="enablePassword"
            type="password"
            :placeholder="t('settings.encryption.passwordPlaceholder')"
            show-password-on="click"
          />
        </n-form-item>
        <n-form-item :label="t('settings.encryption.confirmPassword')">
          <n-input
            v-model:value="enableConfirmPassword"
            type="password"
            :placeholder="t('settings.encryption.confirmPasswordPlaceholder')"
            show-password-on="click"
          />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="cancelEnableDialog">{{
            t("settings.encryption.cancel")
          }}</n-button>
          <n-button
            type="primary"
            @click="handleEnableEncryption"
            :loading="isEnabling"
          >
            {{ t("settings.encryption.confirm") }}
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- Disable Encryption Dialog -->
    <n-modal
      v-model:show="showDisableDialog"
      preset="card"
      :title="t('settings.encryption.disableTitle')"
      style="max-width: 420px"
    >
      <n-form label-placement="top">
        <n-form-item :label="t('settings.encryption.currentPassword')">
          <n-input
            v-model:value="disablePassword"
            type="password"
            :placeholder="t('settings.encryption.currentPasswordPlaceholder')"
            show-password-on="click"
          />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="cancelDisableDialog">{{
            t("settings.encryption.cancel")
          }}</n-button>
          <n-button
            type="primary"
            @click="handleDisableEncryption"
            :loading="isDisabling"
          >
            {{ t("settings.encryption.confirm") }}
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- Change Password Dialog -->
    <n-modal
      v-model:show="showChangePasswordDialog"
      preset="card"
      :title="t('settings.encryption.changePasswordTitle')"
      style="max-width: 420px"
    >
      <n-form label-placement="top">
        <n-form-item :label="t('settings.encryption.oldPassword')">
          <n-input
            v-model:value="changeOldPassword"
            type="password"
            :placeholder="t('settings.encryption.oldPasswordPlaceholder')"
            show-password-on="click"
          />
        </n-form-item>
        <n-form-item :label="t('settings.encryption.newPassword')">
          <n-input
            v-model:value="changeNewPassword"
            type="password"
            :placeholder="t('settings.encryption.passwordPlaceholder')"
            show-password-on="click"
          />
        </n-form-item>
        <n-form-item :label="t('settings.encryption.confirmPassword')">
          <n-input
            v-model:value="changeConfirmPassword"
            type="password"
            :placeholder="t('settings.encryption.confirmPasswordPlaceholder')"
            show-password-on="click"
          />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="cancelChangePasswordDialog">{{
            t("settings.encryption.cancel")
          }}</n-button>
          <n-button
            type="primary"
            @click="handleChangePassword"
            :loading="isChangingPassword"
          >
            {{ t("settings.encryption.confirm") }}
          </n-button>
        </n-space>
      </template>
    </n-modal>
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

.encryption-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.encryption-status {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #d9d9d9;
  transition: background 0.3s;
}

.status-indicator.active .status-dot {
  background: #18a058;
  box-shadow: 0 0 8px rgba(24, 160, 88, 0.5);
}

.status-text {
  font-size: 14px;
  font-weight: 500;
  color: #666;
}

.status-indicator.active .status-text {
  color: #18a058;
}

.encryption-description {
  margin: 0;
  font-size: 13px;
  color: #999;
  line-height: 1.5;
}

.encryption-actions {
  display: flex;
  justify-content: flex-start;
}

.encryption-extra {
  padding-top: 8px;
}

.password-change-section {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.password-hint {
  font-size: 13px;
  color: #666;
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