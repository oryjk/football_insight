import type { DailyTopicItem } from '../../types/dailyTopic'
import { isRongchengTopic } from '../../utils/dailyTopics'

export * from '../../utils/dailyTopics'

export function homeTopicCategory(item: Pick<DailyTopicItem, 'teamTags'>): string {
  return isRongchengTopic(item) ? '蓉城动态' : '其他动态'
}

// Past the last topic the swiper cannot move, so the card is nudged a damped amount and eased back.
const MAX_OVERSCROLL_PX = 48
export function lastSlideOverscroll(dx: number): number {
  if (!Number.isFinite(dx) || dx >= 0) return 0
  return Math.max(dx * 0.35, -MAX_OVERSCROLL_PX)
}
