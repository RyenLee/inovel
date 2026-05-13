<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useRoute, useRouter } from "vue-router";
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
  NInputNumber,
  NTabs,
  NTabPane,
  NImage,
  NTag,
  NProgress,
} from "naive-ui";
import {
  ArrowLeft,
  Target,
  Save,
  FolderOpen,
  Keyboard,
  ImageIcon,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { useFolderDialog } from "../composables/useFolderDialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useProjectStore } from "../stores/project";
import { useLocale } from "../i18n/composables/useLocale";
import { defineAsyncComponent } from "vue";
const ShortcutSettings = defineAsyncComponent(
  () => import("../components/ShortcutSettings.vue")
);
import type {
  EncryptProjectParams,
  DecryptProjectParams,
  ChangePasswordParams,
  DisableEncryptionParams,
  EncryptionProgress,
} from "../types/encryption";

const route = useRoute();
const router = useRouter();
const message = useMessage();
const projectStore = useProjectStore();
const { selectFile } = useFolderDialog();
const { t } = useLocale();

const projectId = computed(() => Number(route.params.projectId));
const isLoading = ref(false);
const isSaving = ref(false);
const activeTab = ref("basic");

// Project settings state
const projectData = ref({
  name: "",
  author: "",
  description: "",
  path: "",
});
const dailyGoal = ref(3000);
const coverUrl = ref<string | null>(null);
const isChangingCover = ref(false);

// 加密相关状态
const encryptPassword = ref("");
const encryptConfirmPassword = ref("");
const oldPassword = ref("");
const newPassword = ref("");
const confirmNewPassword = ref("");
const decryptPassword = ref("");
const isProcessing = ref(false);
const encryptProgress = ref<EncryptionProgress | null>(null);
const isEncrypted = ref(false);

// Get project info
const projectName = computed(() => {
  return projectStore.currentProject?.name || t("projectSettings.project");
});

onMounted(async () => {
  if (
    !projectStore.currentProject ||
    projectStore.currentProject.id !== projectId.value
  ) {
    await projectStore.openProject(projectId.value);
  }
  await loadSettings();

  // 检查项目是否已加密
  if (projectStore.currentProject) {
    isEncrypted.value = await projectStore.isProjectEncrypted(
      projectStore.currentProject.path
    );
  }

  // 监听加密进度
  const app = getCurrentWindow();
  app.listen("encryption-progress", (event: any) => {
    encryptProgress.value = event.payload;
  });

  // 监听解密进度
  app.listen("decryption-progress", (event: any) => {
    encryptProgress.value = event.payload;
  });
});

const loadSettings = async () => {
  isLoading.value = true;
  try {
    // Load project info
    if (projectStore.currentProject) {
      projectData.value = {
        name: projectStore.currentProject.name,
        author: projectStore.currentProject.author,
        description: projectStore.currentProject.description,
        path: projectStore.currentProject.path,
      };
    }

    // Load writing goal
    const goal = await invoke<{ daily_goal: number } | null>(
      "get_writing_goal",
      {
        project_id: projectId.value,
      }
    );
    if (goal) {
      dailyGoal.value = goal.daily_goal;
    }

    // Load cover
    if (projectStore.currentProject?.cover_path) {
      coverUrl.value = convertFileSrc(projectStore.currentProject.cover_path);
    }
  } catch (error) {
    console.error("Failed to load settings:", error);
  } finally {
    isLoading.value = false;
  }
};

// Change cover image
const changeCover = async () => {
  const { path, error } = await selectFile({
    title: t("projectSettings.cover.selectTitle"),
    filters: [
      {
        name: "Images",
        extensions: ["jpg", "jpeg", "png"],
      },
    ],
  });

  if (error) {
    message.error(error);
    return;
  }
  if (!path) return;

  isChangingCover.value = true;
  try {
    const newCoverPath = await invoke<string>("set_cover", {
      id: projectId.value,
      image_path: path,
    });

    coverUrl.value = convertFileSrc(newCoverPath);
    message.success(t("projectSettings.cover.updated"));

    if (projectStore.currentProject) {
      projectStore.currentProject.cover_path = newCoverPath;
    }
  } catch (error) {
    console.error("Failed to change cover:", error);
    message.error(t("projectSettings.cover.changeFailed", { error }));
  } finally {
    isChangingCover.value = false;
  }
};

const saveProjectInfo = async () => {
  if (!projectData.value.name.trim()) {
    message.warning(t("projectSettings.projectInfo.nameRequired"));
    return;
  }

  isSaving.value = true;
  try {
    const project = await projectStore.updateProject(projectId.value, {
      name: projectData.value.name.trim(),
      author: projectData.value.author.trim(),
      description: projectData.value.description.trim(),
    });

    if (project) {
      message.success(t("projectSettings.projectInfo.saved"));
    } else {
      message.error(
        projectStore.error || t("projectSettings.writingGoal.saveFailed")
      );
    }
  } catch (error) {
    message.error(t("projectSettings.projectInfo.saveFailed", { error }));
  } finally {
    isSaving.value = false;
  }
};

const saveDailyGoal = async () => {
  isSaving.value = true;
  try {
    await invoke("save_writing_goal", {
      project_id: projectId.value,
      daily_goal: dailyGoal.value,
    });
    message.success(t("projectSettings.writingGoal.saved"));
  } catch (error) {
    console.error("Failed to save daily goal:", error);
    message.error(t("projectSettings.writingGoal.saveFailed"));
  } finally {
    isSaving.value = false;
  }
};

const goBack = () => {
  router.push(`/editor/${projectId.value}`);
};

// 加密相关方法
const handleEncrypt = async () => {
  if (encryptPassword.value !== encryptConfirmPassword.value) {
    message.error(t("projectSettings.encryption.enable.passwordMismatch"));
    return;
  }
  if (encryptPassword.value.length < 8) {
    message.error(t("projectSettings.encryption.enable.passwordTooShort"));
    return;
  }

  isProcessing.value = true;
  try {
    const params = {
      project_path: projectStore.currentProject?.path || "",
      password: encryptPassword.value,
      confirm_password: encryptConfirmPassword.value,
    };
    await invoke("encrypt_project", { params });
    isEncrypted.value = true;
    encryptPassword.value = "";
    encryptConfirmPassword.value = "";
    message.success(t("projectSettings.encryption.enable.success"));
  } catch (e) {
    message.error(t("projectSettings.encryption.enable.failed", { error: e }));
  } finally {
    isProcessing.value = false;
    encryptProgress.value = null;
  }
};

const handleChangePassword = async () => {
  if (newPassword.value !== confirmNewPassword.value) {
    message.error(t("projectSettings.encryption.change.passwordMismatch"));
    return;
  }
  if (newPassword.value.length < 8) {
    message.error(t("projectSettings.encryption.change.passwordTooShort"));
    return;
  }

  isProcessing.value = true;
  try {
    const params = {
      project_path: projectStore.currentProject?.path || "",
      old_password: oldPassword.value,
      new_password: newPassword.value,
      confirm_password: confirmNewPassword.value,
    };
    await invoke("change_project_password", { params });
    oldPassword.value = "";
    newPassword.value = "";
    confirmNewPassword.value = "";
    message.success(t("projectSettings.encryption.change.success"));
  } catch (e) {
    message.error(t("projectSettings.encryption.change.failed", { error: e }));
  } finally {
    isProcessing.value = false;
  }
};

const handleDecrypt = async () => {
  isProcessing.value = true;
  try {
    const params: DisableEncryptionParams = {
      project_path: projectStore.currentProject?.path || "",
      password: decryptPassword.value,
    };
    await invoke("disable_encryption", { params });
    isEncrypted.value = false;
    decryptPassword.value = "";
    message.success(t("projectSettings.encryption.disable.success"));
  } catch (e) {
    message.error(t("projectSettings.encryption.disable.failed", { error: e }));
  } finally {
    isProcessing.value = false;
    encryptProgress.value = null;
  }
};
</script>

<template>
  <div
    class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300"
  >
    <header
      class="border-b bg-white dark:bg-gray-800 dark:border-gray-700 transition-colors duration-300"
    >
      <div class="max-w-3xl mx-auto px-4 py-4 flex items-center gap-4">
        <n-button quaternary circle @click="goBack">
          <template #icon>
            <NIcon>
              <ArrowLeft />
            </NIcon>
          </template>
        </n-button>
        <Target class="w-6 h-6 text-blue-600" />
        <h1 class="text-xl font-bold text-gray-900 dark:text-white">
          {{ t("projectSettings.title") }}
        </h1>
        <span class="text-sm text-gray-500 dark:text-gray-400">{{
          projectName
        }}</span>
      </div>
    </header>

    <main class="max-w-3xl mx-auto px-4 py-6">
      <!-- Tabs -->
      <n-tabs v-model:value="activeTab" type="line" class="mb-6">
        <n-tab-pane name="basic" :tab="t('projectSettings.tabs.basic')">
          <div v-if="isLoading" class="flex justify-center py-12">
            <n-spin size="large" />
          </div>
          <n-grid v-else :cols="1" :x-gap="16" :y-gap="16">
            <!-- Cover Image -->
            <n-gi>
              <n-card :title="t('projectSettings.cover.title')" hoverable>
                <div class="flex items-center gap-6">
                  <div
                    class="w-32 h-44 rounded-lg overflow-hidden bg-gray-100 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 flex items-center justify-center"
                  >
                    <img
                      v-if="coverUrl"
                      :src="coverUrl"
                      :alt="t('projectSettings.cover.alt')"
                      class="w-full h-full object-cover"
                    />
                    <n-icon size="48" class="text-gray-400" v-else>
                      <ImageIcon />
                    </n-icon>
                  </div>
                  <div class="flex flex-col gap-2">
                    <n-button
                      type="primary"
                      @click="changeCover"
                      :loading="isChangingCover"
                    >
                      {{ t("projectSettings.cover.changeButton") }}
                    </n-button>
                    <span class="text-sm text-gray-500 dark:text-gray-400">
                      {{ t("projectSettings.cover.formatHint") }}
                    </span>
                  </div>
                </div>
              </n-card>
            </n-gi>

            <!-- Project Info -->
            <n-gi>
              <n-card :title="t('projectSettings.projectInfo.title')" hoverable>
                <n-form label-placement="top">
                  <n-form-item
                    :label="t('projectSettings.projectInfo.projectId')"
                  >
                    <n-input
                      :value="projectStore.currentProject?.project_id || ''"
                      readonly
                      :placeholder="
                        t('projectSettings.projectInfo.projectIdPlaceholder')
                      "
                    >
                      <template #suffix>
                        <span class="text-xs text-gray-400">{{
                          t("projectSettings.projectInfo.projectIdReadonly")
                        }}</span>
                      </template>
                    </n-input>
                  </n-form-item>
                  <n-form-item :label="t('projectSettings.projectInfo.name')">
                    <n-input
                      v-model:value="projectData.name"
                      :placeholder="
                        t('projectSettings.projectInfo.namePlaceholder')
                      "
                      maxlength="100"
                      show-count
                    />
                  </n-form-item>
                  <n-form-item :label="t('projectSettings.projectInfo.author')">
                    <n-input
                      v-model:value="projectData.author"
                      :placeholder="
                        t('projectSettings.projectInfo.authorPlaceholder')
                      "
                      maxlength="50"
                    />
                  </n-form-item>
                  <n-form-item
                    :label="t('projectSettings.projectInfo.description')"
                  >
                    <n-input
                      v-model:value="projectData.description"
                      type="textarea"
                      :placeholder="
                        t('projectSettings.projectInfo.descriptionPlaceholder')
                      "
                      :rows="3"
                      maxlength="500"
                      show-count
                    />
                  </n-form-item>
                  <n-form-item :label="t('projectSettings.projectInfo.path')">
                    <n-input v-model:value="projectData.path" readonly />
                  </n-form-item>
                </n-form>
                <template #footer>
                  <n-space justify="end">
                    <n-button
                      type="primary"
                      @click="saveProjectInfo"
                      :loading="isSaving"
                    >
                      {{ t("projectSettings.projectInfo.saveButton") }}
                    </n-button>
                  </n-space>
                </template>
              </n-card>
            </n-gi>

            <!-- Writing Goals -->
            <n-gi>
              <n-card :title="t('projectSettings.writingGoal.title')" hoverable>
                <n-form label-placement="top">
                  <n-form-item
                    :label="t('projectSettings.writingGoal.dailyGoal')"
                  >
                    <div class="flex items-center gap-4">
                      <n-input-number
                        v-model:value="dailyGoal"
                        :min="0"
                        :max="100000"
                        :step="100"
                        class="w-48"
                      />
                      <span class="text-gray-500 dark:text-gray-400">{{
                        t("projectSettings.writingGoal.unit")
                      }}</span>
                    </div>
                    <template #feedback>
                      <span class="text-sm text-gray-500 dark:text-gray-400">
                        {{ t("projectSettings.writingGoal.feedback") }}
                      </span>
                    </template>
                  </n-form-item>
                </n-form>
                <template #footer>
                  <n-space justify="end">
                    <n-button
                      type="primary"
                      @click="saveDailyGoal"
                      :loading="isSaving"
                    >
                      <template #icon>
                        <NIcon><Save /></NIcon>
                      </template>
                      {{ t("projectSettings.writingGoal.saveButton") }}
                    </n-button>
                  </n-space>
                </template>
              </n-card>
            </n-gi>
          </n-grid>
        </n-tab-pane>

        <n-tab-pane name="security" :tab="t('projectSettings.tabs.security')">
          <div v-if="isLoading" class="flex justify-center py-12">
            <n-spin size="large" />
          </div>
          <n-grid v-else :cols="1" :x-gap="16" :y-gap="16">
            <!-- 加密状态显示 -->
            <n-gi>
              <n-card
                :title="t('projectSettings.encryption.status.title')"
                hoverable
              >
                <div class="flex items-center gap-4">
                  <n-tag :type="isEncrypted ? 'success' : 'default'">
                    {{
                      isEncrypted
                        ? t("projectSettings.encryption.status.encrypted")
                        : t("projectSettings.encryption.status.notEncrypted")
                    }}
                  </n-tag>
                  <span class="text-sm text-gray-500 dark:text-gray-400">
                    {{
                      isEncrypted
                        ? t("projectSettings.encryption.status.encryptedDesc")
                        : t(
                            "projectSettings.encryption.status.notEncryptedDesc"
                          )
                    }}
                  </span>
                </div>
              </n-card>
            </n-gi>

            <!-- 设置密码/启用加密 -->
            <n-gi v-if="!isEncrypted">
              <n-card
                :title="t('projectSettings.encryption.enable.title')"
                hoverable
              >
                <n-form label-placement="top">
                  <n-form-item
                    :label="t('projectSettings.encryption.enable.password')"
                  >
                    <n-input
                      v-model:value="encryptPassword"
                      type="password"
                      :placeholder="
                        t(
                          'projectSettings.encryption.enable.passwordPlaceholder'
                        )
                      "
                      show-password-on="mousedown"
                    />
                  </n-form-item>
                  <n-form-item
                    :label="
                      t('projectSettings.encryption.enable.confirmPassword')
                    "
                  >
                    <n-input
                      v-model:value="encryptConfirmPassword"
                      type="password"
                      :placeholder="
                        t(
                          'projectSettings.encryption.enable.confirmPasswordPlaceholder'
                        )
                      "
                      show-password-on="mousedown"
                    />
                  </n-form-item>
                  <n-form-item>
                    <n-button
                      type="primary"
                      @click="handleEncrypt"
                      :loading="isProcessing"
                      :disabled="!encryptPassword || !encryptConfirmPassword"
                    >
                      {{ t("projectSettings.encryption.enable.button") }}
                    </n-button>
                  </n-form-item>
                </n-form>
                <n-progress
                  v-if="encryptProgress"
                  type="line"
                  :percentage="
                    Math.round(
                      (encryptProgress.current / encryptProgress.total) * 100
                    )
                  "
                  :indicator-placement="'inside'"
                  :processing="isProcessing"
                />
                <div v-if="encryptProgress" class="text-sm text-gray-500 mt-2">
                  {{
                    t("projectSettings.encryption.enable.progress", {
                      current: encryptProgress.current,
                      total: encryptProgress.total,
                      file: encryptProgress.currentFile,
                    })
                  }}
                </div>
              </n-card>
            </n-gi>

            <!-- 修改密码 -->
            <n-gi v-if="isEncrypted">
              <n-card
                :title="t('projectSettings.encryption.change.title')"
                hoverable
              >
                <n-form label-placement="top">
                  <n-form-item
                    :label="t('projectSettings.encryption.change.oldPassword')"
                  >
                    <n-input
                      v-model:value="oldPassword"
                      type="password"
                      :placeholder="
                        t(
                          'projectSettings.encryption.change.oldPasswordPlaceholder'
                        )
                      "
                      show-password-on="mousedown"
                    />
                  </n-form-item>
                  <n-form-item
                    :label="t('projectSettings.encryption.change.newPassword')"
                  >
                    <n-input
                      v-model:value="newPassword"
                      type="password"
                      :placeholder="
                        t(
                          'projectSettings.encryption.change.newPasswordPlaceholder'
                        )
                      "
                      show-password-on="mousedown"
                    />
                  </n-form-item>
                  <n-form-item
                    :label="
                      t('projectSettings.encryption.change.confirmNewPassword')
                    "
                  >
                    <n-input
                      v-model:value="confirmNewPassword"
                      type="password"
                      :placeholder="
                        t(
                          'projectSettings.encryption.change.confirmNewPasswordPlaceholder'
                        )
                      "
                      show-password-on="mousedown"
                    />
                  </n-form-item>
                  <n-form-item>
                    <n-button
                      type="warning"
                      @click="handleChangePassword"
                      :loading="isProcessing"
                      :disabled="
                        !oldPassword || !newPassword || !confirmNewPassword
                      "
                    >
                      {{ t("projectSettings.encryption.change.button") }}
                    </n-button>
                  </n-form-item>
                </n-form>
              </n-card>
            </n-gi>

            <!-- 关闭加密 -->
            <n-gi v-if="isEncrypted">
              <n-card
                :title="t('projectSettings.encryption.disable.title')"
                hoverable
              >
                <p class="text-sm text-gray-500 dark:text-gray-400 mb-4">
                  {{ t("projectSettings.encryption.disable.description") }}
                </p>
                <n-form label-placement="top">
                  <n-form-item
                    :label="t('projectSettings.encryption.disable.password')"
                  >
                    <n-input
                      v-model:value="decryptPassword"
                      type="password"
                      :placeholder="
                        t(
                          'projectSettings.encryption.disable.passwordPlaceholder'
                        )
                      "
                      show-password-on="mousedown"
                    />
                  </n-form-item>
                  <n-form-item>
                    <n-button
                      type="error"
                      @click="handleDecrypt"
                      :loading="isProcessing"
                      :disabled="!decryptPassword"
                    >
                      {{ t("projectSettings.encryption.disable.button") }}
                    </n-button>
                  </n-form-item>
                </n-form>
                <n-progress
                  v-if="encryptProgress"
                  type="line"
                  :percentage="
                    Math.round(
                      (encryptProgress.current / encryptProgress.total) * 100
                    )
                  "
                  :indicator-placement="'inside'"
                  :processing="isProcessing"
                />
                <div v-if="encryptProgress" class="text-sm text-gray-500 mt-2">
                  {{
                    t("projectSettings.encryption.disable.progress", {
                      current: encryptProgress.current,
                      total: encryptProgress.total,
                      file: encryptProgress.currentFile,
                    })
                  }}
                </div>
              </n-card>
            </n-gi>
          </n-grid>
        </n-tab-pane>
      </n-tabs>
    </main>
  </div>
</template>
