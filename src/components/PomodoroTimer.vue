<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from "vue";
import { NButton, NTooltip, NModal, NSlider, NSwitch, useMessage, NIcon, useDialog } from "naive-ui";
import { Play, Pause, RotateCcw, Settings, X, Volume2, VolumeX, Maximize2, Minimize2, Timer, Coffee, CoffeeIcon } from "lucide-vue-next";
import { invoke } from "@tauri-apps/api/core";
import {
  type SessionType,
  type PomodoroSettings,
  type PomodoroState,
  DEFAULT_POMODORO_SETTINGS,
  getSessionTypeName,
  getSessionTypeColor,
  formatTime,
  formatMinutes,
} from "../types/pomodoro";

const props = defineProps<{
  projectId: number;
  isDark: boolean;
  visible?: boolean;
}>();

const emit = defineEmits<{
  (e: "zen-mode", enabled: boolean): void;
}>();

// Visible state (default to hidden)
const isVisible = computed(() => props.visible ?? false);

// Drag state
const isDragging = ref(false);
const dragOffset = ref({ x: 0, y: 0 });
const position = ref({ x: 0, y: 0 });

// Handle drag start
const onDragStart = (e: MouseEvent | TouchEvent) => {
  isDragging.value = true;
  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
  
  // Get current position from style or default to right-bottom
  const rect = (e.target as HTMLElement).closest('.pomodoro-timer-container')?.getBoundingClientRect();
  if (rect) {
    dragOffset.value = {
      x: clientX - rect.left,
      y: clientY - rect.top,
    };
  } else {
    dragOffset.value = {
      x: clientX - window.innerWidth + 220,
      y: clientY - window.innerHeight + 200,
    };
  }
  
  e.preventDefault();
};

// Handle drag move
const onDragMove = (e: MouseEvent | TouchEvent) => {
  if (!isDragging.value) return;
  
  const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
  const clientY = 'touches' in e ? e.touches[0].clientY : e.clientY;
  
  const newX = clientX - dragOffset.value.x;
  const newY = clientY - dragOffset.value.y;
  
  // Keep within viewport bounds
  const minX = 0;
  const minY = 0;
  const maxX = window.innerWidth - 250;
  const maxY = window.innerHeight - 300;
  
  position.value = {
    x: Math.max(minX, Math.min(maxX, newX)),
    y: Math.max(minY, Math.min(maxY, newY)),
  };
  
  e.preventDefault();
};

// Handle drag end
const onDragEnd = () => {
  isDragging.value = false;
};

// Add global mouse/touch listeners when dragging
import { onMounted, onUnmounted } from 'vue';

onMounted(() => {
  document.addEventListener('mousemove', onDragMove);
  document.addEventListener('mouseup', onDragEnd);
  document.addEventListener('touchmove', onDragMove, { passive: false });
  document.addEventListener('touchend', onDragEnd);
});

onUnmounted(() => {
  document.removeEventListener('mousemove', onDragMove);
  document.removeEventListener('mouseup', onDragEnd);
  document.removeEventListener('touchmove', onDragMove);
  document.removeEventListener('touchend', onDragEnd);
});

const message = useMessage();
const dialog = useDialog();

// Settings
const showSettings = ref(false);
const showCompletionModal = ref(false);
const completionMessage = ref("");
const completionSessionType = ref<SessionType>("work");

const settings = ref<PomodoroSettings>({ ...DEFAULT_POMODORO_SETTINGS });

// Timer state
const state = ref<PomodoroState>({
  status: "idle",
  currentSessionType: "work",
  timeRemaining: DEFAULT_POMODORO_SETTINGS.workDuration * 60,
  totalTime: DEFAULT_POMODORO_SETTINGS.workDuration * 60,
  completedPomodoros: 0,
  currentStreak: 0,
  todayTotalMinutes: 0,
  todaySessions: 0,
});

let timerInterval: number | null = null;
let tickSound: HTMLAudioElement | null = null;
let completeSound: HTMLAudioElement | null = null;

// Audio context for tick sound
let audioContext: AudioContext | null = null;
let tickOscillator: OscillatorNode | null = null;

// Computed
const progress = computed(() => {
  if (state.value.totalTime === 0) return 0;
  return ((state.value.totalTime - state.value.timeRemaining) / state.value.totalTime) * 100;
});

const currentColor = computed(() => {
  return getSessionTypeColor(state.value.currentSessionType, props.isDark);
});

const circumference = 2 * Math.PI * 90; // radius = 90
const strokeDashoffset = computed(() => {
  return circumference * (1 - progress.value / 100);
});

// Load settings from localStorage
const loadSettings = () => {
  try {
    const stored = localStorage.getItem("pomodoro_settings");
    if (stored) {
      settings.value = { ...DEFAULT_POMODORO_SETTINGS, ...JSON.parse(stored) };
    }
    const storedState = localStorage.getItem("pomodoro_state");
    if (storedState) {
      const parsed = JSON.parse(storedState);
      state.value.todayTotalMinutes = parsed.todayTotalMinutes || 0;
      state.value.todaySessions = parsed.todaySessions || 0;
    }
  } catch {
    // ignore
  }
  updateTimeRemaining();
};

// Save settings to localStorage
const saveSettings = () => {
  localStorage.setItem("pomodoro_settings", JSON.stringify(settings.value));
  updateTimeRemaining();
};

// Update time remaining based on current session type
const updateTimeRemaining = () => {
  let duration: number;
  switch (state.value.currentSessionType) {
    case "work":
      duration = settings.value.workDuration;
      break;
    case "short_break":
      duration = settings.value.shortBreakDuration;
      break;
    case "long_break":
      duration = settings.value.longBreakDuration;
      break;
  }
  state.value.totalTime = duration * 60;
  state.value.timeRemaining = duration * 60;
};

// Initialize audio
const initAudio = () => {
  if (typeof window !== "undefined" && !audioContext) {
    audioContext = new (window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)();
  }
};

// Play tick sound
const playTickSound = () => {
  if (!settings.value.soundEnabled || !audioContext) return;
  
  try {
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();
    
    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);
    
    oscillator.frequency.value = 800;
    oscillator.type = "sine";
    
    gainNode.gain.setValueAtTime(0.1, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.001, audioContext.currentTime + 0.05);
    
    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.05);
  } catch {
    // Audio not available
  }
};

// Play completion sound
const playCompleteSound = () => {
  if (!settings.value.soundEnabled || !audioContext) return;
  
  try {
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();
    
    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);
    
    oscillator.frequency.value = 523.25; // C5
    oscillator.type = "sine";
    
    gainNode.gain.setValueAtTime(0.3, audioContext.currentTime);
    gainNode.gain.exponentialRampToValueAtTime(0.001, audioContext.currentTime + 0.3);
    
    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.3);
    
    // Second beep
    setTimeout(() => {
      if (!audioContext) return;
      const osc2 = audioContext.createOscillator();
      const gain2 = audioContext.createGain();
      osc2.connect(gain2);
      gain2.connect(audioContext.destination);
      osc2.frequency.value = 659.25; // E5
      osc2.type = "sine";
      gain2.gain.setValueAtTime(0.3, audioContext.currentTime);
      gain2.gain.exponentialRampToValueAtTime(0.001, audioContext.currentTime + 0.3);
      osc2.start(audioContext.currentTime);
      osc2.stop(audioContext.currentTime + 0.3);
    }, 200);
    
    // Third beep
    setTimeout(() => {
      if (!audioContext) return;
      const osc3 = audioContext.createOscillator();
      const gain3 = audioContext.createGain();
      osc3.connect(gain3);
      gain3.connect(audioContext.destination);
      osc3.frequency.value = 783.99; // G5
      osc3.type = "sine";
      gain3.gain.setValueAtTime(0.3, audioContext.currentTime);
      gain3.gain.exponentialRampToValueAtTime(0.001, audioContext.currentTime + 0.4);
      osc3.start(audioContext.currentTime);
      osc3.stop(audioContext.currentTime + 0.4);
    }, 400);
  } catch {
    // Audio not available
  }
};

// Start timer
const startTimer = () => {
  if (state.value.status === "running") return;
  
  initAudio();
  state.value.status = "running";
  
  // Enable zen mode if configured
  if (settings.value.zenModeEnabled && state.value.currentSessionType === "work") {
    emit("zen-mode", true);
  }
  
  timerInterval = window.setInterval(() => {
    if (state.value.timeRemaining > 0) {
      state.value.timeRemaining--;
      
      // Play tick sound in last 3 seconds
      if (state.value.timeRemaining <= 3 && state.value.timeRemaining > 0) {
        playTickSound();
      }
      
      // Save state periodically
      if (state.value.timeRemaining % 30 === 0) {
        saveStateToStorage();
      }
    } else {
      // Timer completed
      handleSessionComplete();
    }
  }, 1000);
};

// Pause timer
const pauseTimer = () => {
  if (timerInterval) {
    clearInterval(timerInterval);
    timerInterval = null;
  }
  state.value.status = "paused";
  emit("zen-mode", false);
};

// Reset timer
const resetTimer = () => {
  pauseTimer();
  state.value.status = "idle";
  updateTimeRemaining();
  emit("zen-mode", false);
};

// Handle session completion
const handleSessionComplete = async () => {
  pauseTimer();
  playCompleteSound();
  
  const completedType = state.value.currentSessionType;
  
  // Record session to backend
  try {
    const startedAt = new Date(Date.now() - state.value.totalTime * 1000).toISOString();
    let duration: number;
    
    if (completedType === "work") {
      duration = settings.value.workDuration;
      state.value.completedPomodoros++;
      state.value.currentStreak++;
      state.value.todaySessions++;
      state.value.todayTotalMinutes += duration;
    } else if (completedType === "short_break") {
      duration = settings.value.shortBreakDuration;
      state.value.currentStreak = 0;
    } else {
      duration = settings.value.longBreakDuration;
      state.value.currentStreak = 0;
    }
    
    await invoke("record_focus_session", {
      projectId: props.projectId,
      sessionType: completedType,
      durationMinutes: duration,
      startedAt,
      completed: true,
    });
    
    saveStateToStorage();
  } catch (error) {
    console.error("记录专注会话失败:", error);
  }
  
  // Show completion message
  completionSessionType.value = completedType;
  if (completedType === "work") {
    completionMessage.value = `🎉 恭喜完成一个番茄钟！已连续专注 ${state.value.currentStreak} 次`;
  } else {
    completionMessage.value = "☕ 休息结束，准备开始下一个番茄钟！";
  }
  showCompletionModal.value = true;
  
  // Determine next session
  let nextSession: SessionType;
  if (completedType === "work") {
    if (state.value.completedPomodoros % settings.value.longBreakInterval === 0) {
      nextSession = "long_break";
    } else {
      nextSession = settings.value.autoStartBreaks ? "short_break" : "work";
    }
  } else {
    nextSession = settings.value.autoStartWork ? "work" : "work";
  }
  
  state.value.currentSessionType = nextSession;
  updateTimeRemaining();
  
  // Auto-start if configured
  if ((completedType === "work" && settings.value.autoStartBreaks) ||
      (completedType !== "work" && settings.value.autoStartWork)) {
    setTimeout(() => {
      showCompletionModal.value = false;
      startTimer();
    }, 1500);
  }
};

// Save state to localStorage
const saveStateToStorage = () => {
  localStorage.setItem("pomodoro_state", JSON.stringify({
    todayTotalMinutes: state.value.todayTotalMinutes,
    todaySessions: state.value.todaySessions,
    completedPomodoros: state.value.completedPomodoros,
  }));
};

// Skip to next session
const skipSession = () => {
  pauseTimer();
  emit("zen-mode", false);
  
  let nextSession: SessionType;
  if (state.value.currentSessionType === "work") {
    if (state.value.completedPomodoros % settings.value.longBreakInterval === 0) {
      nextSession = "long_break";
    } else {
      nextSession = "short_break";
    }
  } else {
    nextSession = "work";
  }
  
  state.value.currentSessionType = nextSession;
  state.value.status = "idle";
  updateTimeRemaining();
};

// Switch to specific session type
const switchToSession = (type: SessionType) => {
  if (state.value.status === "running") {
    pauseTimer();
  }
  emit("zen-mode", false);
  state.value.currentSessionType = type;
  state.value.status = "idle";
  updateTimeRemaining();
};

// Watch for settings changes
watch(settings, () => {
  saveSettings();
}, { deep: true });

// Cleanup on unmount
onUnmounted(() => {
  if (timerInterval) {
    clearInterval(timerInterval);
  }
  if (audioContext) {
    audioContext.close();
  }
  saveStateToStorage();
});

// Initialize
loadSettings();
</script>

<template>
  <!-- Only render when visible -->
  <Teleport to="body">
    <div
      v-if="isVisible"
      class="pomodoro-timer-container fixed z-50 flex flex-col items-end gap-2"
      :style="{
        left: `${position.x}px`,
        top: `${position.y}px`,
        right: 'auto',
        bottom: 'auto',
      }"
    >
      <!-- Main Timer Card -->
      <div
        class="relative bg-white dark:bg-gray-800 rounded-2xl shadow-xl p-4 transition-all duration-300 cursor-move select-none"
        :class="[
          props.isDark ? 'shadow-gray-900/50' : 'shadow-gray-300',
          isDragging ? 'opacity-90 scale-105 shadow-2xl' : ''
        ]"
        @mousedown="onDragStart"
        @touchstart="onDragStart"
      >
        <!-- Drag Handle Indicator -->
        <div class="absolute top-2 left-1/2 -translate-x-1/2 w-8 h-1 bg-gray-300 dark:bg-gray-600 rounded-full opacity-50"></div>
      <!-- Session Type Tabs -->
      <div class="flex items-center gap-1 mb-3">
        <button
          v-for="type in ['work', 'short_break', 'long_break'] as SessionType[]"
          :key="type"
          class="px-3 py-1 text-xs font-medium rounded-full transition-all"
          :class="state.currentSessionType === type
            ? 'text-white'
            : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'"
          :style="state.currentSessionType === type ? { backgroundColor: getSessionTypeColor(type, props.isDark) } : {}"
          @click="switchToSession(type)"
        >
          {{ type === 'work' ? '专注' : type === 'short_break' ? '短休' : '长休' }}
        </button>
      </div>

      <!-- Circular Progress Timer -->
      <div class="relative w-48 h-48 mx-auto">
        <svg class="w-full h-full -rotate-90" viewBox="0 0 200 200">
          <!-- Background circle -->
          <circle
            cx="100"
            cy="100"
            r="90"
            fill="none"
            stroke="currentColor"
            stroke-width="8"
            class="text-gray-200 dark:text-gray-700"
          />
          <!-- Progress circle -->
          <circle
            cx="100"
            cy="100"
            r="90"
            fill="none"
            :stroke="currentColor"
            stroke-width="8"
            stroke-linecap="round"
            :stroke-dasharray="circumference"
            :stroke-dashoffset="strokeDashoffset"
            class="transition-all duration-1000"
          />
        </svg>
        
        <!-- Timer Display -->
        <div class="absolute inset-0 flex flex-col items-center justify-center">
          <span class="text-3xl font-bold text-gray-900 dark:text-white">
            {{ formatTime(state.timeRemaining) }}
          </span>
          <span class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            {{ getSessionTypeName(state.currentSessionType) }}
          </span>
        </div>
      </div>

      <!-- Control Buttons -->
      <div class="flex items-center justify-center gap-3 mt-4">
        <n-tooltip trigger="hover">
          <template #trigger>
            <button
              class="p-2 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
              @click="resetTimer"
            >
              <RotateCcw class="w-5 h-5" />
            </button>
          </template>
          重置
        </n-tooltip>

        <button
          class="p-4 rounded-full text-white transition-all hover:scale-105"
          :style="{ backgroundColor: currentColor }"
          @click="state.status === 'running' ? pauseTimer() : startTimer()"
        >
          <Pause v-if="state.status === 'running'" class="w-6 h-6" />
          <Play v-else class="w-6 h-6 ml-0.5" />
        </button>

        <n-tooltip trigger="hover">
          <template #trigger>
            <button
              class="p-2 rounded-full bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
              @click="skipSession"
            >
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polygon points="5,4 15,12 5,20" />
                <line x1="19" y1="5" x2="19" y2="19" />
              </svg>
            </button>
          </template>
          跳过
        </n-tooltip>
      </div>

      <!-- Stats Bar -->
      <div class="flex items-center justify-between mt-4 pt-3 border-t border-gray-100 dark:border-gray-700">
        <div class="flex items-center gap-4 text-xs">
          <div class="flex items-center gap-1">
            <Timer class="w-3.5 h-3.5 text-red-500" />
            <span class="text-gray-600 dark:text-gray-400">{{ state.completedPomodoros }}</span>
          </div>
          <div class="flex items-center gap-1">
            <Coffee class="w-3.5 h-3.5 text-green-500" />
            <span class="text-gray-600 dark:text-gray-400">{{ formatMinutes(state.todayTotalMinutes) }}</span>
          </div>
        </div>
        
        <!-- Quick Settings Buttons -->
        <div class="flex items-center gap-1">
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                class="p-1.5 rounded transition-colors"
                :class="settings.soundEnabled ? 'text-blue-500' : 'text-gray-400'"
                @click="settings.soundEnabled = !settings.soundEnabled"
              >
                <Volume2 v-if="settings.soundEnabled" class="w-4 h-4" />
                <VolumeX v-else class="w-4 h-4" />
              </button>
            </template>
            {{ settings.soundEnabled ? '关闭' : '开启' }}声音
          </n-tooltip>
          
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                class="p-1.5 rounded transition-colors"
                :class="settings.zenModeEnabled ? 'text-purple-500' : 'text-gray-400'"
                @click="settings.zenModeEnabled = !settings.zenModeEnabled"
              >
                <Maximize2 v-if="settings.zenModeEnabled" class="w-4 h-4" />
                <Minimize2 v-else class="w-4 h-4" />
              </button>
            </template>
            {{ settings.zenModeEnabled ? '关闭' : '开启' }}专注模式
          </n-tooltip>
          
          <n-tooltip trigger="hover">
            <template #trigger>
              <button
                class="p-1.5 rounded text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                @click="showSettings = true"
              >
                <Settings class="w-4 h-4" />
              </button>
            </template>
            设置
          </n-tooltip>
        </div>
      </div>
      </div>
    </div>
  </Teleport>

  <!-- Settings Modal -->
  <n-modal v-model:show="showSettings" preset="card" title="番茄钟设置" style="width: 400px">
    <div class="space-y-6">
      <!-- Work Duration -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">工作时长</label>
          <span class="text-sm text-blue-600">{{ settings.workDuration }} 分钟</span>
        </div>
        <n-slider
          v-model:value="settings.workDuration"
          :min="5"
          :max="60"
          :step="5"
          :marks="{ 15: '15', 25: '25', 45: '45' }"
        />
      </div>

      <!-- Short Break Duration -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">短休息</label>
          <span class="text-sm text-green-600">{{ settings.shortBreakDuration }} 分钟</span>
        </div>
        <n-slider
          v-model:value="settings.shortBreakDuration"
          :min="1"
          :max="15"
          :step="1"
        />
      </div>

      <!-- Long Break Duration -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">长休息</label>
          <span class="text-sm text-blue-600">{{ settings.longBreakDuration }} 分钟</span>
        </div>
        <n-slider
          v-model:value="settings.longBreakDuration"
          :min="10"
          :max="30"
          :step="5"
        />
      </div>

      <!-- Long Break Interval -->
      <div>
        <div class="flex items-center justify-between mb-2">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">长休息间隔</label>
          <span class="text-sm text-purple-600">每 {{ settings.longBreakInterval }} 个番茄</span>
        </div>
        <n-slider
          v-model:value="settings.longBreakInterval"
          :min="2"
          :max="8"
          :step="1"
        />
      </div>

      <!-- Auto Start Options -->
      <div class="space-y-3 pt-2 border-t border-gray-100 dark:border-gray-700">
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">自动开始休息</label>
          <n-switch v-model:value="settings.autoStartBreaks" />
        </div>
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium text-gray-700 dark:text-gray-300">自动开始工作</label>
          <n-switch v-model:value="settings.autoStartWork" />
        </div>
      </div>
    </div>
  </n-modal>

  <!-- Completion Modal -->
  <n-modal v-model:show="showCompletionModal" preset="card" :title="getSessionTypeName(completionSessionType) + '完成'" style="width: 320px">
    <div class="text-center py-4">
      <p class="text-lg text-gray-700 dark:text-gray-300">{{ completionMessage }}</p>
      <n-button type="primary" class="mt-4" @click="showCompletionModal = false">
        确定
      </n-button>
    </div>
  </n-modal>
</template>

<style scoped>
.n-slider {
  @apply !mx-0;
}
</style>
