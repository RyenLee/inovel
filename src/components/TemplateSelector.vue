<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import {
  NModal,
  NCard,
  NButton,
  NTabs,
  NTabPane,
  NGrid,
  NGridItem,
  NSpace,
  NText,
  NIcon,
  NEmpty,
  NRadioGroup,
  NRadioButton,
  NTooltip,
  NSpin,
  useMessage,
} from "naive-ui";
import { FileText, X } from "lucide-vue-next";
import { useTemplateStore } from "../stores/template";
import type { WritingTemplate } from "../types/template";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();

const props = withDefaults(
  defineProps<{
    show: boolean;
    projectId?: number;
    insertMode?: "replace" | "insert"; // 来自父组件的模式设置
  }>(),
  {
    projectId: 0,
    insertMode: "replace",
  }
);

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
  (e: "select", payload: { content: string; mode: "replace" | "insert" }): void;
}>();

// 本地模式状态（用于预览）
const localInsertMode = ref<"replace" | "insert">("replace");

const message = useMessage();
const templateStore = useTemplateStore();

const selectedTemplateId = ref<string | null>(null);
const activeCategory = ref<string>(t("templateSelector.categories.all"));

// 加载模板（带重试机制）
const loadTemplates = async () => {
  try {
    if (props.projectId > 0) {
      await templateStore.loadAllTemplates(props.projectId);
    } else {
      await templateStore.loadBuiltinTemplates();
    }
  } catch (error) {
    console.error(t("templateSelector.messages.loadFailed") + ":", error);
    message.error(t("templateSelector.messages.loadFailedRetry"));
    // 可以添加重试逻辑
    setTimeout(() => {
      if (allTemplates.value.length === 0) {
        loadTemplates();
      }
    }, 2000);
  }
};

// 加载模板
onMounted(() => {
  loadTemplates();
});

// 监听 show 变化，重置选中状态
watch(
  () => props.show,
  (newVal) => {
    if (newVal) {
      selectedTemplateId.value = null;
      // 同步父组件的模式设置
      localInsertMode.value = props.insertMode;
    }
  }
);

// 监听父组件模式变化
watch(
  () => props.insertMode,
  (newVal) => {
    localInsertMode.value = newVal;
  }
);

// 所有模板（内置 + 用户）
const allTemplates = computed(() => {
  return [
    ...templateStore.builtinTemplates,
    ...templateStore.userTemplates.map((t) => ({
      id: `user_${t.id}`,
      name: t.name,
      description: t.description,
      category: t.category,
      content: t.content,
      is_builtin: false,
    })),
  ];
});

// 根据分类筛选模板
const filteredTemplates = computed(() => {
  if (activeCategory.value === "全部") {
    return allTemplates.value;
  }
  return allTemplates.value.filter((t) => t.category === activeCategory.value);
});

// 选中的模板
const selectedTemplate = computed(() => {
  if (!selectedTemplateId.value) return null;
  return (
    allTemplates.value.find((t) => t.id === selectedTemplateId.value) || null
  );
});

// 选择模板
function selectTemplate(template: WritingTemplate | any) {
  selectedTemplateId.value = template.id;
}

// 确认使用模板
function confirmSelect() {
  if (selectedTemplate.value) {
    // 同时传递内容和当前选择的模式
    emit("select", {
      content: selectedTemplate.value.content,
      mode: localInsertMode.value,
    });
    emit("update:show", false);
  }
}

// 取消
function cancel() {
  emit("update:show", false);
}

// 关闭模态框
function handleClose() {
  emit("update:show", false);
}

// 简单 Markdown 转 HTML（用于预览）
function markdownToHtml(md: string): string {
  if (!md) return "";

  let html = md;

  // 标题
  html = html.replace(/^### (.*$)/gim, "<h3>$1</h3>");
  html = html.replace(/^## (.*$)/gim, "<h2>$1</h2>");
  html = html.replace(/^# (.*$)/gim, "<h1>$1</h1>");

  // 粗体
  html = html.replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>");

  // 斜体
  html = html.replace(/\*(.*?)\*/g, "<em>$1</em>");

  // 引用
  html = html.replace(/^> (.*$)/gim, "<blockquote>$1</blockquote>");

  // 无序列表
  html = html.replace(/^- (.*$)/gim, "<li>$1</li>");
  html = html.replace(/(<li>.*<\/li>)/gims, "<ul>$1</ul>");

  // 有序列表
  html = html.replace(/^\d+\. (.*$)/gim, "<li>$1</li>");

  // 段落
  html = html.replace(/\n\n/g, "</p><p>");
  html = "<p>" + html + "</p>";

  // 清理
  html = html.replace(/<p><h/g, "<h");
  html = html.replace(/<\/h[123]><\/p>/g, "");
  html = html.replace(/<p><blockquote/g, "<blockquote");
  html = html.replace(/<\/blockquote><\/p>/g, "</blockquote>");
  html = html.replace(/<p><ul>/g, "<ul>");
  html = html.replace(/<\/ul><\/p>/g, "</ul>");

  return html;
}
</script>

<template>
  <n-modal
    :show="show"
    :mask-closable="false"
    :close-on-esc="true"
    @update:show="(val) => emit('update:show', val)"
  >
    <n-card
      :title="t('templateSelector.title')"
      style="width: 1000px; max-width: 92vw; max-height: 88vh"
      :bordered="false"
      size="large"
      role="dialog"
      aria-modal="true"
    >
      <template #header-extra>
        <n-button text @click="handleClose">
          <template #icon>
            <n-icon :component="X" />
          </template>
        </n-button>
      </template>

      <div class="template-selector-content">
        <!-- 分类筛选和模式选择 -->
        <div class="flex items-center justify-between mb-4">
          <n-tabs v-model:value="activeCategory" type="line" style="flex: 1">
            <n-tab-pane
              name="全部"
              :tab="t('templateSelector.categories.all')"
            />
            <n-tab-pane
              name="章节"
              :tab="t('templateSelector.categories.chapter')"
            />
            <n-tab-pane
              name="图文"
              :tab="t('templateSelector.categories.illustrated')"
            />
            <n-tab-pane
              name="对话"
              :tab="t('templateSelector.categories.dialogue')"
            />
            <n-tab-pane
              name="结构化"
              :tab="t('templateSelector.categories.structured')"
            />
          </n-tabs>

          <!-- 插入模式选择 -->
          <div class="flex items-center gap-2 ml-4">
            <n-radio-group
              v-model:value="localInsertMode"
              name="insertMode"
              size="small"
            >
              <n-radio-button value="replace">
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <span>{{ t("templateSelector.insertMode.replace") }}</span>
                  </template>
                  {{ t("templateSelector.insertMode.replaceTooltip") }}
                </n-tooltip>
              </n-radio-button>
              <n-radio-button value="insert">
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <span>{{ t("templateSelector.insertMode.insert") }}</span>
                  </template>
                  {{ t("templateSelector.insertMode.insertTooltip") }}
                </n-tooltip>
              </n-radio-button>
            </n-radio-group>
          </div>
        </div>

        <!-- 模板网格 -->
        <div class="template-grid-scroll">
          <div class="template-grid-inner">
            <n-grid
              v-if="filteredTemplates.length > 0"
              cols="2 m:2 l:3"
              :x-gap="12"
              :y-gap="12"
            >
              <n-grid-item
                v-for="template in filteredTemplates"
                :key="template.id"
              >
                <n-card
                  :class="{
                    'template-card': true,
                    'template-card-selected':
                      selectedTemplateId === template.id,
                  }"
                  size="small"
                  @click="selectTemplate(template)"
                >
                  <template #header>
                    <div class="flex items-center gap-2">
                      <span class="template-card-icon">
                        <FileText class="w-3.5 h-3.5" />
                      </span>
                      <n-text strong class="template-card-title">
                        {{ template.name }}
                      </n-text>
                    </div>
                  </template>

                  <template #default>
                    <div class="template-preview">
                      <div
                        class="preview-content"
                        v-html="markdownToHtml(template.content)"
                      ></div>
                    </div>

                    <n-text depth="3" class="template-desc">
                      {{ template.description }}
                    </n-text>

                    <div class="template-meta">
                      <span
                        class="template-badge"
                        :class="
                          template.is_builtin ? 'badge-builtin' : 'badge-custom'
                        "
                      >
                        {{
                          template.is_builtin
                            ? t("templateSelector.badges.builtin")
                            : t("templateSelector.badges.custom")
                        }}
                      </span>
                      <span class="template-category">{{
                        template.category
                      }}</span>
                    </div>
                  </template>
                </n-card>
              </n-grid-item>
            </n-grid>

            <div
              v-else-if="templateStore.isLoading"
              class="flex flex-col items-center justify-center py-20"
            >
              <n-spin size="large" />
              <n-text depth="3" style="margin-top: 12px">{{
                t("templateSelector.loading")
              }}</n-text>
            </div>

            <n-empty
              v-else
              :description="t('templateSelector.noTemplates')"
              style="padding: 40px 0"
            >
              <template #extra>
                <n-button size="small" @click="loadTemplates">{{
                  t("templateSelector.reload")
                }}</n-button>
              </template>
            </n-empty>
          </div>
        </div>
      </div>

      <template #footer>
        <n-space justify="end">
          <n-text depth="3" style="font-size: 12px; margin-right: auto">
            {{
              t("templateSelector.insertMode.modeLabel", {
                mode:
                  localInsertMode === "replace"
                    ? t("templateSelector.insertMode.replace")
                    : t("templateSelector.insertMode.insert"),
              })
            }}
          </n-text>
          <n-button @click="cancel">{{
            t("templateSelector.cancel")
          }}</n-button>
          <n-button
            type="primary"
            :disabled="!selectedTemplate"
            @click="confirmSelect"
          >
            {{
              localInsertMode === "replace"
                ? t("templateSelector.useReplace")
                : t("templateSelector.useInsert")
            }}
          </n-button>
        </n-space>
      </template>
    </n-card>
  </n-modal>
</template>

<style scoped>
.template-selector-content {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.template-grid-scroll {
  max-height: 520px;
  overflow-y: auto;
  overflow-x: hidden;
  margin: -6px;
  padding: 6px;
}

.template-grid-inner {
  min-height: 0;
}

.template-card {
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  border: 2px solid transparent;
  border-radius: 10px;
  overflow: visible;
}

.template-card :deep(.n-card__content) {
  padding-top: 8px;
}

.template-card:hover {
  transform: translateY(-2px) scale(1.02);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.12);
  border-color: #e2e8f0;
  z-index: 10;
}

.template-card-selected {
  border-color: #3b82f6 !important;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.25),
    0 4px 16px rgba(59, 130, 246, 0.15);
  background-color: #f8faff;
}

.template-card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: linear-gradient(135deg, #3b82f6, #6366f1);
  color: #fff;
  flex-shrink: 0;
}

.template-card-title {
  font-size: 14px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.template-preview {
  max-height: 130px;
  overflow: hidden;
  position: relative;
  background-color: #1a1a2e;
  border-radius: 6px;
  padding: 10px;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.template-preview::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 36px;
  background: linear-gradient(transparent, #1a1a2e);
  pointer-events: none;
}

.preview-content {
  font-size: 12px;
  line-height: 1.6;
  color: #d4d4d8;
}

.preview-content:deep(h1),
.preview-content:deep(h2),
.preview-content:deep(h3) {
  margin: 6px 0 4px;
  font-weight: 600;
  line-height: 1.3;
}

.preview-content:deep(h1) {
  font-size: 16px;
  color: #60a5fa;
}

.preview-content:deep(h2) {
  font-size: 14px;
  color: #a78bfa;
}

.preview-content:deep(h3) {
  font-size: 13px;
  color: #34d399;
}

.preview-content:deep(p) {
  margin: 3px 0;
  color: #d4d4d8;
}

.preview-content:deep(strong) {
  font-weight: 600;
  color: #fafafa;
}

.preview-content:deep(em) {
  font-style: italic;
  color: #fbbf24;
}

.preview-content:deep(blockquote) {
  border-left: 3px solid #6366f1;
  padding: 4px 10px;
  margin: 6px 0;
  color: #a1a1aa;
  background-color: rgba(99, 102, 241, 0.12);
  border-radius: 0 4px 4px 0;
}

.preview-content:deep(ul),
.preview-content:deep(ol) {
  margin: 4px 0;
  padding-left: 18px;
  color: #d4d4d8;
}

.preview-content:deep(li) {
  margin: 2px 0;
  color: #d4d4d8;
}

.preview-content:deep(ul) {
  list-style-type: disc;
}

.preview-content:deep(ul li)::marker {
  color: #22d3ee;
}

.template-desc {
  font-size: 12px;
  display: block;
  margin-top: 10px;
  line-height: 1.5;
}

.template-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}

.template-badge {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
  line-height: 1.4;
}

.badge-builtin {
  background-color: #eff6ff;
  color: #3b82f6;
  border: 1px solid #bfdbfe;
}

.badge-custom {
  background-color: #fef3c7;
  color: #d97706;
  border: 1px solid #fde68a;
}

.template-category {
  font-size: 10px;
  color: #9ca3af;
}

/* ===== 暗色主题 ===== */
:root.dark .template-card:hover {
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.4);
  border-color: #374151;
}

:root.dark .template-card-selected {
  border-color: #60a5fa !important;
  box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.25),
    0 4px 16px rgba(96, 165, 250, 0.15);
  background-color: #111827;
}

:root.dark .template-card-icon {
  background: linear-gradient(135deg, #60a5fa, #818cf8);
}

:root.dark .template-preview {
  background-color: #0f0f1a;
  border-color: rgba(255, 255, 255, 0.04);
}

:root.dark .template-preview::after {
  background: linear-gradient(transparent, #0f0f1a);
}

:root.dark .badge-builtin {
  background-color: #1e3a5f;
  color: #60a5fa;
  border-color: #1e40af;
}

:root.dark .badge-custom {
  background-color: #422006;
  color: #fbbf24;
  border-color: #78350f;
}

:root.dark .template-category {
  color: #6b7280;
}

/* ===== 响应式 ===== */
@media screen and (max-width: 768px) {
  .template-grid-scroll {
    max-height: 420px;
  }

  .template-preview {
    max-height: 100px;
  }

  .template-card-title {
    font-size: 13px;
  }
}

@media screen and (max-width: 480px) {
  .template-grid-scroll {
    max-height: 360px;
  }

  .template-preview {
    max-height: 80px;
    padding: 6px;
  }

  .preview-content {
    font-size: 11px;
  }
}
</style>
