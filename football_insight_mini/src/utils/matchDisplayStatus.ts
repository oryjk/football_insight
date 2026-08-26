import type { MatchCard } from '../types/insight'

export type MatchDisplayStatus = MatchCard['status']

const LIVE_INFERENCE_WINDOW_MS = 3 * 60 * 60 * 1000

function parseMatchKickoff(matchDate: string, matchTime: string): number | null {
  const date = matchDate?.trim()
  const time = matchTime?.trim()

  if (!date || !time) {
    return null
  }

  const normalizedTime = time.length === 5 ? `${time}:00` : time
  const kickoff = new Date(`${date}T${normalizedTime}+08:00`).getTime()

  return Number.isNaN(kickoff) ? null : kickoff
}

/** scheduled 状态按开球时间推断：开球后 3 小时内视为进行中（跨页面共享，勿放页面 helpers）。 */
export function resolveMatchDisplayStatus(
  match: Pick<MatchCard, 'status' | 'match_date' | 'match_time'>,
  nowIso = new Date().toISOString(),
): MatchDisplayStatus {
  if (match.status !== 'scheduled') {
    return match.status
  }

  const kickoffAt = parseMatchKickoff(match.match_date, match.match_time)
  const nowAt = new Date(nowIso).getTime()

  if (!kickoffAt || Number.isNaN(nowAt)) {
    return 'scheduled'
  }

  if (nowAt >= kickoffAt && nowAt < kickoffAt + LIVE_INFERENCE_WINDOW_MS) {
    return 'live'
  }

  return 'scheduled'
}
