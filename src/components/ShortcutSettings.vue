<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { NButton, NModal, NTag, useMessage, NUpload, useDialog } from 'naive-ui'
import { Download, Upload, RotateCcw, Save } from 'lucide-vue-next'
import { useShortcutStore, type Shortcut } from '../stores/shortcuts'
import { useLocale } from '../i18n/composables/useLocale'

const { t } = useLocale()

const props = defineProps<{
  show?: boolean
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
}>()

const message = useMessage()
const dialog = useDialog()
const shortcutStore = useShortcutStore()

// Editing state
const editingId = ref<string | null>(null)
const recordingKeys = ref<string[]>([])
const isRecording = ref(false)

// Key capture handler
const handleKeyDown = (event: KeyboardEvent) => {
  if (!isRecording.value) return

  event.preventDefault()
  event.stopPropagation()

  const keys: string[] = []
  if (event.ctrlKey) keys.push('Ctrl')
  if (event.altKey) keys.push('Alt')
  if (event.shiftKey) keys.push('Shift')
  if (event.metaKey) keys.push('Meta')

  const key = event.key
  if (!['Control', 'Alt', 'Shift', 'Meta'].includes(key)) {
    // Normalize key name
    let normalizedKey = key
    if (key === ' ') normalizedKey = 'Space'
    else if (key.length === 1) normalizedKey = key.toUpperCase()
    keys.push(normalizedKey)
    recordingKeys.value = keys
  }
}

const handleKeyUp = () => {
  if (!isRecording.value) return
  
  if (recordingKeys.value.length > 0) {
    // Save the recorded keys
    if (editingId.value) {
      shortcutStore.updateShortcut(editingId.value, [...recordingKeys.value])
      message.success(t('shortcutSettings.messages.shortcutUpdated'))
    }
    stopRecording()
  }
}

// Start recording
const startRecording = (shortcut: Shortcut) => {
  editingId.value = shortcut.id
  recordingKeys.value = []
  isRecording.value = true
  message.info(t('shortcutSettings.messages.pressNewShortcut'))
}

// Stop recording
const stopRecording = () => {
  isRecording.value = false
  editingId.value = null
  recordingKeys.value = []
}

// Cancel recording
const cancelRecording = () => {
  stopRecording()
}

// Reset single shortcut
const resetShortcut = (shortcut: Shortcut) => {
  dialog.warning({
    title: t('shortcutSettings.resetDefault'),
    content: t('shortcutSettings.messages.resetDefaultConfirm', { name: shortcut.name }),
    positiveText: t('common.action.confirm'),
    negativeText: t('common.action.cancel'),
    onPositiveClick: () => {
      shortcutStore.resetToDefault(shortcut.id)
      message.success(t('shortcutSettings.messages.resetDefaultDone'))
    },
  })
}

// Reset all shortcuts
const resetAllShortcuts = () => {
  dialog.warning({
    title: t('shortcutSettings.resetAll'),
    content: t('shortcutSettings.messages.resetAllConfirm'),
    positiveText: t('common.action.confirm'),
    negativeText: t('common.action.cancel'),
    onPositiveClick: () => {
      shortcutStore.resetAll()
      message.success(t('shortcutSettings.messages.resetAllDone'))
    },
  })
}

// Export shortcuts
const exportShortcuts = () => {
  const json = shortcutStore.exportConfig()
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'inovel-shortcuts.json'
  a.click()
  URL.revokeObjectURL(url)
  message.success(t('shortcutSettings.messages.exportSuccess'))
}

// Import shortcuts
const importShortcuts = (options: { file: File }) => {
  const file = options.file
  const reader = new FileReader()
  reader.onload = (e) => {
    const content = e.target?.result as string
    if (shortcutStore.importConfig(content)) {
      message.success(t('shortcutSettings.messages.importSuccess'))
    } else {
      message.error(t('shortcutSettings.messages.importFailed'))
    }
  }
  reader.readAsText(file)
  return false // Prevent auto upload
}

// Format key combination for display
const formatKeys = (keys: string[]): string => {
  return keys.join(' + ')
}

// Check if shortcut is being edited
const isEditing = (id: string): boolean => {
  return editingId.value === id
}

// Close modal
const closeModal = () => {
  if (isRecording.value) {
    cancelRecording()
  }
  emit('update:show', false)
}

// Register key listener when recording
onMounted(() => {
  window.addEventListener('keydown', handleKeyDown)
  window.addEventListener('keyup', handleKeyUp)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
  window.removeEventListener('keyup', handleKeyUp)
})
</script>

<template>
  <n-modal
    :show="props.show"
    preset="card"
    :title="t('shortcutSettings.title')"
    style="width: 700px; max-width: 90vw;"
    :mask-closable="!isRecording"
    @update:show="(val) => !isRecording && emit('update:show', val)"
  >
    <div class="space-y-6">
      <!-- Recording indicator -->
      <div v-if="isRecording" class="flex items-center gap-3 p-4 bg-blue-50 dark:bg-blue-900/30 rounded-lg border border-blue-200 dark:border-blue-800">
        <div class="w-3 h-3 bg-blue-500 rounded-full animate-pulse"></div>
        <span class="text-blue-700 dark:text-blue-300 font-medium">{{ t('shortcutSettings.recording') }}</span>
        <span class="ml-2 px-3 py-1 bg-blue-100 dark:bg-blue-800 rounded text-blue-600 dark:text-blue-200 font-mono">
          {{ recordingKeys.length > 0 ? formatKeys(recordingKeys) : t('shortcutSettings.pressKeys') }}
        </span>
        <n-button size="small" quaternary class="ml-auto" @click="cancelRecording">
          {{ t('shortcutSettings.cancel') }}
        </n-button>
      </div>

      <!-- Shortcuts list by category -->
      <div v-for="(shortcuts, category) in shortcutStore.groupedShortcuts" :key="category" class="space-y-2">
        <h3 class="text-sm font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wide">
          {{ category }}
        </h3>
        
        <div class="space-y-1">
          <div
            v-for="shortcut in shortcuts"
            :key="shortcut.id"
            class="flex items-center justify-between py-2 px-3 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
            :class="isEditing(shortcut.id) ? 'bg-blue-50 dark:bg-blue-900/20 ring-2 ring-blue-300 dark:ring-blue-600' : ''"
          >
            <div class="flex-1">
              <span class="font-medium">{{ shortcut.name }}</span>
              <span v-if="shortcut.description" class="ml-2 text-sm text-gray-500 dark:text-gray-400">
                {{ shortcut.description }}
              </span>
            </div>
            
            <div class="flex items-center gap-2">
              <!-- Key display -->
              <div class="flex gap-1">
                <n-tag
                  v-if="isEditing(shortcut.id)"
                  type="info"
                  size="small"
                  class="font-mono"
                >
                  {{ recordingKeys.length > 0 ? formatKeys(recordingKeys) : t('shortcutSettings.waitingInput') }}
                </n-tag>
                <template v-else>
                  <n-tag
                    v-for="(key, idx) in shortcut.keys"
                    :key="idx"
                    size="small"
                    :type="JSON.stringify(shortcut.keys) === JSON.stringify(shortcut.defaultKeys) ? 'default' : 'warning'"
                    class="font-mono"
                  >
                    {{ key }}
                  </n-tag>
                </template>
              </div>
              
              <!-- Actions -->
              <div class="flex gap-1 ml-2">
                <n-button
                  v-if="isEditing(shortcut.id)"
                  size="tiny"
                  type="primary"
                  :disabled="recordingKeys.length === 0"
                  @click="stopRecording"
                >
                  <template #icon>
                    <Save class="w-3 h-3" />
                  </template>
                  {{ t('shortcutSettings.save') }}
                </n-button>
                <n-button
                  v-else
                  size="tiny"
                  quaternary
                  @click="startRecording(shortcut)"
                >
                  {{ t('shortcutSettings.edit') }}
                </n-button>
                <n-button
                  v-if="!isEditing(shortcut.id)"
                  size="tiny"
                  quaternary
                  :disabled="JSON.stringify(shortcut.keys) === JSON.stringify(shortcut.defaultKeys)"
                  @click="resetShortcut(shortcut)"
                >
                  <template #icon>
                    <RotateCcw class="w-3 h-3" />
                  </template>
                </n-button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex items-center justify-between pt-4 border-t border-gray-200 dark:border-gray-700">
        <n-button @click="resetAllShortcuts">
          <template #icon>
            <RotateCcw class="w-4 h-4" />
          </template>
          {{ t('shortcutSettings.resetAll') }}
        </n-button>
        
        <div class="flex gap-2">
          <n-upload
            :multiple="false"
            accept=".json"
            :show-file-list="false"
            :custom-request="(importShortcuts as any)"
          >
            <n-button>
              <template #icon>
                <Upload class="w-4 h-4" />
              </template>
              {{ t('shortcutSettings.import') }}
            </n-button>
          </n-upload>
          
          <n-button type="primary" @click="exportShortcuts">
            <template #icon>
              <Download class="w-4 h-4" />
            </template>
            {{ t('shortcutSettings.export') }}
          </n-button>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="flex justify-end gap-2">
        <n-button @click="closeModal">{{ t('shortcutSettings.close') }}</n-button>
      </div>
    </template>
  </n-modal>
</template>

<style scoped>
</style>
