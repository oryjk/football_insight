import { describe, expect, test } from 'bun:test'
import { ref } from 'vue'
import { findTicketSaleMatch, useHomeTicketSale } from './useHomeTicketSale'
import type { TicketWatchMatchSummary } from '../../types/ticketWatch'

function summary(overrides: Partial<TicketWatchMatchSummary> = {}): TicketWatchMatchSummary {
  return {
    match_id: 584,
    external_match_id: '100',
    round_number: 29,
    sale_start_at: '2026-10-22T14:00:00+08:00',
    match_date: '2026-10-24',
    match_time: '15:30',
    kickoff_at: '2026-10-24T15:30:00+08:00',
    home_team_name: '成都蓉城',
    away_team_name: '大连英博海发',
    is_current: true,
    ...overrides,
  }
}

// 助力比赛的 match_id 与余票监控不是同一套 ID，故意取不同值。
const nextMatch = {
  round_number: 29,
  match_date: '2026-10-24',
  match_time: '15:30',
  home_team: { team_name: '成都蓉城' },
} as Parameters<typeof findTicketSaleMatch>[1]

describe('findTicketSaleMatch', () => {
  test('按轮次、日期、开球时间和主队对应，不依赖 match_id', () => {
    const target = summary()
    const others = [
      summary({ match_id: 588, round_number: 1, match_date: '2026-10-13', home_team_name: '成都蓉城希拉谷' }),
      summary({ match_id: 600, match_time: '19:30' }),
      summary({ match_id: 601, home_team_name: '上海海港' }),
    ]
    expect(findTicketSaleMatch([...others, target], nextMatch)).toBe(target)
    expect(findTicketSaleMatch(others, nextMatch)).toBeNull()
    expect(findTicketSaleMatch([target], null)).toBeNull()
  })

  test('队名简称与全称互相包含视为同一队', () => {
    const away = summary({ home_team_name: '大连英博海发' })
    expect(findTicketSaleMatch([away], { ...nextMatch!, home_team: { team_name: '大连英博' } } as typeof nextMatch)).toBe(away)
  })
})

describe('useHomeTicketSale', () => {
  test('加载后给出下一场开票时间，换场后随之变化', async () => {
    const match = ref(nextMatch)
    const { saleStartAt, loadTicketSaleMatches } = useHomeTicketSale(match, async () => [summary()])
    expect(saleStartAt.value).toBeNull()
    await loadTicketSaleMatches()
    expect(saleStartAt.value).toBe('2026-10-22T14:00:00+08:00')
    match.value = { ...nextMatch!, round_number: 30 } as typeof nextMatch
    expect(saleStartAt.value).toBeNull()
  })

  test('请求失败时静默返回 null，不抛错', async () => {
    const { saleStartAt, loadTicketSaleMatches } = useHomeTicketSale(ref(nextMatch), async () => {
      throw new Error('network')
    })
    await loadTicketSaleMatches()
    expect(saleStartAt.value).toBeNull()
  })
})
