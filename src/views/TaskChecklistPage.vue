<script setup lang="ts">
import { useRouter } from 'vue-router'
import { NButton, NIcon } from 'naive-ui'
import { ArrowLeft } from 'lucide-vue-next'
import TaskChecklist from '../components/TaskChecklist.vue'

const router = useRouter()

const goBack = () => {
  router.push('/')
}
</script>

<template>
  <div class="task-page min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300">
    <header class="border-b bg-white dark:bg-gray-800 dark:border-gray-700 transition-colors duration-300">
      <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center gap-4">
        <n-button quaternary circle @click="goBack">
          <template #icon>
            <NIcon><ArrowLeft /></NIcon>
          </template>
        </n-button>
      </div>
    </header>

    <main class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <TaskChecklist />
    </main>
  </div>
</template>

<style scoped>
.task-page {
  height: 100vh;
  overflow-y: auto;
}
</style>
