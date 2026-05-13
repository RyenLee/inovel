<script setup lang="ts">
import { ref, watch } from 'vue'
import { NModal, NButton, NIcon, NSpin, NEmpty, NPopconfirm, NTag, useMessage, NAlert } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { History, RotateCcw, ArrowLeft } from 'lucide-vue-next'
import { useLocale } from '../i18n/composables/useLocale'

const { t } = useLocale()

interface Snapshot {
  hash: string
  message: string
  date: string
}

const props = defineProps<{
  projectId: number
  show: boolean
  currentContent?: string
}>()

const emit = defineEmits<{
  (e: 'update:show', val: boolean): void
  (e: 'restore', content: string): void
}>()

const message = useMessage()
const snapshots = ref<Snapshot[]>([])
const loading = ref(false)

// Diff state
const selectedA = ref<string | null>(null)
const selectedB = ref<string | null>(null)
const diffResult = ref('')
const showDiffPanel = ref(false)
const diffLoading = ref(false)

// Restore state
const restoring = ref(false)

// Load on show
watch(() => props.show, (v) => { if (v) loadSnapshots() })

const loadSnapshots = async () => {
  loading.value = true
  try {
    snapshots.value = await invoke<Snapshot[]>('get_snapshots', { project_id: props.projectId })
  } catch (e) {
    console.error('加载版本失败:', e)
    message.warning(t('history.messages.noHistoryWarning'))
  } finally {
    loading.value = false
  }
}

const formatDate = (iso: string) => {
  try { return new Date(iso).toLocaleString('zh-CN') }
  catch { return iso }
}

const shortHash = (hash: string) => hash.slice(0, 8)

function selectForCompare(hash: string) {
  if (!selectedA.value) {
    selectedA.value = hash
    selectedB.value = null
    showDiffPanel.value = false
  } else if (selectedA.value === hash) {
    // Deselect A
    selectedA.value = null
    selectedB.value = null
    showDiffPanel.value = false
  } else if (!selectedB.value) {
    selectedB.value = hash
    loadDiff()
  } else if (selectedB.value === hash) {
    // Deselect B
    selectedB.value = null
    showDiffPanel.value = false
  } else {
    // Swap: make the new hash B, keep A
    selectedB.value = hash
    loadDiff()
  }
}

async function loadDiff() {
  if (!selectedA.value || !selectedB.value) return
  diffLoading.value = true
  showDiffPanel.value = true
  try {
    diffResult.value = await invoke<string>('get_snapshot_diff', {
      project_id: props.projectId,
      from_hash: selectedA.value,
      to_hash: selectedB.value,
    })
  } catch (e) {
    diffResult.value = t('history.messages.diffLoadFailed', { error: String(e) })
  } finally {
    diffLoading.value = false
  }
}

function formatDiffLine(line: string, index: number): string {
  if (line.startsWith('@@')) {
    return `<div class="diff-hunk">${escapeHtml(line)}</div>`
  }
  if (line.startsWith('+')) {
    return `<div class="diff-add">${escapeHtml(line)}</div>`
  }
  if (line.startsWith('-')) {
    return `<div class="diff-remove">${escapeHtml(line)}</div>`
  }
  if (line.startsWith('---') || line.startsWith('+++')) {
    return `<div class="diff-header">${escapeHtml(line)}</div>`
  }
  return `<div class="diff-context">${escapeHtml(line)}</div>`
}

function escapeHtml(s: string) {
  const el = document.createElement('span')
  el.textContent = s
  return el.innerHTML
}

const diffHtml = () => diffResult.value.split('\n').map(formatDiffLine).join('')

function backToList() {
  selectedA.value = null
  selectedB.value = null
  showDiffPanel.value = false
  diffResult.value = ''
}

const doRestore = async (hash: string) => {
  restoring.value = true
  try {
    await invoke('restore_snapshot', { project_id: props.projectId, commit_hash: hash })
    message.success(t('history.messages.restoreSuccess'))
    emit('restore', '')
    emit('update:show', false)
  } catch (e) {
    message.error(t('history.messages.restoreFailed', { error: String(e) }))
  } finally {
    restoring.value = false
  }
}
</script>

<template>
  <NModal
    :show="show"
    @update:show="$emit('update:show', $event)"
    preset="card"
    :title="t('history.title')"
    style="width: 750px; max-width: 90vw;"
    :mask-closable="false"
  >
    <div class="history-dialog">
      <!-- Back button when in diff view -->
      <div v-if="showDiffPanel" class="mb-3">
        <NButton size="tiny" quaternary @click="backToList">
          <template #icon><NIcon><ArrowLeft /></NIcon></template>
          {{ t('history.backToList') }}
        </NButton>
      </div>

      <!-- Loading -->
      <div v-if="loading && !showDiffPanel" class="flex justify-center py-8">
        <NSpin size="large" />
      </div>

      <!-- Empty -->
      <div v-else-if="snapshots.length === 0 && !showDiffPanel" class="py-8">
        <NEmpty :description="t('history.noHistory')">
          <template #extra>
            <p class="text-sm text-gray-500">{{ t('history.noHistoryHint') }}</p>
          </template>
        </NEmpty>
      </div>

      <!-- Version list -->
      <template v-if="!showDiffPanel">
        <!-- Selected version hint -->
        <NAlert v-if="selectedA" type="info" :bordered="false" class="mb-3">
          <template #header>
            {{ t('history.selectedA', { hash: shortHash(selectedA) }) }}
          </template>
        </NAlert>

        <div v-if="snapshots.length > 0" class="history-list">
          <div
            v-for="snap in snapshots"
            :key="snap.hash"
            class="history-item"
            :class="{
              'selected-a': selectedA === snap.hash && !selectedB,
              'selected-b': selectedB === snap.hash,
              'selected-pair': selectedA && selectedB && (selectedA === snap.hash || selectedB === snap.hash)
            }"
            @click="selectForCompare(snap.hash)"
          >
            <div class="history-item-main">
              <div class="history-item-left">
                <NButton
                  size="tiny"
                  quaternary
                  circle
                  :type="selectedA === snap.hash && !selectedB ? 'primary' : 'default'"
                  class="select-btn"
                >
                  {{ selectedB === snap.hash ? 'B' : selectedA === snap.hash ? 'A' : '' }}
                </NButton>
                <History class="w-4 h-4 text-blue-500 shrink-0" />
                <div>
                  <div class="history-item-msg">{{ snap.message }}</div>
                  <div class="history-item-meta">
                    <NTag size="tiny" type="info">{{ shortHash(snap.hash) }}</NTag>
                    <span class="text-xs text-gray-400">{{ formatDate(snap.date) }}</span>
                  </div>
                </div>
              </div>
              <div class="history-item-actions">
                <NPopconfirm
                  @positive-click="doRestore(snap.hash)"
                  :positive-text="t('history.confirmRestore')"
                  :negative-text="t('history.cancel')"
                >
                  <template #default>
                    <div style="max-width: 280px">
                      <p><strong>{{ t('history.restoreConfirmTitle') }}</strong></p>
                      <ul style="padding-left: 16px; margin: 8px 0;">
                        <li>{{ t('history.restoreConfirmItem1') }}</li>
                        <li>{{ t('history.restoreConfirmItem2') }}</li>
                        <li>{{ t('history.restoreConfirmItem3') }}</li>
                      </ul>
                      <p class="text-xs text-gray-400">{{ t('history.restoreConfirmQuestion') }}</p>
                    </div>
                  </template>
                  <template #trigger>
                    <NButton size="tiny" quaternary type="warning" :loading="restoring">
                      <template #icon><NIcon><RotateCcw /></NIcon></template>
                      {{ t('history.restore') }}
                    </NButton>
                  </template>
                </NPopconfirm>
              </div>
            </div>
          </div>
        </div>

        <!-- Restore hint -->
        <NAlert v-if="snapshots.length > 0" type="warning" :bordered="false" class="mt-3">
          <template #header>
            {{ t('history.restoreHint') }}
          </template>
        </NAlert>
      </template>

      <!-- Diff Panel -->
      <div v-if="showDiffPanel" class="diff-panel">
        <div class="diff-info" v-if="selectedA && selectedB">
          <span class="text-sm font-medium">{{ t('history.compare') }}</span>
          <NTag size="small" type="primary">{{ shortHash(selectedA) }}</NTag>
          <span class="text-sm text-gray-500"> → </span>
          <NTag size="small" type="primary">{{ shortHash(selectedB) }}</NTag>
        </div>

        <div v-if="diffLoading" class="flex justify-center py-4">
          <NSpin size="small" />
        </div>

        <div v-else-if="!diffResult" class="py-4 text-center text-sm text-gray-400">
          {{ t('history.diffLoadFailed') }}
        </div>

        <div
          v-else
          class="diff-content"
          v-html="diffHtml()"
        ></div>
      </div>
    </div>
  </NModal>
</template>

<style scoped>
.history-dialog { min-height: 200px; }

.history-list {
  max-height: 350px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.history-item {
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
  padding: 10px 12px;
  transition: all 0.15s;
  cursor: pointer;
}

.history-item:hover { border-color: #bfdbfe; background: #f0f7ff; }
.dark .history-item:hover { border-color: #1e3a5f; background: #0f1f33; }

.history-item.selected-a { border-color: #3b82f6; background: #eff6ff; }
.dark .history-item.selected-a { border-color: #60a5fa; background: #0f1f33; }

.history-item.selected-b { border-color: #f59e0b; background: #fffbeb; }
.dark .history-item.selected-b { border-color: #fbbf24; background: #1f1a0f; }

.history-item.selected-pair { border-color: #8b5cf6; }
.dark .history-item.selected-pair { border-color: #a78bfa; }

.history-item-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.history-item-left {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.select-btn { min-width: 24px; font-size: 11px; font-weight: 700; flex-shrink: 0; }

.history-item-msg { font-weight: 500; color: var(--color-text-primary, #1f2937); font-size: 14px; word-break: break-all; }
.history-item-meta { display: flex; align-items: center; gap: 8px; margin-top: 4px; }
.history-item-actions { display: flex; gap: 4px; flex-shrink: 0; }

.diff-panel { margin-top: 12px; }
.diff-info { display: flex; align-items: center; gap: 6px; margin-bottom: 8px; }

.diff-content {
  background: #1e1e1e;
  border: 1px solid #333;
  border-radius: 8px;
  padding: 12px;
  max-height: 350px;
  overflow: auto;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.diff-content :deep(.diff-hunk) {
  color: #569cd6;
  background: rgba(86, 156, 214, 0.08);
  padding: 1px 4px;
}

.diff-content :deep(.diff-add) {
  color: #6a9955;
  background: rgba(106, 153, 85, 0.15);
  padding: 1px 4px;
}

.diff-content :deep(.diff-remove) {
  color: #f14c4c;
  background: rgba(241, 76, 76, 0.12);
  padding: 1px 4px;
}

.diff-content :deep(.diff-header) {
  color: #808080;
  font-weight: 600;
  padding: 2px 4px;
  background: rgba(128, 128, 128, 0.1);
  margin-top: 4px;
}

.diff-content :deep(.diff-context) {
  color: #d4d4d4;
  padding: 0 4px;
}
</style>
