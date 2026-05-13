<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from "vue";
import {
  NButton,
  NInput,
  NIcon,
  NModal,
  useMessage,
  NDropdown,
  useDialog,
} from "naive-ui";
import {
  Lightbulb,
  MessageCircle,
  Map,
  Plus,
  Trash2,
  Edit3,
  GripVertical,
  Send,
  X,
  MoreVertical,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import draggable from "vuedraggable";
import { useLocale } from "../i18n/composables/useLocale";
import { getLocale } from "../i18n";

const { t, locale } = useLocale();

const props = defineProps<{
  projectId: number;
  isDark: boolean;
}>();

const emit = defineEmits<{
  (e: "insert-content", content: string): void;
}>();

const message = useMessage();
const dialog = useDialog();

const defaultColumnKeys = ["inspiration", "dialogue", "scene"];

const defaultColumnDefinitions = [
  { key: "inspiration", icon: Lightbulb },
  { key: "dialogue", icon: MessageCircle },
  { key: "scene", icon: Map },
];

const columnIconMap: Record<string, typeof Lightbulb> = {
  inspiration: Lightbulb,
  dialogue: MessageCircle,
  scene: Map,
};

interface InspirationItem {
  id: number;
  project_id: number;
  column_key: string;
  column_name: string;
  content: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface ColumnInfo {
  column_key: string;
  column_name: string;
  items: InspirationItem[];
}

const columns = ref<ColumnInfo[]>([]);
const isLoading = ref(true);

const showNewColumnInput = ref(false);
const newColumnName = ref("");

const editingCardId = ref<number | null>(null);
const editingContent = ref("");
const contextMenuCard = ref<{
  item: InspirationItem;
  x: number;
  y: number;
} | null>(null);

const getDefaultColumns = (): ColumnInfo[] => {
  return defaultColumnDefinitions.map((def) => ({
    column_key: def.key,
    column_name: t(`inspiration.columns.${def.key}`),
    items: [],
  }));
};

const loadBoard = async () => {
  isLoading.value = true;
  try {
    const data = await invoke<{ columns: ColumnInfo[] }>(
      "get_inspiration_board",
      {
        project_id: props.projectId,
        locale: getLocale(),
      }
    );
    columns.value = data.columns;
  } catch (error) {
    console.error("加载灵感看板失败:", error);
    columns.value = getDefaultColumns();
  } finally {
    isLoading.value = false;
  }
};

const saveAllItems = async () => {
  const updates: { id: number; column_key: string; sort_order: number }[] = [];

  columns.value.forEach((col) => {
    col.items.forEach((item, index) => {
      updates.push({
        id: item.id,
        column_key: col.column_key,
        sort_order: index,
      });
    });
  });

  try {
    await invoke("reorder_inspiration_items", {
      updates: updates,
    });
  } catch (error) {
    console.error("保存排序失败:", error);
  }
};

const createItem = async (columnKey: string) => {
  try {
    const newItem = await invoke<InspirationItem>("create_inspiration_item", {
      params: {
        project_id: props.projectId,
        column_key: columnKey,
        content: "",
      },
    });

    const column = columns.value.find((c) => c.column_key === columnKey);
    if (column) {
      column.items.push(newItem);
    }

    nextTick(() => {
      startEditing(newItem.id, newItem.content);
    });
  } catch (error) {
    console.error("创建条目失败:", error);
    message.error(t("inspiration.messages.createFailed"));
  }
};

const updateItem = async (itemId: number) => {
  const item = findItem(itemId);
  if (!item) return;

  try {
    await invoke("update_inspiration_item", {
      item_id: itemId,
      params: {
        content: editingContent.value,
      },
    });

    item.content = editingContent.value;
    editingCardId.value = null;
  } catch (error) {
    console.error("更新条目失败:", error);
    message.error(t("inspiration.messages.updateFailed"));
  }
};

const deleteItem = async (itemId: number) => {
  dialog.warning({
    title: t("inspiration.confirmDelete"),
    content: t("inspiration.deleteItemConfirm"),
    positiveText: t("inspiration.delete"),
    negativeText: t("inspiration.cancel"),
    onPositiveClick: async () => {
      try {
        await invoke("delete_inspiration_item", { item_id: itemId });

        columns.value.forEach((col) => {
          col.items = col.items.filter((item) => item.id !== itemId);
        });

        message.success(t("inspiration.deleted"));
      } catch (error) {
        console.error("删除条目失败:", error);
        message.error(t("inspiration.messages.deleteFailed"));
      }
    },
  });
};

const findItem = (itemId: number): InspirationItem | undefined => {
  for (const col of columns.value) {
    const item = col.items.find((i) => i.id === itemId);
    if (item) return item;
  }
  return undefined;
};

const startEditing = (itemId: number, content: string) => {
  editingCardId.value = itemId;
  editingContent.value = content;
};

const cancelEditing = () => {
  editingCardId.value = null;
  editingContent.value = "";
};

const onColumnChange = () => {
  saveAllItems();
};

const insertToEditor = (content: string) => {
  if (content.trim()) {
    emit("insert-content", content);
    message.success(t("inspiration.insertedToEditor"));
  }
  closeContextMenu();
};

const showContextMenu = (event: MouseEvent, item: InspirationItem) => {
  event.preventDefault();
  contextMenuCard.value = {
    item,
    x: event.clientX,
    y: event.clientY,
  };
};

const closeContextMenu = () => {
  contextMenuCard.value = null;
};

const addColumn = () => {
  if (!newColumnName.value.trim()) {
    message.warning(t("inspiration.enterColumnName"));
    return;
  }

  const customKey = `custom-${Date.now()}`;
  if (columns.value.some((c) => c.column_name === newColumnName.value.trim())) {
    message.warning(t("inspiration.columnExists"));
    return;
  }

  columns.value.push({
    column_key: customKey,
    column_name: newColumnName.value.trim(),
    items: [],
  });

  newColumnName.value = "";
  showNewColumnInput.value = false;
  message.success(t("inspiration.columnAdded"));
};

const deleteColumn = (columnKey: string) => {
  if (defaultColumnKeys.includes(columnKey)) {
    message.warning(t("inspiration.defaultColumnCannotDelete"));
    return;
  }

  const col = columns.value.find((c) => c.column_key === columnKey);
  const colName = col?.column_name || columnKey;

  dialog.warning({
    title: t("inspiration.confirmDelete"),
    content: t("inspiration.deleteColumnConfirm", { name: colName }),
    positiveText: t("inspiration.delete"),
    negativeText: t("inspiration.cancel"),
    onPositiveClick: () => {
      columns.value = columns.value.filter((c) => c.column_key !== columnKey);
      message.success(t("inspiration.columnDeleted"));
    },
  });
};

const getColumnIcon = (columnKey: string) => {
  return columnIconMap[columnKey] || Lightbulb;
};

const isDefaultColumn = (columnKey: string) => {
  return defaultColumnKeys.includes(columnKey);
};

const handleGlobalClick = () => {
  closeContextMenu();
};

onMounted(() => {
  loadBoard();
  document.addEventListener("click", handleGlobalClick);
});

watch(locale, () => {
  loadBoard();
});
</script>

<template>
  <div class="inspiration-board" @click.stop>
    <!-- Loading State -->
    <div v-if="isLoading" class="flex items-center justify-center h-full">
      <n-spin size="large" />
    </div>

    <!-- Board -->
    <div v-else class="board-container">
      <draggable
        v-model="columns"
        class="board"
        :animation="200"
        item-key="column_key"
        handle=".column-header"
        ghost-class="ghost-column"
      >
        <template #item="{ element: column }">
          <div class="column" :class="{ 'is-dark': isDark }">
            <!-- Column Header -->
            <div class="column-header">
              <div class="column-title">
                <component :is="getColumnIcon(column.column_key)" class="w-4 h-4" />
                <span>{{ column.column_name }}</span>
                <span class="item-count">({{ column.items.length }})</span>
              </div>
              <div class="column-actions">
                <button
                  v-if="!isDefaultColumn(column.column_key)"
                  class="action-btn delete"
                  @click="deleteColumn(column.column_key)"
                  :title="t('inspiration.deleteColumn')"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
                <button
                  class="action-btn"
                  @click="createItem(column.column_key)"
                  :title="t('inspiration.addItem')"
                >
                  <Plus class="w-4 h-4" />
                </button>
              </div>
            </div>

            <!-- Cards List -->
            <draggable
              v-model="column.items"
              class="cards-list"
              :animation="200"
              group="cards"
              item-key="id"
              ghost-class="ghost-card"
              @change="onColumnChange"
            >
              <template #item="{ element: item }">
                <div
                  class="card"
                  :class="{ 'is-editing': editingCardId === item.id }"
                  @contextmenu="showContextMenu($event, item)"
                  @dblclick="startEditing(item.id, item.content)"
                >
                  <!-- Card Content -->
                  <div v-if="editingCardId !== item.id" class="card-content">
                    <pre class="card-text">{{
                      item.content || t("inspiration.doubleClickToEdit")
                    }}</pre>
                    <div class="card-drag-handle">
                      <GripVertical class="w-4 h-4" />
                    </div>
                  </div>

                  <!-- Edit Mode -->
                  <div v-else class="card-edit">
                    <textarea
                      v-model="editingContent"
                      class="edit-textarea"
                      :placeholder="t('inspiration.editPlaceholder')"
                      rows="4"
                      @keydown.ctrl.enter="updateItem(item.id)"
                      @keydown.escape="cancelEditing"
                    ></textarea>
                    <div class="edit-actions">
                      <n-button size="tiny" @click="cancelEditing">
                        <X class="w-3 h-3" />
                      </n-button>
                      <n-button
                        size="tiny"
                        type="primary"
                        @click="updateItem(item.id)"
                      >
                        {{ t("inspiration.save") }}
                      </n-button>
                    </div>
                  </div>
                </div>
              </template>
            </draggable>

            <!-- Add Card Button -->
            <button class="add-card-btn" @click="createItem(column.column_key)">
              <Plus class="w-4 h-4" />
              <span>{{ t("inspiration.addItem") }}</span>
            </button>
          </div>
        </template>
      </draggable>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenuCard"
        class="context-menu"
        :style="{
          left: `${contextMenuCard.x}px`,
          top: `${contextMenuCard.y}px`,
        }"
        @click.stop
      >
        <button
          class="context-menu-item"
          @click="insertToEditor(contextMenuCard.item.content)"
        >
          <Send class="w-4 h-4" />
          <span>{{ t("inspiration.insertToEditor") }}</span>
        </button>
        <button
          class="context-menu-item"
          @click="
            startEditing(contextMenuCard.item.id, contextMenuCard.item.content);
            closeContextMenu();
          "
        >
          <Edit3 class="w-4 h-4" />
          <span>{{ t("inspiration.edit") }}</span>
        </button>
        <button
          class="context-menu-item danger"
          @click="deleteItem(contextMenuCard.item.id)"
        >
          <Trash2 class="w-4 h-4" />
          <span>{{ t("inspiration.delete") }}</span>
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.inspiration-board {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.board-container {
  height: 100%;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 16px;
}

.board {
  display: flex;
  gap: 16px;
  height: 100%;
  min-width: min-content;
}

.column {
  flex-shrink: 0;
  width: 280px;
  max-height: 100%;
  display: flex;
  flex-direction: column;
  background: v-bind("isDark ? '#1f2937' : '#f3f4f6'");
  border-radius: 12px;
  overflow: hidden;
}

.column.is-dark {
  background: #1f2937;
}

.column-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  cursor: grab;
  user-select: none;
  background: v-bind("isDark ? '#374151' : '#e5e7eb'");
}

.column-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: v-bind("isDark ? '#f3f4f6' : '#374151'");
}

.item-count {
  font-weight: 400;
  font-size: 0.875rem;
  opacity: 0.7;
}

.column-actions {
  display: flex;
  gap: 4px;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: v-bind("isDark ? '#9ca3af' : '#6b7280'");
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: v-bind("isDark ? '#4b5563' : '#d1d5db'");
  color: v-bind("isDark ? '#f3f4f6' : '#374151'");
}

.action-btn.delete:hover {
  background: #fee2e2;
  color: #dc2626;
}

.cards-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 100px;
}

.card {
  background: v-bind("isDark ? '#374151' : '#ffffff'");
  border-radius: 8px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.card:hover {
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.card.is-editing {
  padding: 8px;
}

.card-content {
  position: relative;
}

.card-text {
  font-family: inherit;
  font-size: 0.875rem;
  line-height: 1.5;
  color: v-bind("isDark ? '#e5e7eb' : '#374151'");
  white-space: pre-wrap;
  word-break: break-word;
  margin: 0;
  padding-right: 24px;
}

.card-drag-handle {
  position: absolute;
  top: 0;
  right: 0;
  opacity: 0;
  transition: opacity 0.2s;
  color: v-bind("isDark ? '#6b7280' : '#9ca3af'");
}

.card:hover .card-drag-handle {
  opacity: 1;
}

.card-edit {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.edit-textarea {
  width: 100%;
  padding: 8px;
  border: 1px solid v-bind("isDark ? '#4b5563' : '#d1d5db'");
  border-radius: 6px;
  background: v-bind("isDark ? '#1f2937' : '#ffffff'");
  color: v-bind("isDark ? '#f3f4f6' : '#374151'");
  font-family: inherit;
  font-size: 0.875rem;
  line-height: 1.5;
  resize: vertical;
  min-height: 80px;
}

.edit-textarea:focus {
  outline: none;
  border-color: #3b82f6;
}

.edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.add-card-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 12px;
  border: none;
  background: transparent;
  color: v-bind("isDark ? '#9ca3af' : '#6b7280'");
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
}

.add-card-btn:hover {
  background: v-bind("isDark ? '#374151' : '#e5e7eb'");
  color: v-bind("isDark ? '#f3f4f6' : '#374151'");
}

.add-column {
  flex-shrink: 0;
  width: 280px;
}

.add-column-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 16px;
  border: 2px dashed v-bind("isDark ? '#4b5563' : '#d1d5db'");
  border-radius: 12px;
  background: transparent;
  color: v-bind("isDark ? '#9ca3af' : '#6b7280'");
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s;
}

.add-column-btn:hover {
  border-color: #3b82f6;
  color: #3b82f6;
}

.new-column-input {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  background: v-bind("isDark ? '#1f2937' : '#ffffff'");
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.input-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.ghost-card {
  opacity: 0.5;
  background: #3b82f6 !important;
}

.ghost-column {
  opacity: 0.5;
}

.context-menu {
  position: fixed;
  z-index: 9999;
  min-width: 160px;
  background: v-bind("isDark ? '#374151' : '#ffffff'");
  border-radius: 8px;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
  padding: 4px;
  animation: fadeIn 0.15s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 12px;
  border: none;
  background: transparent;
  color: v-bind("isDark ? '#e5e7eb' : '#374151'");
  font-size: 0.875rem;
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s;
}

.context-menu-item:hover {
  background: v-bind("isDark ? '#4b5563' : '#f3f4f6'");
}

.context-menu-item.danger {
  color: #dc2626;
}

.context-menu-item.danger:hover {
  background: #fee2e2;
}

.cards-list::-webkit-scrollbar {
  width: 6px;
}

.cards-list::-webkit-scrollbar-track {
  background: transparent;
}

.cards-list::-webkit-scrollbar-thumb {
  background: v-bind("isDark ? '#4b5563' : '#d1d5db'");
  border-radius: 3px;
}

.cards-list::-webkit-scrollbar-thumb:hover {
  background: v-bind("isDark ? '#6b7280' : '#9ca3af'");
}
</style>
