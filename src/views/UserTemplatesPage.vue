<script setup lang="ts">
import { ref, computed, onMounted, h } from "vue";
import { useRouter } from "vue-router";
import {
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
import {
  Plus,
  Edit,
  Trash2,
  Eye,
  FileText,
  ArrowLeft,
  X,
} from "lucide-vue-next";
import { useTemplateStore } from "../stores/template";
import type { UserTemplate } from "../types/template";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();
const router = useRouter();
const message = useMessage();
const templateStore = useTemplateStore();

const isLoading = ref(false);
const isSaving = ref(false);

type PanelMode = "create" | "edit" | "preview";

const showSlidePanel = ref(false);
const panelMode = ref<PanelMode>("create");
const editingTemplate = ref<UserTemplate | null>(null);

const formData = ref({
  name: "",
  description: "",
  category: "自定义",
  content: "",
});

const panelTitle = computed(() => {
  if (panelMode.value === "create") return t("userTemplates.modal.createTitle");
  if (panelMode.value === "edit") return t("userTemplates.modal.editTitle");
  return t("userTemplates.modal.previewTitle");
});

const panelSubtitle = computed(() => {
  if (panelMode.value === "create")
    return t("userTemplates.modal.createSubtitle");
  if (panelMode.value === "edit") return t("userTemplates.modal.editSubtitle");
  return t("userTemplates.modal.previewSubtitle");
});

const panelIcon = computed(() => {
  if (panelMode.value === "create") return Plus;
  if (panelMode.value === "edit") return Edit;
  return Eye;
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
      return h(
        NSpace,
        { size: "small" },
        {
          default: () => [
            h(
              NButton,
              {
                size: "small",
                quaternary: true,
                onClick: () => openPreview(row),
              },
              {
                icon: () => h(NIcon, { component: Eye }),
              }
            ),
            h(
              NButton,
              {
                size: "small",
                quaternary: true,
                onClick: () => openEdit(row),
              },
              {
                icon: () => h(NIcon, { component: Edit }),
              }
            ),
            h(
              NPopconfirm,
              {
                onPositiveClick: () => deleteTemplate(row.id),
              },
              {
                default: () =>
                  t("userTemplates.confirm.deleteMessage", { name: row.name }),
                trigger: () =>
                  h(
                    NButton,
                    {
                      size: "small",
                      quaternary: true,
                      type: "error",
                    },
                    {
                      icon: () => h(NIcon, { component: Trash2 }),
                    }
                  ),
              }
            ),
          ],
        }
      );
    },
  },
];

// 加载模板
const loadTemplates = async () => {
  isLoading.value = true;
  try {
    await templateStore.loadUserTemplates(0);
  } catch (error) {
    console.error(t("userTemplates.messages.loadError") + ":", error);
    message.error(t("userTemplates.messages.loadError"));
  } finally {
    isLoading.value = false;
  }
};

onMounted(() => {
  loadTemplates();
});

const resetForm = () => {
  formData.value = {
    name: "",
    description: "",
    category: "customize",
    content: "",
  };
  editingTemplate.value = null;
};

const openCreate = () => {
  resetForm();
  panelMode.value = "create";
  showSlidePanel.value = true;
};

const openEdit = (template: UserTemplate) => {
  editingTemplate.value = template;
  panelMode.value = "edit";
  formData.value = {
    name: template.name,
    description: template.description,
    category: template.category,
    content: template.content,
  };
  showSlidePanel.value = true;
};

const openPreview = (template: UserTemplate) => {
  editingTemplate.value = template;
  panelMode.value = "preview";
  formData.value = {
    name: template.name,
    description: template.description,
    category: template.category,
    content: template.content,
  };
  showSlidePanel.value = true;
};

const closePanel = () => {
  showSlidePanel.value = false;
};

const saveTemplate = async () => {
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
    if (panelMode.value === "create") {
      await templateStore.saveUserTemplate({
        project_id: 0,
        ...formData.value,
      });
      message.success(t("userTemplates.messages.saveSuccess"));
    } else if (editingTemplate.value) {
      await templateStore.updateUserTemplate(
        editingTemplate.value.id,
        formData.value
      );
      message.success(t("userTemplates.messages.updateSuccess"));
    }
    showSlidePanel.value = false;
    resetForm();
  } catch (error) {
    console.error(t("userTemplates.messages.saveError") + ":", error);
    message.error(
      panelMode.value === "create"
        ? t("userTemplates.messages.saveError")
        : t("userTemplates.messages.updateError")
    );
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
      <div class="max-w-5xl mx-auto px-4 py-4 flex items-center gap-4">
        <n-button quaternary circle @click="goBack">
          <template #icon>
            <NIcon>
              <ArrowLeft />
            </NIcon>
          </template>
        </n-button>
        <FileText class="w-6 h-6 text-blue-600" />
        <h1 class="text-xl font-bold text-gray-900 dark:text-white">
          {{ t("userTemplates.title") }}
        </h1>
        <div class="flex-1" />
        <n-button type="primary" @click="openCreate">
          <template #icon>
            <NIcon :component="Plus" />
          </template>
          {{ t("userTemplates.newTemplate") }}
        </n-button>
      </div>
    </header>

    <main class="max-w-5xl mx-auto px-4 py-8">
      <div v-if="isLoading" class="flex justify-center py-12">
        <n-spin size="large" />
      </div>

      <n-card v-else :bordered="false" :segmented="{ content: true }">
        <n-data-table
          :columns="columns"
          :data="templateStore.userTemplates"
          :loading="isLoading"
          :pagination="{ pageSize: 10 }"
          :bordered="false"
          size="small"
        >
          <template #empty>
            <NEmpty :description="t('userTemplates.messages.emptyList')" />
          </template>
        </n-data-table>
      </n-card>
    </main>

    <Teleport to="body">
      <Transition name="panel-overlay">
        <div
          v-if="showSlidePanel"
          class="fixed inset-0 z-50 bg-black/40 transition-colors duration-300"
          @click.self="closePanel"
        />
      </Transition>

      <Transition name="panel-slide">
        <div
          v-if="showSlidePanel"
          class="fixed top-0 right-0 z-50 h-full w-[90vw] sm:w-[480px] md:w-[520px] lg:w-[560px] bg-white dark:bg-gray-800 shadow-2xl flex flex-col transition-colors duration-300"
        >
          <div class="shrink-0 border-b border-gray-200 dark:border-gray-700">
            <div class="flex items-center justify-between px-6 py-4">
              <div class="flex items-center gap-3 min-w-0">
                <div
                  class="p-2 rounded-lg shrink-0 transition-colors duration-200"
                  :class="
                    panelMode === 'create'
                      ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-600'
                      : panelMode === 'edit'
                      ? 'bg-amber-100 dark:bg-amber-900/30 text-amber-600'
                      : 'bg-purple-100 dark:bg-purple-900/30 text-purple-600'
                  "
                >
                  <NIcon :component="panelIcon" size="20" />
                </div>
                <div class="min-w-0">
                  <h2
                    class="text-lg font-semibold text-gray-900 dark:text-white truncate"
                  >
                    {{ panelTitle }}
                  </h2>
                  <p class="text-sm text-gray-500 dark:text-gray-400 truncate">
                    {{ panelSubtitle }}
                  </p>
                </div>
              </div>
              <n-button
                quaternary
                circle
                @click="closePanel"
                class="shrink-0 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-200"
              >
                <template #icon>
                  <NIcon :component="X" />
                </template>
              </n-button>
            </div>
          </div>

          <div class="flex-1 overflow-y-auto px-6 py-5 scroll-smooth">
            <template v-if="panelMode === 'preview'">
              <div class="space-y-6 animate-fade-in">
                <div class="space-y-2">
                  <n-text
                    depth="3"
                    class="text-xs font-medium uppercase tracking-wide"
                    >{{ t("userTemplates.templateName") }}</n-text
                  >
                  <n-text strong class="block text-base">{{
                    formData.name
                  }}</n-text>
                </div>

                <div v-if="formData.description" class="space-y-2">
                  <n-text
                    depth="3"
                    class="text-xs font-medium uppercase tracking-wide"
                    >{{ t("userTemplates.templateDescription") }}</n-text
                  >
                  <n-text depth="2" class="block text-sm">{{
                    formData.description
                  }}</n-text>
                </div>

                <div class="border-t border-gray-200 dark:border-gray-700" />

                <div class="space-y-2">
                  <n-text
                    depth="3"
                    class="text-xs font-medium uppercase tracking-wide"
                    >{{ t("userTemplates.templateContent") }}</n-text
                  >
                  <div
                    class="preview-content rounded-lg p-5 bg-gray-50 dark:bg-gray-700/50 leading-relaxed border border-gray-100 dark:border-gray-600"
                    v-html="markdownToHtml(formData.content)"
                  />
                </div>
              </div>
            </template>

            <template v-else>
              <n-form
                :model="formData"
                label-placement="top"
                size="large"
                class="panel-form animate-fade-in"
              >
                <n-form-item
                  :label="t('userTemplates.templateName')"
                  path="name"
                >
                  <n-input
                    v-model:value="formData.name"
                    :placeholder="t('userTemplates.placeholder.name')"
                    maxlength="100"
                    show-count
                  />
                  <template #feedback>
                    <span class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      {{ t("userTemplates.help.name") }}
                    </span>
                  </template>
                </n-form-item>

                <n-form-item
                  :label="t('userTemplates.templateDescription')"
                  path="description"
                >
                  <n-input
                    v-model:value="formData.description"
                    :placeholder="t('userTemplates.placeholder.description')"
                    maxlength="200"
                    show-count
                  />
                  <template #feedback>
                    <span class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      {{ t("userTemplates.help.description") }}
                    </span>
                  </template>
                </n-form-item>

                <n-form-item
                  :label="t('userTemplates.templateContent')"
                  path="content"
                >
                  <n-input
                    v-model:value="formData.content"
                    :placeholder="t('userTemplates.placeholder.content')"
                    type="textarea"
                    :rows="18"
                  />
                  <template #feedback>
                    <span class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      {{ t("userTemplates.help.content") }}
                    </span>
                  </template>
                </n-form-item>
              </n-form>
            </template>
          </div>

          <div
            class="panel-footer shrink-0 border-t border-gray-200 dark:border-gray-700 px-6 py-4 flex items-center justify-end gap-3"
          >
            <n-button size="large" @click="closePanel" :disabled="isSaving">
              {{ t("userTemplates.buttons.cancel") }}
            </n-button>
            <n-button
              v-if="panelMode !== 'preview'"
              type="primary"
              size="large"
              :loading="isSaving"
              @click="saveTemplate"
            >
              {{ t("userTemplates.buttons.save") }}
            </n-button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
/* Panel slide transitions */
.panel-overlay-enter-active,
.panel-overlay-leave-active {
  transition: opacity 0.3s ease;
}
.panel-overlay-enter-from,
.panel-overlay-leave-to {
  opacity: 0;
}
.panel-slide-enter-active,
.panel-slide-leave-active {
  transition: transform 0.3s cubic-bezier(0.22, 1, 0.36, 1);
}
.panel-slide-enter-from,
.panel-slide-leave-to {
  transform: translateX(100%);
}

.panel-form {
  --n-feedback-padding: 8px 0 0 0;
}

.panel-form:deep(.n-form-item) {
  margin-bottom: 24px;
  transition: all 0.2s ease;
}

.panel-form:deep(.n-form-item:last-child) {
  margin-bottom: 0;
}

/* Input hover/focus enhancements */
.panel-form:deep(.n-input) {
  transition: all 0.2s ease;
}

.panel-form:deep(.n-input:hover) {
  box-shadow: 0 0 0 1px rgba(99, 102, 241, 0.1);
}

.panel-form:deep(.n-input:focus-within) {
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
}

/* Button hover effects */
.panel-footer:deep(.n-button) {
  transition: all 0.2s ease;
}

/* Fade-in animation for content */
.animate-fade-in {
  animation: fadeIn 0.3s ease-out forwards;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
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

.panel-overlay-enter-active,
.panel-overlay-leave-active {
  transition: opacity 0.3s ease;
}

.panel-overlay-enter-from,
.panel-overlay-leave-to {
  opacity: 0;
}

.panel-slide-enter-active,
.panel-slide-leave-active {
  transition: transform 0.35s cubic-bezier(0.25, 0.8, 0.25, 1.2);
}

.panel-slide-enter-from,
.panel-slide-leave-to {
  transform: translateX(100%);
}
</style>
