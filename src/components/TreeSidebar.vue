<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessage } from "naive-ui";
import {
  NTree,
  NIcon,
  NButton,
  NDropdown,
  NInput,
  NPopconfirm,
  NEmpty,
  NSelect,
} from "naive-ui";
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
import {
  CHAPTER_STATUS_OPTIONS,
  getStatusColor,
  getStatusLabel,
  type ChapterStatus,
} from "../types/chapter";
import { useLocale } from "../i18n/composables/useLocale";

const { t } = useLocale();

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
const statusFilter = ref<ChapterStatus | "all">("all");
const showStatusFilter = ref(false);

// 模板选择器状态
const showTemplateSelector = ref(false);
const currentVolumeIdForTemplate = ref<number | null>(null);

const statusFilterOptions = computed(() => {
  const base: Array<{ label: string; value: string; color?: string }> = [
    { label: t("treeSidebar.allChapters"), value: "all" },
  ];
  const hardcoded: Array<{ label: string; value: string; color: string }> = [
    {
      label: t("treeSidebar.status.outline"),
      value: "outline",
      color: "#9CA3AF",
    },
    { label: t("treeSidebar.status.draft"), value: "draft", color: "#F59E0B" },
    {
      label: t("treeSidebar.status.revised"),
      value: "revised",
      color: "#3B82F6",
    },
    { label: t("treeSidebar.status.final"), value: "final", color: "#10B981" },
    {
      label: t("treeSidebar.status.abandoned"),
      value: "abandoned",
      color: "#EF4444",
    },
  ];
  return [...base, ...hardcoded];
});

// Filtered volumes based on status filter
const filteredVolumes = computed(() => {
  if (statusFilter.value === "all") {
    return volumes.value;
  }
  return volumes.value
    .map((volume) => ({
      ...volume,
      chapters: volume.chapters.filter(
        (chapter) => chapter.status === statusFilter.value
      ),
    }))
    .filter((volume) => volume.chapters.length > 0);
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
const editingChapterStatus = ref<ChapterStatus>("draft");

// Update chapter status
const updateChapterStatus = async (
  chapterId: number,
  newStatus: ChapterStatus
) => {
  try {
    await invoke("update_chapter_status", {
      chapter_id: chapterId,
      status: newStatus,
    });
    for (const volume of volumes.value) {
      const chapter = volume.chapters.find((c) => c.id === chapterId);
      if (chapter) {
        chapter.status = newStatus;
        break;
      }
    }
    message.success(
      t("treeSidebar.statusUpdated") + `: ${getStatusLabel(newStatus)}`
    );
  } catch (error) {
    console.error(t("treeSidebar.updateStatusFailed") + ":", error);
    message.error(t("treeSidebar.updateStatusFailed"));
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
    updateChapterStatus(
      editingChapterStatusId.value,
      editingChapterStatus.value
    );
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
      project_id: Number(props.projectId),
    });
    volumes.value = tree;
  } catch (error) {
    console.error(t("treeSidebar.loadTreeFailed") + ":", error);
  } finally {
    isLoading.value = false;
  }
};

// Create volume
const createVolume = async () => {
  try {
    const newVolume = await invoke<VolumeWithChapters>("create_volume", {
      project_id: Number(props.projectId),
      name: t("treeSidebar.newVolume"),
    });
    // 后端返回的 Volume 缺少 chapters 数组，需要补充
    newVolume.chapters = [];
    volumes.value.push(newVolume);
    expandedVolumes.value.push(newVolume.id);
    startEditVolume(newVolume.id, newVolume.name);
  } catch (error) {
    console.error(t("treeSidebar.createVolumeFailed") + ":", error);
  }
};

// Create chapter with debounce protection
const isCreatingChapter = ref(false);
const creatingChapterTimeout = ref<number | null>(null);

// 实际创建章节的函数
const doCreateChapter = async (
  volumeId: number,
  initialContent?: string,
  _mode?: string
) => {
  try {
    const newChapter = await invoke<Chapter>("create_chapter", {
      project_id: Number(props.projectId),
      volume_id: volumeId,
      title: t("treeSidebar.newChapter"),
      initial_content: initialContent || null,
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
    console.error(t("treeSidebar.createChapterFailed") + ":", error);
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
const handleTemplateSelect = async (
  data: { content: string; mode: string } | string
) => {
  if (!currentVolumeIdForTemplate.value) return;

  // 兼容两种调用方式：从 TemplateSelector 收到对象，或直接收到字符串
  const templateData =
    typeof data === "string" ? { content: data, mode: "replace" } : data;

  showTemplateSelector.value = false;
  await doCreateChapter(
    currentVolumeIdForTemplate.value,
    templateData.content,
    templateData.mode
  );
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
      volume_id: volumeId,
      new_name: newName,
    });
    const volume = volumes.value.find((v) => v.id === volumeId);
    if (volume) {
      volume.name = newName;
    }
  } catch (error) {
    console.error(t("treeSidebar.updateVolumeNameFailed") + ":", error);
  }
};

// Update chapter title
const updateChapterTitle = async (chapterId: number, newTitle: string) => {
  try {
    await invoke("update_chapter_title", {
      chapter_id: chapterId,
      new_title: newTitle,
    });
    for (const volume of volumes.value) {
      const chapter = volume.chapters.find((c) => c.id === chapterId);
      if (chapter) {
        chapter.title = newTitle;
        break;
      }
    }
  } catch (error) {
    console.error(t("treeSidebar.updateChapterTitleFailed") + ":", error);
  }
};

// Delete volume - 直接删除，不保留文件
const deleteVolume = async (volumeId: number) => {
  try {
    await invoke("delete_volume", { volume_id: volumeId });
    volumes.value = volumes.value.filter((v) => v.id !== volumeId);
    message.success(t("treeSidebar.volumeDeleted"));
  } catch (error) {
    console.error(t("treeSidebar.deleteVolumeFailed") + ":", error);
    message.error(t("treeSidebar.deleteVolumeFailed"));
  }
};

// Delete chapter - 直接删除，不保留文件
const deleteChapter = async (chapterId: number) => {
  try {
    await invoke("delete_chapter", { chapter_id: chapterId, keep_file: false });
    for (const volume of volumes.value) {
      volume.chapters = volume.chapters.filter((c) => c.id !== chapterId);
    }
    if (selectedChapterId.value === chapterId) {
      selectedChapterId.value = null;
    }
    message.success(t("treeSidebar.chapterDeleted"));
  } catch (error) {
    console.error(t("treeSidebar.deleteChapterFailed") + ":", error);
    message.error(t("treeSidebar.deleteChapterFailed"));
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
    await invoke("delete_chapter", {
      chapter_id: chapterToDelete.value.id,
      keep_file: keepFile,
    });
    for (const volume of volumes.value) {
      volume.chapters = volume.chapters.filter(
        (c) => c.id !== chapterToDelete.value!.id
      );
    }
    if (selectedChapterId.value === chapterToDelete.value.id) {
      selectedChapterId.value = null;
    }
    message.success(
      keepFile
        ? t("treeSidebar.chapterRemoved")
        : t("treeSidebar.chapterDeleted")
    );
  } catch (error) {
    console.error(t("treeSidebar.deleteChapterFailed") + ":", error);
    message.error(t("treeSidebar.deleteChapterFailed"));
  }
  showDeleteChapterModal.value = false;
  chapterToDelete.value = null;
};

// Reorder volumes
const onVolumeReorder = async () => {
  try {
    const orderedIds = volumes.value.map((v) => v.id);
    await invoke("reorder_volumes", {
      project_id: Number(props.projectId),
      ordered_ids: orderedIds,
    });
  } catch (error) {
    console.error(t("treeSidebar.reorderVolumesFailed") + ":", error);
    loadChapterTree();
  }
};

// Reorder chapters within a volume
const onChapterReorder = async (volumeId: number) => {
  try {
    const volume = volumes.value.find((v) => v.id === volumeId);
    if (volume) {
      const orderedIds = volume.chapters.map((c) => c.id);
      await invoke("reorder_chapters", {
        volume_id: volumeId,
        ordered_ids: orderedIds,
      });
    }
  } catch (error) {
    console.error(t("treeSidebar.reorderChaptersFailed") + ":", error);
    loadChapterTree();
  }
};

// Handle chapter added from another volume (cross-volume drag)
const onChapterAdd = async (
  evt: {
    item: HTMLElement;
    from: HTMLElement;
    to: HTMLElement;
    newIndex: number;
  },
  targetVolumeId: number
) => {
  try {
    const targetVolume = volumes.value.find((v) => v.id === targetVolumeId);
    if (!targetVolume || evt.newIndex >= targetVolume.chapters.length) return;

    const movedChapter = targetVolume.chapters[evt.newIndex];
    const sourceVolumeId = movedChapter.volume_id;

    if (sourceVolumeId !== targetVolumeId) {
      await invoke("move_chapter_to_volume", {
        chapter_id: movedChapter.id,
        target_volume_id: targetVolumeId,
        sort_order: evt.newIndex,
      });
      movedChapter.volume_id = targetVolumeId;

      const sourceVolume = volumes.value.find((v) => v.id === sourceVolumeId);
      if (sourceVolume) {
        const sourceOrderedIds = sourceVolume.chapters.map((c) => c.id);
        if (sourceOrderedIds.length > 0) {
          await invoke("reorder_chapters", {
            volume_id: sourceVolumeId,
            ordered_ids: sourceOrderedIds,
          });
        }
      }
    }

    const orderedIds = targetVolume.chapters.map((c) => c.id);
    await invoke("reorder_chapters", {
      volume_id: targetVolumeId,
      ordered_ids: orderedIds,
    });
  } catch (error) {
    console.error(t("treeSidebar.moveChapterFailed") + ":", error);
    loadChapterTree();
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
const showContextMenu = (
  event: MouseEvent,
  type: "volume" | "chapter" | "empty",
  volumeId?: number,
  chapterId?: number
) => {
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
  if (
    contextMenuType.value === "volume" &&
    contextMenuVolumeId.value !== null
  ) {
    return [
      {
        label: t("treeSidebar.addChapter"),
        key: "add-chapter",
        icon: () => null,
      },
      {
        label: t("treeSidebar.rename"),
        key: "rename",
        icon: () => null,
      },
      {
        label: t("treeSidebar.delete"),
        key: "delete",
        icon: () => null,
      },
    ];
  } else if (
    contextMenuType.value === "chapter" &&
    contextMenuChapterId.value !== null
  ) {
    // Get current chapter status for menu
    let currentStatus: ChapterStatus = "draft";
    for (const volume of volumes.value) {
      const chapter = volume.chapters.find(
        (c) => c.id === contextMenuChapterId.value
      );
      if (chapter) {
        currentStatus = chapter.status;
        break;
      }
    }

    // Build status submenu
    const statusOptions = CHAPTER_STATUS_OPTIONS.map((opt) => ({
      label: opt.label,
      key: `status-${opt.value}`,
      icon: () => null,
      disabled: opt.value === currentStatus,
    }));

    return [
      {
        label: t("treeSidebar.changeStatus"),
        key: "change-status",
        icon: () => null,
        children: statusOptions,
      },
      {
        label: t("treeSidebar.rename"),
        key: "rename",
        icon: () => null,
      },
      {
        label: t("treeSidebar.delete"),
        key: "delete",
        icon: () => null,
      },
    ];
  } else {
    return [
      {
        label: t("treeSidebar.addVolume"),
        key: "add-volume",
        icon: () => null,
      },
    ];
  }
});

const handleContextMenuSelect = async (key: string) => {
  hideContextMenu();

  // Handle status change
  if (key.startsWith("status-")) {
    const newStatus = key.replace("status-", "") as ChapterStatus;
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
      if (
        contextMenuType.value === "volume" &&
        contextMenuVolumeId.value !== null
      ) {
        const volume = volumes.value.find(
          (v) => v.id === contextMenuVolumeId.value
        );
        if (volume) {
          startEditVolume(volume.id, volume.name);
        }
      } else if (
        contextMenuType.value === "chapter" &&
        contextMenuChapterId.value !== null
      ) {
        for (const volume of volumes.value) {
          const chapter = volume.chapters.find(
            (c) => c.id === contextMenuChapterId.value
          );
          if (chapter) {
            startEditChapter(chapter.id, chapter.title);
            break;
          }
        }
      }
      break;
    case "delete":
      if (
        contextMenuType.value === "volume" &&
        contextMenuVolumeId.value !== null
      ) {
        const volume = volumes.value.find(
          (v) => v.id === contextMenuVolumeId.value
        );
        if (volume) {
          openDeleteVolumeModal(volume);
        }
      } else if (
        contextMenuType.value === "chapter" &&
        contextMenuChapterId.value !== null
      ) {
        for (const volume of volumes.value) {
          const chapter = volume.chapters.find(
            (c) => c.id === contextMenuChapterId.value
          );
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
    updateChapterTitle(
      editingChapterId.value,
      editingChapterTitle.value.trim()
    );
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
    <div
      class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700"
    >
      <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-200">
        {{ t("treeSidebar.title") }}
      </h2>
      <div class="flex items-center gap-2">
        <!-- Status Filter -->
        <NButton
          size="tiny"
          quaternary
          @click="showStatusFilter = !showStatusFilter"
          :type="statusFilter !== 'all' ? 'primary' : undefined"
        >
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
    <div
      v-if="showStatusFilter"
      class="px-4 py-2 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800"
    >
      <div class="flex items-center gap-2 flex-wrap">
        <span class="text-xs text-gray-500">{{ t("treeSidebar.filter") }}</span>
        <button
          v-for="opt in statusFilterOptions"
          :key="opt.value"
          class="flex items-center gap-1 px-2 py-0.5 text-xs rounded-full transition-colors"
          :class="
            statusFilter === opt.value
              ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300'
              : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
          "
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
      <NEmpty
        v-if="!isLoading && filteredVolumes.length === 0"
        :description="t('treeSidebar.noChapters')"
        class="py-8"
      >
        <template #extra>
          <NButton
            v-if="volumes.length === 0"
            size="small"
            @click="createVolume"
            >{{ t("treeSidebar.createFirstVolume") }}</NButton
          >
          <span v-else class="text-sm text-gray-500">{{
            t("treeSidebar.noMatchingChapters")
          }}</span>
        </template>
      </NEmpty>

      <!-- Volume list with drag -->
      <draggable
        v-model="volumes"
        item-key="id"
        handle=".volume-handle"
        ghost-class="drag-ghost"
        chosen-class="drag-chosen"
        drag-class="drag-dragging"
        animation="200"
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
              <GripVertical
                class="w-3.5 h-3.5 text-gray-400 opacity-40 group-hover:opacity-100 cursor-grab volume-handle transition-opacity"
              />

              <component
                :is="
                  expandedVolumes.includes(volume.id)
                    ? ChevronDown
                    : ChevronRight
                "
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
              <div
                class="flex items-center gap-1 opacity-0 group-hover:opacity-100"
              >
                <button
                  class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
                  @click.stop="createChapter(volume.id)"
                  :title="t('treeSidebar.addChapter')"
                >
                  <Plus class="w-3 h-3 text-gray-500" />
                </button>
                <button
                  class="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600"
                  @click.stop="showContextMenu($event, 'volume', volume.id)"
                  :title="t('treeSidebar.moreActions')"
                >
                  <MoreVertical class="w-3 h-3 text-gray-500" />
                </button>
              </div>
            </div>

            <!-- Chapters -->
            <div
              v-if="expandedVolumes.includes(volume.id)"
              class="ml-4 mt-1 space-y-0.5"
            >
              <draggable
                v-model="volume.chapters"
                item-key="id"
                handle=".chapter-handle"
                ghost-class="drag-ghost"
                chosen-class="drag-chosen"
                drag-class="drag-dragging"
                group="chapters"
                animation="200"
                @end="onChapterReorder(volume.id)"
                @add="onChapterAdd($event, volume.id)"
                class="space-y-0.5"
              >
                <template #item="{ element: chapter }">
                  <div
                    class="flex items-center gap-1 px-2 py-1 rounded-md cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 group"
                    :class="{
                      'bg-blue-50 dark:bg-blue-900/30':
                        selectedChapterId === chapter.id,
                    }"
                    @contextmenu="
                      showContextMenu($event, 'chapter', volume.id, chapter.id)
                    "
                    @click="selectChapter(chapter)"
                  >
                    <GripVertical
                      class="w-3.5 h-3.5 text-gray-400 opacity-40 group-hover:opacity-100 cursor-grab chapter-handle transition-opacity"
                    />

                    <FileText class="w-4 h-4 text-gray-400 shrink-0" />

                    <!-- Status indicator dot -->
                    <span
                      class="w-2 h-2 rounded-full shrink-0 cursor-pointer hover:scale-125 transition-transform"
                      :style="{
                        backgroundColor: getStatusColor(chapter.status),
                      }"
                      :title="
                        t('treeSidebar.statusTitle', {
                          status: getStatusLabel(chapter.status),
                        })
                      "
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
                        @dblclick.stop="
                          startEditChapter(chapter.id, chapter.title)
                        "
                      >
                        {{ chapter.title }}
                      </span>
                    </template>

                    <!-- Word count -->
                    <span class="text-xs text-gray-400">
                      {{ chapter.word_count_cache
                      }}{{ t("treeSidebar.wordCountSuffix") }}
                    </span>

                    <!-- Status edit dropdown -->
                    <div
                      v-if="editingChapterStatusId === chapter.id"
                      class="relative"
                    >
                      <NSelect
                        v-model:value="editingChapterStatus"
                        size="tiny"
                        :options="
                          CHAPTER_STATUS_OPTIONS.map((o) => ({
                            label: o.label,
                            value: o.value,
                          }))
                        "
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
                {{ t("treeSidebar.addChapter") }}
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
                :is="
                  expandedVolumes.includes(volume.id)
                    ? ChevronDown
                    : ChevronRight
                "
                class="w-4 h-4 text-gray-500 shrink-0"
              />
              <FolderOpen class="w-4 h-4 text-blue-500 shrink-0" />
              <span
                class="flex-1 text-sm font-medium text-gray-700 dark:text-gray-200 truncate"
              >
                {{ volume.name }}
              </span>
              <span class="text-xs text-gray-400"
                >{{ volume.chapters.length
                }}{{ t("treeSidebar.chapterCountSuffix") }}</span
              >
            </div>

            <!-- Chapters -->
            <div
              v-if="expandedVolumes.includes(volume.id)"
              class="ml-4 mt-1 space-y-0.5"
            >
              <div
                v-for="chapter in volume.chapters"
                :key="chapter.id"
                class="flex items-center gap-1 px-2 py-1 rounded-md cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 group"
                :class="{
                  'bg-blue-50 dark:bg-blue-900/30':
                    selectedChapterId === chapter.id,
                }"
                @contextmenu="
                  showContextMenu($event, 'chapter', volume.id, chapter.id)
                "
                @click="selectChapter(chapter)"
              >
                <FileText class="w-4 h-4 text-gray-400 shrink-0" />

                <!-- Status indicator dot -->
                <span
                  class="w-2 h-2 rounded-full shrink-0 cursor-pointer hover:scale-125 transition-transform"
                  :style="{ backgroundColor: getStatusColor(chapter.status) }"
                  :title="
                    t('treeSidebar.statusTitle', {
                      status: getStatusLabel(chapter.status),
                    })
                  "
                  @click.stop="startEditChapterStatus(chapter)"
                ></span>

                <span
                  class="flex-1 text-sm text-gray-600 dark:text-gray-300 truncate"
                >
                  {{ chapter.title }}
                </span>

                <span class="text-xs text-gray-400">
                  {{ chapter.word_count_cache
                  }}{{ t("treeSidebar.wordCountSuffix") }}
                </span>

                <!-- Status edit dropdown -->
                <div
                  v-if="editingChapterStatusId === chapter.id"
                  class="relative"
                >
                  <NSelect
                    v-model:value="editingChapterStatus"
                    size="tiny"
                    :options="
                      CHAPTER_STATUS_OPTIONS.map((o) => ({
                        label: o.label,
                        value: o.value,
                      }))
                    "
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
        {{ t("treeSidebar.addVolume") }}
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
      :title="t('treeSidebar.deleteVolumeTitle')"
      :message="
        t('treeSidebar.deleteVolumeMessage', { name: volumeToDelete?.name })
      "
      :confirm-text="t('treeSidebar.delete')"
      @confirm="handleConfirmDeleteVolume"
    />

    <!-- 删除章节确认弹窗 -->
    <DeleteConfirmModal
      v-model:show="showDeleteChapterModal"
      :title="t('treeSidebar.deleteChapterTitle')"
      :message="
        t('treeSidebar.deleteChapterMessage', { title: chapterToDelete?.title })
      "
      :confirm-text="t('treeSidebar.delete')"
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
.drag-ghost {
  opacity: 0.3;
  background: #e0e7ff;
  border: 1px dashed #6366f1;
  border-radius: 6px;
}

.drag-chosen {
  background: #ede9fe;
  border-radius: 6px;
}

.drag-dragging {
  opacity: 0.85;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-radius: 6px;
  z-index: 9999;
}

:deep(.dark) .drag-ghost {
  background: #312e81;
  border-color: #818cf8;
}

:deep(.dark) .drag-chosen {
  background: #4c1d95;
}

.volume-item,
.chapter-item {
  user-select: none;
}

.sortable-ghost {
  opacity: 0.3;
}
</style>
