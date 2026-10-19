import { computed, ref, type Ref } from 'vue'
import { getTicketWatchMatches } from '../../api/ticketWatch'
import type { SupportMatchDetail } from '../../types/support'
import type { TicketWatchMatchSummary } from '../../types/ticketWatch'

type SupportMatchKey = Pick<SupportMatchDetail, 'round_number' | 'match_date' | 'match_time' | 'home_team'>

/** 两边队名写法不完全一致（如「大连英博」/「大连英博海发」），互相包含即视为同一队。 */
function isSameTeamName(left: string, right: string): boolean {
  return left.includes(right) || right.includes(left)
}

/**
 * 余票监控的 match_id 来自外部抢票监控服务，与助力比赛（f_i_matches）不是同一套 ID，
 * 所以按「轮次 + 比赛日期 + 开球时间 + 主队」对应到同一场比赛。
 */
export function findTicketSaleMatch(
  matches: TicketWatchMatchSummary[],
  match: SupportMatchKey | null,
): TicketWatchMatchSummary | null {
  if (!match) return null
  return matches.find((candidate) => candidate.round_number === match.round_number
    && candidate.match_date === match.match_date
    && candidate.match_time === match.match_time
    && isSameTeamName(candidate.home_team_name, match.home_team.team_name)) ?? null
}

/**
 * 首页「我的主队」的抢票开始时间：从公开的余票监控比赛列表里找下一场。
 * 只有被监控的比赛才有开票时间，找不到或请求失败时返回 null，面板不显示抢票倒计时。
 */
export function useHomeTicketSale(
  nextMatch: Ref<SupportMatchKey | null>,
  fetchMatches: () => Promise<TicketWatchMatchSummary[]> = getTicketWatchMatches,
) {
  const matches = ref<TicketWatchMatchSummary[]>([])

  async function loadTicketSaleMatches(): Promise<void> {
    try {
      matches.value = await fetchMatches()
    } catch (error) {
      console.warn('[home] ticket sale matches load failed', error)
    }
  }

  const saleStartAt = computed(() => findTicketSaleMatch(matches.value, nextMatch.value)?.sale_start_at ?? null)

  return { saleStartAt, loadTicketSaleMatches }
}
