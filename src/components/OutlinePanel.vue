<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NInput, NEmpty, NButton, NIcon, NPopconfirm, NTooltip } from "naive-ui";
import {
  ChevronDown,
  ChevronRight,
  FileText,
  Edit3,
  Trash2,
  AlignLeft,
} from "lucide-vue-next";
import draggable from "vuedraggable";
import { useTheme } from "../composables/useTheme";

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
  selectedChapterId?: number | null;
}>();

// Emits
const emit = defineEmits<{
  (e: "select-chapter", chapterId: number, chapter: Chapter): void;
  (e: "update"): void;
}>();

// Theme
const { isDark } = useTheme();

// State
const volumes = ref<VolumeWithChapters[]>([]);
const isLoading = ref(true);
const expandedVolumes = ref<number[]>([]);

// Edit state
const editingChapterId = ref<number | null>(null);
const editingSummary = ref("");

// Load chapter tree
const loadChapterTree = async () => {
  isLoading.value = true;
  try {
    const tree = await invoke<VolumeWithChapters[]>("get_chapter_tree", {
      projectId: Number(props.projectId),
    });
    volumes.value = tree;
    // Auto expand all volumes
    expandedVolumes.value = tree.map((v) => v.id);
  } catch (error) {
    console.error("加载章节树失败:", error);
  } finally {
    isLoading.value = false;
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
  emit("select-chapter", chapter.id, chapter);
};

// Start editing summary
const startEditSummary = (chapter: Chapter, event: Event) => {
  event.stopPropagation();
  editingChapterId.value = chapter.id;
  editingSummary.value = chapter.summary;
};

// Finish editing summary
const finishEditSummary = async () => {
  if (editingChapterId.value === null) return;

  const chapterId = editingChapterId.value;
  const newSummary = editingSummary.value.trim();

  // Find current chapter
  let currentChapter: Chapter | undefined;
  for (const volume of volumes.value) {
    const ch = volume.chapters.find((c) => c.id === chapterId);
    if (ch) {
      currentChapter = ch;
      break;
    }
  }

  // Only update if changed
  if (currentChapter && currentChapter.summary !== newSummary) {
    try {
      await invoke("update_chapter_summary", {
        chapterId,
        newSummary,
      });
      currentChapter.summary = newSummary;
      emit("update");
    } catch (error) {
      console.error("更新摘要失败:", error);
    }
  }

  editingChapterId.value = null;
  editingSummary.value = "";
};

// Handle key events in summary input
const handleSummaryKeydown = (event: KeyboardEvent) => {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    finishEditSummary();
  } else if (event.key === "Escape") {
    editingChapterId.value = null;
    editingSummary.value = "";
  }
};

// Delete chapter
const deleteChapter = async (chapterId: number) => {
  try {
    await invoke("delete_chapter", { chapterId });
    for (const volume of volumes.value) {
      volume.chapters = volume.chapters.filter((c) => c.id !== chapterId);
    }
    emit("update");
  } catch (error) {
    console.error("删除章节失败:", error);
  }
};

// Delete volume
const deleteVolume = async (volumeId: number) => {
  try {
    await invoke("delete_volume", { volumeId });
    volumes.value = volumes.value.filter((v) => v.id !== volumeId);
    emit("update");
  } catch (error) {
    console.error("删除卷失败:", error);
  }
};

// Handle chapter drag end
const onChapterDragEnd = async (volumeId: number) => {
  const volume = volumes.value.find((v) => v.id === volumeId);
  if (!volume) return;

  try {
    const orderedIds = volume.chapters.map((c) => c.id);
    await invoke("reorder_chapters", {
      volumeId,
      orderedIds,
    });
    emit("update");
  } catch (error) {
    console.error("排序章节失败:", error);
    loadChapterTree();
  }
};

// Handle volume drag end
const onVolumeDragEnd = async () => {
  try {
    const orderedIds = volumes.value.map((v) => v.id);
    await invoke("reorder_volumes", {
      projectId: Number(props.projectId),
      orderedIds,
    });
    emit("update");
  } catch (error) {
    console.error("排序卷失败:", error);
    loadChapterTree();
  }
};

// Watch for project ID changes
watch(
  () => props.projectId,
  () => {
    loadChapterTree();
  },
  { immediate: true }
);
</script>

<template>
  <div
    class="h-full flex flex-col bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700"
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700"
    >
      <h2 class="text-sm font-semibold text-gray-700 dark:text-gray-200 flex items-center gap-2">
        <AlignLeft class="w-4 h-4" />
        大纲视图
      </h2>
    </div>

    <!-- Tree Content -->
    <div class="flex-1 overflow-y-auto p-2">
      <NEmpty
        v-if="!isLoading && volumes.length === 0"
        description="暂无章节"
        class="py-8"
      />

      <!-- Volume list with drag -->
      <draggable
        v-model="volumes"
        item-key="id"
        handle=".volume-handle"
        ghost-class="ghost"
        @end="onVolumeDragEnd"
        class="space-y-2"
      >
        <template #item="{ element: volume }">
          <div class="volume-item">
            <!-- Volume Header -->
            <div
              class="flex items-center gap-1 px-2 py-2 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 group"
              :class="isDark ? 'bg-gray-800' : 'bg-gray-50'"
              @click="toggleVolume(volume.id)"
            >
              <component
                :is="expandedVolumes.includes(volume.id) ? ChevronDown : ChevronRight"
                class="w-4 h-4 text-gray-500 flex-shrink-0 cursor-grab volume-handle"
              />

              <span
                class="flex-1 text-sm font-semibold text-gray-700 dark:text-gray-200 truncate"
              >
                {{ volume.name }}
              </span>

              <span
                class="text-xs text-gray-400 px-1"
              >
                {{ volume.chapters.length }}章
              </span>

              <!-- Delete volume -->
              <NPopconfirm
                @positive-click="() => deleteVolume(volume.id)"
              >
                <template #trigger>
                  <button
                    class="p-1 rounded opacity-0 group-hover:opacity-100 hover:bg-red-100 dark:hover:bg-red-900/30"
                    @click.stop
                    title="删除卷"
                  >
                    <Trash2 class="w-3 h-3 text-red-500" />
                  </button>
                </template>
                确定删除"{{ volume.name }}"及其所有章节？
              </NPopconfirm>
            </div>

            <!-- Chapters -->
            <div
              v-if="expandedVolumes.includes(volume.id)"
              class="ml-4 mt-1 space-y-1"
            >
              <draggable
                v-model="volume.chapters"
                item-key="id"
                handle=".chapter-handle"
                ghost-class="ghost"
                @end="() => onChapterDragEnd(volume.id)"
                class="space-y-1"
              >
                <template #item="{ element: chapter }">
                  <div
                    class="flex flex-col gap-1 px-3 py-2 rounded-lg cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 group"
                    :class="{
                      'bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-800': selectedChapterId === chapter.id,
                    }"
                    @click="selectChapter(chapter)"
                  >
                    <div class="flex items-center gap-2">
                      <GripVertical class="w-3 h-3 text-gray-400 opacity-0 group-hover:opacity-100 cursor-grab chapter-handle flex-shrink-0" />
                      
                      <FileText class="w-4 h-4 text-gray-400 flex-shrink-0" />

                      <span
                        class="flex-1 text-sm font-medium text-gray-700 dark:text-gray-200 truncate"
                      >
                        {{ chapter.title }}
                      </span>

                      <span class="text-xs text-gray-400 flex-shrink-0">
                        {{ chapter.word_count_cache }}字
                      </span>
                    </div>

                    <!-- Summary section -->
                    <div class="ml-6">
                      <template v-if="editingChapterId === chapter.id">
                        <NInput
                          v-model:value="editingSummary"
                          type="textarea"
                          :autosize="{ minRows: 1, maxRows: 3 }"
                          placeholder="输入章节摘要..."
                          autofocus
                          @blur="finishEditSummary"
                          @keydown="handleSummaryKeydown"
                          @click.stop
                        />
                      </template>
                      <template v-else>
                        <div
                          class="text-xs text-gray-500 dark:text-gray-400 cursor-pointer hover:text-blue-500 dark:hover:text-blue-400 line-clamp-2"
                          :class="{ 'text-gray-400 italic': !chapter.summary }"
                          @click="startEditSummary(chapter, $event)"
                        >
                          {{ chapter.summary || '点击添加摘要...' }}
                        </div>
                      </template>
                    </div>
                  </div>
                </template>
              </draggable>
            </div>
          </div>
        </template>
      </draggable>
    </div>
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

.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
