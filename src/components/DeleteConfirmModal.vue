<script setup lang="ts">
import {
  NModal,
  NCard,
  NButton,
  NSwitch,
  NSpace,
  NText,
  NIcon,
  NInput,
} from 'naive-ui';
import { computed, ref, watch } from 'vue';
import { AlertTriangle } from 'lucide-vue-next';

const props = withDefaults(
  defineProps<{
    show: boolean;
    title?: string;
    message?: string;
    confirmText?: string;
    cancelText?: string;
    danger?: boolean;
    // 项目删除专用
    showKeepFiles?: boolean;
    defaultKeepFiles?: boolean;
    // 确认输入
    requireConfirmInput?: boolean;
    confirmInputPattern?: string;
    confirmInputPlaceholder?: string;
  }>(),
  {
    title: '确认删除',
    message: '确定要删除吗？此操作不可撤销。',
    confirmText: '删除',
    cancelText: '取消',
    danger: true,
    showKeepFiles: false,
    defaultKeepFiles: true,
    requireConfirmInput: false,
    confirmInputPattern: '',
    confirmInputPlaceholder: '',
  }
);

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void;
  (e: 'confirm', keepFiles: boolean): void;
  (e: 'cancel'): void;
}>();

// 本地状态
const keepFiles = ref(props.defaultKeepFiles);
const confirmInput = ref('');

// 监听 show 属性变化，重置状态
watch(
  () => props.show,
  (newVal) => {
    if (newVal) {
      keepFiles.value = props.defaultKeepFiles;
      confirmInput.value = '';
    }
  }
);

// 确认按钮是否可用
const canConfirm = computed(() => {
  if (props.requireConfirmInput) {
    if (props.confirmInputPattern) {
      const regex = new RegExp(props.confirmInputPattern);
      return regex.test(confirmInput.value);
    }
    return confirmInput.value.trim() !== '';
  }
  return true;
});

const handleConfirm = () => {
  emit('confirm', keepFiles.value);
};

const handleCancel = () => {
  emit('cancel');
  emit('update:show', false);
};

const handleClose = () => {
  emit('update:show', false);
};
</script>

<template>
  <n-modal
    :show="show"
    :mask-closable="false"
    :close-on-esc="false"
    @update:show="(val) => emit('update:show', val)"
  >
    <n-card
      :title="title"
      style="width: 420px; max-width: 90vw"
      :bordered="false"
      size="large"
      role="dialog"
      aria-modal="true"
    >
      <template #header-extra>
        <n-icon size="24" :color="danger ? '#ef4444' : '#f59e0b'">
          <AlertTriangle />
        </n-icon>
      </template>

      <div class="delete-confirm-content">
        <!-- 警告图标和消息 -->
        <div class="message-row">
          <n-icon size="20" class="warning-icon" :color="danger ? '#ef4444' : '#f59e0b'">
            <AlertTriangle />
          </n-icon>
          <n-text depth="2" style="flex: 1">{{ message }}</n-text>
        </div>

        <!-- 确认输入框 -->
        <div v-if="requireConfirmInput" class="confirm-input-section">
          <n-text depth="1" style="font-size: 13px; margin-bottom: 8px; display: block">
            请输入以下内容以确认：
            <n-text v-if="confirmInputPattern" type="warning"> {{ confirmInputPattern }} </n-text>
          </n-text>
          <n-input
            v-model:value="confirmInput"
            :placeholder="confirmInputPlaceholder || confirmInputPattern"
            clearable
            autofocus
          />
        </div>

        <!-- 保留文件开关（项目删除专用） -->
        <div v-if="showKeepFiles" class="keep-files-section">
          <div class="keep-files-row">
            <div class="keep-files-info">
              <n-text strong>保留本地文件</n-text>
              <n-text depth="3" style="font-size: 12px; display: block; margin-top: 2px">
                {{ keepFiles ? '仅从列表移除，文件夹保留' : '同时删除文件夹和数据' }}
              </n-text>
            </div>
            <n-switch v-model:value="keepFiles" />
          </div>
        </div>
      </div>

      <template #footer>
        <n-space justify="end">
          <n-button @click="handleCancel" :disabled="false">
            {{ cancelText }}
          </n-button>
          <n-button
            type="error"
            @click="handleConfirm"
            :disabled="!canConfirm"
            :loading="false"
          >
            {{ confirmText }}
          </n-button>
        </n-space>
      </template>
    </n-card>
  </n-modal>
</template>

<style scoped>
.delete-confirm-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.message-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}

.warning-icon {
  flex-shrink: 0;
  margin-top: 2px;
}

.confirm-input-section {
  margin-top: 8px;
}

.keep-files-section {
  margin-top: 8px;
  padding: 12px;
  background: var(--n-color-hover);
  border-radius: 8px;
}

.keep-files-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.keep-files-info {
  flex: 1;
}
</style>
