<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import {
  NModal,
  NCard,
  NButton,
  NInput,
  NSpace,
  NIcon,
  NEmpty,
  NSpin,
  NDataTable,
  NPopconfirm,
  NText,
  useMessage,
} from "naive-ui";
import { Plus, Edit, Trash2, X, Eye, FileText } from "lucide-vue-next";
import { useTemplateStore } from "../stores/template";
import type { UserTemplate } from "../types/template";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();

const props = withDefaults(
  defineProps<{
    show: boolean;
    projectId?: number;
  }>(),
  {
    projectId: 0,
  }
);

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
}>();

const message = useMessage();
const templateStore = useTemplateStore();

// 模态框状态
const showEditModal = ref(false);
const showPreviewModal = ref(false);
const isLoading = ref(false);
const isSaving = ref(false);

// 编辑状态
const editingTemplate = ref<UserTemplate | null>(null);
const isNewTemplate = ref(false);

// 表单数据
const formData = ref({
  name: "",
  description: "",
  category: "customize",
  content: "",
});

// 表格列定义
const columns = [
  {
    title: () => t("userTemplates.table.name"),
    key: "name",
    width: 180,
  },
  {
    title: () => t("userTemplates.table.description"),
    key: "description",
    ellipsis: true,
  },
  {
    title: () => t("userTemplates.table.createdAt"),
    key: "created_at",
    width: 160,
    render: (row: UserTemplate) => {
      return new Date(row.created_at).toLocaleString();
    },
  },
  {
    title: () => t("userTemplates.table.actions"),
    key: "actions",
    width: 160,
    fixed: "right" as const,
    render: (row: UserTemplate) => {
      return h(NSpace, { size: "small" }, {
        default: () => [
          h(NButton, {
            size: "small",
            quaternary: true,
            onClick: () => openPreview(row),
          }, {
            icon: () => h(NIcon, { component: Eye }),
          }),
          h(NButton, {
            size: "small",
            quaternary: true,
            onClick: () => openEdit(row),
          }, {
            icon: () => h(NIcon, { component: Edit }),
          }),
          h(NPopconfirm, {
            onPositiveClick: () => deleteTemplate(row.id),
          }, {
            default: () => t("userTemplates.confirm.deleteMessage", { name: row.name }),
            trigger: () => h(NButton, {
              size: "small",
              quaternary: true,
              type: "error",
            }, {
              icon: () => h(NIcon, { component: Trash2 }),
            }),
          }),
        ],
      });
    },
  },
];

// 加载模板
const loadTemplates = async () => {
  isLoading.value = true;
  try {
    await templateStore.loadUserTemplates(props.projectId);
  } catch (error) {
    console.error(t("userTemplates.messages.loadError") + ":", error);
    message.error(t("userTemplates.messages.loadError"));
  } finally {
    isLoading.value = false;
  }
};

onMounted(() => {
  if (props.show) {
    loadTemplates();
  }
});

// 监听 show 变化
watch(
  () => props.show,
  (newVal) => {
    if (newVal) {
      loadTemplates();
    }
  }
);

// 重置表单
const resetForm = () => {
  formData.value = {
    name: "",
    description: "",
    category: "customize",
    content: "",
  };
  editingTemplate.value = null;
  isNewTemplate.value = false;
};

// 打开新建模板
const openCreate = () => {
  resetForm();
  isNewTemplate.value = true;
  showEditModal.value = true;
};

// 打开编辑模板
const openEdit = (template: UserTemplate) => {
  editingTemplate.value = template;
  isNewTemplate.value = false;
  formData.value = {
    name: template.name,
    description: template.description,
    category: template.category,
    content: template.content,
  };
  showEditModal.value = true;
};

// 打开预览
const openPreview = (template: UserTemplate) => {
  editingTemplate.value = template;
  formData.value = {
    name: template.name,
    description: template.description,
    category: template.category,
    content: template.content,
  };
  showPreviewModal.value = true;
};

// 保存模板
const saveTemplate = async () => {
  // 验证表单
  if (!formData.value.name.trim()) {
    message.error(t("userTemplates.messages.nameRequired"));
    return;
  }
  if (!formData.value.content.trim()) {
    message.error(t("userTemplates.messages.contentRequired"));
    return;
  }

  isSaving.value = true;
  try {
    if (isNewTemplate.value) {
      await templateStore.saveUserTemplate({
        project_id: props.projectId,
        ...formData.value,
      });
      message.success(t("userTemplates.messages.saveSuccess"));
    } else if (editingTemplate.value) {
      await templateStore.updateUserTemplate(editingTemplate.value.id, formData.value);
      message.success(t("userTemplates.messages.updateSuccess"));
    }
    showEditModal.value = false;
    resetForm();
  } catch (error) {
    console.error(t("userTemplates.messages.saveError") + ":", error);
    message.error(isNewTemplate.value ? t("userTemplates.messages.saveError") : t("userTemplates.messages.updateError"));
  } finally {
    isSaving.value = false;
  }
};

// 删除模板
const deleteTemplate = async (id: number) => {
  try {
    await templateStore.deleteUserTemplate(id);
    message.success(t("userTemplates.messages.deleteSuccess"));
  } catch (error) {
    console.error(t("userTemplates.messages.deleteError") + ":", error);
    message.error(t("userTemplates.messages.deleteError"));
  }
};

// 简单 Markdown 转 HTML（用于预览）
const markdownToHtml = (md: string): string => {
  if (!md) return "";
  let html = md;
  html = html.replace(/^### (.*$)/gim, "<h3>$1</h3>");
  html = html.replace(/^## (.*$)/gim, "<h2>$1</h2>");
  html = html.replace(/^# (.*$)/gim, "<h1>$1</h1>");
  html = html.replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/\*(.*?)\*/g, "<em>$1</em>");
  html = html.replace(/^> (.*$)/gim, "<blockquote>$1</blockquote>");
  html = html.replace(/^- (.*$)/gim, "<li>$1</li>");
  html = html.replace(/(<li>.*<\/li>)/gims, "<ul>$1</ul>");
  html = html.replace(/^\d+\. (.*$)/gim, "<li>$1</li>");
  html = html.replace(/\n\n/g, "</p><p>");
  html = "<p>" + html + "</p>";
  html = html.replace(/<p><h/g, "<h");
  html = html.replace(/<\/h[123]><\/p>/g, "");
  html = html.replace(/<p><blockquote/g, "<blockquote");
  html = html.replace(/<\/blockquote><\/p>/g, "</blockquote>");
  html = html.replace(/<p><ul>/g, "<ul>");
  html = html.replace(/<\/ul><\/p>/g, "</ul>");
  return html;
};

import { watch, h } from "vue";
</script>

<template>
  <n-modal
    :show="show"
    :mask-closable="false"
    :close-on-esc="true"
    preset="card"
    style="width: 1000px; max-width: 92vw"
    @update:show="(val) => emit('update:show', val)"
  >
    <template #header>
      <div class="flex items-center gap-2">
        <FileText class="w-5 h-5" />
        <span>{{ t('userTemplates.title') }}</span>
      </div>
    </template>
    <template #header-extra>
      <n-button text @click="() => emit('update:show', false)">
        <template #icon>
          <n-icon :component="X" />
        </template>
      </n-button>
    </template>

    <div class="template-manager-content">
      <!-- 工具栏 -->
      <div class="flex items-center justify-between mb-4">
        <n-button type="primary" @click="openCreate">
          <template #icon>
            <n-icon :component="Plus" />
          </template>
          {{ t('userTemplates.newTemplate') }}
        </n-button>
      </div>

      <!-- 模板列表 -->
      <div class="template-list-container">
        <n-data-table
          :columns="columns"
          :data="templateStore.userTemplates"
          :loading="isLoading"
          :pagination="{
            pageSize: 10,
          }"
          :bordered="false"
          size="small"
        >
          <template #empty>
            <n-empty :description="t('userTemplates.messages.emptyList')" />
          </template>
        </n-data-table>
      </div>
    </div>
  </n-modal>

  <!-- 编辑模态框 -->
  <n-modal
    v-model:show="showEditModal"
    preset="card"
    :title="isNewTemplate ? t('userTemplates.modal.createTitle') : t('userTemplates.modal.editTitle')"
    style="width: 700px; max-width: 92vw"
    :mask-closable="false"
  >
    <n-form :model="formData" label-placement="top">
      <n-form-item :label="t('userTemplates.templateName')">
        <n-input
          v-model:value="formData.name"
          :placeholder="t('userTemplates.placeholder.name')"
          maxlength="100"
          show-count
        />
      </n-form-item>

      <n-form-item :label="t('userTemplates.templateDescription')">
        <n-input
          v-model:value="formData.description"
          :placeholder="t('userTemplates.placeholder.description')"
          maxlength="200"
          show-count
        />
      </n-form-item>

      <n-form-item :label="t('userTemplates.templateContent')">
        <n-input
          v-model:value="formData.content"
          :placeholder="t('userTemplates.placeholder.content')"
          type="textarea"
          :rows="15"
        />
      </n-form-item>
    </n-form>

    <template #footer>
      <n-space justify="end">
        <n-button @click="() => showEditModal = false" :disabled="isSaving">
          {{ t('userTemplates.buttons.cancel') }}
        </n-button>
        <n-button
          type="primary"
          :loading="isSaving"
          @click="saveTemplate"
        >
          {{ t('userTemplates.buttons.save') }}
        </n-button>
      </n-space>
    </template>
  </n-modal>

  <!-- 预览模态框 -->
  <n-modal
    v-model:show="showPreviewModal"
    preset="card"
    :title="t('userTemplates.modal.previewTitle')"
    style="width: 700px; max-width: 92vw"
  >
    <div class="template-preview-container">
      <div class="preview-header">
        <n-text strong style="font-size: 16px">{{ formData.name }}</n-text>
      </div>
      <n-text depth="3" class="preview-description">{{ formData.description }}</n-text>
      <div class="preview-divider"></div>
      <div class="preview-content" v-html="markdownToHtml(formData.content)"></div>
    </div>

    <template #footer>
      <n-space justify="end">
        <n-button @click="() => showPreviewModal = false">
          {{ t('userTemplates.buttons.cancel') }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<style scoped>
.template-manager-content {
  display: flex;
  flex-direction: column;
  min-height: 400px;
}

.template-list-container {
  flex: 1;
  min-height: 0;
}

.template-preview-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.preview-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.preview-description {
  display: block;
  font-size: 14px;
}

.preview-divider {
  height: 1px;
  background: #e5e7eb;
  margin: 4px 0;
}

.preview-content {
  padding: 16px;
  background: #f9fafb;
  border-radius: 8px;
  line-height: 1.6;
}

.preview-content:deep(h1),
.preview-content:deep(h2),
.preview-content:deep(h3) {
  margin: 12px 0 8px;
  font-weight: 600;
  line-height: 1.3;
}

.preview-content:deep(h1) {
  font-size: 20px;
  color: #1f2937;
}

.preview-content:deep(h2) {
  font-size: 18px;
  color: #374151;
}

.preview-content:deep(h3) {
  font-size: 16px;
  color: #4b5563;
}

.preview-content:deep(p) {
  margin: 8px 0;
  color: #374151;
}

.preview-content:deep(strong) {
  font-weight: 600;
  color: #111827;
}

.preview-content:deep(em) {
  font-style: italic;
  color: #7c3aed;
}

.preview-content:deep(blockquote) {
  border-left: 3px solid #6366f1;
  padding: 8px 16px;
  margin: 12px 0;
  color: #6b7280;
  background: rgba(99, 102, 241, 0.05);
  border-radius: 0 4px 4px 0;
}

.preview-content:deep(ul),
.preview-content:deep(ol) {
  margin: 8px 0;
  padding-left: 24px;
  color: #374151;
}

.preview-content:deep(li) {
  margin: 4px 0;
  color: #374151;
}

.preview-content:deep(ul) {
  list-style-type: disc;
}

/* 暗色主题 */
:root.dark .preview-content {
  background: #1f2937;
}

:root.dark .preview-content:deep(h1),
:root.dark .preview-content:deep(h2),
:root.dark .preview-content:deep(h3) {
  color: #f9fafb;
}

:root.dark .preview-content:deep(p) {
  color: #d1d5db;
}

:root.dark .preview-content:deep(strong) {
  color: #f3f4f6;
}

:root.dark .preview-content:deep(em) {
  color: #a78bfa;
}

:root.dark .preview-content:deep(blockquote) {
  color: #9ca3af;
  background: rgba(99, 102, 241, 0.1);
}

:root.dark .preview-content:deep(ul),
:root.dark .preview-content:deep(ol),
:root.dark .preview-content:deep(li) {
  color: #d1d5db;
}

:root.dark .preview-divider {
  background: #374151;
}
</style>
