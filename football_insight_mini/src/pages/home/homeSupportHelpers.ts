import type { SupportMatchDetail } from '../../types/support'

export function resolveHomeSupportWindowShortLabel(
  match: Pick<SupportMatchDetail, 'support_window_status'> | null,
): string {
  if (!match) {
    return ''
  }

  if (match.support_window_status !== 'closed') {
    return 'VS'
  }

  return '已关闭'
}

export function resolveHomeSupportNextMatchLabel(
  match: Pick<SupportMatchDetail, 'support_window_status' | 'status'> | null,
): string {
  if (!match) {
    return ''
  }

  if (match.support_window_status !== 'closed') {
    return '随时进入助力页'
  }

  return match.status === 'finished' ? '比赛已完赛' : '比赛已开始'
}

export { resolveSupportVoteState as resolveHomeSupportVoteState } from '../../utils/supportVoteState'

/** Compare Beijing calendar dates, not rounded 24-hour durations or the phone timezone. */
const DAY_MS = 24 * 60 * 60 * 1000
const BEIJING_OFFSET_MS = 8 * 60 * 60 * 1000

/** Beijing calendar days until an offset-qualified instant; null when the instant is missing, malformed or passed. */
function resolveBeijingDaysUntil(instant: string | null | undefined, now: number): number | null {
  if (!instant || !Number.isFinite(now) || !/(?:Z|[+-]\d{2}:\d{2})$/.test(instant)) return null
  const target = Date.parse(instant)
  if (!Number.isFinite(target) || target <= now) return null
  return Math.floor((target + BEIJING_OFFSET_MS) / DAY_MS) - Math.floor((now + BEIJING_OFFSET_MS) / DAY_MS)
}

function formatCountdownClock(instant: string, now: number): string {
  const totalSeconds = Math.max(0, Math.ceil((Date.parse(instant) - now) / 1000))
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${pad(Math.floor(totalSeconds / 3600))}:${pad(Math.floor((totalSeconds % 3600) / 60))}:${pad(totalSeconds % 60)}`
}

/** HH:MM:SS until the instant, only on its Beijing calendar day (so the hour part is below 24). */
function resolveSameDayCountdownClock(instant: string | null | undefined, now: number): string | null {
  if (resolveBeijingDaysUntil(instant, now) !== 0 || !instant) return null
  return formatCountdownClock(instant, now)
}

export function resolveHomeSupportMatchDaysRemaining(
  match: Pick<SupportMatchDetail, 'kickoff_at' | 'status'> | null,
  now: number = Date.now(),
): number | null {
  if (!match || !['scheduled', '1'].includes(match.status)) return null
  return resolveBeijingDaysUntil(match.kickoff_at, now)
}

/** HH:MM:SS until kickoff, only on the Beijing match day itself (so the hour part is below 24). */
export function resolveHomeSupportMatchCountdownClock(
  match: Pick<SupportMatchDetail, 'kickoff_at' | 'status'> | null,
  now: number = Date.now(),
): string | null {
  if (resolveHomeSupportMatchDaysRemaining(match, now) !== 0 || !match) return null
  return resolveSameDayCountdownClock(match.kickoff_at, now)
}

/** 距抢票开始的北京自然日；无开票时间（如非监控比赛）或已开票时返回 null。 */
export function resolveHomeTicketSaleDaysRemaining(
  saleStartAt: string | null | undefined,
  now: number = Date.now(),
): number | null {
  return resolveBeijingDaysUntil(saleStartAt, now)
}

/**
 * 距开票不足 24 小时显示 HH:MM:SS，其余时间返回 null。
 * 与比赛倒计时（按自然日）不同：开票多在下午，按自然日会让前一晚一直显示「1 天」。
 */
export function resolveHomeTicketSaleCountdownClock(
  saleStartAt: string | null | undefined,
  now: number = Date.now(),
): string | null {
  if (resolveBeijingDaysUntil(saleStartAt, now) === null || !saleStartAt) return null
  return Date.parse(saleStartAt) - now < DAY_MS ? formatCountdownClock(saleStartAt, now) : null
}

export function resolveHomeSupportTeamRankLabel(rankNo: number | null | undefined): string {
  return rankNo && rankNo > 0 ? `积分榜第 ${rankNo}` : '排名待同步'
}

export function resolveHomeSupportMatchWeekdayLabel(matchDate: string | null | undefined): string {
  const normalizedDate = matchDate?.trim()
  if (!normalizedDate) {
    return ''
  }

  const dateMatch = /^(\d{4})-(\d{2})-(\d{2})$/.exec(normalizedDate)
  if (!dateMatch) {
    return ''
  }

  const year = Number(dateMatch[1])
  const monthIndex = Number(dateMatch[2]) - 1
  const day = Number(dateMatch[3])
  const parsedDate = new Date(year, monthIndex, day)

  if (
    parsedDate.getFullYear() !== year
    || parsedDate.getMonth() !== monthIndex
    || parsedDate.getDate() !== day
  ) {
    return ''
  }

  return ['周日', '周一', '周二', '周三', '周四', '周五', '周六'][parsedDate.getDay()]
}
