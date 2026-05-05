export type ChapterStatus = 'outline' | 'draft' | 'revised' | 'final' | 'abandoned';

export const CHAPTER_STATUS_OPTIONS = [
  { label: '大纲', value: 'outline', color: '#9CA3AF' },
  { label: '草稿', value: 'draft', color: '#F59E0B' },
  { label: '修订', value: 'revised', color: '#3B82F6' },
  { label: '定稿', value: 'final', color: '#10B981' },
  { label: '废弃', value: 'abandoned', color: '#EF4444' },
] as const;

export function getStatusColor(status: ChapterStatus): string {
  const option = CHAPTER_STATUS_OPTIONS.find(o => o.value === status);
  return option?.color ?? '#9CA3AF';
}

export function getStatusLabel(status: ChapterStatus): string {
  const option = CHAPTER_STATUS_OPTIONS.find(o => o.value === status);
  return option?.label ?? status;
}
