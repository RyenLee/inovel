<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  watch,
  nextTick,
  defineAsyncComponent,
} from "vue";
import { useRoute, useRouter } from "vue-router";
import { useProjectStore } from "../stores/project";
import { useEditorStore } from "../stores/editor";
import {
  useWorldbuildingStore,
  type Character,
  type Location,
  type Organization,
} from "../stores/worldbuilding";
import { useEnumDictionary } from "../stores/enumDictionary";
import {
  NButton,
  NIcon,
  NProgress,
  useMessage,
  NTooltip,
  NModal,
  NSelect,
  NInputNumber,
  NRadioGroup,
  NRadio,
  NTag,
  NSpace,
} from "naive-ui";
import {
  ArrowLeft,
  Save,
  FileText,
  Sun,
  Moon,
  ChevronLeft,
  ChevronRight,
  Target,
  Settings,
  BarChart3,
  User,
  Globe,
  X,
  GitBranch,
  AlertTriangle,
  Download,
  Package,
  Maximize2,
  Minimize2,
  Sparkles,
  Keyboard,
  Timer,
  Lightbulb,
} from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTheme } from "../composables/useTheme";
import { useLocale } from "../i18n/composables/useLocale";
import { useSnapshotMessage } from "../i18n/composables/useSnapshotMessage";

import TreeSidebar from "../components/TreeSidebar.vue";
import OutlinePanel from "../components/OutlinePanel.vue";
import MarkdownEditor from "../components/MarkdownEditor.vue";

const WorldbuildingPanel = defineAsyncComponent(
  () => import("../components/WorldbuildingPanel.vue")
);
const RelationshipGraph = defineAsyncComponent(
  () => import("../components/RelationshipGraph.vue")
);
const Timeline = defineAsyncComponent(
  () => import("../components/Timeline.vue")
);
const HistoryDialog = defineAsyncComponent(
  () => import("../components/HistoryDialog.vue")
);
const SensitiveWordsManager = defineAsyncComponent(
  () => import("../components/SensitiveWordsManager.vue")
);
const ExportDialog = defineAsyncComponent(
  () => import("../components/ExportDialog.vue")
);
const BackupDialog = defineAsyncComponent(
  () => import("../components/BackupDialog.vue")
);
const ShortcutSettings = defineAsyncComponent(
  () => import("../components/ShortcutSettings.vue")
);
const PomodoroTimer = defineAsyncComponent(
  () => import("../components/PomodoroTimer.vue")
);
const InspirationBoard = defineAsyncComponent(
  () => import("../components/InspirationBoard.vue")
);

const { isDark, toggleDark } = useTheme();
const { t, isZhCN, isEnUS } = useLocale();
const { formatSnapshotMessage } = useSnapshotMessage();

// Initialize enum dictionary
const enumDictionary = useEnumDictionary();
enumDictionary.loadDictionary();

// Types
interface SimpleCharacter {
  id: number;
  name: string;
  gender?: string;
  age?: number | null;
  appearance?: string;
  personality?: string;
  background?: string;
}

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
const sidebarTab = ref<
  "chapters" | "worldbuilding" | "relationship" | "timeline" | "inspiration"
>("chapters"); // 侧边栏内容切换
const chapterTree = ref<VolumeWithChapters[]>([]);
const editorRef = ref<InstanceType<typeof MarkdownEditor> | null>(null);
const worldbuildingPanelRef = ref<{
  viewCharacterDetail: (character: Character) => void;
  viewLocationDetail: (location: Location) => void;
  viewOrganizationDetail: (organization: Organization) => void;
} | null>(null);
const pomodoroTimerRef = ref<InstanceType<typeof PomodoroTimer> | null>(null);

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
  } catch {
    /* ignore */
  }
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
// 保存打开名称生成器时的光标位置
const savedSelection = ref<{ from: number; to: number } | null>(null);

const nameCategoryOptions = computed(() => [
  { label: t("editor.nameGen.categories.chineseName"), value: "chinese_name" },
  { label: t("editor.nameGen.categories.westernName"), value: "western_name" },
  {
    label: t("editor.nameGen.categories.chinesePlace"),
    value: "chinese_place",
  },
  {
    label: t("editor.nameGen.categories.westernPlace"),
    value: "western_place",
  },
]);

const genderOptions = computed(() => [
  { label: t("editor.nameGen.any"), value: "any" },
  ...enumDictionary.genderOptions.value.map((opt) => ({
    label: opt.label,
    value: opt.value,
  })),
]);

const showHistory = ref(false);
const showSensitiveWords = ref(false);
const showExport = ref(false);
const showBackup = ref(false);
const showShortcuts = ref(false);

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
    message.warning(t("editor.zenMode.pomodoroActive"));
    return;
  }

  // If enabling zen mode, perform pre-checks
  if (!isZenMode.value) {
    // Check if editor instance exists and has content
    const hasEditorInstance = editorRef.value?.editor !== undefined;
    const hasCurrentChapter = currentChapter.value !== null;

    if (!hasEditorInstance || !hasCurrentChapter) {
      // Scan chapter tree for the last chapter
      let lastChapter: Chapter | null = null;
      for (const volume of chapterTree.value) {
        if (volume.chapters.length > 0) {
          lastChapter = volume.chapters[volume.chapters.length - 1];
        }
      }

      if (lastChapter) {
        message.info(t("editor.zenMode.openingLastChapter"));
        await handleSelectChapter(lastChapter.id, lastChapter);
      } else {
        message.warning(t("editor.zenMode.noChapterAvailable"));
        return;
      }
    }
  }

  isZenMode.value = !isZenMode.value;
  if (isZenMode.value) {
    message.info(t("editor.zenMode.activated"));
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
    message.warning(t("editor.fullscreenFailed"));
  }
};

// Listen to fullscreen state changes
const unlistenFullscreen = ref<(() => void) | null>(null);
const unlistenWindowMove = ref<(() => void) | null>(null);
const unlistenWindowResize = ref<(() => void) | null>(null);
const setupFullscreenListener = async () => {
  // Use listen to capture resize events which include fullscreen changes
  unlistenFullscreen.value = await appWindow.onResized(async () => {
    isFullscreen.value = await appWindow.isFullscreen();
    // Also reposition Pomodoro timer on resize!
    nextTick(() => {
      pomodoroTimerRef.value?.reposition?.();
    });
  });
  // Listen to window move to reposition timer!
  unlistenWindowMove.value = await appWindow.listen("tauri://move", () => {
    nextTick(() => {
      pomodoroTimerRef.value?.reposition?.();
    });
  });
};

// Exit zen mode handler
const exitZenMode = () => {
  if (isZenMode.value) {
    isZenMode.value = false;
    message.info(t("editor.zenMode.exited"));
  }
  // Note: pomodoro zen mode is controlled by the timer itself
};

const handleSelectCharacter = async (character: SimpleCharacter) => {
  sidebarTab.value = "worldbuilding";
  showWorldbuilding.value = true;
  await nextTick();
  worldbuildingPanelRef.value?.viewCharacterDetail(character as any);
};

const onSelectCharacterFromGraph = (character: SimpleCharacter) => {
  handleSelectCharacter(character);
};

// Handle timeline chapter navigation
const handleNavigateChapter = async (chapterId: number) => {
  // Find and open the chapter
  for (const volume of chapterTree.value) {
    const chapter = volume.chapters.find((c) => c.id === chapterId);
    if (chapter) {
      await handleSelectChapter(chapter.id, chapter);
      message.success(
        t("editor.navigate.chapterJumped", { title: chapter.title })
      );
      return;
    }
  }
  message.warning(t("editor.navigate.chapterNotFound"));
};

// Handle mention click from editor
const handleMentionClick = async (id: string) => {
  const m = id.match(/^(character|location|organization)-(\d+)$/);
  if (!m) return;
  const [, type, idStr] = m;
  const numericId = parseInt(idStr, 10);

  const worldbuildingStore = useWorldbuildingStore();

  if (type === "character") {
    const char = worldbuildingStore.getCharacterById(numericId);
    if (char) {
      handleSelectCharacter(char);
    } else {
      message.warning(t("editor.navigate.characterNotFound"));
    }
  } else if (type === "location") {
    const location = worldbuildingStore.getLocationById(numericId);
    if (location) {
      sidebarTab.value = "worldbuilding";
      showWorldbuilding.value = true;
      await nextTick();
      worldbuildingPanelRef.value?.viewLocationDetail(location);
    } else {
      message.warning(t("editor.navigate.locationNotFound"));
    }
  } else if (type === "organization") {
    const org = worldbuildingStore.getOrganizationById(numericId);
    if (org) {
      sidebarTab.value = "worldbuilding";
      showWorldbuilding.value = true;
      await nextTick();
      worldbuildingPanelRef.value?.viewOrganizationDetail(org);
    } else {
      message.warning(t("editor.navigate.organizationNotFound"));
    }
  }
};

const projectId = computed(() => route.params.projectId as string);

const projectName = computed(() => {
  const project = projectStore.recentProjects.find(
    (p) => p.id === Number(projectId.value)
  );
  return project?.name || t("editor.project") + ` ${projectId.value}`;
});

const truncatedProjectName = computed(() => {
  const name = projectName.value;
  if (name.length > 5) {
    return name.slice(0, 5) + "...";
  }
  return name;
});

// 计算本卷字数（优先使用 currentChapter 的实时字数）
const volumeWordCount = computed(() => {
  if (!currentChapter.value) return 0;
  const volume = chapterTree.value.find(
    (v) => v.id === currentChapter.value!.volume_id
  );
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
    return (
      sum +
      volume.chapters.reduce((vs, ch) => {
        // 如果是当前章节，优先使用 currentChapter 的实时字数
        if (currentChapter.value && ch.id === currentChapter.value.id) {
          return vs + (currentChapter.value.word_count_cache ?? 0);
        }
        return vs + ch.word_count_cache;
      }, 0)
    );
  }, 0);
});

// 名称生成
const handleGenerateNames = async () => {
  if (!currentChapter.value) {
    message.warning(t("editor.nameGen.selectChapterFirst"));
    return;
  }
  if (!nameCategory.value) {
    message.warning(t("editor.nameGen.selectType"));
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
    message.error(t("editor.nameGen.generateFailed", { error: String(error) }));
  }
};

// 打开名称生成器
const openNameGenerator = () => {
  // 保存当前光标位置
  if (editorRef.value?.editor) {
    const selection = editorRef.value.editor.state.selection;
    savedSelection.value = {
      from: selection.from,
      to: selection.to,
    };
  }
  showNameGenerator.value = true;
};

// 名称生成器关闭时重置保存的光标位置
const onNameGeneratorClose = (value: boolean) => {
  if (!value) {
    savedSelection.value = null;
  }
};

// 插入名称到编辑器
const handleInsertName = (name: string) => {
  if (!editorRef.value?.editor) {
    message.warning(t("editor.nameGen.editorNotReady"));
    return;
  }

  const editor = editorRef.value.editor;

  // 使用 Tiptap 的 chain API 进行链式操作
  editor.chain().focus().insertContent(name).run();

  message.success(t("editor.nameGen.inserted", { name }));
};

// 加载章节树（用于字数统计）
const loadChapterTree = async () => {
  try {
    const tree = await invoke<VolumeWithChapters[]>("get_chapter_tree", {
      project_id: Number(projectId.value),
    });
    chapterTree.value = tree;
  } catch (error) {
    console.error("加载章节树失败:", error);
  }
};

// 加载写作目标和今日数据
const loadWritingGoal = async () => {
  try {
    const goal = await invoke<{ daily_goal: number } | null>(
      "get_writing_goal",
      {
        project_id: Number(projectId.value),
      }
    );
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
    const record = await invoke<{
      total_words: number;
      duration: number;
    } | null>("get_today_words", {
      project_id: Number(projectId.value),
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
  if (!projectId.value) {
    console.error("加载章节失败: 项目ID为空");
    throw new Error("项目ID为空");
  }

  try {
    const content = await invoke<string>("get_chapter_content", {
      project_id: String(projectId.value),
      chapter_id: String(chapterId),
    });

    if (content === undefined || content === null) {
      console.warn(`章节 ${chapterId} 内容为空`);
      return "";
    }

    return content;
  } catch (error) {
    console.error(`加载章节 ${chapterId} 失败:`, error);
    throw error;
  }
};

// 保存章节内容（支持 HTML）
const saveChapter = async (autoCreateSnapshot: boolean = true) => {
  if (isSaving.value || !currentChapter.value) return;

  isSaving.value = true;
  try {
    // 从编辑器获取实时内容（优先），回退到 currentContent
    const contentToSave = editorRef.value?.getHTML() ?? currentContent.value;

    await invoke("save_chapter_content", {
      project_id: String(projectId.value),
      chapter_id: String(currentChapter.value.id),
      content: contentToSave,
    });
    // Auto-commit to git after save
    if (autoCreateSnapshot) {
      try {
        const now = new Date().toLocaleString();
        const commitMessage = formatSnapshotMessage("auto", {
          title: currentChapter.value.title,
          time: now,
        });
        await invoke("create_snapshot", {
          project_id: Number(projectId.value),
          message: commitMessage,
        });
      } catch (_e) {
        // If git repo doesn't exist yet, ignore silently
      }
    }
    // 获取当前编辑器中的实时字数
    const finalWordCount = editorRef.value?.getWordCount() ?? 0;

    // 更新字数到数据库
    await invoke("update_chapter_word_count", {
      chapter_id: currentChapter.value.id,
      word_count: finalWordCount,
    });

    // 同步更新章节树中的缓存（避免重新加载导致的闪烁）
    for (const volume of chapterTree.value) {
      const chapter = volume.chapters.find(
        (c) => c.id === currentChapter.value!.id
      );
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

    message.success(t("editor.saveSuccess"));

    // 延迟重新加载章节树，确保 TreeSidebar 等组件也能同步
    setTimeout(async () => {
      await loadChapterTree();
    }, 100);
  } catch (error) {
    console.error("保存失败:", error);
    message.error(t("editor.saveError", { error: String(error) }));
  } finally {
    isSaving.value = false;
  }
};

// 手动创建快照（先保存，再创建 Git 快照）
const manualSnapshot = async () => {
  if (!currentChapter.value) {
    message.warning(t("editor.snapshot.noChapter"));
    return;
  }

  if (!projectId.value) {
    message.warning(t("editor.snapshot.noProject"));
    return;
  }

  try {
    // 先保存当前章节，但是不自动创建快照（避免重复快照）
    await saveChapter(false);

    const now = new Date().toLocaleString();
    const commitMessage = formatSnapshotMessage("manual", { time: now });

    console.log("准备创建快照，消息:", commitMessage);
    const result = await invoke("create_snapshot", {
      project_id: Number(projectId.value),
      message: commitMessage,
    });

    console.log("快照创建成功:", result);
    message.success(t("editor.snapshot.created"));
  } catch (error: any) {
    console.error("快照创建失败:", error);

    // 提取详细错误信息
    const errorMessage =
      typeof error === "string"
        ? error
        : error.message ||
          error.toString() ||
          t("editor.snapshot.unknownError");

    // 根据错误类型显示不同的提示
    if (errorMessage.includes("项目目录不存在")) {
      message.error(t("editor.snapshot.projectNotFound"));
    } else if (errorMessage.includes("权限")) {
      message.error(t("editor.snapshot.permissionDenied"));
    } else {
      message.error(t("editor.snapshot.createFailed", { error: errorMessage }));
    }
  }
};

// 更新写作记录
const upsertWritingRecord = async (currentChapterWords: number) => {
  try {
    // 计算今日新增字数 = 当前全书累计字数 - 当日初始字数
    const newWords = Math.max(
      0,
      totalWordCount.value - todayInitialWords.value
    );
    // 增加写作时长（每次保存增加30秒，约0.5分钟）
    writingDuration.value += 0.5;

    await invoke("upsert_writing_record", {
      project_id: Number(projectId.value),
      total_words: todayInitialWords.value + newWords,
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
    message.info(
      editorStore.isTypewriter
        ? t("editor.mode.typewriterOn")
        : t("editor.mode.typewriterOff")
    );
  }
  // Ctrl+Shift+F 聚焦模式
  if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === "F") {
    event.preventDefault();
    editorStore.toggleFocus();
    message.info(
      editorStore.isFocus ? t("editor.mode.focusOn") : t("editor.mode.focusOff")
    );
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
      message.info(t("editor.mode.specialExited"));
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

  // 设置加载状态
  isLoading.value = true;

  try {
    // 设置当前章节
    currentChapter.value = chapter;
    currentContent.value = ""; // 清空内容

    // 加载新章节内容
    const content = await loadChapterContentByPath(chapter.id);

    // 内容加载完成后再更新，避免中间状态导致的闪烁
    currentContent.value = content;

    // 重新加载章节树以获取最新字数
    await loadChapterTree();

    // 同步 currentChapter 的字数缓存（如果章节树中有最新数据）
    const updatedChapter = chapterTree.value
      .flatMap((v) => v.chapters)
      .find((c) => c.id === chapterId);
    if (updatedChapter) {
      currentChapter.value.word_count_cache = updatedChapter.word_count_cache;
    }

    console.log(`章节 ${chapterId} 加载成功，内容长度: ${content.length} 字符`);
  } catch (error) {
    console.error(`加载章节 ${chapterId} 失败:`, error);
    message.error(
      t("editor.loadChapterFailed", { chapterTitle: chapter.title })
    );

    // 重置状态
    currentChapter.value = null;
    currentContent.value = "";
  } finally {
    isLoading.value = false;
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

// Watch for pomodoro visibility to reposition timer
watch(showPomodoro, (visible) => {
  if (visible) {
    nextTick(() => {
      pomodoroTimerRef.value?.reposition?.();
    });
  }
});

onMounted(async () => {
  if (projectStore.recentProjects.length === 0) {
    await projectStore.getRecentProjects();
  }

  await loadChapterTree();
  await loadWritingGoal();
  await loadTodayRecord();

  if (projectId.value) {
    const worldbuildingStore = useWorldbuildingStore();
    await worldbuildingStore.loadAll(Number(projectId.value));
  }

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
  if (unlistenWindowMove.value) {
    unlistenWindowMove.value();
  }
  if (unlistenWindowResize.value) {
    unlistenWindowResize.value();
  }

  try {
    const now = new Date().toLocaleString();
    const commitMessage = formatSnapshotMessage("appClose", { time: now });
    await invoke("create_snapshot", {
      project_id: Number(projectId.value),
      message: commitMessage,
    });
  } catch {
    /* ok */
  }

  if (
    projectStore.currentProject?.encrypted &&
    projectStore.encryptionPassword
  ) {
    try {
      await invoke("reencrypt_project", {
        project_path: projectStore.currentProject.path,
        password: projectStore.encryptionPassword,
      });
    } catch {
      /* ok */
    }
  }
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
  <div
    class="h-screen flex flex-col transition-colors duration-300"
    :class="[
      isDark ? 'bg-gray-900 text-white' : 'bg-gray-50 text-gray-900',
      isZenMode ? 'bg-gray-900!' : '',
    ]"
  >
    <!-- Zen Mode Exit Button (floating) -->
    <button
      v-if="isZenMode && !isPomodoroZenMode"
      @click="exitZenMode"
      class="fixed top-4 right-4 z-50 flex items-center gap-2 px-4 py-2 bg-gray-800 text-white rounded-lg shadow-lg hover:bg-gray-700 transition-colors"
    >
      <X class="w-4 h-4" />
      {{ t("editor.zenMode.exitButton") }}
    </button>

    <!-- Pomodoro Zen Mode Indicator -->
    <button
      v-if="isPomodoroZenMode"
      @click="isPomodoroZenMode = false"
      class="fixed top-4 right-4 z-50 flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg shadow-lg hover:bg-purple-700 transition-colors"
    >
      <X class="w-4 h-4" />
      {{ t("editor.zenMode.exitFocus") }}
    </button>

    <!-- Header (hidden in zen mode) -->
    <header
      v-if="!isZenModeActive"
      class="border-b transition-colors duration-300 shrink-0"
      :class="
        isDark ? 'border-gray-700 bg-gray-800' : 'border-gray-200 bg-white'
      "
    >
      <div class="px-4 py-3 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <button
            @click="goBack"
            class="p-2 rounded-lg transition-colors duration-300"
            :class="
              isDark
                ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
            "
          >
            <ArrowLeft class="w-5 h-5" />
          </button>
          <FileText class="w-6 h-6 text-blue-600" />
          <h1 class="text-lg font-bold">{{ t("welcome.title") }}</h1>
          <n-tooltip trigger="hover">
            <template #trigger>
              <span
                class="px-3 py-1 rounded-full text-sm font-medium cursor-pointer"
                :class="
                  isDark
                    ? 'bg-gray-700 text-gray-300'
                    : 'bg-gray-100 text-gray-600'
                "
              >
                {{ truncatedProjectName }}
              </span>
            </template>
            {{ projectName }}
          </n-tooltip>
        </div>
        <div class="flex items-center gap-2">
          <n-button
            type="primary"
            size="small"
            @click="() => saveChapter()"
            :loading="isSaving"
            :disabled="!currentChapter"
          >
            <template #icon>
              <NIcon>
                <Save />
              </NIcon>
            </template>
            {{ t("editor.save") }}
          </n-button>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                @click="toggleZenMode"
                class="p-2 rounded-lg transition-colors duration-300"
                :class="
                  isZenMode
                    ? 'bg-blue-600 text-white'
                    : isDark
                    ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                    : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
                "
              >
                <Sparkles class="w-5 h-5" />
              </button>
            </template>
            {{ t("editor.zenMode.tooltip") }}
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                @click="toggleFullscreen"
                class="p-2 rounded-lg transition-colors duration-300"
                :class="
                  isFullscreen
                    ? 'bg-blue-600 text-white'
                    : isDark
                    ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                    : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
                "
              >
                <Minimize2 v-if="isFullscreen" class="w-5 h-5" />
                <Maximize2 v-else class="w-5 h-5" />
              </button>
            </template>
            {{ t("editor.fullscreenTooltip") }}
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                @click="goToProjectStats"
                class="p-2 rounded-lg transition-colors duration-300"
                :class="
                  isDark
                    ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                    : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
                "
              >
                <BarChart3 class="w-5 h-5" />
              </button>
            </template>
            {{ t("editor.projectStats") }}
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                @click="showShortcuts = true"
                class="p-2 rounded-lg transition-colors duration-300"
                :class="
                  isDark
                    ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                    : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
                "
              >
                <Keyboard class="w-5 h-5" />
              </button>
            </template>
            {{ t("editor.shortcutSettings") }}
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                @click="showPomodoro = !showPomodoro"
                class="p-2 rounded-lg transition-colors duration-300"
                :class="
                  showPomodoro
                    ? 'bg-red-500 text-white'
                    : isDark
                    ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                    : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
                "
              >
                <Timer class="w-5 h-5" />
              </button>
            </template>
            {{
              showPomodoro
                ? t("editor.pomodoroToggle.hide")
                : t("editor.pomodoroToggle.show")
            }}
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                @click="showInspirationBoard = !showInspirationBoard"
                class="p-2 rounded-lg transition-colors duration-300"
                :class="
                  showInspirationBoard
                    ? 'bg-yellow-500 text-white'
                    : isDark
                    ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                    : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
                "
              >
                <Lightbulb class="w-5 h-5" />
              </button>
            </template>
            {{
              showInspirationBoard
                ? t("editor.inspirationToggle.hide")
                : t("editor.inspirationToggle.show")
            }}
          </n-tooltip>
          <button
            @click="() => toggleDark()"
            class="p-2 rounded-lg transition-colors duration-300"
            :class="
              isDark
                ? 'bg-gray-700 hover:bg-gray-600 text-yellow-400'
                : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
            "
          >
            <Sun v-if="isDark" class="w-5 h-5" />
            <Moon v-else class="w-5 h-5" />
          </button>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                @click="goToProjectSettings"
                class="p-2 rounded-lg transition-colors duration-300"
                :class="
                  isDark
                    ? 'bg-gray-700 hover:bg-gray-600 text-gray-300'
                    : 'bg-gray-100 hover:bg-gray-200 text-gray-600'
                "
              >
                <Settings class="w-5 h-5" />
              </button>
            </template>
            {{ t("editor.projectSettings") }}
          </n-tooltip>
        </div>
      </div>
    </header>

    <div
      class="flex-1 flex overflow-hidden"
      :class="isZenModeActive ? 'p-0!' : ''"
    >
      <!-- Sidebar Toggle Button -->
      <button
        v-if="!showSidebar && !isZenModeActive"
        @click="toggleSidebar"
        class="absolute left-0 top-1/2 -translate-y-1/2 z-10 p-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-r-lg shadow-md"
      >
        <ChevronRight class="w-4 h-4" />
      </button>

      <!-- Tree Sidebar (hidden in zen mode) -->
      <div
        v-if="showSidebar && !isZenModeActive"
        class="w-64 shrink-0 relative"
      >
        <button
          @click="toggleSidebar"
          class="absolute -right-3 top-1/2 -translate-y-1/2 z-10 p-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-full shadow-md"
        >
          <ChevronLeft class="w-3 h-3" />
        </button>
        <!-- Sidebar Header with Mode Toggle -->
        <div
          class="flex items-center border-b border-gray-200 dark:border-gray-700"
        >
          <button
            class="flex-1 px-3 py-2 text-xs font-medium transition-colors"
            :class="
              sidebarMode === 'tree'
                ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
                : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
            "
            @click="sidebarMode = 'tree'"
          >
            {{ t("editor.sidebarTabs.chapterTree") }}
          </button>
          <button
            class="flex-1 px-3 py-2 text-xs font-medium transition-colors"
            :class="
              sidebarMode === 'outline'
                ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
                : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
            "
            @click="sidebarMode = 'outline'"
          >
            {{ t("editor.sidebarTabs.outline") }}
          </button>
        </div>
        <!-- Tree Sidebar -->
        <TreeSidebar
          v-if="sidebarMode === 'tree'"
          :project-id="projectId"
          @select-chapter="handleSelectChapter"
        />
        <!-- Outline Panel -->
        <OutlinePanel
          v-else
          :project-id="projectId"
          :selected-chapter-id="currentChapter?.id"
          @select-chapter="handleSelectChapter"
          @update="loadChapterTree"
        />
      </div>

      <!-- Editor Area -->
      <main
        class="flex-1 overflow-hidden flex justify-center"
        :class="isZenModeActive ? 'bg-gray-900' : 'p-4'"
      >
        <!-- Zen Mode: Centered editor container with 50% width -->
        <div
          v-if="isZenModeActive"
          class="w-1/2 mx-auto h-full flex flex-col px-8 py-6"
        >
          <div
            v-if="!currentChapter"
            class="h-full flex flex-col items-center justify-center text-gray-500"
          >
            <FileText class="w-16 h-16 mb-4 opacity-50" />
            <p class="text-lg">{{ t("editor.emptyState.selectChapter") }}</p>
          </div>
          <div v-else class="h-full flex flex-col">
            <!-- Minimal title bar in zen mode -->
            <div
              class="flex items-center gap-2 mb-3 pb-3 border-b border-gray-700"
            >
              <FileText class="w-4 h-4 text-blue-500" />
              <h2 class="text-base font-medium text-gray-300">
                {{ currentChapter.title }}
              </h2>
              <span class="text-xs text-gray-500 ml-auto"
                >{{ currentChapter.word_count_cache || 0 }}
                {{ t("editor.words") }}</span
              >
            </div>
            <!-- Markdown Editor -->
            <MarkdownEditor
              ref="editorRef"
              v-model="currentContent"
              :chapter-id="currentChapter.id"
              :project-id="Number(projectId)"
              :volume-word-count="volumeWordCount"
              :total-word-count="totalWordCount"
              :is-dark="isDark"
              :editor-mode="editorStore.mode"
              @update:model-value="handleContentUpdate"
              @mention-click="handleMentionClick"
              @show-history="showHistory = true"
              @create-snapshot="manualSnapshot"
              @word-count-updated="handleWordCountUpdated"
              @open-name-generator="openNameGenerator"
            />
          </div>
        </div>
        <!-- Normal mode editor -->
        <div v-else class="h-full w-full max-w-full flex flex-col">
          <div
            v-if="!currentChapter"
            class="h-full flex flex-col items-center justify-center text-gray-500"
          >
            <FileText class="w-16 h-16 mb-4 opacity-50" />
            <p class="text-lg">{{ t("editor.emptyState.selectChapter") }}</p>
          </div>
          <div v-else class="h-full flex flex-col">
            <!-- Chapter Title Bar -->
            <div
              class="flex items-center gap-2 mb-3 pb-3 border-b"
              :class="isDark ? 'border-gray-700' : 'border-gray-200'"
            >
              <FileText class="w-5 h-5 text-blue-600" />
              <h2 class="text-lg font-semibold">{{ currentChapter.title }}</h2>
            </div>
            <!-- Markdown Editor -->
            <MarkdownEditor
              ref="editorRef"
              v-model="currentContent"
              :chapter-id="currentChapter.id"
              :project-id="Number(projectId)"
              :volume-word-count="volumeWordCount"
              :total-word-count="totalWordCount"
              :is-dark="isDark"
              :editor-mode="editorStore.mode"
              @update:model-value="handleContentUpdate"
              @mention-click="handleMentionClick"
              @show-history="showHistory = true"
              @create-snapshot="manualSnapshot"
              @word-count-updated="handleWordCountUpdated"
              @open-name-generator="openNameGenerator"
            />
          </div>
        </div>
      </main>

      <!-- Worldbuilding Panel (Right Sidebar, hidden in zen mode) -->
      <div
        v-if="showWorldbuilding && !isZenModeActive"
        class="w-[480px] shrink-0 border-l border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 overflow-hidden flex flex-col"
      >
        <!-- Panel Header with Tabs -->
        <div
          class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700"
        >
          <div class="flex items-center gap-2">
            <Globe class="w-5 h-5 text-blue-600" />
            <h3 class="font-semibold text-gray-900 dark:text-white">
              {{ t("editor.worldbuildingPanel.title") }}
            </h3>
          </div>
          <button
            @click="showWorldbuilding = false"
            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          >
            <X class="w-5 h-5 text-gray-500" />
          </button>
        </div>
        <!-- Tabs -->
        <div class="flex border-b border-gray-200 dark:border-gray-700">
          <button
            class="flex-1 px-4 py-2 text-sm font-medium transition-colors"
            :class="
              sidebarTab === 'worldbuilding'
                ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
                : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
            "
            @click="sidebarTab = 'worldbuilding'"
          >
            {{ t("editor.worldbuildingPanel.settings") }}
          </button>
          <button
            class="flex-1 px-4 py-2 text-sm font-medium transition-colors"
            :class="
              sidebarTab === 'relationship'
                ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
                : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
            "
            @click="sidebarTab = 'relationship'"
          >
            {{ t("editor.worldbuildingPanel.relationship") }}
          </button>
          <button
            class="flex-1 px-4 py-2 text-sm font-medium transition-colors"
            :class="
              sidebarTab === 'timeline'
                ? 'text-blue-600 dark:text-blue-400 border-b-2 border-blue-600 dark:border-blue-400'
                : 'text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
            "
            @click="sidebarTab = 'timeline'"
          >
            {{ t("editor.worldbuildingPanel.timeline") }}
          </button>
        </div>
        <!-- Panel Content -->
        <div class="flex-1 overflow-hidden">
          <WorldbuildingPanel
            ref="worldbuildingPanelRef"
            v-if="sidebarTab === 'worldbuilding'"
          />
          <RelationshipGraph
            v-else-if="sidebarTab === 'relationship'"
            :project-id="Number(projectId)"
            @select-character="onSelectCharacterFromGraph"
          />
          <Timeline
            v-else-if="sidebarTab === 'timeline'"
            :project-id="Number(projectId)"
            @navigate-chapter="handleNavigateChapter"
          />
        </div>
      </div>

      <!-- Worldbuilding Toggle Button (hidden in zen mode) -->
      <button
        v-if="!showWorldbuilding && !isZenModeActive"
        @click="showWorldbuilding = true"
        class="absolute right-0 top-1/2 -translate-y-1/2 z-10 p-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-l-lg shadow-md transition-transform hover:translate-x-0"
        :style="{ top: 'calc(50% + 36px)' }"
      >
        <Globe class="w-4 h-4 text-blue-600" />
      </button>
    </div>

    <!-- Status Bar with Writing Progress (hidden in zen mode) -->
    <footer
      v-if="!isZenModeActive"
      class="border-t transition-colors duration-300 shrink-0 px-4 py-2"
      :class="
        isDark ? 'border-gray-700 bg-gray-800' : 'border-gray-200 bg-white'
      "
    >
      <div class="flex items-center gap-6">
        <!-- Daily Goal Progress -->
        <n-tooltip trigger="hover">
          <template #trigger>
            <div class="daily-goal-container">
              <Target class="w-4 h-4 text-blue-500 shrink-0" />
              <span
                class="text-sm text-gray-600 dark:text-gray-400 whitespace-nowrap"
              >
                {{
                  t("editor.statusBar.todayProgress", {
                    today: todayWords,
                    goal: dailyGoal,
                  })
                }}
              </span>
              <n-progress
                type="line"
                :percentage="dailyProgress"
                :height="8"
                :border-radius="4"
                :fill-border-radius="4"
                :color="dailyProgress >= 100 ? '#52c41a' : '#3b82f6'"
                :rail-color="isDark ? '#374151' : '#e5e7eb'"
                class="daily-goal-progress"
                :show-indicator="false"
              />
              <span
                class="text-sm font-medium whitespace-nowrap"
                :class="
                  dailyProgress >= 100 ? 'text-green-500' : 'text-blue-500'
                "
              >
                {{ dailyProgress }}%
              </span>
            </div>
          </template>
          <div>
            {{
              t("editor.statusBar.dailyGoalTooltip", {
                today: todayWords,
                goal: dailyGoal,
              })
            }}
          </div>
        </n-tooltip>

        <!-- Total Word Count -->
        <div
          class="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400"
        >
          <FileText class="w-4 h-4" />
          <span>{{
            t("editor.statusBar.totalWords", { total: totalWordCount })
          }}</span>
        </div>

        <!-- Export Button -->
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button
              size="tiny"
              quaternary
              @click="showExport = true"
              class="text-gray-500! hover:text-blue-500!"
            >
              <template #icon>
                <n-icon>
                  <Download />
                </n-icon>
              </template>
            </n-button>
          </template>
          {{ t("editor.exportDoc") }}
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button
              size="tiny"
              quaternary
              @click="showBackup = true"
              class="text-gray-500! hover:text-green-500!"
            >
              <template #icon>
                <n-icon>
                  <Package />
                </n-icon>
              </template>
            </n-button>
          </template>
          {{ t("editor.backupProject") }}
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button
              size="tiny"
              quaternary
              @click="showSensitiveWords = true"
              class="text-gray-500! hover:text-red-500!"
            >
              <template #icon>
                <n-icon>
                  <AlertTriangle />
                </n-icon>
              </template>
            </n-button>
          </template>
          {{ t("editor.sensitiveWords") }}
        </n-tooltip>
      </div>
    </footer>

    <!-- 名称生成模态框 -->
    <n-modal
      v-model:show="showNameGenerator"
      preset="card"
      :title="t('editor.nameGen.title')"
      style="width: 480px"
      :mask-closable="true"
      @update:show="onNameGeneratorClose"
    >
      <div class="space-y-4">
        <div>
          <label
            class="block text-sm font-medium mb-2"
            :class="isDark ? 'text-gray-300' : 'text-gray-700'"
            >{{ t("editor.nameGen.type") }}</label
          >
          <n-select
            v-model:value="nameCategory"
            :options="nameCategoryOptions"
            :placeholder="t('editor.nameGen.selectTypePlaceholder')"
          />
        </div>

        <div
          v-if="
            nameCategory === 'chinese_name' || nameCategory === 'western_name'
          "
        >
          <label
            class="block text-sm font-medium mb-2"
            :class="isDark ? 'text-gray-300' : 'text-gray-700'"
            >{{ t("editor.nameGen.gender") }}</label
          >
          <n-radio-group v-model:value="nameGender" name="gender">
            <n-space>
              <n-radio
                v-for="opt in genderOptions"
                :key="opt.value"
                :value="opt.value"
              >
                {{ opt.label }}
              </n-radio>
            </n-space>
          </n-radio-group>
        </div>

        <div>
          <label
            class="block text-sm font-medium mb-2"
            :class="isDark ? 'text-gray-300' : 'text-gray-700'"
            >{{ t("editor.nameGen.count") }}</label
          >
          <n-input-number v-model:value="nameCount" :min="1" :max="100" />
        </div>

        <n-button type="primary" block @click="handleGenerateNames">
          {{ t("editor.nameGen.generate") }}
        </n-button>
        <label
          class="block text-sm font-medium mb-2"
          :class="isDark ? 'text-gray-300' : 'text-gray-700'"
          >{{ t("editor.nameGen.result") }}</label
        >
        <div class="flex flex-wrap gap-2">
          <n-tag
            v-for="(name, index) in generatedNames"
            :key="index"
            :bordered="false"
            checkable
            @click="handleInsertName(name)"
            class="cursor-pointer hover:bg-blue-100 dark:hover:bg-blue-900"
          >
            {{ name }}
          </n-tag>
        </div>
      </div>
    </n-modal>

    <!-- Version History Dialog -->
    <HistoryDialog
      v-model:show="showHistory"
      :project-id="Number(projectId)"
      :current-content="currentContent"
      @restore="(content: string) => { if (content) currentContent = content }"
    />

    <!-- Sensitive Words Manager -->
    <SensitiveWordsManager
      v-model:show="showSensitiveWords"
      :project-id="Number(projectId)"
    />

    <!-- Export Dialog -->
    <ExportDialog v-model:show="showExport" :project-id="Number(projectId)" />
    <!-- Backup Dialog -->
    <BackupDialog v-model:show="showBackup" :project-id="Number(projectId)" />
    <!-- Shortcut Settings -->
    <ShortcutSettings v-model:show="showShortcuts" />

    <!-- Pomodoro Timer (floating) -->
    <PomodoroTimer
      ref="pomodoroTimerRef"
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
      :title="t('editor.inspiration.title')"
      :style="{ width: '90vw', height: '80vh' }"
      :max-height="600"
      :mask-closable="true"
    >
      <template #header-extra>
        <n-tag type="warning" size="small">{{
          t("editor.inspirationToggle.hint")
        }}</n-tag>
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
