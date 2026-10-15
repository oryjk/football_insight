import type { DailyTopicEdition, DailyTopicImage, DailyTopicItem, DailyTopicSource, DailyTopicTag } from '../types/dailyTopic'

export const DAILY_TOPICS_TEAM_ID = 77680
export const DAILY_TOPICS_LIMIT = 5

export function isRongchengTopic(item: Pick<DailyTopicItem, 'teamTags'>): boolean {
  // The edition owner is shared by all clubs; only article tags identify Rongcheng.
  return item.teamTags?.some(tag => tag.teamId === DAILY_TOPICS_TEAM_ID) ?? false
}

function record(value: unknown): Record<string, unknown> | null {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown> : null
}
function text(value: unknown): string { return typeof value === 'string' ? value.trim() : '' }
function timestamp(value: unknown): string | null {
  const candidate = text(value)
  return /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$/.test(candidate)
    && Number.isFinite(Date.parse(candidate)) ? candidate : null
}
function editionDate(value: unknown): string | null {
  const candidate = text(value)
  if (!/^\d{4}-\d{2}-\d{2}$/.test(candidate)) return null
  const date = new Date(`${candidate}T00:00:00Z`)
  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === candidate ? candidate : null
}
export function isTopicSourceUrl(value: unknown): value is string {
  return typeof value === 'string'
    && /^https?:\/\/[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?(?::\d{1,5})?(?:[/?#][^\s]*)?$/i.test(value)
}
function isAudioUrl(value: unknown): value is string {
  return isTopicSourceUrl(value) && /^https:\/\//i.test(value)
}
/** 固定北京时间，不受手机所在时区影响。 */
export function formatTopicTimestamp(value: string): string {
  const date = new Date(Date.parse(value) + 8 * 60 * 60 * 1000)
  const pad = (number: number) => String(number).padStart(2, '0')
  return `${date.getUTCFullYear()}/${pad(date.getUTCMonth() + 1)}/${pad(date.getUTCDate())} ${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}`
}
export function emptyDailyTopicEdition(): DailyTopicEdition {
  return { teamId: DAILY_TOPICS_TEAM_ID, editionDate: null, updatedAt: null, dateLabel: '', updatedLabel: '', items: [] }
}
function mapSource(value: unknown): DailyTopicSource | null {
  const source = record(value)
  if (!source) return null
  const title = text(source.title), publisher = text(source.publisher), publishedAt = timestamp(source.published_at)
  if (!title || !publisher || !publishedAt || !isTopicSourceUrl(source.url)) return null
  return { title, publisher, publishedAt, url: source.url, dateLabel: formatTopicTimestamp(publishedAt) }
}
function mapImages(value: unknown, sources: DailyTopicSource[], paragraphs: number): DailyTopicImage[] {
  if (!Array.isArray(value)) return []
  const images: DailyTopicImage[] = [], seen = new Set<string>()
  for (const candidate of value) {
    const image = record(candidate)
    if (!image || !isAudioUrl(image.url) || seen.has(image.url)
      || !sources.some(source => source.url === image.source_url && source.publisher === image.publisher)
      || ![image.width, image.height, image.after_paragraph].every(n => typeof n === 'number' && Number.isSafeInteger(n) && n > 0)
      || (image.width as number) * (image.height as number) > 40000000
      || (image.after_paragraph as number) > paragraphs) continue
    seen.add(image.url)
    images.push({ url: image.url, width: image.width as number, height: image.height as number,
      sourceUrl: image.source_url as string, publisher: image.publisher as string, afterParagraph: image.after_paragraph as number })
    if (images.length === 4) break
  }
  return images
}

function mapTeamTags(value: unknown): DailyTopicTag[] {
  if (!Array.isArray(value)) return []
  const tags: DailyTopicTag[] = [], seen = new Set<number>()
  for (const candidate of value) {
    const tag = record(candidate)
    if (!tag || typeof tag.team_id !== 'number' || !Number.isSafeInteger(tag.team_id) || tag.team_id <= 0
      || !text(tag.name) || text(tag.name).length > 80 || /[<>\x00-\x1f]/.test(text(tag.name)) || seen.has(tag.team_id)) continue
    seen.add(tag.team_id)
    tags.push({ teamId: tag.team_id, name: text(tag.name) })
  }
  return tags
}
export function mapDailyTopicItem(value: unknown): DailyTopicItem | null {
  const item = record(value)
  if (!item || item.team_id !== DAILY_TOPICS_TEAM_ID) return null
  const id = text(item.id), title = text(item.title), summary = text(item.summary), publishedAt = timestamp(item.published_at)
  const paragraphs = Array.isArray(item.paragraphs) ? item.paragraphs.map(text).filter(Boolean) : []
  const sources = Array.isArray(item.sources) ? item.sources.map(mapSource).filter((source): source is DailyTopicSource => source !== null) : []
  if (!id || !title || !summary || !publishedAt || !paragraphs.length || !sources.length) return null
  const audio = record(item.audio)
  const ready = audio?.status === 'ready' && audio.dialect === 'sichuan' && isAudioUrl(audio.url)
  return {
    id, teamId: DAILY_TOPICS_TEAM_ID, title, summary, paragraphs, publishedAt,
    publishedLabel: formatTopicTimestamp(publishedAt), sources,
    images: mapImages(item.images, sources, paragraphs.length),
    teamTags: mapTeamTags(item.team_tags),
    sourceLabel: `${Array.from(new Set(sources.map(source => source.publisher))).join(' · ')} · ${sources.length} 个来源`,
    heat: typeof item.heat === 'number' && Number.isSafeInteger(item.heat) && item.heat >= 0 ? item.heat : 0,
    readMinutes: Math.max(1, Math.ceil(paragraphs.join('').length / 350)),
    audio: {
      status: ready ? 'ready' : audio?.status === 'pending' ? 'pending' : 'failed',
      url: ready ? audio!.url as string : null,
      durationSeconds: ready && typeof audio?.duration_seconds === 'number' && Number.isFinite(audio.duration_seconds) && audio.duration_seconds > 0
        ? audio.duration_seconds : null,
    },
  }
}
export function mapDailyTopicEdition(value: unknown): DailyTopicEdition {
  const payload = record(value)
  if (!payload || payload.team_id !== DAILY_TOPICS_TEAM_ID || !Array.isArray(payload.items)) throw new Error('话题数据格式异常，请重试')
  const date = editionDate(payload.edition_date), updatedAt = timestamp(payload.updated_at)
  if (payload.items.length && (!date || !updatedAt)) throw new Error('话题发布日期缺失，请重试')
  const items: DailyTopicItem[] = [], seen = new Set<string>()
  for (const candidate of payload.items) {
    const item = mapDailyTopicItem(candidate)
    if (!item || seen.has(item.id)) continue
    seen.add(item.id)
    items.push(item)
    if (items.length === DAILY_TOPICS_LIMIT) break
  }
  if (payload.items.length && !items.length) throw new Error('话题内容暂时不可用，请重试')
  return { teamId: DAILY_TOPICS_TEAM_ID, editionDate: date, updatedAt, dateLabel: date?.replace(/-/g, '/') ?? '', updatedLabel: updatedAt ? `更新于 ${formatTopicTimestamp(updatedAt)} · 北京时间` : '', items }
}
/** 版本、文章正文或音频状态变更都必须销毁原播放，避免读到旧稿。 */
export function topicPlaybackSignature(edition: DailyTopicEdition, id: string): string {
  const item = edition.items.find(item => item.id === id)
  if (!item || item.audio.status !== 'ready' || !isAudioUrl(item.audio.url)) return ''
  return JSON.stringify([edition.updatedAt, item.id, item.publishedAt, item.title, item.summary, item.paragraphs, item.audio.url])
}
export function formatTopicAudioTime(seconds: number): string {
  const time = Number.isFinite(seconds) ? Math.max(0, Math.floor(seconds)) : 0
  return `${Math.floor(time / 60)}:${String(time % 60).padStart(2, '0')}`
}

export interface TopicHistoryPage { items: DailyTopicItem[]; nextCursor: string | null }
export function mapTopicHistory(value: unknown): TopicHistoryPage {
  const payload = record(value)
  if (!payload || payload.team_id !== DAILY_TOPICS_TEAM_ID || !Array.isArray(payload.items)
    || typeof payload.has_more !== 'boolean'
    || (payload.has_more ? typeof payload.next_cursor !== 'string' || !payload.next_cursor : payload.next_cursor !== null)) {
    throw new Error('话题加载失败，请重试')
  }
  const items = payload.items.map(mapDailyTopicItem)
  if (items.some(item => !item)) throw new Error('话题内容暂时不可用，请重试')
  if (payload.has_more && !items.length) throw new Error('话题加载失败，请重试')
  return { items: mergeTopicHistory([], items as DailyTopicItem[]), nextCursor: payload.next_cursor as string | null }
}

export function mapTopicTimeline(value: unknown, anchor: DailyTopicItem, tagTeamId: number): TopicHistoryPage {
  const payload = record(value)
  const tag = record(payload?.tag)
  if (!payload || payload.anchor_id !== anchor.id || tag?.team_id !== tagTeamId
    || !anchor.teamTags?.some(team => team.teamId === tagTeamId)) throw new Error('话题追踪数据异常，请重试')
  const page = mapTopicHistory(value)
  if (page.items.some(item => item.id === anchor.id || Date.parse(item.publishedAt) >= Date.parse(anchor.publishedAt)
    || !item.teamTags?.some(team => team.teamId === tagTeamId))) throw new Error('话题追踪数据异常，请重试')
  return page
}
export function mergeTopicHistory(previous: DailyTopicItem[], next: DailyTopicItem[]): DailyTopicItem[] {
  const items = new Map(previous.map(item => [item.id, item]))
  for (const item of next) items.set(item.id, item)
  return [...items.values()].sort((a, b) => Date.parse(b.publishedAt) - Date.parse(a.publishedAt) || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))
}
export function mapTopicDetail(value: unknown): DailyTopicItem {
  const item = mapDailyTopicItem(value)
  if (!item) throw new Error('话题内容暂时不可用，请重试')
  return item
}
export function topicDetailUrl(id: string): string {
  return `/pages/topic-detail/index?id=${encodeURIComponent(id)}`
}
export function editionForTopic(item: DailyTopicItem | null): DailyTopicEdition {
  return item ? { ...emptyDailyTopicEdition(), updatedAt: item.publishedAt, items: [item] } : emptyDailyTopicEdition()
}

export function formatTopicHeat(value: number): string {
  const heat = Number.isSafeInteger(value) && value >= 0 ? value : 0
  if (heat >= 100000000) return `${Math.floor(heat / 10000000) / 10}亿`
  if (heat >= 10000) return `${Math.floor(heat / 1000) / 10}万`
  return String(heat)
}
