import { request } from '../utils/request'

/** 独立于 overview / public-config，文章与四川话音频由 Python 预生成。 */
export function getDailyTopics(teamId: number, limit = 5): Promise<unknown> {
  return request<unknown>({ url: `/daily-topics?team_id=${teamId}&limit=${Math.min(5, Math.max(1, limit))}` })
}
