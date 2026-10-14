import { describe, expect, test } from 'bun:test'
import { buildSupportPosterHeadline } from './poster'

const home = { team_id: 1, team_name: '成都蓉城' }
const away = { team_id: 2, team_name: '上海申花' }

describe('buildSupportPosterHeadline', () => {
  test('已助力时点名对手拉票', () => {
    const detail = { home_team: home, away_team: away }
    expect(buildSupportPosterHeadline(detail as never, away as never)).toBe('我已为上海申花助力，来把成都蓉城压过去')
    expect(buildSupportPosterHeadline(detail as never, home as never)).toBe('我已为成都蓉城助力，来把上海申花压过去')
  })

  test('未助力时展示双方对抗', () => {
    const detail = { home_team: home, away_team: away }
    expect(buildSupportPosterHeadline(detail as never, null)).toBe('成都蓉城 和 上海申花 的热度对抗已经打响')
  })
})
