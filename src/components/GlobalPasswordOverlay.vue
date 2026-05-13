<script setup lang="ts">
import { ref, onMounted } from "vue";
import { NCard, NInput, NButton, NSpin, NSpace } from "naive-ui";
import { invoke } from "@tauri-apps/api/core";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();

const isLoading = ref(true);
const isEncrypted = ref(false);
const isVerifying = ref(false);
const password = ref("");
const errorMessage = ref("");

onMounted(async () => {
  try {
    isEncrypted.value = await invoke<boolean>("get_global_encryption_status");
  } catch (error) {
    console.error("Failed to check encryption status:", error);
    isEncrypted.value = false;
  } finally {
    isLoading.value = false;
  }
});

const handleVerify = async () => {
  if (!password.value) {
    errorMessage.value = t("settings.encryption.passwordRequired");
    return;
  }

  isVerifying.value = true;
  errorMessage.value = "";

  try {
    const verified = await invoke<boolean>("verify_global_password", {
      params: { password: password.value },
    });

    if (verified) {
      isEncrypted.value = false;
    } else {
      errorMessage.value = t("settings.encryption.incorrectPassword");
      password.value = "";
    }
  } catch (error) {
    console.error("Failed to verify password:", error);
    errorMessage.value = String(error);
    password.value = "";
  } finally {
    isVerifying.value = false;
  }
};

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === "Enter") {
    handleVerify();
  }
};
</script>

<template>
  <div v-if="!isLoading && isEncrypted" class="password-overlay">
    <div class="password-modal">
      <n-card
        :title="t('settings.encryption.verifyTitle')"
        style="width: 360px"
      >
        <div class="verify-content">
          <p class="verify-description">
            {{ t("settings.encryption.verifyDescription") }}
          </p>
          <n-input
            v-model:value="password"
            type="password"
            :placeholder="t('settings.encryption.passwordPlaceholder')"
            show-password-on="click"
            :disabled="isVerifying"
            @keydown="handleKeydown"
            autofocus
          />
          <p v-if="errorMessage" class="error-message">
            {{ errorMessage }}
          </p>
        </div>
        <template #footer>
          <n-space justify="end">
            <n-button
              type="primary"
              :loading="isVerifying"
              @click="handleVerify"
            >
              {{ t("settings.encryption.confirm") }}
            </n-button>
          </n-space>
        </template>
      </n-card>
      <div v-if="isVerifying" class="loading-overlay">
        <n-spin size="large" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.password-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.password-modal {
  position: relative;
}

.verify-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.verify-description {
  margin: 0 0 8px 0;
  color: #666;
  font-size: 14px;
}

.error-message {
  margin: 0;
  color: #e53e3e;
  font-size: 13px;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>