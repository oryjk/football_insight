export interface DailyTopicSource {
  title: string
  url: string
  publisher: string
  publishedAt: string
  dateLabel: string
}

export interface DailyTopicAudio {
  status: 'pending' | 'ready' | 'failed'
  url: string | null
  durationSeconds: number | null
}

export interface DailyTopicItem {
  /** 生成端的不可变内容 ID；禁止使用榜单分类或球队 ID 代替。 */
  id: string
  teamId: number
  teamTags?: DailyTopicTag[]
  title: string
  summary: string
  paragraphs: string[]
  publishedAt: string
  publishedLabel: string
  sources: DailyTopicSource[]
  images?: DailyTopicImage[]
  sourceLabel: string
  heat: number
  readMinutes: number
  audio: DailyTopicAudio
  /** 每日电报的快讯条目（新到旧）；旧版多篇话题为空数组。 */
  entries: DailyTopicEntry[]
  /** 每日电报按分类各一段的语音；旧数据为空对象。 */
  categoryAudio: Partial<Record<TelegraphCategory, DailyTopicAudio>>
}

/** 国内 / 国际（五大联赛热门）/ 留洋。 */
export type TelegraphCategory = 'domestic' | 'world' | 'abroad'

export interface DailyTopicEntry {
  id: string
  /** 改版前发布的条目没有分类，按国内处理。 */
  category: TelegraphCategory
  title: string
  text: string
  publishedAt: string
  /** 当天的条目只显示「HH:mm」，前一天深夜的条目带日期「MM-DD HH:mm」。 */
  timeLabel: string
  sources: DailyTopicSource[]
}

export interface DailyTopicTag { teamId: number; name: string }

export interface DailyTopicImage {
  url: string
  width: number
  height: number
  sourceUrl: string
  publisher: string
  afterParagraph: number
}

export interface DailyTopicEdition {
  teamId: number
  editionDate: string | null
  updatedAt: string | null
  dateLabel: string
  updatedLabel: string
  items: DailyTopicItem[]
}
