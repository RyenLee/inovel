<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useProjectStore } from "../stores/project";
import { useEditorStore } from "../stores/editor";
import { NButton, NIcon, NProgress, useMessage, NTooltip, NModal, NSelect, NInputNumber, NRadioGroup, NRadio, NTag } from "naive-ui";
import { ArrowLeft, Save, FileText, Sun, Moon, ChevronLeft, ChevronRight, Target, Settings, BarChart3, User, Globe, X, GitBranch, AlertTriangle, Download, Package, Maximize2, Minimize2, Sparkles, Keyboard } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import TreeSidebar from "../components/TreeSidebar.vue";
import OutlinePanel from "../components/OutlinePanel.vue";
import MarkdownEditor from "../components/MarkdownEditor.vue";
import WorldbuildingPanel from "../components/WorldbuildingPanel.vue";
import RelationshipGraph from "../components/RelationshipGraph.vue";
import Timeline from "../components/Timeline.vue";
import HistoryDialog from "../components/HistoryDialog.vue";
import SensitiveWordsManager from "../components/SensitiveWordsManager.vue";
import ExportDialog from "../components/ExportDialog.vue";
import BackupDialog from "../components/BackupDialog.vue";
import ShortcutSettings from "../components/ShortcutSettings.vue";
import PomodoroTimer from "../components/PomodoroTimer.vue";
import InspirationBoard from "../components/InspirationBoard.vue";
import { Timer, Lightbulb } from "lucide-vue-next";
import { useTheme } from "../composables/useTheme";

const { isDark, toggleDark } = useTheme();

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

const route = useRoute();
const router = useRouter();
const projectStore = useProjectStore();
const editorStore = useEditorStore();
const message = useMessage();

const currentContent = ref("");
const isLoading = ref(true);
const currentChapter = ref<Chapter | null>(null);
const autoSaveTimer = ref<number | null>(null);
const isSaving = ref(false);
const showSidebar = ref(true);
const showWorldbuilding = ref(false); // 右侧世界观面板
const sidebarMode = ref<"tree" | "outline">("tree"); // 侧边栏模式切换
const sidebarTab = ref<"chapters" | "worldbuilding" | "relationship" | "timeline" | "inspiration">("chapters"); // 侧边栏内容切换
const chapterTree = ref<VolumeWithChapters[]>([]);
const editorRef = ref<InstanceType<typeof MarkdownEditor> | null>(null);
const worldbuildingPanelRef = ref<{ viewCharacterDetail: (character: Character) => void } | null>(null);

// Reload trigger for TreeSidebar
const reloadTrigger = ref(0);

const SETTINGS_KEY = "inovel_settings";

// 读取自动保存间隔（分钟），默认1分钟
const getAutoSaveIntervalMs = (): number => {
  try {
    const stored = localStorage.getItem(SETTINGS_KEY);
    if (stored) {
      const settings = JSON.parse(stored);
      if (settings.autoSaveInterval !== undefined) {
        return Math.round(settings.autoSaveInterval * 60 * 1000);
      }
    }
  } catch { /* ignore */ }
  return 60000; // 默认1分钟
};

// Writing goal state
const dailyGoal = ref(3000);
const todayWords = ref(0);
const todayInitialWords = ref(0); // 当日打开时的初始字数
const writingDuration = ref(0); // 写作时长（分钟）

// 名称生成相关状态
const showNameGenerator = ref(false);
const nameCategory = ref<string | null>("chinese_name");
const nameGender = ref<string>("any");
const nameCount = ref(10);
const generatedNames = ref<string[]>([]);

const nameCategoryOptions = [
  { label: "中文姓名", value: "chinese_name" },
  { label: "英文姓名", value: "western_name" },
  { label: "中文地名", value: "chinese_place" },
  { label: "英文地名", value: "western_place" },
];

const genderOptions = [
  { label: "不限", value: "any" },
  { label: "男", value: "male" },
  { label: "女", value: "female" },
];

// Character detail sidebar state
interface Character {
  id: number
  name: string
  gender: string
  age: number | null
  appearance: string
  personality: string
  background: string
}

const showHistory = ref(false)
const showSensitiveWords = ref(false)
const showExport = ref(false)
const showBackup = ref(false)
const showShortcuts = ref(false)

// Pomodoro Timer visibility state (default to hidden)
const showPomodoro = ref(false);

// Inspiration Board visibility state (default to hidden)
const showInspirationBoard = ref(false);

// Zen Mode / Fullscreen state
const isZenMode = ref(false);
const isPomodoroZenMode = ref(false); // 番茄钟触发的专注模式
const isFullscreen = ref(false);
const appWindow = getCurrentWindow();

// Pomodoro zen mode handler
const handlePomodoroZenMode = (enabled: boolean) => {
  isPomodoroZenMode.value = enabled;
};

// Combined zen mode state
const isZenModeActive = computed(() => {
  return isZenMode.value || isPomodoroZenMode.value;
});

// Handle inserting content from inspiration board
const handleInsertFromInspiration = (content: string) => {
  if (editorRef.value?.editor) {
    editorRef.value.editor.commands.insertContent(content);
  }
};

// Toggle Zen Mode (pseudo fullscreen - hides UI elements, centers editor)
const toggleZenMode = async () => {
  if (isPomodoroZenMode.value) {
    // Can't toggle while pomodoro zen mode is active
    message.warning("番茄钟专注模式进行中，请等待结束后再切换");
    return;
  }
  isZenMode.value = !isZenMode.value;
  if (isZenMode.value) {
    message.info("禅模式已开启，按 Esc 或点击退出按钮退出");
  }
};

// Toggle real fullscreen using Tauri API
const toggleFullscreen = async () => {
  try {
    const isFs = await appWindow.isFullscreen();
    await appWindow.setFullscreen(!isFs);
    isFullscreen.value = !isFs;
  } catch (e) {
    console.error("Fullscreen toggle failed:", e);
    message.warning("全屏切换失败");
  }
};

// Listen to fullscreen state changes
const unlistenFullscreen = ref<(() => void) | null>(null);
const setupFullscreenListener = async () => {
  // Use listen to capture resize events which include fullscreen changes
  unlistenFullscreen.value = await appWindow.onResized(async () => {
    isFullscreen.value = await appWindow.isFullscreen();
  });
};

// Exit zen mode handler
const exitZenMode = () => {
  if (isZenMode.value) {
    isZenMode.value = false;
    message.info("已退出禅模式");
  }
  // Note: pomodoro zen mode is controlled by the timer itself
};

const handleSelectCharacter = async (character: Character) => {
  // 切换到世界观面板并显示人物详情
  sidebarTab.value = 'worldbuilding'
  showWorldbuilding.value = true
  // 等待组件挂载完成后再调用方法
  await nextTick()
  worldbuildingPanelRef.value?.viewCharacterDetail(character)
}

// Handle timeline chapter navigation
const handleNavigateChapter = async (chapterId: number) => {
  // Find and open the chapter
  for (const volume of chapterTree.value) {
    const chapter = volume.chapters.find(c => c.id === chapterId)
    if (chapter) {
      await handleSelectChapter(chapter.id, chapter)
      message.success(`已跳转到：${chapter.title}`)
      return
    }
  }
  message.warning('未找到对应章节')
}

// Handle mention click from editor
const handleMentionClick = (id: string) => {
  const m = id.match(/^(character|location|organization)-(\d+)$/)
  if (!m) return
  const [, type, idStr] = m
  const numericId = parseInt(idStr, 10)

  // Import worldbuilding store to find the item
  import('../stores/worldbuilding').then(({ useWorldbuildingStore }) => {
    const store = useWorldbuildingStore()

    if (type === 'character') {
      const char = store.getCharacterById(numericId)
      if (char) {
        handleSelectCharacter(char)
      } else {
        message.warning('未找到该人物')
      }
    } else if (type === 'location' || type === 'organization') {
      // For location/organization, show the worldbuilding panel on the relevant tab
      sidebarTab.value = 'worldbuilding'
      message.success(`已切换到世界观面板`)
    }
  })
}

const projectId = computed(() => route.params.projectId as string);

const projectName = computed(() => {
  const project = projectStore.recentProjects.find((p) => p.id === Number(projectId.value));
  return project?.name || `项目 ${projectId.value}`;
});

// 计算本卷字数（优先使用 currentChapter 的实时字数）
const volumeWordCount = computed(() => {
  if (!currentChapter.value) return 0;
  const volume = chapterTree.value.find((v) => v.id === currentChapter.value!.volume_id);
  if (!volume) return 0;
  // 使用 currentChapter 的实时字数 + 其他章节的缓存字数
  return volume.chapters.reduce((sum, ch) => {
    // 如果是当前章节，优先使用 currentChapter 的实时字数
    if (ch.id === currentChapter.value!.id) {
      return sum + (currentChapter.value!.word_count_cache ?? 0);
    }
    return sum + ch.word_count_cache;
  }, 0);
});

// 计算全书字数（优先使用 currentChapter 的实时字数）
const totalWordCount = computed(() => {
  return chapterTree.value.reduce((sum, volume) => {
    return sum + volume.chapters.reduce((vs, ch) => {
      // 如果是当前章节，优先使用 currentChapter 的实时字数
      if (currentChapter.value && ch.id === currentChapter.value.id) {
        return vs + (currentChapter.value.word_count_cache ?? 0);
      }
      return vs + ch.word_count_cache;
    }, 0);
  }, 0);
});

// 名称生成
const handleGenerateNames = async () => {
  if (!currentChapter.value) {
    message.warning("请先选择一个章节");
    return;
  }
  if (!nameCategory.value) {
    message.warning("请选择生成类型");
    return;
  }
  try {
    const gender = nameGender.value === "any" ? null : nameGender.value;
    const names = await invoke<string[]>("generate_names", {
      category: nameCategory.value,
      gender,
      count: nameCount.value,
    });
    generatedNames.value = names;
  } catch (error) {
    console.error("生成名称失败:", error);
    message.error("生成失败: " + error);
  }
};

// 插入名称到编辑器
const handleInsertName = (name: string) => {
  if (editorRef.value?.editor) {
    editorRef.value.editor.commands.insertContent(name);
    message.success(`已插入: ${name}`);
  }
};

// 加载章节树（用于字数统计）
const loadChapterTree = async () => {
  try {
    const tree = await invoke<VolumeWithChapters[]>("get_chapter_tree", {
      projectId: Number(projectId.value),
    });
    chapterTree.value = tree;
  } catch (error) {
    console.error("加载章节树失败:", error);
  }
};

// 加载写作目标和今日数据
const loadWritingGoal = async () => {
  try {
    const goal = await invoke<{ daily_goal: number } | null>("get_writing_goal", {
      projectId: Number(projectId.value),
    });
    if (goal) {
      dailyGoal.value = goal.daily_goal;
    }
  } catch (error) {
    console.error("加载写作目标失败:", error);
  }
};

// 加载今日写作记录
const loadTodayRecord = async () => {
  try {
    const record = await invoke<{ total_words: number; duration: number } | null>("get_today_words", {
      projectId: Number(projectId.value),
    });
    if (record) {
      todayWords.value = record.total_words;
      todayInitialWords.value = record.total_words;
      writingDuration.value = record.duration;
    }
  } catch (error) {
    console.error("加载今日记录失败:", error);
  }
};

// 加载章节内容（使用 chapterId）
const loadChapterContentByPath = async (chapterId: number): Promise<string> => {
  try {
    const content = await invoke<string>("get_chapter_content", {
      projectId: String(projectId.value),
      chapterId: String(chapterId),
    });
    return content;
  } catch (error) {
    console.error("加载章节失败:", error);
    return "";
  }
};

// 保存章节内容（支持 HTML）
const saveChapter = async () => {
  if (isSaving.value || !currentChapter.value) return;

  isSaving.value = true;
  try {
    // 保存内容
    await invoke("save_chapter_content", {
      projectId: String(projectId.value),
      chapterId: String(currentChapter.value.id),
      content: currentContent.value,
    });
    // Auto-commit to git after save
    try {
      const now = new Date().toLocaleString('zh-CN')
      await invoke('create_snapshot', {
        projectId: Number(projectId.value),
        message: `自动保存 - ${currentChapter.value.title} @ ${now}`,
      })
    } catch (_e) {
      // If git repo doesn't exist yet, ignore silently
    }
    // 获取当前编辑器中的实时字数
    const finalWordCount = editorRef.value?.getWordCount() ?? 0;
    
    // 更新字数到数据库
    await invoke("update_chapter_word_count", {
      chapterId: currentChapter.value.id,
      wordCount: finalWordCount,
    });
    
    // 同步更新章节树中的缓存（避免重新加载导致的闪烁）
    for (const volume of chapterTree.value) {
      const chapter = volume.chapters.find(c => c.id === currentChapter.value!.id);
      if (chapter) {
        chapter.word_count_cache = finalWordCount;
        break;
      }
    }
    
    // 更新当前章节的缓存字数
    if (currentChapter.value) {
      currentChapter.value.word_count_cache = finalWordCount;
    }

    // 更新写作记录
    await upsertWritingRecord(finalWordCount);

    message.success("保存成功");
    
    // 延迟重新加载章节树，确保 TreeSidebar 等组件也能同步
    setTimeout(async () => {
      await loadChapterTree();
    }, 100);
  } catch (error) {
    console.error("保存失败:", error);
    message.error("保存失败: " + error);
  } finally {
    isSaving.value = false;
  }
};

// 手动创建快照（先保存，再创建 Git 快照）
const manualSnapshot = async () => {
  if (!currentChapter.value) return;
  await saveChapter();
  try {
    const now = new Date().toLocaleString('zh-CN');
    await invoke('create_snapshot', {
      projectId: Number(projectId.value),
      message: `手动快照 @ ${now}`,
    });
    message.success('快照已创建');
  } catch (e) {
    console.error('快照创建失败:', e);
    // Try to init git first, then retry
    try {
      await invoke('init_project_git', { projectId: Number(projectId.value), gitignore: null });
      await invoke('create_snapshot', {
        projectId: Number(projectId.value),
        message: `手动快照 @ ${new Date().toLocaleString('zh-CN')}`,
      });
      message.success('Git 已初始化，快照已创建');
    } catch (e2) {
      message.error('Git 初始化失败，快照创建失败。请在项目目录下手动执行 git init');
      console.error('Git init 失败:', e2);
    }
  }
};

// 更新写作记录
const upsertWritingRecord = async (currentChapterWords: number) => {
  try {
    // 计算今日新增字数 = 当前全书累计字数 - 当日初始字数
    const newWords = Math.max(0, totalWordCount.value - todayInitialWords.value);
    // 增加写作时长（每次保存增加30秒，约0.5分钟）
    writingDuration.value += 0.5;

    await invoke("upsert_writing_record", {
      projectId: Number(projectId.value),
      totalWords: todayInitialWords.value + newWords,
      duration: Math.floor(writingDuration.value),
    });

    todayWords.value = todayInitialWords.value + newWords;
  } catch (error) {
    console.error("更新写作记录失败:", error);
  }
};

// 处理内容更新
const handleContentUpdate = (html: string) => {
  currentContent.value = html;
};

// 自动保存定时器
const startAutoSave = () => {
  if (autoSaveTimer.value) {
    clearInterval(autoSaveTimer.value);
  }
  const intervalMs = getAutoSaveIntervalMs();
  autoSaveTimer.value = window.setInterval(() => {
    saveChapter();
  }, intervalMs);
};

// 键盘快捷键
const handleKeyDown = (event: KeyboardEvent) => {
  // Ctrl+S 保存
  if ((event.ctrlKey || event.metaKey) && event.key === "s") {
    event.preventDefault();
    saveChapter();
  }
  // Ctrl+Shift+T 打字机模式
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === "T") {
    event.preventDefault();
    editorStore.toggleTypewriter();
    message.info(editorStore.isTypewriter ? "打字机模式已开启" : "打字机模式已关闭");
  }
  // Ctrl+Shift+F 聚焦模式
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === "F") {
    event.preventDefault();
    editorStore.toggleFocus();
    message.info(editorStore.isFocus ? "聚焦模式已开启" : "聚焦模式已关闭");
  }
  // Ctrl+Shift+Z 禅模式
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === "Z") {
    event.preventDefault();
    toggleZenMode();
  }
  // F11 全屏切换
  if (event.key === "F11") {
    event.preventDefault();
    toggleFullscreen();
  }
  // Esc 退出特殊模式
  if (event.key === "Escape") {
    if (isZenMode.value) {
      exitZenMode();
    } else if (!editorStore.isNormal) {
      editorStore.exitSpecialMode();
      message.info("已退出特殊模式");
    }
  }
};

// 处理章节选择
const handleSelectChapter = async (chapterId: number, chapter: Chapter) => {
  // 如果是同一个章节，无需处理
  if (currentChapter.value?.id === chapterId) {
    return;
  }

  // 先保存当前章节
  if (currentChapter.value) {
    await saveChapter();
  }

  // 设置加载状态，编辑器会显示占位符
  currentChapter.value = chapter;
  currentContent.value = ''; // 清空内容
  
  // 加载新章节内容
  const content = await loadChapterContentByPath(chapter.id);
  
  // 内容加载完成后再更新，避免中间状态导致的闪烁
  currentContent.value = content;
  
  // 重新加载章节树以获取最新字数
  await loadChapterTree();
  
  // 同步 currentChapter 的字数缓存（如果章节树中有最新数据）
  const updatedChapter = chapterTree.value
    .flatMap(v => v.chapters)
    .find(c => c.id === chapterId);
  if (updatedChapter) {
    currentChapter.value.word_count_cache = updatedChapter.word_count_cache;
  }
};

// 处理编辑器字数更新事件
const handleWordCountUpdated = (count: number) => {
  // 当编辑器字数更新时，实时更新当前章节的缓存
  if (currentChapter.value) {
    currentChapter.value.word_count_cache = count;
  }
};

// 监听字数变化并更新
watch(
  () => currentChapter.value?.word_count_cache,
  async () => {
    await loadChapterTree();
  }
);

onMounted(async () => {
  if (projectStore.recentProjects.length === 0) {
    await projectStore.getRecentProjects();
  }

  await loadChapterTree();
  await loadWritingGoal();
  await loadTodayRecord();
  startAutoSave();
  window.addEventListener("keydown", handleKeyDown);
  await setupFullscreenListener();
  isLoading.value = false;
});

onUnmounted(async () => {
  if (autoSaveTimer.value) {
    clearInterval(autoSaveTimer.value);
  }
  window.removeEventListener("keydown", handleKeyDown);
  if (unlistenFullscreen.value) {
    unlistenFullscreen.value();
  }
  // Auto-commit on close
  try {
    await invoke('create_snapshot', {
      projectId: Number(projectId.value),
      message: `应用关闭 @ ${new Date().toLocaleString('zh-CN')}`,
    })
  } catch { /* ok */ }
});

const goBack = () => {
  router.push("/");
};

const goToProjectSettings = () => {
  router.push(`/editor/${projectId.value}/project-settings`);
};

const goToProjectStats = () => {
  router.push(`/editor/${projectId.value}/project-stats`);
};

const goToWorldbuilding = () => {
  showWorldbuilding.value = !showWorldbuilding.value;
};

const toggleSidebar = () => {
  showSidebar.value = !showSidebar.value;
};

// 计算今日进度百分比
const dailyProgress = computed(() => {
  if (dailyGoal.value <= 0) return 0;
  return Math.min(100, Math.round((todayWords.value / dailyGoal.value) * 100));
});

// 计算今日新增字数
const todayNewWords = computed(() => {
  return Math.max(0, todayWords.value - todayInitialWords.value);
});


</script>

<template>
  <div class="h-screen flex flex-col transition-colors duration-300"
    :class="[
      isDark ? 'bg-gray-900 text-white' : 'bg-gray-50 text-gray-900',
      isZenMode ? 'bg-gray-900!' : ''
    ]">
    <!-- Zen Mode Exit Button (floating) -->
    <button v-if="isZenMode && !isPomodoroZenMode" @click="exitZenMode"
      class="fixed top-4 right-4 z-50 flex items-center gap-2 px-4 py-2 bg-gray-800 text-white rounded-lg shadow-lg hover:bg-gray-700 transition-colors">
      <X class="w-4 h-4" />
      退出禅模式 (Esc)
    </button>

    <!-- Pomodoro Zen Mode Indicator -->
    <button v-if="isPomodoroZenMode" @click="isPomodoroZenMode = false"
      class="fixed top-4 right-4 z-50 flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg shadow-lg hover:bg-purple-700 transition-colors">
      <X class="w-4 h-4" />
      退出专注模式
    </button>

    <!-- Header (hidden in zen mode) -->
    <header v-if="!isZenModeActive" class="border-b transition-colors duration-300 shrink-0"
      :class="isDark ? 'border-gray-700 bg-gray-800' : 'border-gray-200 bg-white'">
      <div class="px-4 py-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <button @click="goBack" class="p-2 rounded-lg transition-colors duration-300" :class="isDark
            ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
            : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
            ">
            <ArrowLeft class="w-5 h-5" />
          </button>
          <FileText class="w-6 h-6 text-blue-600" />
          <h1 class="text-lg font-bold">小说工坊</h1>
          <span class="px-3 py-1 rounded-full text-sm font-medium"
            :class="isDark ? 'bg-gray-700 text-gray-300' : 'bg-gray-100 text-gray-600'">
            {{ projectName }}
          </span>
        </div>
        <div class="flex items-center gap-2">
          <n-button type="primary" size="small" @click="saveChapter" :loading="isSaving" :disabled="!currentChapter">
            <template #icon>
              <NIcon>
                <Save />
              </NIcon>
            </template>
            保存
          </n-button>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button @click="showNameGenerator = true" class="p-2 rounded-lg transition-colors duration-300"
                :class="isDark ? 'bg-gray-700 hover:bg-gray-600 text-gray-300' : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                <User class="w-5 h-5" />
              </button>
            </template>
            名称生成
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button @click="toggleZenMode" class="p-2 rounded-lg transition-colors duration-300"
                :class="isZenMode ? 'bg-blue-600 text-white' : isDark ? 'bg-gray-700 hover:bg-gray-600 text-gray-300' : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                <Sparkles class="w-5 h-5" />
              </button>
            </template>
            禅模式 (Ctrl+Shift+Z)
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button @click="toggleFullscreen" class="p-2 rounded-lg transition-colors duration-300"
                :class="isFullscreen ? 'bg-blue-600 text-white' : isDark ? 'bg-gray-700 hover:bg-gray-600 text-gray-300' : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                <Minimize2 v-if="isFullscreen" class="w-5 h-5" />
                <Maximize2 v-else class="w-5 h-5" />
              </button>
            </template>
            全屏 (F11)
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button @click="goToProjectSettings" class="p-2 rounded-lg transition-colors duration-300"
                :class="isDark ? 'bg-gray-700 hover:bg-gray-600 text-gray-300' : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                <Settings class="w-5 h-5" />
              </button>
            </template>
            项目设置
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button @click="goToProjectStats" class="p-2 rounded-lg transition-colors duration-300"
                :class="isDark ? 'bg-gray-700 hover:bg-gray-600 text-gray-300' : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                <BarChart3 class="w-5 h-5" />
              </button>
            </template>
            项目统计
          </n-tooltip>
          <button @click="() => toggleDark()" class="p-2 rounded-lg transition-colors duration-300" :class="isDark
            ? 'bg-gray-700 hover:bg-gray-600 text-yellow-400'
            : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
            ">
            <Sun v-if="isDark" class="w-5 h-5" />
            <Moon v-else class="w-5 h-5" />
          </button>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button @click="showShortcuts = true" class="p-2 rounded-lg transition-colors duration-300"
                :class="isDark ? 'bg-gray-700 hover:bg-gray-600 text-gray-300' : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                <Keyboard class="w-5 h-5" />
              </button>
            </template>
            快捷键设置
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button @click="showPomodoro = !showPomodoro" class="p-2 rounded-lg transition-colors duration-300"
                :class="showPomodoro ? 'bg-red-500 text-white' : isDark ? 'bg-gray-700 hover:bg-gray-600 text-gray-300' : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                <Timer class="w-5 h-5" />
              </button>
            </template>
            {{ showPomodoro ? '隐藏' : '显示' }}番茄钟
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button @click="showInspirationBoard = !showInspirationBoard" class="p-2 rounded-lg transition-colors duration-300"
                :class="showInspirationBoard ? 'bg-yellow-500 text-white' : isDark ? 'bg-gray-700 hover:bg-gray-600 text-gray-300' : 'bg-gray-100 hover:bg-gray-200 text-gray-600'">
                <Lightbulb class="w-5 h-5" />
              </button>
            </template>
            {{ showInspirationBoard ? '隐藏' : '显示' }}灵感看板
          </n-tooltip>
        </div>
      </div>
    </header>

    <div class="flex-1 flex overflow-hidden" :class="isZenModeActive ? 'p-0!' : ''">
      <!-- Sidebar Toggle Button -->
      <button v-if="!showSidebar && !isZenModeActive" @click="toggleSidebar"
        class="absolute left-0 top-1/2 -translate-y-1/2 z-10 p-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-r-lg shadow-md">
        <ChevronRight class="w-4 h-4" />
      </button>

      <!-- Tree Sidebar (hidden in zen mode) -->
      <div v-if="showSidebar && !isZenModeActive" class="w-64 shrink-0 relative">
        <button @click="toggleSidebar"
          class="absolute -right-3 top-1/2 -translate-y-1/2 z-10 p-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-full shadow-md">
          <ChevronLeft class="w-3 h-3" />
        </button>
        <!-- Sidebar Header with Mode Toggle -->
        <div class="flex items-center border-b border-gray-200 dark:border-gray-700">
          <button class="flex-1 px-3 py-2 text-xs font-medium transition-colors" :class="sidebarMode === 'tree'
            ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
            : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'"
            @click="sidebarMode = 'tree'">
            章节树
          </button>
          <button class="flex-1 px-3 py-2 text-xs font-medium transition-colors" :class="sidebarMode === 'outline'
            ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
            : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'"
            @click="sidebarMode = 'outline'">
            大纲
          </button>
        </div>
        <!-- Tree Sidebar -->
        <TreeSidebar v-if="sidebarMode === 'tree'" :project-id="projectId" @select-chapter="handleSelectChapter" />
        <!-- Outline Panel -->
        <OutlinePanel v-else :project-id="projectId" :selected-chapter-id="currentChapter?.id"
          @select-chapter="handleSelectChapter" @update="loadChapterTree" />
      </div>

      <!-- Editor Area -->
      <main class="flex-1 overflow-hidden flex justify-center"
        :class="isZenModeActive ? 'bg-gray-900' : 'p-4'">
        <!-- Zen Mode: Centered editor container with 50% width -->
        <div v-if="isZenModeActive" class="w-1/2 mx-auto h-full flex flex-col px-8 py-6">
          <div v-if="!currentChapter" class="h-full flex flex-col items-center justify-center text-gray-500">
            <FileText class="w-16 h-16 mb-4 opacity-50" />
            <p class="text-lg">请从左侧选择或创建章节</p>
          </div>
          <div v-else class="h-full flex flex-col">
            <!-- Minimal title bar in zen mode -->
            <div class="flex items-center gap-2 mb-3 pb-3 border-b border-gray-700">
              <FileText class="w-4 h-4 text-blue-500" />
              <h2 class="text-base font-medium text-gray-300">{{ currentChapter.title }}</h2>
              <span class="text-xs text-gray-500 ml-auto">{{ totalWordCount }} 字</span>
            </div>
            <!-- Markdown Editor -->
            <MarkdownEditor ref="editorRef" v-model="currentContent" :chapter-id="currentChapter.id"
              :project-id="Number(projectId)" :volume-word-count="volumeWordCount" :total-word-count="totalWordCount"
              :is-dark="isDark" :editor-mode="editorStore.mode" @update:model-value="handleContentUpdate"
              @mention-click="handleMentionClick" @show-history="showHistory = true" @create-snapshot="manualSnapshot"
              @word-count-updated="handleWordCountUpdated" />
          </div>
        </div>
        <!-- Normal mode editor -->
        <div v-else class="h-full w-full max-w-full flex flex-col">
          <div v-if="!currentChapter" class="h-full flex flex-col items-center justify-center text-gray-500">
            <FileText class="w-16 h-16 mb-4 opacity-50" />
            <p class="text-lg">请从左侧选择或创建章节</p>
          </div>
          <div v-else class="h-full flex flex-col">
            <!-- Chapter Title Bar -->
            <div class="flex items-center gap-2 mb-3 pb-3 border-b"
              :class="isDark ? 'border-gray-700' : 'border-gray-200'">
              <FileText class="w-5 h-5 text-blue-600" />
              <h2 class="text-lg font-semibold">{{ currentChapter.title }}</h2>
            </div>
            <!-- Markdown Editor -->
            <MarkdownEditor ref="editorRef" v-model="currentContent" :chapter-id="currentChapter.id"
              :project-id="Number(projectId)" :volume-word-count="volumeWordCount" :total-word-count="totalWordCount"
              :is-dark="isDark" :editor-mode="editorStore.mode" @update:model-value="handleContentUpdate"
              @mention-click="handleMentionClick" @show-history="showHistory = true" @create-snapshot="manualSnapshot"
              @word-count-updated="handleWordCountUpdated" />
          </div>
        </div>
      </main>

      <!-- Worldbuilding Panel (Right Sidebar, hidden in zen mode) -->
      <div v-if="showWorldbuilding && !isZenModeActive"
        class="w-[480px] shrink-0 border-l border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 overflow-hidden flex flex-col">
        <!-- Panel Header with Tabs -->
        <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
          <div class="flex items-center gap-2">
            <Globe class="w-5 h-5 text-blue-600" />
            <h3 class="font-semibold text-gray-900 dark:text-white">世界观设定</h3>
          </div>
          <button @click="showWorldbuilding = false"
            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors">
            <X class="w-5 h-5 text-gray-500" />
          </button>
        </div>
        <!-- Tabs -->
        <div class="flex border-b border-gray-200 dark:border-gray-700">
          <button class="flex-1 px-4 py-2 text-sm font-medium transition-colors" :class="sidebarTab === 'worldbuilding'
            ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
            : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'"
            @click="sidebarTab = 'worldbuilding'">
            设定
          </button>
          <button class="flex-1 px-4 py-2 text-sm font-medium transition-colors" :class="sidebarTab === 'relationship'
            ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
            : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'"
            @click="sidebarTab = 'relationship'">
            关系图谱
          </button>
          <button class="flex-1 px-4 py-2 text-sm font-medium transition-colors" :class="sidebarTab === 'timeline'
            ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
            : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'"
            @click="sidebarTab = 'timeline'">
            时间轴
          </button>
        </div>
        <!-- Panel Content -->
        <div class="flex-1 overflow-hidden">
          <WorldbuildingPanel ref="worldbuildingPanelRef" v-if="sidebarTab === 'worldbuilding'" />
          <RelationshipGraph v-else-if="sidebarTab === 'relationship'" :project-id="Number(projectId)"
            @select-character="handleSelectCharacter" />
          <Timeline v-else-if="sidebarTab === 'timeline'" :project-id="Number(projectId)"
            @navigate-chapter="handleNavigateChapter" />
        </div>
      </div>

      <!-- Worldbuilding Toggle Button (hidden in zen mode) -->
      <button v-if="!showWorldbuilding && !isZenModeActive" @click="showWorldbuilding = true"
        class="absolute right-0 top-1/2 -translate-y-1/2 z-10 p-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-l-lg shadow-md transition-transform hover:translate-x-0"
        :style="{ top: 'calc(50% + 36px)' }">
        <Globe class="w-4 h-4 text-blue-600" />
      </button>
    </div>

    <!-- Status Bar with Writing Progress (hidden in zen mode) -->
    <footer v-if="!isZenModeActive" class="border-t transition-colors duration-300 shrink-0 px-4 py-2"
      :class="isDark ? 'border-gray-700 bg-gray-800' : 'border-gray-200 bg-white'">
      <div class="flex items-center gap-6">
        <!-- Daily Goal Progress -->
        <n-tooltip trigger="hover">
          <template #trigger>
            <div class="daily-goal-container">
              <Target class="w-4 h-4 text-blue-500 shrink-0" />
              <span class="text-sm text-gray-600 dark:text-gray-400 whitespace-nowrap">
                今日: {{ todayWords }} / {{ dailyGoal }} 字
              </span>
              <n-progress type="line" :percentage="dailyProgress" :height="8" :border-radius="4" :fill-border-radius="4"
                :color="dailyProgress >= 100 ? '#52c41a' : '#3b82f6'" :rail-color="isDark ? '#374151' : '#e5e7eb'"
                class="daily-goal-progress" :show-indicator="false" />
              <span class="text-sm font-medium whitespace-nowrap" :class="dailyProgress >= 100 ? 'text-green-500' : 'text-blue-500'">
                {{ dailyProgress }}%
              </span>
            </div>
          </template>
          <div>每日目标进度: {{ todayWords }} / {{ dailyGoal }} 字</div>
        </n-tooltip>

        <!-- Total Word Count -->
        <div class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
          <FileText class="w-4 h-4" />
          <span>全书: {{ totalWordCount }} 字</span>
        </div>

        <!-- Export Button -->
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="tiny" quaternary @click="showExport = true" class="text-gray-500! hover:text-blue-500!">
              <template #icon>
                <n-icon>
                  <Download />
                </n-icon>
              </template>
            </n-button>
          </template>
          导出文档
        </n-tooltip>

        <!-- Backup Button -->
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="tiny" quaternary @click="showBackup = true"
              class="text-gray-500! hover:text-green-500!">
              <template #icon>
                <n-icon>
                  <Package />
                </n-icon>
              </template>
            </n-button>
          </template>
          备份项目
        </n-tooltip>

        <!-- Sensitive Words Manager Button -->
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="tiny" quaternary @click="showSensitiveWords = true"
              class="text-gray-500! hover:text-red-500!">
              <template #icon>
                <n-icon>
                  <AlertTriangle />
                </n-icon>
              </template>
            </n-button>
          </template>
          敏感词管理
        </n-tooltip>
      </div>
    </footer>

    <!-- 名称生成模态框 -->
    <n-modal v-model:show="showNameGenerator" preset="card" title="名称生成器" style="width: 480px" :mask-closable="true">
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-2" :class="isDark ? 'text-gray-300' : 'text-gray-700'">类型</label>
          <n-select v-model:value="nameCategory" :options="nameCategoryOptions" placeholder="选择生成类型" />
        </div>

        <div v-if="nameCategory === 'chinese_name' || nameCategory === 'western_name'">
          <label class="block text-sm font-medium mb-2" :class="isDark ? 'text-gray-300' : 'text-gray-700'">性别</label>
          <n-radio-group v-model:value="nameGender" name="gender">
            <n-space>
              <n-radio v-for="opt in genderOptions" :key="opt.value" :value="opt.value">
                {{ opt.label }}
              </n-radio>
            </n-space>
          </n-radio-group>
        </div>

        <div>
          <label class="block text-sm font-medium mb-2" :class="isDark ? 'text-gray-300' : 'text-gray-700'">数量</label>
          <n-input-number v-model:value="nameCount" :min="1" :max="100" />
        </div>

        <n-button type="primary" block @click="handleGenerateNames">
          生成
        </n-button>

        <div v-if="generatedNames.length > 0">
          <label class="block text-sm font-medium mb-2"
            :class="isDark ? 'text-gray-300' : 'text-gray-700'">生成结果（点击插入）</label>
          <div class="flex flex-wrap gap-2">
            <n-tag v-for="(name, index) in generatedNames" :key="index" :bordered="false" checkable
              @click="handleInsertName(name)" class="cursor-pointer hover:bg-blue-100 dark:hover:bg-blue-900">
              {{ name }}
            </n-tag>
          </div>
        </div>
      </div>
    </n-modal>

    <!-- Version History Dialog -->
    <HistoryDialog v-model:show="showHistory" :project-id="Number(projectId)" :current-content="currentContent"
      @restore="(content: string) => { if (content) currentContent = content }" />

    <!-- Sensitive Words Manager -->
    <SensitiveWordsManager v-model:show="showSensitiveWords" :project-id="Number(projectId)" />

    <!-- Export Dialog -->
    <ExportDialog v-model:show="showExport" :project-id="Number(projectId)" />
    <!-- Backup Dialog -->
    <BackupDialog v-model:show="showBackup" :project-id="Number(projectId)" />
    <!-- Shortcut Settings -->
    <ShortcutSettings v-model:show="showShortcuts" />

    <!-- Pomodoro Timer (floating) -->
    <PomodoroTimer
      v-if="projectId && !isLoading"
      :project-id="Number(projectId)"
      :is-dark="isDark"
      :visible="showPomodoro"
      @zen-mode="handlePomodoroZenMode"
    />

    <!-- Inspiration Board Modal -->
    <n-modal
      v-model:show="showInspirationBoard"
      preset="card"
      title="灵感看板"
      :style="{ width: '90vw', height: '80vh' }"
      :max-height="600"
      :mask-closable="true"
    >
      <template #header-extra>
        <n-tag type="warning" size="small">双击卡片编辑，右键插入到编辑器</n-tag>
      </template>
      <InspirationBoard
        :project-id="Number(projectId)"
        :is-dark="isDark"
        @insert-content="handleInsertFromInspiration"
      />
    </n-modal>
  </div>
</template>

<style scoped>
/* 每日目标进度容器 - 防止文本换行 */
.daily-goal-container {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}

.daily-goal-container .whitespace-nowrap {
  white-space: nowrap;
  flex-shrink: 0;
}

.daily-goal-progress {
  width: 80px;
  min-width: 80px;
  flex-shrink: 0;
}
</style>
