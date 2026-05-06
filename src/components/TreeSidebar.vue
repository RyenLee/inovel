<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessage } from "naive-ui";
import { NTree, NIcon, NButton, NDropdown, NInput, NPopconfirm, NEmpty, NSelect } from "naive-ui";
import {
  ChevronRight,
  ChevronDown,
  FolderOpen,
  FileText,
  Plus,
  Edit3,
  Trash2,
  MoreVertical,
  GripVertical,
  Filter,
} from "lucide-vue-next";
import draggable from "vuedraggable";
import DeleteConfirmModal from "./DeleteConfirmModal.vue";
import TemplateSelector from "./TemplateSelector.vue";
import { CHAPTER_STATUS_OPTIONS, getStatusColor, getStatusLabel, type ChapterStatus } from "../types/chapter";

interface Chapter {
  id: number;
  volume_id: number;
  title: string;
  file_path: string;
  sort_order: number;
  summary: string;
  word_count_cache: number;
  status: ChapterStatus;
  created_at: string;
  updated_at: string;
}

interface VolumeWithChapters {
  id: number;
  project_id: number;
  name: string;
  sort_order: number;
  chapters: Chapter[];
}

// Props
const props = defineProps<{
  projectId: string;
}>();

// Emits
const emit = defineEmits<{
  (e: "select-chapter", chapterId: number, chapter: Chapter): void;
}>();

// Message
const message = useMessage();

// State
const volumes = ref<VolumeWithChapters[]>([]);
const isLoading = ref(true);
const expandedVolumes = ref<number[]>([]);
const selectedChapterId = ref<number | null>(null);

// Status filter
const statusFilter = ref<ChapterStatus | 'all'>('all');
const showStatusFilter = ref(false);

// 模板选择器状态
const showTemplateSelector = ref(false);
const currentVolumeIdForTemplate = ref<number | null>(null);

const statusFilterOptions = [
  { label: '全部章节', value: 'all' },
  { label: '大纲', value: 'outline', color: '#9CA3AF' },
  { label: '草稿', value: 'draft', color: '#F59E0B' },
  { label: '修订', value: 'revised', color: '#3B82F6' },
  { label: '定稿', value: 'final', color: '#10B981' },
  { label: '废弃', value: 'abandoned', color: '#EF4444' },
];

// Filtered volumes based on status filter
const filteredVolumes = computed(() => {
  if (statusFilter.value === 'all') {
    return volumes.value;
  }
  return volumes.value
    .map(volume => ({
      ...volume,
      chapters: volume.chapters.filter(chapter => chapter.status === statusFilter.value),
    }))
    .filter(volume => volume.chapters.length > 0);
});

// Context menu state
const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuType = ref<"volume" | "chapter" | "empty">("empty");
const contextMenuVolumeId = ref<number | null>(null);
const contextMenuChapterId = ref<number | null>(null);

// Edit state
const editingVolumeId = ref<number | null>(null);
const editingChapterId = ref<number | null>(null);
const editingVolumeName = ref("");
const editingChapterTitle = ref("");

// Status edit state
const editingChapterStatusId = ref<number | null>(null);
const editingChapterStatus = ref<ChapterStatus>('draft');

// Update chapter status
const updateChapterStatus = async (chapterId: number, newStatus: ChapterStatus) => {
  try {
    await invoke("update_chapter_status", {
      chapterId,
      status: newStatus,
    });
    for (const volume of volumes.value) {
      const chapter = volume.chapters.find((c) => c.id === chapterId);
      if (chapter) {
        chapter.status = newStatus;
        break;
      }
    }
    message.success(`章节状态已更新为: ${getStatusLabel(newStatus)}`);
  } catch (error) {
    console.error("更新章节状态失败:", error);
    message.error("更新章节状态失败");
  }
};

// Start editing chapter status
const startEditChapterStatus = (chapter: Chapter) => {
  editingChapterStatusId.value = chapter.id;
  editingChapterStatus.value = chapter.status;
};

// Confirm status edit
const confirmEditChapterStatus = () => {
  if (editingChapterStatusId.value !== null) {
    updateChapterStatus(editingChapterStatusId.value, editingChapterStatus.value);
  }
  editingChapterStatusId.value = null;
};

// Cancel status edit
const cancelEditChapterStatus = () => {
  editingChapterStatusId.value = null;
};

// 删除确认弹窗状态
const showDeleteVolumeModal = ref(false);
const showDeleteChapterModal = ref(false);
const volumeToDelete = ref<VolumeWithChapters | null>(null);
const chapterToDelete = ref<{ id: number; title: string } | null>(null);

// Load chapter tree
const loadChapterTree = async () => {
  isLoading.value = true;
  try {
    const tree = await invoke<VolumeWithChapters[]>("get_chapter_tree", {
      projectId: Number(props.projectId),
    });
    volumes.value = tree;
  } catch (error) {
    console.error("加载章节树失败:", error);
  } finally {
    isLoading.value = false;
  }
};

// Create volume
const createVolume = async () => {
  try {
    const newVolume = await invoke<VolumeWithChapters>("create_volume", {
      projectId: Number(props.projectId),
      name: "新建卷",
    });
    // 后端返回的 Volume 缺少 chapters 数组，需要补充
    newVolume.chapters = [];
    volumes.value.push(newVolume);
    expandedVolumes.value.push(newVolume.id);
    startEditVolume(newVolume.id, newVolume.name);
  } catch (error) {
    console.error("创建卷失败:", error);
  }
};

// Create chapter with debounce protection
const isCreatingChapter = ref(false);
const creatingChapterTimeout = ref<number | null>(null);

// 实际创建章节的函数
const doCreateChapter = async (volumeId: number, initialContent?: string) => {
  try {
    const newChapter = await invoke<Chapter>("create_chapter", {
      projectId: Number(props.projectId),
      volumeId,
      title: "新建章节",
      initialContent: initialContent || null,
    });
    const volume = volumes.value.find((v) => v.id === volumeId);
    if (volume) {
      volume.chapters.push(newChapter);
    }
    // 创建后选择章节，触发编辑器加载模板内容
    selectChapter(newChapter);
    // 然后开始编辑标题
    startEditChapter(newChapter.id, newChapter.title);
  } catch (error) {
    console.error("创建章节失败:", error);
  }
};

const createChapter = (volumeId: number) => {
  // Clear any pending timeout
  if (creatingChapterTimeout.value) {
    clearTimeout(creatingChapterTimeout.value);
    creatingChapterTimeout.value = null;
  }
  
  // Prevent multiple simultaneous chapter creation
  if (isCreatingChapter.value) {
    return;
  }
  
  // Set a brief lock to prevent rapid re-triggers
  isCreatingChapter.value = true;
  
  // Small delay to allow any double-click browser event to be filtered
  creatingChapterTimeout.value = window.setTimeout(() => {
    // 显示模板选择器
    currentVolumeIdForTemplate.value = volumeId;
    showTemplateSelector.value = true;
    isCreatingChapter.value = false;
    creatingChapterTimeout.value = null;
  }, 100);
};

// 处理模板选择
const handleTemplateSelect = async (data: { content: string; mode: string } | string) => {
  if (!currentVolumeIdForTemplate.value) return;
  
  // 兼容两种调用方式：从 TemplateSelector 收到对象，或直接收到字符串
  const content = typeof data === 'string' ? data : data.content;
  
  showTemplateSelector.value = false;
  await doCreateChapter(currentVolumeIdForTemplate.value, content);
  currentVolumeIdForTemplate.value = null;
};

// 处理模板选择器关闭
const handleTemplateSelectorClose = () => {
  showTemplateSelector.value = false;
  currentVolumeIdForTemplate.value = null;
};

// Update volume name
const updateVolumeName = async (volumeId: number, newName: string) => {
  try {
    await invoke("update_volume_name", {
      volumeId,
      newName,
    });
    const volume = volumes.value.find((v) => v.id === volumeId);
    if (volume) {
      volume.name = newName;
    }
  } catch (error) {
    console.error("更新卷名失败:", error);
  }
};

// Update chapter title
const updateChapterTitle = async (chapterId: number, newTitle: string) => {
  try {
    await invoke("update_chapter_title", {
      chapterId,
      newTitle,
    });
    for (const volume of volumes.value) {
      const chapter = volume.chapters.find((c) => c.id === chapterId);
      if (chapter) {
        chapter.title = newTitle;
        break;
      }
    }
  } catch (error) {
    console.error("更新章节标题失败:", error);
  }
};

// Delete volume - 直接删除，不保留文件
const deleteVolume = async (volumeId: number) => {
  try {
    await invoke("delete_volume", { volumeId });
    volumes.value = volumes.value.filter((v) => v.id !== volumeId);
    message.success("卷已删除");
  } catch (error) {
    console.error("删除卷失败:", error);
    message.error("删除卷失败");
  }
};

// Delete chapter - 直接删除，不保留文件
const deleteChapter = async (chapterId: number) => {
  try {
    await invoke("delete_chapter", { chapterId, keepFile: false });
    for (const volume of volumes.value) {
      volume.chapters = volume.chapters.filter((c) => c.id !== chapterId);
    }
    if (selectedChapterId.value === chapterId) {
      selectedChapterId.value = null;
    }
    message.success("章节已删除");
  } catch (error) {
    console.error("删除章节失败:", error);
    message.error("删除章节失败");
  }
};

// 打开删除卷确认弹窗
const openDeleteVolumeModal = (volume: VolumeWithChapters) => {
  volumeToDelete.value = volume;
  showDeleteVolumeModal.value = true;
};

// 打开删除章节确认弹窗
const openDeleteChapterModal = (chapter: { id: number; title: string }) => {
  chapterToDelete.value = chapter;
  showDeleteChapterModal.value = true;
};

// 确认删除卷
const handleConfirmDeleteVolume = async () => {
  if (!volumeToDelete.value) return;
  await deleteVolume(volumeToDelete.value.id);
  showDeleteVolumeModal.value = false;
  volumeToDelete.value = null;
};

// 确认删除章节
const handleConfirmDeleteChapter = async (keepFile: boolean) => {
  if (!chapterToDelete.value) return;
  try {
    await invoke("delete_chapter", { chapterId: chapterToDelete.value.id, keepFile });
    for (const volume of volumes.value) {
      volume.chapters = volume.chapters.filter((c) => c.id !== chapterToDelete.value!.id);
    }
    if (selectedChapterId.value === chapterToDelete.value.id) {
      selectedChapterId.value = null;
    }
    message.success(keepFile ? "章节已从列表移除" : "章节已删除");
  } catch (error) {
    console.error("删除章节失败:", error);
    message.error("删除章节失败");
  }
  showDeleteChapterModal.value = false;
  chapterToDelete.value = null;
};

// Reorder volumes
const onVolumeReorder = async () => {
  try {
    const orderedIds = volumes.value.map((v) => v.id);
    await invoke("reorder_volumes", {
      projectId: Number(props.projectId),
      orderedIds,
    });
  } catch (error) {
    console.error("排序卷失败:", error);
    loadChapterTree(); // Reload on error
  }
};

// Reorder chapters within a volume
const onChapterReorder = async (volumeId: number) => {
  try {
    const volume = volumes.value.find((v) => v.id === volumeId);
    if (volume) {
      const orderedIds = volume.chapters.map((c) => c.id);
      await invoke("reorder_chapters", {
        volumeId,
        orderedIds,
      });
    }
  } catch (error) {
    console.error("排序章节失败:", error);
    loadChapterTree(); // Reload on error
  }
};

// Toggle volume expansion
const toggleVolume = (volumeId: number) => {
  const index = expandedVolumes.value.indexOf(volumeId);
  if (index === -1) {
    expandedVolumes.value.push(volumeId);
  } else {
    expandedVolumes.value.splice(index, 1);
  }
};

// Select chapter
const selectChapter = (chapter: Chapter) => {
  selectedChapterId.value = chapter.id;
  emit("select-chapter", chapter.id, chapter);
};

// Context menu handlers
const showContextMenu = (event: MouseEvent, type: "volume" | "chapter" | "empty", volumeId?: number, chapterId?: number) => {
  event.preventDefault();
  contextMenuVisible.value = true;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
  contextMenuType.value = type;
  contextMenuVolumeId.value = volumeId || null;
  contextMenuChapterId.value = chapterId || null;
};

const hideContextMenu = () => {
  contextMenuVisible.value = false;
};

// Context menu actions
const contextMenuOptions = computed(() => {
  if (contextMenuType.value === "volume" && contextMenuVolumeId.value !== null) {
    return [
      {
        label: "新增章节",
        key: "add-chapter",
        icon: () => null,
      },
      {
        label: "重命名",
        key: "rename",
        icon: () => null,
      },
      {
        label: "删除",
        key: "delete",
        icon: () => null,
      },
    ];
  } else if (contextMenuType.value === "chapter" && contextMenuChapterId.value !== null) {
    // Get current chapter status for menu
    let currentStatus: ChapterStatus = 'draft';
    for (const volume of volumes.value) {
      const chapter = volume.chapters.find((c) => c.id === contextMenuChapterId.value);
      if (chapter) {
        currentStatus = chapter.status;
        break;
      }
    }

    // Build status submenu
    const statusOptions = CHAPTER_STATUS_OPTIONS.map(opt => ({
      label: opt.label,
      key: `status-${opt.value}`,
      icon: () => null,
      disabled: opt.value === currentStatus,
    }));

    return [
      {
        label: "修改状态",
        key: "change-status",
        icon: () => null,
        children: statusOptions,
      },
      {
        label: "重命名",
        key: "rename",
        icon: () => null,
      },
      {
        label: "删除",
        key: "delete",
        icon: () => null,
      },
    ];
  } else {
    return [
      {
        label: "新增卷",
        key: "add-volume",
        icon: () => null,
      },
    ];
  }
});

const handleContextMenuSelect = async (key: string) => {
  hideContextMenu();

  // Handle status change
  if (key.startsWith('status-')) {
    const newStatus = key.replace('status-', '') as ChapterStatus;
    if (contextMenuChapterId.value !== null) {
      await updateChapterStatus(contextMenuChapterId.value, newStatus);
    }
    return;
  }

  switch (key) {
    case "add-volume":
      await createVolume();
      break;
    case "add-chapter":
      if (contextMenuVolumeId.value !== null) {
        await createChapter(contextMenuVolumeId.value);
      }
      break;
    case "change-status":
      // Open status dropdown
      break;
    case "rename":
      if (contextMenuType.value === "volume" && contextMenuVolumeId.value !== null) {
        const volume = volumes.value.find((v) => v.id === contextMenuVolumeId.value);
        if (volume) {
          startEditVolume(volume.id, volume.name);
        }
      } else if (contextMenuType.value === "chapter" && contextMenuChapterId.value !== null) {
        for (const volume of volumes.value) {
          const chapter = volume.chapters.find((c) => c.id === contextMenuChapterId.value);
          if (chapter) {
            startEditChapter(chapter.id, chapter.title);
            break;
          }
        }
      }
      break;
    case "delete":
      if (contextMenuType.value === "volume" && contextMenuVolumeId.value !== null) {
        const volume = volumes.value.find((v) => v.id === contextMenuVolumeId.value);
        if (volume) {
          openDeleteVolumeModal(volume);
        }
      } else if (contextMenuType.value === "chapter" && contextMenuChapterId.value !== null) {
        for (const volume of volumes.value) {
          const chapter = volume.chapters.find((c) => c.id === contextMenuChapterId.value);
          if (chapter) {
            openDeleteChapterModal({ id: chapter.id, title: chapter.title });
            break;
          }
        }
      }
      break;
  }
};

// Edit handlers
const startEditVolume = (volumeId: number, currentName: string) => {
  editingVolumeId.value = volumeId;
  editingVolumeName.value = currentName;
};

const finishEditVolume = () => {
  if (editingVolumeId.value !== null && editingVolumeName.value.trim()) {
    updateVolumeName(editingVolumeId.value, editingVolumeName.value.trim());
  }
  editingVolumeId.value = null;
  editingVolumeName.value = "";
};

const startEditChapter = (chapterId: number, currentTitle: string) => {
  editingChapterId.value = chapterId;
  editingChapterTitle.value = currentTitle;
};

const finishEditChapter = () => {
  if (editingChapterId.value !== null && editingChapterTitle.value.trim()) {
    updateChapterTitle(editingChapterId.value, editingChapterTitle.value.trim());
  }
  editingChapterId.value = null;
  editingChapterTitle.value = "";
};

// Watch for project ID changes
watch(
  () => props.projectId,
  () => {
    loadChapterTree();
  },
  { immediate: true }
);

// Initialize - expand first volume by default
onMounted(async () => {
  await loadChapterTree();
  if (volumes.value.length > 0 && expandedVolumes.value.length === 0) {
    expandedVolumes.value.push(volumes.value[0].id);
  }
});
</script>

<template>
  <div
    class="h-full flex flex-col bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700"
    @click="hideContextMenu"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
      <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-200">章节结构</h2>
      <div class="flex items-center gap-2">
        <!-- Status Filter -->
        <NButton size="tiny" quaternary @click="showStatusFilter = !showStatusFilter" :type="statusFilter !== 'all' ? 'primary' : undefined">
          <template #icon>
            <Filter class="w-4 h-4" />
          </template>
        </NButton>
        <NButton size="tiny" @click="createVolume">
          <template #icon>
            <Plus class="w-4 h-4" />
          </template>
        </NButton>
      </div>
    </div>

    <!-- Status Filter Bar -->
    <div v-if="showStatusFilter" class="px-4 py-2 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
      <div class="flex items-center gap-2 flex-wrap">
        <span class="text-xs text-gray-500">筛选:</span>
        <button
          v-for="opt in statusFilterOptions"
          :key="opt.value"
          class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full transition-colors"
          :class="statusFilter === opt.value
            ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
            : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'"
          @click="statusFilter = opt.value as ChapterStatus | 'all'"
        >
          <span
            v-if="opt.value !== 'all'"
            class="w-2 h-2 rounded-full"
            :style="{ backgroundColor: opt.color }"
          ></span>
          {{ opt.label }}
        </button>
      </div>
    </div>

    <!-- Tree Content -->
    <div class="flex-1 overflow-y-auto p-2">
      <NEmpty v-if="!isLoading && filteredVolumes.length === 0" description="暂无章节" class="py-8">
        <template #extra>
          <NButton v-if="volumes.length === 0" size="small" @click="createVolume">创建第一个卷</NButton>
          <span v-else class="text-sm text-gray-500">没有符合条件的章节</span>
        </template>
      </NEmpty>

      <!-- Volume list with drag -->
      <draggable
        v-model="volumes"
        item-key="id"
        handle=".volume-handle"
        ghost-class="ghost"
        @end="onVolumeReorder"
        class="space-y-1"
        v-if="statusFilter === 'all'"
      >
        <template #item="{ element: volume }">
          <div class="volume-item">
            <!-- Volume Header -->
            <div
              class="flex items-center gap-1 px-2 py-1.5 rounded-md cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 group"
              @contextmenu="showContextMenu($event, 'volume', volume.id)"
              @click="toggleVolume(volume.id)"
            >
              <GripVertical class="w-3 h-3 text-gray-400 opacity-0 group-hover:opacity-100 cursor-grab volume-handle" />
              
              <component
                :is="expandedVolumes.includes(volume.id) ? ChevronDown : ChevronRight"
                class="w-4 h-4 text-gray-500 shrink-0"
              />
              
              <FolderOpen class="w-4 h-4 text-blue-500 shrink-0" />
              
              <!-- Volume name (editable) -->
              <template v-if="editingVolumeId === volume.id">
                <NInput
                  v-model:value="editingVolumeName"
                  size="small"
                  class="flex-1"
                  autofocus
                  @click.stop
                  @blur="finishEditVolume"
                  @keyup.enter="finishEditVolume"
                  @keyup.escape="editingVolumeId = null"
                />
              </template>
              <template v-else>
                <span
                  class="flex-1 text-sm font-medium text-gray-700 dark:text-gray-200 truncate"
                  @dblclick.stop="startEditVolume(volume.id, volume.name)"
                >
                  {{ volume.name }}
                </span>
              </template>

              <!-- Volume actions -->
              <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100">
                <button
                  class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
                  @click.stop="createChapter(volume.id)"
                  title="新增章节"
                >
                  <Plus class="w-3 h-3 text-gray-500" />
                </button>
                <button
                  class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
                  @click.stop="showContextMenu($event, 'volume', volume.id)"
                  title="更多操作"
                >
                  <MoreVertical class="w-3 h-3 text-gray-500" />
                </button>
              </div>
            </div>

            <!-- Chapters -->
            <div v-if="expandedVolumes.includes(volume.id)" class="ml-4 mt-1 space-y-0.5">
              <draggable
                v-model="volume.chapters"
                item-key="id"
                handle=".chapter-handle"
                ghost-class="ghost"
                group="chapters"
                @end="onChapterReorder(volume.id)"
                class="space-y-0.5"
              >
                <template #item="{ element: chapter }">
                  <div
                    class="flex items-center gap-1 px-2 py-1 rounded-md cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 group"
                    :class="{ 'bg-blue-50 dark:bg-blue-900/30': selectedChapterId === chapter.id }"
                    @contextmenu="showContextMenu($event, 'chapter', volume.id, chapter.id)"
                    @click="selectChapter(chapter)"
                  >
                    <GripVertical class="w-3 h-3 text-gray-400 opacity-0 group-hover:opacity-100 cursor-grab chapter-handle" />
                    
                    <FileText class="w-4 h-4 text-gray-400 shrink-0" />

                    <!-- Status indicator dot -->
                    <span
                      class="w-2 h-2 rounded-full shrink-0 cursor-pointer hover:scale-125 transition-transform"
                      :style="{ backgroundColor: getStatusColor(chapter.status) }"
                      :title="`状态: ${getStatusLabel(chapter.status)}`"
                      @click.stop="startEditChapterStatus(chapter)"
                    ></span>

                    <!-- Chapter title (editable) -->
                    <template v-if="editingChapterId === chapter.id">
                      <NInput
                        v-model:value="editingChapterTitle"
                        size="small"
                        class="flex-1"
                        autofocus
                        @click.stop
                        @blur="finishEditChapter"
                        @keyup.enter="finishEditChapter"
                        @keyup.escape="editingChapterId = null"
                      />
                    </template>
                    <template v-else>
                      <span
                        class="flex-1 text-sm text-gray-600 dark:text-gray-300 truncate"
                        @dblclick.stop="startEditChapter(chapter.id, chapter.title)"
                      >
                        {{ chapter.title }}
                      </span>
                    </template>

                    <!-- Word count -->
                    <span class="text-xs text-gray-400">
                      {{ chapter.word_count_cache }}字
                    </span>

                    <!-- Status edit dropdown -->
                    <div v-if="editingChapterStatusId === chapter.id" class="relative">
                      <NSelect
                        v-model:value="editingChapterStatus"
                        size="tiny"
                        :options="CHAPTER_STATUS_OPTIONS.map(o => ({ label: o.label, value: o.value }))"
                        class="w-20"
                        autofocus
                        @click.stop
                        @update:value="confirmEditChapterStatus"
                        @blur="cancelEditChapterStatus"
                      />
                    </div>
                  </div>
                </template>
              </draggable>

              <!-- Add chapter button -->
              <button
                class="flex items-center gap-2 w-full px-2 py-1 text-sm text-gray-500 hover:text-gray-700 hover:bg-gray-100 dark:hover:text-gray-300 dark:hover:bg-gray-700 rounded-md"
                @click="createChapter(volume.id)"
              >
                <Plus class="w-4 h-4" />
                新增章节
              </button>
            </div>
          </div>
        </template>
      </draggable>

      <!-- Simple volume list when filter is active (no drag) -->
      <div v-if="statusFilter !== 'all'" class="space-y-1">
        <template v-for="volume in filteredVolumes" :key="volume.id">
          <div class="volume-item">
            <!-- Volume Header -->
            <div
              class="flex items-center gap-1 px-2 py-1.5 rounded-md cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 group"
              @click="toggleVolume(volume.id)"
            >
              <component
                :is="expandedVolumes.includes(volume.id) ? ChevronDown : ChevronRight"
                class="w-4 h-4 text-gray-500 shrink-0"
              />
              <FolderOpen class="w-4 h-4 text-blue-500 shrink-0" />
              <span class="flex-1 text-sm font-medium text-gray-700 dark:text-gray-200 truncate">
                {{ volume.name }}
              </span>
              <span class="text-xs text-gray-400">{{ volume.chapters.length }}章节</span>
            </div>

            <!-- Chapters -->
            <div v-if="expandedVolumes.includes(volume.id)" class="ml-4 mt-1 space-y-0.5">
              <div
                v-for="chapter in volume.chapters"
                :key="chapter.id"
                class="flex items-center gap-1 px-2 py-1 rounded-md cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 group"
                :class="{ 'bg-blue-50 dark:bg-blue-900/30': selectedChapterId === chapter.id }"
                @contextmenu="showContextMenu($event, 'chapter', volume.id, chapter.id)"
                @click="selectChapter(chapter)"
              >
                <FileText class="w-4 h-4 text-gray-400 shrink-0" />

                <!-- Status indicator dot -->
                <span
                  class="w-2 h-2 rounded-full shrink-0 cursor-pointer hover:scale-125 transition-transform"
                  :style="{ backgroundColor: getStatusColor(chapter.status) }"
                  :title="`状态: ${getStatusLabel(chapter.status)}`"
                  @click.stop="startEditChapterStatus(chapter)"
                ></span>

                <span class="flex-1 text-sm text-gray-600 dark:text-gray-300 truncate">
                  {{ chapter.title }}
                </span>

                <span class="text-xs text-gray-400">
                  {{ chapter.word_count_cache }}字
                </span>

                <!-- Status edit dropdown -->
                <div v-if="editingChapterStatusId === chapter.id" class="relative">
                  <NSelect
                    v-model:value="editingChapterStatus"
                    size="tiny"
                    :options="CHAPTER_STATUS_OPTIONS.map(o => ({ label: o.label, value: o.value }))"
                    class="w-20"
                    autofocus
                    @click.stop
                    @update:value="confirmEditChapterStatus"
                    @blur="cancelEditChapterStatus"
                  />
                </div>
              </div>
            </div>
          </div>
        </template>
      </div>

      <!-- Add volume button (when empty) -->
      <button
        v-if="volumes.length > 0"
        class="flex items-center gap-2 w-full mt-2 px-2 py-1.5 text-sm text-gray-500 hover:text-gray-700 hover:bg-gray-100 dark:hover:text-gray-300 dark:hover:bg-gray-700 rounded-md"
        @click="createVolume"
        @contextmenu="showContextMenu($event, 'empty')"
      >
        <Plus class="w-4 h-4" />
        新增卷
      </button>
    </div>

    <!-- Context Menu -->
    <NDropdown
      v-model:show="contextMenuVisible"
      trigger="manual"
      :x="contextMenuX"
      :y="contextMenuY"
      :options="contextMenuOptions"
      @select="handleContextMenuSelect"
      @click.stop
    />

    <!-- 删除卷确认弹窗 -->
    <DeleteConfirmModal
      v-model:show="showDeleteVolumeModal"
      title="确认删除卷"
      :message="`确定要删除卷 &quot;${volumeToDelete?.name}&quot; 吗？该卷下的所有章节也会被删除。`"
      confirm-text="删除"
      @confirm="handleConfirmDeleteVolume"
    />

    <!-- 删除章节确认弹窗 -->
    <DeleteConfirmModal
      v-model:show="showDeleteChapterModal"
      title="确认删除章节"
      :message="`确定要删除章节 &quot;${chapterToDelete?.title}&quot; 吗？`"
      confirm-text="删除"
      :show-keep-files="true"
      :default-keep-files="false"
      @confirm="handleConfirmDeleteChapter"
    />
    </div>
    
    <!-- 模板选择器 -->
    <TemplateSelector
      v-model:show="showTemplateSelector"
      :project-id="Number(projectId)"
      @select="handleTemplateSelect"
      @update:show="handleTemplateSelectorClose"
    />
  </template>

<style scoped>
.ghost {
  opacity: 0.5;
  background: var(--color-hover, #cce5ff);
}

.volume-item,
.chapter-item {
  user-select: none;
}
</style>
