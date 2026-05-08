import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { useEnumDictionary } from './enumDictionary'

export type ChapterStatus = 'outline' | 'draft' | 'revised' | 'final' | 'abandoned';
export type ChapterStatusFilter = ChapterStatus | 'all';

export const CHAPTER_STATUS_OPTIONS = [
  { label: '大纲', value: 'outline', color: '#9CA3AF' },
  { label: '草稿', value: 'draft', color: '#F59E0B' },
  { label: '修订', value: 'revised', color: '#3B82F6' },
  { label: '定稿', value: 'final', color: '#10B981' },
  { label: '废弃', value: 'abandoned', color: '#EF4444' },
] as const;

const STATUS_COLOR_MAP: Record<string, string> = {
  outline: '#9CA3AF',
  draft: '#F59E0B',
  revised: '#3B82F6',
  final: '#10B981',
  abandoned: '#EF4444',
};

export const getStatusColor = (status: ChapterStatus): string => {
  return STATUS_COLOR_MAP[status] ?? '#9CA3AF';
};

export const getStatusLabel = (status: ChapterStatus): string => {
  try {
    const { getChapterStatusName, isLoaded } = useEnumDictionary()
    if (isLoaded.value) {
      const name = getChapterStatusName(status)
      if (name !== status) return name
    }
  } catch {}
  const option = CHAPTER_STATUS_OPTIONS.find(o => o.value === status);
  return option?.label || status;
};

export const useTreeStore = defineStore("tree", () => {
  const statusFilter = ref<ChapterStatusFilter>('all');

  const statusFilterOptions = computed(() => {
    const base = [{ label: '全部章节', value: 'all' }];
    try {
      const { chapterStatuses, isLoaded } = useEnumDictionary()
      if (isLoaded.value && chapterStatuses.value.length > 0) {
        return [
          ...base,
          ...chapterStatuses.value.map(e => ({
            label: e.name,
            value: e.code,
            color: STATUS_COLOR_MAP[e.code] ?? '#9CA3AF',
          })),
        ];
      }
    } catch {}
    return [...base, ...CHAPTER_STATUS_OPTIONS];
  });

  const isFilterActive = computed(() => statusFilter.value !== 'all');

  function setStatusFilter(filter: ChapterStatusFilter) {
    statusFilter.value = filter;
  }

  function clearFilter() {
    statusFilter.value = 'all';
  }

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
