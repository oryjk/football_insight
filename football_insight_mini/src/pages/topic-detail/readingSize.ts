import type { FiSegmentedItem } from '../../types/segmented'

const STORAGE_KEY = 'fi_topic_reading_size'

export type ReadingSize = 'standard' | 'large' | 'xlarge'

export const READING_SIZES: { value: ReadingSize; label: string; rpx: number }[] = [
  { value: 'standard', label: '标准', rpx: 32 },
  { value: 'large', label: '大', rpx: 36 },
  { value: 'xlarge', label: '特大', rpx: 40 },
]

export function normalizeReadingSize(value: unknown): ReadingSize {
  return READING_SIZES.find(size => size.value === value)?.value ?? 'standard'
}

export function readingSizeRpx(size: ReadingSize): number {
  return (READING_SIZES.find(item => item.value === size) ?? READING_SIZES[0]).rpx
}

export const READING_SIZE_ITEMS: FiSegmentedItem[] = READING_SIZES.map(size => ({ key: size.value, label: size.label }))

export function loadReadingSize(): ReadingSize {
  try { return normalizeReadingSize(uni.getStorageSync(STORAGE_KEY)) } catch { return 'standard' }
}

export function saveReadingSize(size: ReadingSize) {
  try { uni.setStorageSync(STORAGE_KEY, size) } catch { /* storage unavailable: choice applies to this visit only */ }
}
