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
  title: string
  summary: string
  paragraphs: string[]
  publishedAt: string
  publishedLabel: string
  sources: DailyTopicSource[]
  sourceLabel: string
  heat: number
  readMinutes: number
  audio: DailyTopicAudio
}

export interface DailyTopicEdition {
  teamId: number
  editionDate: string | null
  updatedAt: string | null
  dateLabel: string
  updatedLabel: string
  items: DailyTopicItem[]
}
