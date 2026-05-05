<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from "vue";
import { NButton, NInput, NIcon, NModal, useMessage, NDropdown, useDialog } from "naive-ui";
import { Lightbulb, MessageCircle, Map, Plus, Trash2, Edit3, GripVertical, Send, X, MoreVertical } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import draggable from "vuedraggable";

const props = defineProps<{
  projectId: number;
  isDark: boolean;
}>();

const emit = defineEmits<{
  (e: "insert-content", content: string): void;
}>();

const message = useMessage();
const dialog = useDialog();

// Default columns
const defaultColumnNames = ["灵感", "对白", "场景"];

// Column icons mapping
const columnIcons: Record<string, typeof Lightbulb> = {
  "灵感": Lightbulb,
  "对白": MessageCircle,
  "场景": Map,
};

// Board data
interface InspirationItem {
  id: number;
  project_id: number;
  column_name: string;
  content: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface ColumnInfo {
  name: string;
  items: InspirationItem[];
}

const columns = ref<ColumnInfo[]>([]);
const isLoading = ref(true);

// New column name input
const showNewColumnInput = ref(false);
const newColumnName = ref("");

// Card editing state
const editingCardId = ref<number | null>(null);
const editingContent = ref("");
const contextMenuCard = ref<{ item: InspirationItem; x: number; y: number } | null>(null);

// Get or create default columns
const getDefaultColumns = (): ColumnInfo[] => {
  return defaultColumnNames.map((name) => ({
    name,
    items: [],
  }));
};

// Load board data
const loadBoard = async () => {
  isLoading.value = true;
  try {
    const data = await invoke<{ columns: ColumnInfo[] }>("get_inspiration_board", {
      projectId: props.projectId,
    });

    // Merge with defaults to ensure all default columns exist
    const columnMap: Record<string, ColumnInfo> = {};
    data.columns.forEach((col) => { columnMap[col.name] = col; });

    const mergedColumns: ColumnInfo[] = [];
    defaultColumnNames.forEach((name) => {
      if (columnMap[name]) {
        mergedColumns.push(columnMap[name]);
      } else {
        mergedColumns.push({ name, items: [] });
      }
    });

    // Add any custom columns
    data.columns.forEach((col) => {
      if (!defaultColumnNames.includes(col.name)) {
        mergedColumns.push(col);
      }
    });

    columns.value = mergedColumns;
  } catch (error) {
    console.error("加载灵感看板失败:", error);
    columns.value = getDefaultColumns();
  } finally {
    isLoading.value = false;
  }
};

// Save all items
const saveAllItems = async () => {
  const updates: { id: number; column_name: string; sort_order: number }[] = [];

  columns.value.forEach((col) => {
    col.items.forEach((item, index) => {
      updates.push({
        id: item.id,
        column_name: col.name,
        sort_order: index,
      });
    });
  });

  try {
    await invoke("reorder_inspiration_items", {
      projectId: props.projectId,
      updates,
    });
  } catch (error) {
    console.error("保存排序失败:", error);
  }
};

// Create new item in column
const createItem = async (columnName: string) => {
  try {
    const newItem = await invoke<InspirationItem>("create_inspiration_item", {
      params: {
        project_id: props.projectId,
        column_name: columnName,
        content: "",
      },
    });

    const column = columns.value.find((c) => c.name === columnName);
    if (column) {
      column.items.push(newItem);
    }

    // Start editing the new item
    nextTick(() => {
      startEditing(newItem.id, newItem.content);
    });
  } catch (error) {
    console.error("创建条目失败:", error);
    message.error("创建条目失败");
  }
};

// Update item content
const updateItem = async (itemId: number) => {
  const item = findItem(itemId);
  if (!item) return;

  try {
    await invoke("update_inspiration_item", {
      itemId,
      params: {
        content: editingContent.value,
      },
    });

    item.content = editingContent.value;
    editingCardId.value = null;
  } catch (error) {
    console.error("更新条目失败:", error);
    message.error("更新条目失败");
  }
};

// Delete item
const deleteItem = async (itemId: number) => {
  dialog.warning({
    title: "确认删除",
    content: "确定要删除这个灵感条目吗？",
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        await invoke("delete_inspiration_item", { itemId });

        columns.value.forEach((col) => {
          col.items = col.items.filter((item) => item.id !== itemId);
        });

        message.success("已删除");
      } catch (error) {
        console.error("删除条目失败:", error);
        message.error("删除条目失败");
      }
    },
  });
};

// Find item by id
const findItem = (itemId: number): InspirationItem | undefined => {
  for (const col of columns.value) {
    const item = col.items.find((i) => i.id === itemId);
    if (item) return item;
  }
  return undefined;
};

// Start editing
const startEditing = (itemId: number, content: string) => {
  editingCardId.value = itemId;
  editingContent.value = content;
};

// Cancel editing
const cancelEditing = () => {
  editingCardId.value = null;
  editingContent.value = "";
};

// Handle column change during drag
const onColumnChange = () => {
  saveAllItems();
};

// Insert content to editor
const insertToEditor = (content: string) => {
  if (content.trim()) {
    emit("insert-content", content);
    message.success("已插入到编辑器");
  }
  closeContextMenu();
};

// Context menu
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

// Add new column
const addColumn = () => {
  if (!newColumnName.value.trim()) {
    message.warning("请输入列名称");
    return;
  }

  if (columns.value.some((c) => c.name === newColumnName.value.trim())) {
    message.warning("该列已存在");
    return;
  }

  columns.value.push({
    name: newColumnName.value.trim(),
    items: [],
  });

  newColumnName.value = "";
  showNewColumnInput.value = false;
  message.success("已添加新列");
};

// Delete column
const deleteColumn = (columnName: string) => {
  if (defaultColumnNames.includes(columnName)) {
    message.warning("默认列不能删除");
    return;
  }

  dialog.warning({
    title: "确认删除",
    content: `确定要删除"${columnName}"列吗？该列下的所有内容也会被删除。`,
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: () => {
      columns.value = columns.value.filter((c) => c.name !== columnName);
      message.success("已删除列");
    },
  });
};

// Get column icon
const getColumnIcon = (columnName: string) => {
  return columnIcons[columnName] || Lightbulb;
};

// Click outside to close context menu
const handleGlobalClick = () => {
  closeContextMenu();
};

onMounted(() => {
  loadBoard();
  document.addEventListener("click", handleGlobalClick);
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
        item-key="name"
        handle=".column-header"
        ghost-class="ghost-column"
      >
        <template #item="{ element: column }">
          <div class="column" :class="{ 'is-dark': isDark }">
            <!-- Column Header -->
            <div class="column-header">
              <div class="column-title">
                <component :is="getColumnIcon(column.name)" class="w-4 h-4" />
                <span>{{ column.name }}</span>
                <span class="item-count">({{ column.items.length }})</span>
              </div>
              <div class="column-actions">
                <button
                  v-if="!defaultColumnNames.includes(column.name)"
                  class="action-btn delete"
                  @click="deleteColumn(column.name)"
                  title="删除列"
                >
                  <Trash2 class="w-3.5 h-3.5" />
                </button>
                <button
                  class="action-btn"
                  @click="createItem(column.name)"
                  title="添加条目"
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
                    <pre class="card-text">{{ item.content || '双击编辑...' }}</pre>
                    <div class="card-drag-handle">
                      <GripVertical class="w-4 h-4" />
                    </div>
                  </div>

                  <!-- Edit Mode -->
                  <div v-else class="card-edit">
                    <textarea
                      v-model="editingContent"
                      class="edit-textarea"
                      placeholder="输入灵感内容..."
                      rows="4"
                      @keydown.ctrl.enter="updateItem(item.id)"
                      @keydown.escape="cancelEditing"
                    ></textarea>
                    <div class="edit-actions">
                      <n-button size="tiny" @click="cancelEditing">
                        <X class="w-3 h-3" />
                      </n-button>
                      <n-button size="tiny" type="primary" @click="updateItem(item.id)">
                        保存
                      </n-button>
                    </div>
                  </div>
                </div>
              </template>
            </draggable>

            <!-- Add Card Button -->
            <button class="add-card-btn" @click="createItem(column.name)">
              <Plus class="w-4 h-4" />
              <span>添加条目</span>
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
        :style="{ left: `${contextMenuCard.x}px`, top: `${contextMenuCard.y}px` }"
        @click.stop
      >
        <button class="context-menu-item" @click="insertToEditor(contextMenuCard.item.content)">
          <Send class="w-4 h-4" />
          <span>插入到编辑器</span>
        </button>
        <button class="context-menu-item" @click="startEditing(contextMenuCard.item.id, contextMenuCard.item.content); closeContextMenu()">
          <Edit3 class="w-4 h-4" />
          <span>编辑</span>
        </button>
        <button class="context-menu-item danger" @click="deleteItem(contextMenuCard.item.id)">
          <Trash2 class="w-4 h-4" />
          <span>删除</span>
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

/* Ghost styles for drag */
.ghost-card {
  opacity: 0.5;
  background: #3b82f6 !important;
}

.ghost-column {
  opacity: 0.5;
}

/* Context Menu */
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

/* Scrollbar */
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
