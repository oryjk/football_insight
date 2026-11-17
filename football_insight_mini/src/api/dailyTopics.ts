import { request } from '../utils/request'

/** 独立于 overview / public-config，文章与四川话音频由 Python 预生成。 */
export function getDailyTopics(teamId: number, limit = 5): Promise<unknown> {
  return request<unknown>({ url: `/daily-topics?team_id=${teamId}&limit=${Math.min(5, Math.max(1, limit))}`, auth: true })
}

export function getTopicHistory(teamId: number, cursor: string | null = null, limit = 20): Promise<unknown> {
  const after = cursor ? `&cursor=${encodeURIComponent(cursor)}` : ''
  return request<unknown>({ url: `/daily-topics/history?team_id=${teamId}&limit=${limit}${after}`, auth: true })
}
export function getTopicDetail(teamId: number, id: string): Promise<unknown> {
  return request<unknown>({ url: `/daily-topics/${encodeURIComponent(id)}?team_id=${teamId}`, auth: true })
}

export function getTopicTimeline(teamId: number, anchorId: string, tagTeamId: number, cursor: string | null = null, limit = 10): Promise<unknown> {
  const after = cursor ? `&cursor=${encodeURIComponent(cursor)}` : ''
  return request<unknown>({ url: `/daily-topics/${encodeURIComponent(anchorId)}/timeline?team_id=${teamId}&tag_team_id=${tagTeamId}&limit=${limit}${after}`, auth: true, silentNetworkError: true })
}

export function recordTopicVisit(teamId: number, id: string, visitorId: string): Promise<{ heat: number }> {
  return request<{ heat: number }>({ url: `/daily-topics/${encodeURIComponent(id)}/visits?team_id=${teamId}`,
    method: 'POST', data: { visitor_id: visitorId }, auth: true, silentNetworkError: true })
}

export function setTelegraphEntryLike(teamId: number, topicId: string, entryId: string, active: boolean): Promise<unknown> {
  return request<unknown>({
    url: `/daily-topics/${encodeURIComponent(topicId)}/entries/${encodeURIComponent(entryId)}/like?team_id=${teamId}`,
    method: active ? 'PUT' : 'DELETE',
    auth: true,
  })
}

export function setTelegraphEntryDislike(teamId: number, topicId: string, entryId: string, active: boolean): Promise<unknown> {
  return request<unknown>({
    url: `/daily-topics/${encodeURIComponent(topicId)}/entries/${encodeURIComponent(entryId)}/dislike?team_id=${teamId}`,
    method: active ? 'PUT' : 'DELETE',
    auth: true,
  })
}

export function getTelegraphEntryComments(
  teamId: number,
  topicId: string,
  entryId: string,
  cursor: string | null = null,
  limit = 20,
): Promise<unknown> {
  const after = cursor ? `&cursor=${encodeURIComponent(cursor)}` : ''
  return request<unknown>({
    url: `/daily-topics/${encodeURIComponent(topicId)}/entries/${encodeURIComponent(entryId)}/comments?team_id=${teamId}&limit=${limit}${after}`,
    auth: true,
  })
}

export function createTelegraphEntryComment(
  teamId: number,
  topicId: string,
  entryId: string,
  payload: { content: string; reply_to_comment_id?: string | null },
): Promise<{ comment: unknown; comments_count: number }> {
  return request<{ comment: unknown; comments_count: number }>({
    url: `/daily-topics/${encodeURIComponent(topicId)}/entries/${encodeURIComponent(entryId)}/comments?team_id=${teamId}`,
    method: 'POST',
    auth: true,
    data: payload,
  })
}

export function deleteTelegraphEntryComment(
  teamId: number,
  topicId: string,
  entryId: string,
  commentId: string,
): Promise<{ comments_count: number }> {
  return request<{ comments_count: number }>({
    url: `/daily-topics/${encodeURIComponent(topicId)}/entries/${encodeURIComponent(entryId)}/comments/${encodeURIComponent(commentId)}?team_id=${teamId}`,
    method: 'DELETE',
    auth: true,
  })
}
