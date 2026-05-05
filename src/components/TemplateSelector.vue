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

const props = withDefaults(
  defineProps<{
    show: boolean;
    projectId?: number;
    insertMode?: 'replace' | 'insert'; // 来自父组件的模式设置
  }>(),
  {
    projectId: 0,
    insertMode: 'replace',
  }
);

const emit = defineEmits<{
  (e: "update:show", value: boolean): void;
  (e: "select", payload: { content: string; mode: 'replace' | 'insert' }): void;
}>();

// 本地模式状态（用于预览）
const localInsertMode = ref<'replace' | 'insert'>('replace');

const message = useMessage();
const templateStore = useTemplateStore();

const selectedTemplateId = ref<string | null>(null);
const activeCategory = ref<string>("全部");

// 加载模板（带重试机制）
const loadTemplates = async () => {
  try {
    if (props.projectId > 0) {
      await templateStore.loadAllTemplates(props.projectId);
    } else {
      await templateStore.loadBuiltinTemplates();
    }
  } catch (error) {
    console.error("加载模板失败:", error);
    message.error("加载模板失败，请重试");
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
  return allTemplates.value.find((t) => t.id === selectedTemplateId.value) || null;
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
      mode: localInsertMode.value
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
      title="选择写作模板"
      style="width: 800px; max-width: 90vw; max-height: 85vh"
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
          <n-tabs
            v-model:value="activeCategory"
            type="line"
            style="flex: 1"
          >
            <n-tab-pane name="全部" tab="全部" />
            <n-tab-pane name="章节" tab="章节" />
            <n-tab-pane name="图文" tab="图文" />
            <n-tab-pane name="对话" tab="对话" />
            <n-tab-pane name="结构化" tab="结构化" />
          </n-tabs>
          
          <!-- 插入模式选择 -->
          <div class="flex items-center gap-2 ml-4">
            <n-radio-group v-model:value="localInsertMode" name="insertMode" size="small">
              <n-radio-button value="replace">
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <span>替换内容</span>
                  </template>
                  清空当前内容，使用模板
                </n-tooltip>
              </n-radio-button>
              <n-radio-button value="insert">
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <span>插入内容</span>
                  </template>
                  在当前光标位置插入模板
                </n-tooltip>
              </n-radio-button>
            </n-radio-group>
          </div>
        </div>

        <!-- 模板网格 -->
        <div class="template-grid-container">
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
                  'template-card-selected': selectedTemplateId === template.id,
                }"
                size="small"
                @click="selectTemplate(template)"
              >
                <template #header>
                  <n-text strong style="font-size: 14px">
                    {{ template.name }}
                  </n-text>
                </template>
                
                <template #default>
                  <!-- 模板预览 -->
                  <div class="template-preview">
                    <div
                      class="preview-content"
                      v-html="markdownToHtml(template.content)"
                    ></div>
                  </div>
                  
                  <!-- 模板描述 -->
                  <n-text
                    depth="3"
                    style="font-size: 11px; display: block; margin-top: 8px"
                  >
                    {{ template.description }}
                  </n-text>
                  
                  <!-- 内置/自定义标签 -->
                  <n-text
                    :depth="3"
                    style="font-size: 10px; display: block; margin-top: 4px"
                  >
                    {{ template.is_builtin ? "内置" : "自定义" }}
                  </n-text>
                </template>
              </n-card>
            </n-grid-item>
          </n-grid>

          <!-- 加载状态 -->
          <div v-else-if="templateStore.isLoading" class="flex flex-col items-center justify-center py-20">
            <n-spin size="large" />
            <n-text depth="3" style="margin-top: 12px">加载模板中...</n-text>
          </div>
          
          <!-- 空状态 -->
          <n-empty
            v-else
            description="暂无模板，请稍后重试"
            style="padding: 40px 0"
          >
            <template #extra>
              <n-button size="small" @click="loadTemplates">重新加载</n-button>
            </template>
          </n-empty>
        </div>
      </div>

      <template #footer>
        <n-space justify="end">
          <n-text depth="3" style="font-size: 12px; margin-right: auto;">
            模式：{{ localInsertMode === 'replace' ? '替换内容' : '插入内容' }}
          </n-text>
          <n-button @click="cancel">取消</n-button>
          <n-button
            type="primary"
            :disabled="!selectedTemplate"
            @click="confirmSelect"
          >
            {{ localInsertMode === 'replace' ? '使用模板替换' : '插入模板内容' }}
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
}

.template-grid-container {
  max-height: 450px;
  overflow-y: auto;
  padding-right: 8px;
}

.template-card {
  cursor: pointer;
  transition: all 0.2s ease;
  border: 2px solid transparent;
}

.template-card:hover {
  transform: scale(1.02);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.template-card-selected {
  border-color: var(--n-primary-color, #18a058);
  box-shadow: 0 0 0 2px rgba(24, 160, 88, 0.3);
}

/* 模板预览区域 - 使用深色背景确保文字清晰可见 */
.template-preview {
  max-height: 100px;
  overflow: hidden;
  position: relative;
  background-color: #1e1e2e;
  border-radius: 4px;
  padding: 8px;
}

.template-preview::after {
  content: "";
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 30px;
  background: linear-gradient(transparent, #1e1e2e);
  pointer-events: none;
}

/* 预览内容样式 - 高对比度配色 */
.preview-content {
  font-size: 12px;
  line-height: 1.5;
  color: #e4e4e7;
}

/* 标题样式 */
.preview-content:deep(h1),
.preview-content:deep(h2),
.preview-content:deep(h3) {
  margin: 4px 0;
  font-weight: 600;
  color: #fafafa;
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
  margin: 2px 0;
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
  padding-left: 8px;
  margin: 4px 0;
  color: #a1a1aa;
  background-color: rgba(99, 102, 241, 0.1);
  border-radius: 0 4px 4px 0;
}

.preview-content:deep(ul),
.preview-content:deep(ol) {
  margin: 2px 0;
  padding-left: 16px;
  color: #d4d4d8;
}

.preview-content:deep(li) {
  margin: 1px 0;
  color: #d4d4d8;
}

/* 列表项标记颜色 */
.preview-content:deep(ul) {
  list-style-type: disc;
}

.preview-content:deep(ul li)::marker {
  color: #22d3ee;
}
</style>
