<template>
  <div class="config-page">
    <ConfigManager @config-updated="handleConfigUpdated" />
  </div>
</template>

<script setup lang="ts">
import { useConfig } from '../composables/useConfig'
import ConfigManager from '../components/ConfigManager.vue'
import type { TomlConfig } from '../config/types'

const { loadConfig } = useConfig()

function handleConfigUpdated(config: TomlConfig) {
  console.log('Config updated:', config)
}
</script>

<style scoped>
.config-page {
  min-height: 100vh;
  background: #f5f7fa;
  padding: 20px;
}
</style>