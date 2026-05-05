<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessage } from "naive-ui";
import { NTree, NIcon, NButton, NDropdown, NInput, NPopconfirm, NEmpty } from "naive-ui";
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
} from "lucide-vue-next";
import draggable from "vuedraggable";
import DeleteConfirmModal from "./DeleteConfirmModal.vue";

// Types
interface Chapter {
  id: number;
  volume_id: number;
  title: string;
  file_path: string;
  sort_order: number;
  summary: string;
  word_count_cache: number;
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

const createChapter = async (volumeId: number) => {
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
  creatingChapterTimeout.value = window.setTimeout(async () => {
    try {
      const newChapter = await invoke<Chapter>("create_chapter", {
        projectId: Number(props.projectId),
        volumeId,
        title: "新建章节",
      });
      const volume = volumes.value.find((v) => v.id === volumeId);
      if (volume) {
        volume.chapters.push(newChapter);
      }
      startEditChapter(newChapter.id, newChapter.title);
    } catch (error) {
      console.error("创建章节失败:", error);
    } finally {
      isCreatingChapter.value = false;
      creatingChapterTimeout.value = null;
    }
  }, 100);
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
  } else if (contextMenuType.value === "chapter") {
    return [
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

  switch (key) {
    case "add-volume":
      await createVolume();
      break;
    case "add-chapter":
      if (contextMenuVolumeId.value !== null) {
        await createChapter(contextMenuVolumeId.value);
      }
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
      <NButton size="tiny" @click="createVolume">
        <template #icon>
          <Plus class="w-4 h-4" />
        </template>
      </NButton>
    </div>

    <!-- Tree Content -->
    <div class="flex-1 overflow-y-auto p-2">
      <NEmpty v-if="!isLoading && volumes.length === 0" description="暂无章节" class="py-8">
        <template #extra>
          <NButton size="small" @click="createVolume">创建第一个卷</NButton>
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
                class="w-4 h-4 text-gray-500 flex-shrink-0"
              />
              
              <FolderOpen class="w-4 h-4 text-blue-500 flex-shrink-0" />
              
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
                    
                    <FileText class="w-4 h-4 text-gray-400 flex-shrink-0" />
                    
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
</template>

<style scoped>
.ghost {
  opacity: 0.5;
  background: #cce5ff;
}

.volume-item,
.chapter-item {
  user-select: none;
}
</style>
