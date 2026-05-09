<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  NCard,
  NButton,
  NSpace,
  NInput,
  NCheckbox,
  NSelect,
  NDatePicker,
  NTag,
  NProgress,
  NEmpty,
  NIcon,
  NPopconfirm,
  NModal,
  NForm,
  NFormItem,
  NText,
  NDivider,
  NGrid,
  NGi,
} from 'naive-ui'
import { Plus, Trash2, Edit3, CheckCircle2, Circle, Download, Upload, X } from 'lucide-vue-next'
import { useLocale } from '../i18n/composables/useLocale'
import type { AppLocale } from '../i18n/index'

export type TaskPriority = 'high' | 'medium' | 'low'
export type TaskStatus = 'pending' | 'in_progress' | 'completed'

export interface TaskItem {
  id: string
  name: string
  completed: boolean
  priority: TaskPriority
  dueDate: number | null
  assignee: string
  notes: string
  tags: string[]
  createdAt: number
  updatedAt: number
}

type FilterType = 'all' | 'active' | 'completed'
type SortType = 'name' | 'date' | 'priority'

const { t } = useLocale()

const tasks = ref<TaskItem[]>([])
const filter = ref<FilterType>('all')
const sortType = ref<SortType>('date')
const showAddModal = ref(false)
const showEditModal = ref(false)
const editingTask = ref<TaskItem | null>(null)

const newTaskName = ref('')
const newTaskPriority = ref<TaskPriority>('medium')
const newTaskDueDate = ref<number | null>(null)
const newTaskAssignee = ref('')
const newTaskNotes = ref('')

const priorityOptions = computed(() => [
  { label: t('task.priorityHigh'), value: 'high' },
  { label: t('task.priorityMedium'), value: 'medium' },
  { label: t('task.priorityLow'), value: 'low' },
])

const filterOptions = computed(() => [
  { label: t('task.filterAll'), value: 'all' },
  { label: t('task.filterActive'), value: 'active' },
  { label: t('task.filterCompleted'), value: 'completed' },
])

const sortOptions = computed(() => [
  { label: t('task.sortByName'), value: 'name' },
  { label: t('task.sortByDate'), value: 'date' },
  { label: t('task.sortByPriority'), value: 'priority' },
])

const priorityWeight: Record<TaskPriority, number> = {
  high: 0,
  medium: 1,
  low: 2,
}

const filteredAndSortedTasks = computed(() => {
  let result = [...tasks.value]

  if (filter.value === 'active') {
    result = result.filter((task) => !task.completed)
  } else if (filter.value === 'completed') {
    result = result.filter((task) => task.completed)
  }

  result.sort((a, b) => {
    switch (sortType.value) {
      case 'name':
        return a.name.localeCompare(b.name)
      case 'date':
        return b.createdAt - a.createdAt
      case 'priority':
        return priorityWeight[a.priority] - priorityWeight[b.priority]
      default:
        return 0
    }
  })

  return result
})

const completedCount = computed(() => tasks.value.filter((t) => t.completed).length)
const remainingCount = computed(() => tasks.value.filter((t) => !t.completed).length)
const progressPercent = computed(() => {
  if (tasks.value.length === 0) return 0
  return Math.round((completedCount.value / tasks.value.length) * 100)
})

const progressStatus = computed(() => {
  if (progressPercent.value === 100) return 'success' as const
  if (progressPercent.value >= 50) return 'info' as const
  return 'default' as const
})

function generateId(): string {
  return `task_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`
}

function resetNewTaskForm() {
  newTaskName.value = ''
  newTaskPriority.value = 'medium'
  newTaskDueDate.value = null
  newTaskAssignee.value = ''
  newTaskNotes.value = ''
}

function handleAddTask() {
  if (!newTaskName.value.trim()) return

  const task: TaskItem = {
    id: generateId(),
    name: newTaskName.value.trim(),
    completed: false,
    priority: newTaskPriority.value,
    dueDate: newTaskDueDate.value,
    assignee: newTaskAssignee.value.trim(),
    notes: newTaskNotes.value.trim(),
    tags: [],
    createdAt: Date.now(),
    updatedAt: Date.now(),
  }

  tasks.value.push(task)
  resetNewTaskForm()
  showAddModal.value = false
}

function handleEditTask() {
  if (!editingTask.value || !newTaskName.value.trim()) return

  const index = tasks.value.findIndex((t) => t.id === editingTask.value!.id)
  if (index !== -1) {
    tasks.value[index] = {
      ...tasks.value[index],
      name: newTaskName.value.trim(),
      priority: newTaskPriority.value,
      dueDate: newTaskDueDate.value,
      assignee: newTaskAssignee.value.trim(),
      notes: newTaskNotes.value.trim(),
      updatedAt: Date.now(),
    }
  }

  showEditModal.value = false
  editingTask.value = null
}

function openEditModal(task: TaskItem) {
  editingTask.value = task
  newTaskName.value = task.name
  newTaskPriority.value = task.priority
  newTaskDueDate.value = task.dueDate
  newTaskAssignee.value = task.assignee
  newTaskNotes.value = task.notes
  showEditModal.value = true
}

function toggleTaskComplete(task: TaskItem) {
  const index = tasks.value.findIndex((t) => t.id === task.id)
  if (index !== -1) {
    tasks.value[index].completed = !tasks.value[index].completed
    tasks.value[index].updatedAt = Date.now()
  }
}

function deleteTask(taskId: string) {
  tasks.value = tasks.value.filter((t) => t.id !== taskId)
}

function clearCompleted() {
  tasks.value = tasks.value.filter((t) => !t.completed)
}

function getDueDateLabel(task: TaskItem): string {
  if (!task.dueDate) return t('task.noDueDate')

  const now = new Date()
  now.setHours(0, 0, 0, 0)
  const due = new Date(task.dueDate)
  due.setHours(0, 0, 0, 0)

  const diffMs = due.getTime() - now.getTime()
  const diffDays = Math.round(diffMs / (1000 * 60 * 60 * 24))

  if (diffDays < 0) return t('task.overdue')
  if (diffDays === 0) return t('task.dueToday')
  if (diffDays === 1) return t('task.dueTomorrow')
  return t('task.dueInDays', { days: diffDays })
}

function isOverdue(task: TaskItem): boolean {
  if (!task.dueDate || task.completed) return false
  return new Date(task.dueDate).getTime() < Date.now()
}

function getPriorityColor(priority: TaskPriority): string {
  switch (priority) {
    case 'high':
      return '#ef4444'
    case 'medium':
      return '#f59e0b'
    case 'low':
      return '#22c55e'
  }
}

function getPriorityLabel(priority: TaskPriority): string {
  switch (priority) {
    case 'high':
      return t('task.priorityHigh')
    case 'medium':
      return t('task.priorityMedium')
    case 'low':
      return t('task.priorityLow')
  }
}

function exportTasks() {
  const data = JSON.stringify(tasks.value, null, 2)
  const blob = new Blob([data], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `tasks_${new Date().toISOString().slice(0, 10)}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function handleImportClick() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return

    const reader = new FileReader()
    reader.onload = (ev) => {
      try {
        const imported = JSON.parse(ev.target?.result as string)
        if (Array.isArray(imported)) {
          tasks.value = imported
        }
      } catch {
        // import error silently handled
      }
    }
    reader.readAsText(file)
  }
  input.click()
}

watch(showAddModal, (val) => {
  if (!val) resetNewTaskForm()
})

watch(showEditModal, (val) => {
  if (!val) {
    editingTask.value = null
    resetNewTaskForm()
  }
})
</script>

<template>
  <n-card class="task-checklist" :bordered="false">
    <template #header>
      <div class="task-header">
        <span class="task-title">{{ t('task.title') }}</span>
        <n-tag v-if="tasks.length > 0" :bordered="false" type="info" size="small" round>
          {{ t('task.allCount', { count: tasks.length }) }}
        </n-tag>
      </div>
    </template>

    <template #header-extra>
      <n-space align="center" :size="8">
        <n-button size="small" quaternary @click="handleImportClick">
          <template #icon><n-icon :size="16"><Upload /></n-icon></template>
          {{ t('task.importTasks') }}
        </n-button>
        <n-button size="small" quaternary :disabled="tasks.length === 0" @click="exportTasks">
          <template #icon><n-icon :size="16"><Download /></n-icon></template>
          {{ t('task.exportTasks') }}
        </n-button>
        <n-button type="primary" size="small" @click="showAddModal = true">
          <template #icon><n-icon :size="16"><Plus /></n-icon></template>
          {{ t('task.addTask') }}
        </n-button>
      </n-space>
    </template>

    <div v-if="tasks.length > 0" class="task-progress-section">
      <div class="progress-info">
        <n-text depth="3" style="font-size: 13px">
          {{ t('task.progressLabel') }}: {{ t('task.completedCount', { count: completedCount }) }} / {{ t('task.remainingCount', { count: remainingCount }) }}
        </n-text>
        <n-text depth="3" style="font-size: 13px">{{ progressPercent }}%</n-text>
      </div>
      <n-progress
        :percentage="progressPercent"
        :status="progressStatus"
        :show-indicator="false"
        :height="6"
        :border-radius="3"
      />
    </div>

    <div v-if="tasks.length > 0" class="task-toolbar">
      <n-select
        v-model:value="filter"
        :options="filterOptions"
        size="small"
        style="width: 120px; min-width: 100px"
      />
      <n-select
        v-model:value="sortType"
        :options="sortOptions"
        size="small"
        style="width: 140px; min-width: 120px"
      />
      <n-button
        v-if="completedCount > 0"
        size="small"
        quaternary
        type="warning"
        @click="clearCompleted"
      >
        {{ t('task.clearCompleted') }}
      </n-button>
    </div>

    <n-empty v-if="tasks.length === 0" :description="t('task.empty')" style="padding: 40px 0" />

    <div v-else class="task-list">
      <div
        v-for="task in filteredAndSortedTasks"
        :key="task.id"
        class="task-item"
        :class="{ 'task-item--completed': task.completed, 'task-item--overdue': isOverdue(task) }"
      >
        <div class="task-item-left">
          <n-checkbox
            :checked="task.completed"
            @update:checked="toggleTaskComplete(task)"
          />
          <div class="task-item-content">
            <span class="task-item-name" :class="{ 'task-item-name--completed': task.completed }">
              {{ task.name }}
            </span>
            <div class="task-item-meta">
              <n-tag
                :color="{ color: getPriorityColor(task.priority) + '20', textColor: getPriorityColor(task.priority), borderColor: getPriorityColor(task.priority) + '40' }"
                size="tiny"
                round
              >
                {{ getPriorityLabel(task.priority) }}
              </n-tag>
              <n-text v-if="task.dueDate" depth="3" style="font-size: 12px" :type="isOverdue(task) ? 'error' : undefined">
                {{ getDueDateLabel(task) }}
              </n-text>
              <n-text v-if="task.assignee" depth="3" style="font-size: 12px">
                {{ task.assignee }}
              </n-text>
            </div>
            <n-text v-if="task.notes" depth="3" style="font-size: 12px; display: block; margin-top: 4px">
              {{ task.notes }}
            </n-text>
          </div>
        </div>
        <div class="task-item-actions">
          <n-button quaternary circle size="tiny" @click="openEditModal(task)">
            <template #icon><n-icon :size="14"><Edit3 /></n-icon></template>
          </n-button>
          <n-popconfirm @positive-click="deleteTask(task.id)">
            <template #trigger>
              <n-button quaternary circle size="tiny" type="error">
                <template #icon><n-icon :size="14"><Trash2 /></n-icon></template>
              </n-button>
            </template>
            {{ t('task.deleteConfirm', { name: task.name }) }}
          </n-popconfirm>
        </div>
      </div>
    </div>

    <n-modal
      v-model:show="showAddModal"
      preset="card"
      :title="t('task.addTask')"
      style="width: 480px; max-width: 90vw"
      :mask-closable="false"
    >
      <n-form label-placement="left" label-width="auto" :show-feedback="false">
        <n-form-item :label="t('task.addTask')">
          <n-input
            v-model:value="newTaskName"
            :placeholder="t('task.addTaskPlaceholder')"
            autofocus
            @keydown.enter="handleAddTask"
          />
        </n-form-item>
        <n-grid :cols="2" :x-gap="12">
          <n-gi>
            <n-form-item :label="t('task.sortByPriority')">
              <n-select v-model:value="newTaskPriority" :options="priorityOptions" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item :label="t('task.dueDate')">
              <n-date-picker v-model:value="newTaskDueDate" clearable style="width: 100%" />
            </n-form-item>
          </n-gi>
        </n-grid>
        <n-form-item :label="t('task.assignee')">
          <n-input v-model:value="newTaskAssignee" :placeholder="t('task.assigneePlaceholder')" />
        </n-form-item>
        <n-form-item :label="t('task.notes')">
          <n-input
            v-model:value="newTaskNotes"
            type="textarea"
            :placeholder="t('task.notesPlaceholder')"
            :rows="3"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-space justify="end">
          <n-button @click="showAddModal = false">{{ t('common.action.cancel') }}</n-button>
          <n-button type="primary" :disabled="!newTaskName.trim()" @click="handleAddTask">
            {{ t('common.action.create') }}
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <n-modal
      v-model:show="showEditModal"
      preset="card"
      :title="t('task.editTask')"
      style="width: 480px; max-width: 90vw"
      :mask-closable="false"
    >
      <n-form label-placement="left" label-width="auto" :show-feedback="false">
        <n-form-item :label="t('task.addTask')">
          <n-input
            v-model:value="newTaskName"
            :placeholder="t('task.addTaskPlaceholder')"
            @keydown.enter="handleEditTask"
          />
        </n-form-item>
        <n-grid :cols="2" :x-gap="12">
          <n-gi>
            <n-form-item :label="t('task.sortByPriority')">
              <n-select v-model:value="newTaskPriority" :options="priorityOptions" />
            </n-form-item>
          </n-gi>
          <n-gi>
            <n-form-item :label="t('task.dueDate')">
              <n-date-picker v-model:value="newTaskDueDate" clearable style="width: 100%" />
            </n-form-item>
          </n-gi>
        </n-grid>
        <n-form-item :label="t('task.assignee')">
          <n-input v-model:value="newTaskAssignee" :placeholder="t('task.assigneePlaceholder')" />
        </n-form-item>
        <n-form-item :label="t('task.notes')">
          <n-input
            v-model:value="newTaskNotes"
            type="textarea"
            :placeholder="t('task.notesPlaceholder')"
            :rows="3"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-space justify="end">
          <n-button @click="showEditModal = false">{{ t('common.action.cancel') }}</n-button>
          <n-button type="primary" :disabled="!newTaskName.trim()" @click="handleEditTask">
            {{ t('common.action.save') }}
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </n-card>
</template>

<style scoped>
.task-checklist {
  height: 100%;
}

.task-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.task-title {
  font-size: 16px;
  font-weight: 600;
}

.task-progress-section {
  margin-bottom: 16px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.task-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.task-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 8px;
  transition: background-color 0.2s;
  gap: 8px;
}

.task-item:hover {
  background-color: var(--n-color-hover, rgba(0, 0, 0, 0.04));
}

.task-item--overdue {
  border-left: 3px solid #ef4444;
  padding-left: 9px;
}

.task-item-left {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.task-item-content {
  flex: 1;
  min-width: 0;
}

.task-item-name {
  font-size: 14px;
  word-break: break-word;
  line-height: 1.5;
}

.task-item-name--completed {
  text-decoration: line-through;
  opacity: 0.5;
}

.task-item-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
  flex-wrap: wrap;
}

.task-item-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.2s;
}

.task-item:hover .task-item-actions {
  opacity: 1;
}
</style>
