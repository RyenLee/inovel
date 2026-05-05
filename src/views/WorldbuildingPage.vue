<template>
  <div class="h-full flex flex-col bg-gray-50 dark:bg-gray-900">
    <!-- Header -->
    <div class="flex items-center gap-3 px-4 py-3 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <n-button quaternary circle @click="goBack">
        <template #icon>
          <n-icon>
            <ArrowLeft />
          </n-icon>
        </template>
      </n-button>
      <WorldbuildingPanel />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router'
import { computed, onMounted } from 'vue'
import { NButton, NIcon } from 'naive-ui'
import { ArrowLeft } from 'lucide-vue-next'
import WorldbuildingPanel from '@/components/WorldbuildingPanel.vue'
import { useProjectStore } from '@/stores/project'

const router = useRouter()
const route = useRoute()
const projectStore = useProjectStore()

const projectId = computed(() => Number(route.params.projectId))

onMounted(async () => {
  if (projectId.value) {
    await projectStore.openProject(projectId.value)
  }
})

const goBack = () => {
  router.push(`/editor/${projectId.value}`)
}
</script>
