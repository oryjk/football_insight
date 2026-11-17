import type { DailyTopicEntry, DailyTopicItem, TelegraphCategory } from '../types/dailyTopic'
import { traceRoundRect } from './canvasPoster'
import {
  TELEGRAPH_CATEGORIES,
  telegraphCategoryCounts,
  telegraphCategoryLabel,
  telegraphEntryUrl,
} from './dailyTopics'
import { wrapShareCardText } from './shareCardText'
import { buildSharePath, buildShareQuery } from './shareSource'

export const TELEGRAPH_SHARE_CARD = { width: 1000, height: 800 } as const

const VALID_ID_RE = /^[a-zA-Z0-9_-]{1,128}$/

/** 黑金快讯风分享卡调色板（导出的位图色值，与首页深色电报卡一致）。 */
const PALETTE = {
  outerBg: '#0d0f14',
  cardTop: '#1a1f2b',
  cardBottom: '#13161f',
  cardFallback: '#171b26',
  cardBorder: 'rgba(255, 255, 255, 0.09)',
  subCardBg: '#1f2431',
  subCardBorder: 'rgba(255, 255, 255, 0.06)',
  divider: 'rgba(255, 255, 255, 0.08)',
  brandRed: '#dc2626',
  white: '#ffffff',
  headline: '#f8fafc',
  body: '#cbd5e1',
  subHeadline: '#e2e8f0',
  muted: '#94a3b8',
  caption: '#8b93a5',
  gold: '#e9c477',
} as const

const CATEGORY_BADGES: Record<TelegraphCategory, { bg: string; border: string; text: string }> = {
  domestic: { bg: 'rgba(220, 38, 38, 0.20)', border: 'rgba(255, 123, 125, 0.34)', text: '#ff7b7d' },
  world: { bg: 'rgba(59, 130, 246, 0.20)', border: 'rgba(138, 180, 255, 0.34)', text: '#8ab4ff' },
  abroad: { bg: 'rgba(245, 158, 11, 0.20)', border: 'rgba(255, 190, 107, 0.34)', text: '#ffbe6b' },
}

const IMPACT_WORDS: [RegExp, number][] = [
  [/官宣|正式|确认|宣布/, 3],
  [/夺冠|冠军|降级|升入|冲超|逆转|绝杀|崩盘|丢冠/, 3],
  [/离队|加盟|签约|续约|解约|下课|换帅|留队|转会/, 2],
  [/\d+\s*[-:比]\s*\d+/, 2],
  [/传闻|据报道|有意|接触|商谈|谈判/, 1],
]

function impactScore(entry: DailyTopicEntry): number {
  const content = entry.title + entry.text
  return IMPACT_WORDS.reduce((score, [pattern, weight]) => score + (pattern.test(content) ? weight : 0), 0)
}

function rankedEntries(entries: DailyTopicEntry[]): DailyTopicEntry[] {
  return entries
    .map((entry, index) => ({ entry, index, score: impactScore(entry) }))
    .sort((a, b) => b.score - a.score || a.index - b.index)
    .map(({ entry }) => entry)
}

/** 每日电报汇总看点：各非空分类先取最重磅一条，不足再按分量补齐。 */
export function roundupShareHighlights(item: Pick<DailyTopicItem, 'entries'>, count = 3): DailyTopicEntry[] {
  const picks = TELEGRAPH_CATEGORIES
    .map(({ value }) => rankedEntries(item.entries.filter(entry => entry.category === value))[0])
    .filter((entry): entry is DailyTopicEntry => Boolean(entry))
    .slice(0, count)
  for (const entry of rankedEntries(item.entries)) {
    if (picks.length >= count) break
    if (!picks.includes(entry)) picks.push(entry)
  }
  return picks
}

function beijingDate(publishedAt: string): Date | null {
  const parsed = Date.parse(publishedAt)
  return Number.isFinite(parsed) ? new Date(parsed + 8 * 60 * 60 * 1000) : null
}

function roundupDate(item: Pick<DailyTopicItem, 'publishedAt'>): string {
  const date = beijingDate(item.publishedAt)
  if (!date) return '今日'
  return `${date.getUTCMonth() + 1}月${date.getUTCDate()}日`
}

function roundupWeekday(publishedAt: string): string {
  const date = beijingDate(publishedAt)
  return date ? `周${'日一二三四五六'[date.getUTCDay()]}` : ''
}

export function roundupShareDate(item: Pick<DailyTopicItem, 'publishedAt'>): string {
  return roundupDate(item)
}

export function roundupShareTitle(item: Pick<DailyTopicItem, 'publishedAt' | 'entries'>): string {
  const [lead] = rankedEntries(item.entries)
  if (!lead) return `${roundupDate(item)}足坛快讯`
  return `${roundupDate(item)}足坛${item.entries.length}条快讯｜${lead.title}`
}

/** 卡片内部排版仅当整条标题被成对【】或[]完整包裹时剥离外层，避免把“[快讯] 标题”裁残。 */
export function cleanTelegraphHeadline(title: string): string {
  let result = title.replace(/\s+/g, ' ').trim()
  while (/^(?:【[^】]*】|\[[^\]]*\])$/.test(result)) {
    result = result.slice(1, -1).trim()
  }
  return result
}

function normalizeCompact(text: string): string {
  return text.replace(/\s+/g, ' ').trim()
}

export function cleanTelegraphExcerpt(text: string, cleanTitle: string): string {
  const normalized = normalizeCompact(text)
  if (!normalized || normalizeCompact(cleanTitle) === normalized) return ''
  return normalized
}

export function formatEntryShareTime(entry: Pick<DailyTopicEntry, 'publishedAt' | 'timeLabel'>): string {
  const date = beijingDate(entry.publishedAt)
  if (date) {
    const pad = (n: number) => String(n).padStart(2, '0')
    return `${pad(date.getUTCMonth() + 1)}月${pad(date.getUTCDate())}日 ${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}`
  }
  return entry.timeLabel.trim()
}

function truncateCodePoints(text: string, maxChars: number): string {
  const chars = Array.from(normalizeCompact(text))
  if (chars.length <= maxChars) return chars.join('')
  return `${chars.slice(0, Math.max(1, maxChars - 1)).join('')}…`
}

export function telegraphEntryShareTitle(entry: Pick<DailyTopicEntry, 'title' | 'text'>): string {
  const title = cleanTelegraphHeadline(entry.title) || '每日电报'
  const excerpt = cleanTelegraphExcerpt(entry.text, title)
  if (!excerpt) return `【${truncateCodePoints(title, 36)}】`
  const prefix = `【${title}】`
  const prefixLength = Array.from(prefix).length
  if (prefixLength >= 36) return `【${truncateCodePoints(title, 36)}】`
  return `${prefix}${truncateCodePoints(excerpt, 42 - prefixLength)}`
}

export interface TelegraphEntryShareOptions {
  topicId: string
  entryId: string
  entry: Pick<DailyTopicEntry, 'title' | 'text'> | null
  imageUrl?: string
}

export function buildTelegraphEntrySharePayload({ topicId, entryId, entry, imageUrl }: TelegraphEntryShareOptions) {
  const valid = VALID_ID_RE.test(topicId) && VALID_ID_RE.test(entryId)
  const target = valid ? telegraphEntryUrl(topicId, entryId) : '/pages/topics/index'
  const title = entry ? telegraphEntryShareTitle(entry) : '每日电报 · 足球洞察'
  return {
    title,
    path: buildSharePath(target, 'telegraph_entry'),
    ...(imageUrl ? { imageUrl } : {}),
  }
}

export function buildTelegraphEntryTimelinePayload({ topicId, entryId, entry, imageUrl }: TelegraphEntryShareOptions) {
  const valid = VALID_ID_RE.test(topicId) && VALID_ID_RE.test(entryId)
  const title = entry ? `每日电报 · ${cleanTelegraphHeadline(entry.title) || entry.title}` : '每日电报 · 足球洞察'
  const prefix = valid ? `topicId=${encodeURIComponent(topicId)}&entryId=${encodeURIComponent(entryId)}&` : ''
  return {
    title,
    query: `${prefix}${buildShareQuery('telegraph_entry')}`,
    ...(imageUrl ? { imageUrl } : {}),
  }
}

export function telegraphEntryShareSignature(
  entry: DailyTopicEntry,
  options?: { publishedAt?: string; commentsVisible?: boolean },
): string {
  const commentsVisible = options?.commentsVisible !== false
  return JSON.stringify([
    entry.id,
    entry.category,
    entry.title,
    entry.text,
    entry.publishedAt,
    entry.timeLabel,
    entry.likesCount,
    commentsVisible ? entry.commentsCount : -1,
    options?.publishedAt ?? '',
  ])
}

function setFont(context: UniApp.CanvasContext, size: number, bold = false): void {
  context.setFontSize(size)
  ;(context as unknown as { font?: string }).font = `${bold ? 'bold ' : ''}${size}px sans-serif`
}

function estimateTextWidth(text: string, size: number): number {
  return Array.from(text).reduce((sum, ch) => sum + (ch.charCodeAt(0) <= 0xff ? size * 0.58 : size), 0)
}

function measureText(context: UniApp.CanvasContext, text: string, size: number, bold = false): number {
  if (!text) return 0
  setFont(context, size, bold)
  const measured = context.measureText?.(text)?.width
  return typeof measured === 'number' && Number.isFinite(measured) && measured > 0
    ? measured
    : estimateTextWidth(text, size)
}

function drawTextLeft(
  context: UniApp.CanvasContext,
  text: string,
  x: number,
  y: number,
  size: number,
  color: string,
  bold = false,
): void {
  if (!text) return
  setFont(context, size, bold)
  context.setTextAlign('left')
  context.setFillStyle(color)
  context.fillText(text, x, y)
}

function drawRoundCard(
  context: UniApp.CanvasContext,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
  fill: string | CanvasGradient,
  stroke?: { color: string; lineWidth: number },
): void {
  traceRoundRect(context, x, y, width, height, radius)
  context.setFillStyle(fill)
  context.fill()
  if (stroke) {
    context.setStrokeStyle(stroke.color)
    context.setLineWidth(stroke.lineWidth)
    context.stroke()
  }
}

function drawChevron(
  context: UniApp.CanvasContext,
  centerX: number,
  centerY: number,
  stroke: string,
  size = 9,
): void {
  context.setStrokeStyle(stroke)
  context.setLineWidth(3.5)
  context.setLineCap('round')
  context.setLineJoin('round')
  context.beginPath()
  context.moveTo(centerX - size * 0.55, centerY - size)
  context.lineTo(centerX + size * 0.65, centerY)
  context.lineTo(centerX - size * 0.55, centerY + size)
  context.stroke()
}

function drawCardShell(context: UniApp.CanvasContext, accentBarX: number): void {
  const { width, height } = TELEGRAPH_SHARE_CARD
  context.setFillStyle(PALETTE.outerBg)
  context.fillRect(0, 0, width, height)

  let fill: string | CanvasGradient = PALETTE.cardFallback
  if (typeof context.createLinearGradient === 'function') {
    const gradient = context.createLinearGradient(0, 16, 0, height - 16)
    gradient.addColorStop(0, PALETTE.cardTop)
    gradient.addColorStop(1, PALETTE.cardBottom)
    fill = gradient
  }
  drawRoundCard(context, 16, 16, width - 32, height - 32, 32, fill, {
    color: PALETTE.cardBorder,
    lineWidth: 2,
  })

  // 顶部内侧品牌红微光装饰短线。
  drawRoundCard(context, accentBarX, 16, 140, 6, 3, PALETTE.brandRed)
}

function drawPillBadge(
  context: UniApp.CanvasContext,
  x: number,
  y: number,
  height: number,
  paddingX: number,
  text: string,
  fontSize: number,
  textColor: string,
  bgColor: string,
  borderColor?: string,
): number {
  const textWidth = measureText(context, text, fontSize, true)
  const pillWidth = Math.ceil(textWidth + paddingX * 2)
  drawRoundCard(
    context,
    x,
    y,
    pillWidth,
    height,
    Math.floor(height / 2),
    bgColor,
    borderColor ? { color: borderColor, lineWidth: 1.5 } : undefined,
  )
  const baseline = y + Math.round((height + fontSize * 0.72) / 2)
  drawTextLeft(context, text, x + paddingX, baseline, fontSize, textColor, true)
  return pillWidth
}

function drawBrandMark(context: UniApp.CanvasContext, rightX: number, baselineY: number, centerY: number): number {
  const label = '足球洞察'
  const labelWidth = measureText(context, label, 30, true)
  const textX = rightX - labelWidth
  drawTextLeft(context, label, textX, baselineY, 30, PALETTE.caption, true)
  const dotRadius = 6
  const dotCx = textX - 14 - dotRadius
  context.beginPath()
  context.arc(dotCx, centerY, dotRadius, 0, Math.PI * 2)
  context.closePath()
  context.setFillStyle(PALETTE.brandRed)
  context.fill()
  return dotCx - dotRadius
}

function drawCtaPill(
  context: UniApp.CanvasContext,
  rightX: number,
  centerY: number,
  label: string,
): number {
  const fontSize = 34
  const height = 72
  const paddingLeft = 30
  const chevronArea = 44
  const labelWidth = measureText(context, label, fontSize, true)
  const pillWidth = Math.ceil(paddingLeft + labelWidth + chevronArea)
  const pillX = rightX - pillWidth
  const pillY = centerY - Math.floor(height / 2)

  drawRoundCard(context, pillX, pillY, pillWidth, height, Math.floor(height / 2), PALETTE.brandRed)
  const baseline = pillY + Math.round((height + fontSize * 0.72) / 2)
  drawTextLeft(context, label, pillX + paddingLeft, baseline, fontSize, PALETTE.white, true)
  drawChevron(context, rightX - 28, centerY, PALETTE.white, 9)
  return pillX
}

interface TextSegment {
  text: string
  color: string
  bold?: boolean
  size?: number
}

function drawFooterSegments(
  context: UniApp.CanvasContext,
  startX: number,
  maxRightX: number,
  baselineY: number,
  fontSize: number,
  segments: TextSegment[],
): void {
  let cursor = startX
  for (const segment of segments) {
    const segSize = segment.size ?? fontSize
    const segBold = Boolean(segment.bold)
    const width = measureText(context, segment.text, segSize, segBold)
    if (cursor + width > maxRightX) {
      const remaining = maxRightX - cursor
      if (remaining > segSize * 1.5) {
        const [clipped] = wrapShareCardText(
          segment.text,
          sample => measureText(context, sample, segSize, segBold),
          remaining,
          1,
        )
        if (clipped) drawTextLeft(context, clipped, cursor, baselineY, segSize, segment.color, segBold)
      }
      break
    }
    drawTextLeft(context, segment.text, cursor, baselineY, segSize, segment.color, segBold)
    cursor += width
  }
}

function buildEntryFooterSegments(entry: DailyTopicEntry, commentsVisible: boolean): TextSegment[] {
  const likes = entry.likesCount > 0 ? entry.likesCount : 0
  const comments = entry.commentsCount > 0 ? entry.commentsCount : 0
  if (commentsVisible) {
    if (likes > 0 && comments > 0) {
      return [
        { text: '赞同 ', color: PALETTE.muted, bold: true },
        { text: String(likes), color: PALETTE.gold, bold: true, size: 38 },
        { text: ' · 评论 ', color: PALETTE.muted, bold: true },
        { text: String(comments), color: PALETTE.gold, bold: true, size: 38 },
      ]
    }
    if (comments > 0) {
      return [
        { text: String(comments), color: PALETTE.gold, bold: true, size: 38 },
        { text: ' 条球迷看法', color: PALETTE.muted, bold: true },
      ]
    }
    if (likes > 0) {
      return [
        { text: String(likes), color: PALETTE.gold, bold: true, size: 38 },
        { text: ' 位球迷赞同', color: PALETTE.muted, bold: true },
      ]
    }
    return [{ text: '进来聊聊你的看法', color: PALETTE.muted, bold: true }]
  }
  if (likes > 0) {
    return [
      { text: String(likes), color: PALETTE.gold, bold: true, size: 38 },
      { text: ' 人赞同 · 即时快讯', color: PALETTE.muted, bold: true },
    ]
  }
  return [{ text: '足坛即时快讯速递', color: PALETTE.muted, bold: true }]
}

/**
 * 单条电报详情 5:4 微信分享卡（方案 A：黑金快讯风，不绘制任何来源信息）。
 */
export function drawTelegraphEntryShareCard(
  context: UniApp.CanvasContext,
  entry: DailyTopicEntry,
  options?: { publishedAt?: string; commentsVisible?: boolean },
): void {
  const left = 52
  const right = TELEGRAPH_SHARE_CARD.width - left
  const contentWidth = right - left
  const commentsVisible = options?.commentsVisible !== false

  drawCardShell(context, left)

  // 眉栏：每日电报 + 分类胶囊 + 发布时间；右侧品牌。
  const headerY = 48
  const pillH = 60
  const headerBaseline = headerY + Math.round((pillH + 34 * 0.72) / 2)
  const headerCenterY = headerY + pillH / 2

  let cursorX = left
  cursorX += drawPillBadge(context, cursorX, headerY, pillH, 26, '每日电报', 34, PALETTE.white, PALETTE.brandRed)
  cursorX += 12

  const badge = CATEGORY_BADGES[entry.category]
  cursorX += drawPillBadge(
    context,
    cursorX,
    headerY,
    pillH,
    24,
    telegraphCategoryLabel(entry.category),
    34,
    badge.text,
    badge.bg,
    badge.border,
  )
  cursorX += 16

  const brandLeftX = drawBrandMark(context, right, headerBaseline, headerCenterY)
  const timeText = formatEntryShareTime(entry)
  const timeMaxWidth = brandLeftX - 16 - cursorX
  if (timeText && timeMaxWidth > 60) {
    const [clippedTime] = wrapShareCardText(
      timeText,
      sample => measureText(context, sample, 32, true),
      timeMaxWidth,
      1,
    )
    if (clippedTime) drawTextLeft(context, clippedTime, cursorX, headerBaseline, 32, PALETTE.muted, true)
  }

  // 主标题区：优先 66px 粗体排 <=2 行；超过 2 行时切到 58px 粗体最多 3 行（无摘要可到 4 行）。
  const cleanTitle = cleanTelegraphHeadline(entry.title) || entry.title.trim()
  const excerpt = cleanTelegraphExcerpt(entry.text, cleanTitle)
  const probe66 = wrapShareCardText(
    cleanTitle,
    sample => measureText(context, sample, 66, true),
    contentWidth,
    3,
  )
  const useLargeTitle = probe66.length <= 2
  const titleSize = useLargeTitle ? 66 : 58
  const titleLineHeight = useLargeTitle ? 84 : 74
  const titleStartY = useLargeTitle ? 198 : 190
  const titleMaxLines = useLargeTitle ? 2 : excerpt ? 3 : 4
  const titleLines = useLargeTitle
    ? probe66
    : wrapShareCardText(
        cleanTitle,
        sample => measureText(context, sample, titleSize, true),
        contentWidth,
        titleMaxLines,
      )
  titleLines.forEach((line, index) => {
    drawTextLeft(context, line, left, titleStartY + index * titleLineHeight, titleSize, PALETTE.headline, true)
  })

  // 正文摘要区：竖向分类色引述条 + 44px 大字号正文。
  if (excerpt) {
    const titleLastBaseline = titleStartY + Math.max(0, titleLines.length - 1) * titleLineHeight
    const excerptBaseY = titleLastBaseline + 78
    const maxLinesByTitle = titleLines.length <= 1 ? 5 : titleLines.length === 2 ? 4 : 3
    const maxLinesByBottom = Math.max(1, Math.floor((596 - excerptBaseY) / 62) + 1)
    const maxExcerptLines = Math.min(maxLinesByTitle, maxLinesByBottom)
    const excerptX = left + 28
    const excerptLines = wrapShareCardText(
      excerpt,
      sample => measureText(context, sample, 44, false),
      right - excerptX,
      maxExcerptLines,
    )
    if (excerptLines.length) {
      const barTop = excerptBaseY - 38
      const barHeight = Math.max(46, (excerptLines.length - 1) * 62 + 44)
      drawRoundCard(context, left, barTop, 8, barHeight, 4, badge.text)
      excerptLines.forEach((line, index) => {
        drawTextLeft(context, line, excerptX, excerptBaseY + index * 62, 44, PALETTE.body)
      })
    }
  }

  // 底栏：分割线 + 左侧互动热度 + 右侧行动胶囊（不画来源）。
  context.setFillStyle(PALETTE.divider)
  context.fillRect(left, 636, contentWidth, 1)

  const footerCenterY = 710
  const footerBaseline = footerCenterY + 12
  const ctaLeftX = drawCtaPill(context, right, footerCenterY, commentsVisible ? '看热议' : '看详情')
  const segments = buildEntryFooterSegments(entry, commentsVisible)
  drawFooterSegments(context, left, ctaLeftX - 24, footerBaseline, 34, segments)
}

/**
 * 整期《每日电报》汇总 5:4 微信分享卡（方案 A：黑金快讯风，不绘制任何来源信息）。
 */
export function drawRoundupShareCard(context: UniApp.CanvasContext, item: DailyTopicItem): void {
  const left = 52
  const right = TELEGRAPH_SHARE_CARD.width - left
  const contentWidth = right - left

  drawCardShell(context, left)

  // 第 1 行报头：每日电报红胶囊 + 分类计数胶囊 + 右侧品牌。
  const row1Y = 44
  const row1H = 56
  const row1Baseline = row1Y + Math.round((row1H + 32 * 0.72) / 2)
  const row1CenterY = row1Y + row1H / 2

  let cursorX = left
  cursorX += drawPillBadge(context, cursorX, row1Y, row1H, 22, '每日电报', 32, PALETTE.white, PALETTE.brandRed)
  cursorX += 12

  const brandLeftX = drawBrandMark(context, right, row1Baseline, row1CenterY)
  const counts = telegraphCategoryCounts(item)
  const activeCategories: string[] = []
  for (const { value, label } of TELEGRAPH_CATEGORIES) {
    if (counts[value] <= 0) continue
    activeCategories.push(label)
    const badge = CATEGORY_BADGES[value]
    const pillText = `${label} ${counts[value]}`
    const estWidth = measureText(context, pillText, 30, true) + 32
    if (cursorX + estWidth > brandLeftX - 14) continue
    cursorX += drawPillBadge(context, cursorX, row1Y + 1, 54, 16, pillText, 30, badge.text, badge.bg, badge.border)
    cursorX += 10
  }

  // 第 2 行报头：大号日期 + 星期；右侧金色总条数。
  const dateText = roundupShareDate(item)
  const dateWidth = measureText(context, dateText, 66, true)
  drawTextLeft(context, dateText, left, 166, 66, PALETTE.headline, true)
  const weekday = roundupWeekday(item.publishedAt)
  if (weekday) {
    drawTextLeft(context, weekday, left + dateWidth + 16, 166, 34, PALETTE.muted, true)
  }

  const suffixText = ' 条快讯'
  const countText = String(item.entries.length)
  const suffixWidth = measureText(context, suffixText, 34, true)
  const countWidth = measureText(context, countText, 52, true)
  drawTextLeft(context, suffixText, right - suffixWidth, 166, 34, PALETTE.subHeadline, true)
  drawTextLeft(context, countText, right - suffixWidth - countWidth, 166, 52, PALETTE.gold, true)

  // 中部精选快讯子卡片：最多 3 条，42px 双行大字标题。
  const highlights = roundupShareHighlights(item, 3)
  highlights.forEach((entry, index) => {
    const cardY = 192 + index * 151
    const cardH = 138
    drawRoundCard(context, left, cardY, contentWidth, cardH, 24, PALETTE.subCardBg, {
      color: PALETTE.subCardBorder,
      lineWidth: 1.5,
    })

    const badge = CATEGORY_BADGES[entry.category]
    const catLabel = telegraphCategoryLabel(entry.category)
    const colX = left + 20
    const colW = 104
    drawRoundCard(context, colX, cardY + 24, colW, 44, 12, badge.bg, { color: badge.border, lineWidth: 1 })
    const catWidth = measureText(context, catLabel, 28, true)
    drawTextLeft(context, catLabel, colX + (colW - catWidth) / 2, cardY + 56, 28, badge.text, true)

    const shortTime = entry.timeLabel.trim().slice(-5)
    if (shortTime) {
      const timeWidth = measureText(context, shortTime, 26, true)
      drawTextLeft(context, shortTime, colX + (colW - timeWidth) / 2, cardY + 106, 26, PALETTE.muted, true)
    }

    const titleX = left + 144
    const titleMaxWidth = right - 22 - titleX
    const cleanTitle = cleanTelegraphHeadline(entry.title) || entry.title.trim()
    const lines = wrapShareCardText(
      cleanTitle,
      sample => measureText(context, sample, 42, true),
      titleMaxWidth,
      2,
    )
    if (lines.length === 1) {
      drawTextLeft(context, lines[0]!, titleX, cardY + 84, 42, PALETTE.headline, true)
    } else if (lines.length >= 2) {
      drawTextLeft(context, lines[0]!, titleX, cardY + 58, 42, PALETTE.headline, true)
      drawTextLeft(context, lines[1]!, titleX, cardY + 110, 42, PALETTE.headline, true)
    }
  })

  // 底栏：分割线 + 覆盖分类导语 + 右侧行动胶囊（不画来源）。
  context.setFillStyle(PALETTE.divider)
  context.fillRect(left, 650, contentWidth, 1)

  const footerCenterY = 716
  const footerBaseline = footerCenterY + 12
  const ctaLeftX = drawCtaPill(context, right, footerCenterY, `看全部 ${item.entries.length} 条`)
  const footerSummary = activeCategories.length
    ? `今日重点 · ${activeCategories.join('/')}`
    : '今日足坛重点速览'
  drawFooterSegments(context, left, ctaLeftX - 24, footerBaseline, 34, [
    { text: footerSummary, color: PALETTE.muted, bold: true },
  ])
}
