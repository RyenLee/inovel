import { defineStore } from "pinia";
import { ref, computed } from "vue";

export type ChapterStatus = 'outline' | 'draft' | 'revised' | 'final' | 'abandoned';
export type ChapterStatusFilter = ChapterStatus | 'all';

export const CHAPTER_STATUS_OPTIONS = [
  { label: '大纲', value: 'outline', color: '#9CA3AF' },
  { label: '草稿', value: 'draft', color: '#F59E0B' },
  { label: '修订', value: 'revised', color: '#3B82F6' },
  { label: '定稿', value: 'final', color: '#10B981' },
  { label: '废弃', value: 'abandoned', color: '#EF4444' },
] as const;

export const getStatusColor = (status: ChapterStatus): string => {
  const option = CHAPTER_STATUS_OPTIONS.find(o => o.value === status);
  return option?.color || '#9CA3AF';
};

export const getStatusLabel = (status: ChapterStatus): string => {
  const option = CHAPTER_STATUS_OPTIONS.find(o => o.value === status);
  return option?.label || status;
};

export const useTreeStore = defineStore("tree", () => {
  // Status filter state
  const statusFilter = ref<ChapterStatusFilter>('all');

  // Status filter options for UI
  const statusFilterOptions = [
    { label: '全部章节', value: 'all' },
    ...CHAPTER_STATUS_OPTIONS,
  ];

  // Check if filter is active
  const isFilterActive = computed(() => statusFilter.value !== 'all');

  // Set status filter
  function setStatusFilter(filter: ChapterStatusFilter) {
    statusFilter.value = filter;
  }

  // Clear filter
  function clearFilter() {
    statusFilter.value = 'all';
  }

  // Toggle filter
  function toggleStatusFilter(status: ChapterStatus) {
    if (statusFilter.value === status) {
      statusFilter.value = 'all';
    } else {
      statusFilter.value = status;
    }
  }

  return {
    statusFilter,
    statusFilterOptions,
    isFilterActive,
    setStatusFilter,
    clearFilter,
    toggleStatusFilter,
  };
});
