<script setup lang="ts">
import { computed } from 'vue'
import { NPopover, NTag } from 'naive-ui'
import { NodeViewWrapper } from '@tiptap/vue-3'
import type { NodeViewProps } from '@tiptap/vue-3'
import { useWorldbuildingStore } from '../stores/worldbuilding'
import { useEnumDictionary } from '../stores/enumDictionary'
import { parseMentionId } from './MentionExtension'

const props = defineProps<NodeViewProps>()

const worldbuildingStore = useWorldbuildingStore()
const enumDictionary = useEnumDictionary()

const parsed = computed(() => parseMentionId(props.node.attrs.id))

const mentionData = computed(() => {
  if (!parsed.value.type || !parsed.value.numericId) return null
  switch (parsed.value.type) {
    case 'character': return worldbuildingStore.getCharacterById(parsed.value.numericId)
    case 'location': return worldbuildingStore.getLocationById(parsed.value.numericId)
    case 'organization': return worldbuildingStore.getOrganizationById(parsed.value.numericId)
    default: return null
  }
})

const typeTagLabel = computed(() => {
  switch (parsed.value.type) {
    case 'character': return '人物'
    case 'location': return '地点'
    case 'organization': return '组织'
    default: return ''
  }
})

const typeTagType = computed(() => {
  switch (parsed.value.type) {
    case 'character': return 'info'
    case 'location': return 'success'
    case 'organization': return 'warning'
    default: return 'default'
  }
})

const typeDotClass = computed(() => {
  if (!parsed.value.type) return 'bg-gray-400'
  return ({
    character: 'bg-blue-500',
    location: 'bg-green-500',
    organization: 'bg-amber-500',
  } as Record<string, string>)[parsed.value.type]
})

function handleClick() {
  props.editor.emit('mention-click' as any, props.node.attrs.id)
}
</script>

<template>
  <NodeViewWrapper as="span">
    <NPopover
      trigger="hover"
      placement="top"
      :delay="300"
      :width="280"
      :scrollable="true"
    >
      <template #trigger>
        <span
          class="mention-inline"
          :class="{ 'mention-selected': props.selected }"
          @click="handleClick"
        >
          <span
            class="mention-dot"
            :class="typeDotClass"
          ></span>
          @{{ props.node.attrs.label }}
        </span>
      </template>

      <!-- Hover card -->
      <div class="mention-card">
        <div class="mention-card-header">
          <NTag :type="typeTagType as any" size="small">{{ typeTagLabel }}</NTag>
          <span class="mention-card-name">{{ props.node.attrs.label }}</span>
        </div>

        <template v-if="parsed.type === 'character' && mentionData">
          <div class="mention-card-body">
            <div v-if="(mentionData as any).gender" class="mention-row">
              <span class="mention-label">性别</span>
              <span class="mention-value">{{ enumDictionary.getGenderName((mentionData as any).gender) }}</span>
            </div>
            <div v-if="(mentionData as any).age" class="mention-row">
              <span class="mention-label">年龄</span>
              <span class="mention-value">{{ (mentionData as any).age }}</span>
            </div>
            <div v-if="(mentionData as any).personality" class="mention-row">
              <span class="mention-label">性格</span>
              <span class="mention-value">{{ (mentionData as any).personality }}</span>
            </div>
            <div v-if="(mentionData as any).appearance" class="mention-desc">
              {{ (mentionData as any).appearance }}
            </div>
          </div>
        </template>

        <template v-else-if="parsed.type === 'location' && mentionData">
          <div class="mention-card-body">
            <div v-if="(mentionData as any).location_type" class="mention-row">
              <span class="mention-label">类型</span>
              <span class="mention-value">{{ enumDictionary.getLocationTypeName((mentionData as any).location_type) }}</span>
            </div>
            <div v-if="(mentionData as any).description" class="mention-desc">
              {{ (mentionData as any).description }}
            </div>
          </div>
        </template>

        <template v-else-if="parsed.type === 'organization' && mentionData">
          <div class="mention-card-body">
            <div v-if="(mentionData as any).org_type" class="mention-row">
              <span class="mention-label">类型</span>
              <span class="mention-value">{{ enumDictionary.getOrganizationTypeName((mentionData as any).org_type) }}</span>
            </div>
            <div v-if="(mentionData as any).leader" class="mention-row">
              <span class="mention-label">首领</span>
              <span class="mention-value">{{ (mentionData as any).leader }}</span>
            </div>
            <div v-if="(mentionData as any).description" class="mention-desc">
              {{ (mentionData as any).description }}
            </div>
          </div>
        </template>

        <div class="mention-card-footer" @click.stop="handleClick">点击查看详情</div>
      </div>
    </NPopover>
  </NodeViewWrapper>
</template>

<style scoped>
.mention-inline {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.125rem 0.375rem;
  background-color: rgba(59, 130, 246, 0.1);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 0.25rem;
  color: #3b82f6;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  vertical-align: baseline;
  white-space: nowrap;
}

.mention-inline:hover {
  background-color: rgba(59, 130, 246, 0.2);
  border-color: rgba(59, 130, 246, 0.6);
}

.mention-selected {
  background-color: rgba(59, 130, 246, 0.3);
  border-color: #3b82f6;
  box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.3);
}

.mention-dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
}

.mention-card {
  max-width: 280px;
}

.mention-card-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--color-border, #e5e7eb);
  margin-bottom: 8px;
}

.mention-card-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--color-text-primary, #1f2937);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mention-card-body {
  font-size: 13px;
}

.mention-row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 4px;
}

.mention-label {
  color: var(--color-text-secondary, #6b7280);
  min-width: 3em;
  flex-shrink: 0;
}

.mention-value {
  color: var(--color-text-primary, #374151);
}

.mention-desc {
  margin-top: 4px;
  line-height: 1.4;
  color: var(--color-text-secondary, #6b7280);
  font-size: 12px;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.mention-card-footer {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--color-border, #e5e7eb);
  font-size: 11px;
  color: var(--color-text-secondary, #9ca3af);
  text-align: center;
}
</style>
