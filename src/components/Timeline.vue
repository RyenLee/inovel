<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NButton, NIcon, NModal, NInput, NDatePicker, NSelect, NPopconfirm, useMessage, NEmpty, NCard, NSpin } from 'naive-ui'
import { Plus, Trash2, Edit3, Calendar, BookOpen, GripVertical } from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import Sortable from 'sortablejs'
import { useLocale } from '../i18n/composables/useLocale'

const { t } = useLocale()

// Types
interface Event {
  id: number
  project_id: number
  title: string
  story_time: number
  description: string
  chapter_id: number | null
  created_at: string
  updated_at: string
}

interface Chapter {
  id: number
  volume_id: number
  title: string
  sort_order: number
}

// Props
const props = defineProps<{
  projectId: number
}>()

// Emits
const emit = defineEmits<{
  (e: 'navigate-chapter', chapterId: number): void
}>()

// State
const message = useMessage()
const events = ref<Event[]>([])
const chapters = ref<Chapter[]>([])
const isLoading = ref(true)

// Modal state
const showModal = ref(false)
const isEditing = ref(false)
const currentEvent = ref<Partial<Event>>({
  title: '',
  story_time: 0,
  description: '',
  chapter_id: null
})

// Load data
const loadData = async () => {
  isLoading.value = true
  try {
    // Load events
    const eventList = await invoke<Event[]>('list_events', { projectId: props.projectId })
    events.value = eventList

    // Load chapters for selection
    const chapterTree = await invoke<any[]>('get_chapter_tree', { projectId: props.projectId })
    chapters.value = flattenChapters(chapterTree)
  } catch (error) {
    console.error('Failed to load data:', error)
    message.error(t('timeline.messages.loadFailed'))
  } finally {
    isLoading.value = false
  }
}

// Flatten chapter tree for dropdown
function flattenChapters(tree: any[]): Chapter[] {
  const result: Chapter[] = []
  for (const volume of tree) {
    for (const chapter of volume.chapters || []) {
      result.push({
        id: chapter.id,
        volume_id: volume.id,
        title: `${volume.name} / ${chapter.title}`,
        sort_order: chapter.sort_order
      })
    }
  }
  return result
}

// Chapter options for select
const chapterOptions = computed(() => [
  { label: t('timeline.noChapter'), value: null },
  ...chapters.value.map(c => ({
    label: c.title,
    value: c.id
  }))
])

// Sorted events by story_time
const sortedEvents = computed(() => {
  return [...events.value].sort((a, b) => {
    return a.story_time - b.story_time
  })
})

// Open add modal
const openAddModal = () => {
  isEditing.value = false
  currentEvent.value = {
    title: '',
    story_time: Date.now(),
    description: '',
    chapter_id: null
  }
  showModal.value = true
}

// Open edit modal
const openEditModal = (event: Event) => {
  isEditing.value = true
  currentEvent.value = {
    ...event,
    story_time: event.story_time ? new Date(event.story_time).getTime() : Date.now()
  }
  showModal.value = true
}

// Save event
const saveEvent = async () => {
  if (!currentEvent.value.title?.trim()) {
    message.warning(t('timeline.messages.titleRequired'))
    return
  }

  const storyTimeStr = currentEvent.value.story_time
    ? new Date(currentEvent.value.story_time as number).toISOString()
    : ''

  try {
    if (isEditing.value && currentEvent.value.id) {
      // Update existing
      const updated = await invoke<Event>('update_event', {
        eventId: currentEvent.value.id,
        params: {
          title: currentEvent.value.title,
          story_time: storyTimeStr,
          description: currentEvent.value.description || '',
          chapter_id: currentEvent.value.chapter_id
        }
      })
      const index = events.value.findIndex(e => e.id === updated.id)
      if (index !== -1) {
        events.value[index] = updated
      }
      message.success(t('timeline.messages.eventUpdated'))
    } else {
      // Create new
      const created = await invoke<Event>('create_event', {
        params: {
          project_id: props.projectId,
          title: currentEvent.value.title,
          story_time: storyTimeStr,
          description: currentEvent.value.description || '',
          chapter_id: currentEvent.value.chapter_id
        }
      })
      events.value.push(created)
      message.success(t('timeline.messages.eventCreated'))
    }
    showModal.value = false
  } catch (error) {
    console.error('Failed to save event:', error)
    message.error(t('timeline.messages.saveFailed'))
  }
}

// Delete event
const deleteEvent = async (eventId: number) => {
  try {
    await invoke('delete_event', { eventId })
    events.value = events.value.filter(e => e.id !== eventId)
    message.success(t('timeline.messages.eventDeleted'))
  } catch (error) {
    console.error('Failed to delete event:', error)
    message.error(t('timeline.messages.deleteFailed'))
  }
}

// Navigate to chapter
const navigateToChapter = (chapterId: number) => {
  emit('navigate-chapter', chapterId)
}

// Get chapter title by id
const getChapterTitle = (chapterId: number | null) => {
  if (!chapterId) return null
  const chapter = chapters.value.find(c => c.id === chapterId)
  return chapter?.title || null
}

// Format date for display
const formatDate = (ts: number | string) => {
  if (!ts) return ''
  try {
    const date = new Date(ts)
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    })
  } catch {
    return String(ts)
  }
}

// Initialize sortable
const timelineRef = ref<HTMLElement | null>(null)
let sortableInstance: Sortable | null = null

onMounted(() => {
  loadData()
  initSortable()
})

function initSortable() {
  if (!timelineRef.value) return
  
  sortableInstance = Sortable.create(timelineRef.value, {
    handle: '.drag-handle',
    animation: 150,
    ghostClass: 'sortable-ghost',
    onEnd: async (_evt: any) => {
      // Reorder is handled by story_time, so no need to update order here
      // This is just for visual feedback
    }
  })
}
</script>

<template>
  <div class="h-full flex flex-col bg-gray-50 dark:bg-gray-900">
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-3 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <div class="flex items-center gap-2">
        <Calendar class="w-5 h-5 text-purple-600" />
        <h3 class="font-semibold text-gray-900 dark:text-white">{{ t('timeline.title') }}</h3>
        <span class="text-sm text-gray-500 dark:text-gray-400">({{ t('timeline.eventCount', { count: events.length }) }})</span>
      </div>
      <NButton type="primary" size="small" @click="openAddModal">
        <template #icon>
          <NIcon><Plus /></NIcon>
        </template>
        {{ t('timeline.addEvent') }}
      </NButton>
    </div>

    <!-- Timeline -->
    <div class="flex-1 overflow-auto p-4">
      <div v-if="isLoading" class="flex items-center justify-center h-full">
        <NSpin size="large" />
      </div>

      <div v-else-if="events.length === 0" class="flex items-center justify-center h-full">
        <NEmpty :description="t('timeline.emptyDescription')" />
      </div>

      <div v-else ref="timelineRef" class="relative">
        <!-- Timeline line -->
        <div class="absolute left-6 top-0 bottom-0 w-0.5 bg-purple-200 dark:bg-purple-800"></div>

        <!-- Events -->
        <div
          v-for="event in sortedEvents"
          :key="event.id"
          class="relative pl-14 pb-6 last:pb-0"
        >
          <!-- Timeline dot -->
          <div class="absolute left-4 top-1 w-5 h-5 rounded-full bg-purple-500 border-4 border-white dark:border-gray-800 shadow z-10"></div>

          <!-- Event card -->
          <NCard
            class="cursor-pointer hover:shadow-md transition-shadow"
            size="small"
          >
            <div class="flex items-start gap-3">
              <!-- Drag handle -->
              <div class="drag-handle cursor-grab text-gray-400 hover:text-gray-600">
                <GripVertical class="w-4 h-4" />
              </div>

              <div class="flex-1 min-w-0">
                <!-- Title and actions -->
                <div class="flex items-center justify-between gap-2 mb-1">
                  <h4 class="font-medium text-gray-900 dark:text-white truncate">
                    {{ event.title }}
                  </h4>
                  <div class="flex items-center gap-1 shrink-0">
                    <NButton
                      quaternary
                      size="tiny"
                      @click.stop="openEditModal(event)"
                    >
                      <template #icon>
                        <NIcon><Edit3 class="w-3.5 h-3.5" /></NIcon>
                      </template>
                    </NButton>
                    <NPopconfirm @positive-click="deleteEvent(event.id)">
                      <template #trigger>
                        <NButton quaternary size="tiny" type="error">
                          <template #icon>
                            <NIcon><Trash2 class="w-3.5 h-3.5" /></NIcon>
                          </template>
                        </NButton>
                      </template>
                      {{ t('timeline.deleteConfirm') }}
                    </NPopconfirm>
                  </div>
                </div>

                <!-- Story time -->
                <div class="flex items-center gap-1.5 text-sm text-gray-500 mb-2">
                  <Calendar class="w-3.5 h-3.5" />
                  <span>{{ formatDate(event.story_time) }}</span>
                </div>

                <!-- Description -->
                <p v-if="event.description" class="text-sm text-gray-600 dark:text-gray-400 mb-2 line-clamp-2">
                  {{ event.description }}
                </p>

                <!-- Related chapter -->
                <div
                  v-if="event.chapter_id"
                  class="flex items-center gap-1.5 text-sm text-blue-600 dark:text-blue-400 cursor-pointer hover:underline"
                  @click="navigateToChapter(event.chapter_id!)"
                >
                  <BookOpen class="w-3.5 h-3.5" />
                  <span class="truncate">{{ getChapterTitle(event.chapter_id) }}</span>
                </div>
              </div>
            </div>
          </NCard>
        </div>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <NModal
      v-model:show="showModal"
      preset="card"
      :title="isEditing ? t('timeline.editEvent') : t('timeline.addEventTitle')"
      style="width: 500px"
    >
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-2">{{ t('timeline.eventTitle') }} <span class="text-red-500">{{ t('timeline.eventTitleRequired') }}</span></label>
          <NInput
            v-model:value="currentEvent.title"
            :placeholder="t('timeline.eventTitlePlaceholder')"
          />
        </div>

        <div>
          <label class="block text-sm font-medium mb-2">{{ t('timeline.storyTime') }}</label>
          <NDatePicker
            v-model:value="currentEvent.story_time"
            type="date"
            clearable
            class="w-full"
            :placeholder="t('timeline.storyTimePlaceholder')"
          />
        </div>

        <div>
          <label class="block text-sm font-medium mb-2">{{ t('timeline.eventDescription') }}</label>
          <NInput
            v-model:value="currentEvent.description"
            type="textarea"
            :rows="3"
            :placeholder="t('timeline.eventDescriptionPlaceholder')"
          />
        </div>

        <div>
          <label class="block text-sm font-medium mb-2">{{ t('timeline.relatedChapter') }}</label>
          <NSelect
            v-model:value="currentEvent.chapter_id"
            :options="(chapterOptions as any)"
            :placeholder="t('timeline.relatedChapterPlaceholder')"
            clearable
            filterable
          />
        </div>
      </div>

      <template #footer>
        <div class="flex justify-end gap-2">
          <NButton @click="showModal = false">{{ t('common.action.cancel') }}</NButton>
          <NButton type="primary" @click="saveEvent">{{ t('common.action.save') }}</NButton>
        </div>
      </template>
    </NModal>
  </div>
</template>

<style scoped>
.sortable-ghost {
  opacity: 0.4;
  background-color: var(--color-hover, #e0e7ff);
}
</style>
