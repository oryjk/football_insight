import { reactive } from 'vue'
import type { DailyTopicItem } from '../types/dailyTopic'

// Successful reads/visits keep existing home and history pages current when returning from detail.
const values = reactive(new Map<string, number>())
export function topicHeatValue(id: string, fallback: number): number { return values.get(id) ?? fallback }
export function setTopicHeat(id: string, heat: number) {
  if (Number.isSafeInteger(heat) && heat >= 0) values.set(id, heat)
}
export function syncTopicHeat(items: DailyTopicItem[]) { for (const item of items) setTopicHeat(item.id, item.heat) }
