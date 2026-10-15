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
export function resolveHomeSupportMatchDaysRemaining(
  match: Pick<SupportMatchDetail, 'kickoff_at' | 'status'> | null,
  now: number = Date.now(),
): number | null {
  if (!match || !['scheduled', '1'].includes(match.status) || !Number.isFinite(now)
    || !/(?:Z|[+-]\d{2}:\d{2})$/.test(match.kickoff_at)) return null
  const kickoff = Date.parse(match.kickoff_at)
  if (!Number.isFinite(kickoff) || kickoff <= now) return null
  const day = 24 * 60 * 60 * 1000
  const beijingOffset = 8 * 60 * 60 * 1000
  return Math.floor((kickoff + beijingOffset) / day) - Math.floor((now + beijingOffset) / day)
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
