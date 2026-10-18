import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import ts from 'typescript'
import { buildSharePath, buildShareQuery } from '../src/utils/shareSource'
import { parseAcquisitionLaunch } from '../src/utils/acquisitionSource'
import { buildTopicSharePayload, buildTopicTimelinePayload } from '../src/pages/topic-detail/share'
import { recapSharePath } from '../src/pages/season-recap/helpers'

// Evaluate the actual registered callbacks without mounting unrelated page/API state.
function pageShare(page: string, hook: string, state: Record<string, unknown> = {}) {
  const source = readFileSync(new URL(`../src/pages/${page}/index.vue`, import.meta.url), 'utf8')
  const script = source.match(/<script setup[^>]*>([\s\S]*?)<\/script>/)![1]!
  const ast = ts.createSourceFile('page.ts', script, ts.ScriptTarget.Latest, true)
  const calls = ast.statements.filter(ts.isExpressionStatement).map(node => node.expression)
    .filter(ts.isCallExpression).filter(node => node.expression.getText(ast) === hook)
  expect(calls).toHaveLength(1)
  const helpers = ast.statements.filter(ts.isFunctionDeclaration)
    .filter(node => node.name?.text === 'buildTicketSharePayload').map(node => node.getText(ast)).join('\n')
  const scope = { buildSharePath, buildShareQuery, buildTopicSharePayload, buildTopicTimelinePayload, ...state }
  return Function(...Object.keys(scope), `${helpers}\nreturn (${calls[0]!.arguments[0]!.getText(ast)})();`)(...Object.values(scope))
}

const ref = (value: unknown) => ({ value })
const match = { home_team_name: '成都蓉城', away_team_name: '客队', match_id: 42 }

test('all public page share callbacks retain their target and correct attribution', () => {
  const cases: [string, string, string, Record<string, unknown>][] = [
    ['home', 'home', '/pages/home/index', { supportFavoriteTeam: ref(null) }],
    ['matches', 'matches', '/pages/matches/index', {}],
    ['rankings', 'rankings', '/pages/rankings/index', { rankingsContentRef: ref(null) }],
    ['insights', 'insights', '/pages/insights/index', { selectedInsight: ref(null) }],
    ['team-board', 'team_board', '/pages/team-board/index?teamId=77680', { board: ref(null), teamId: ref(77680) }],
    ['team-board', 'team_board', '/pages/insights/index', { board: ref(null), teamId: ref(null) }],
    ['support', 'support', '/pages/support/index?matchId=42', { detail: ref({ match_id: 42, home_team: { team_name: '主队' }, away_team: { team_name: '客队' } }), supportedTeam: ref(null) }],
    ['support', 'support', '/pages/home/index', { detail: ref(null) }],
    ['topics', 'topics', '/pages/topics/index', {}],
    ['seat-swap', 'swap', '/pages/seat-swap/index', {}],
    ...(['current', 'history'] as const).map(tab => ['ticket-watch', tab === 'history' ? 'ticket_history' : 'ticket_current', '/pages/ticket-watch/index',
      { activeTab: ref(tab), displayedHistoryMatch: ref(match), currentMatch: ref(match) }] as [string, string, string, Record<string, unknown>]),
    ['ticket-watch', 'ticket_current', '/pages/ticket-watch/index', { activeTab: ref('history'), displayedHistoryMatch: ref(null), currentMatch: ref(null) }],
  ]
  for (const [page, cid, target, state] of cases) {
    const share = pageShare(page, 'onShareAppMessage', state)
    expect(share.path).toBe(buildSharePath(target, cid))
    expect(parseAcquisitionLaunch({ query: Object.fromEntries(new URLSearchParams(share.path.split('?')[1])) })).toEqual({ src: 'share', cid })
  }
})

test('only the intended pages offer timeline sharing and preserve their content context', () => {
  expect(pageShare('home', 'onShareTimeline', { supportFavoriteTeam: ref(null) }).query).toBe('src=share&cid=home')
  for (const tab of ['current', 'history']) {
    const state = { activeTab: ref(tab), displayedHistoryMatch: ref(match), currentMatch: ref(match) }
    expect(pageShare('ticket-watch', 'onShareTimeline', state)).toEqual({
      title: pageShare('ticket-watch', 'onShareAppMessage', state).title,
      query: `src=share&cid=ticket_${tab}`,
    })
  }
  const article = { id: 'topic_2026-10-13', title: '比赛话题' }
  const state = { shareArticle: ref(article), shareCard: { imageUrl: ref('/tmp/topic.png') } }
  expect(pageShare('topic-detail', 'onShareAppMessage', state).path).toBe('/pages/topic-detail/index?id=topic_2026-10-13&src=share&cid=topic_detail')
  expect(pageShare('topic-detail', 'onShareTimeline', state)).toEqual({ title: article.title, query: 'id=topic_2026-10-13&src=share&cid=topic_detail', imageUrl: '/tmp/topic.png' })
  for (const page of ['matches', 'rankings', 'insights', 'team-board', 'support', 'topics', 'seat-swap']) {
    const source = readFileSync(new URL(`../src/pages/${page}/index.vue`, import.meta.url), 'utf8')
    expect(source.includes('onShareTimeline(')).toBe(false)
  }
})

test('season recap keeps its existing campaign, season and cover', () => {
  const state = { season: ref(2026), sharePath: ref(recapSharePath(2026)), posterPath: ref('/tmp/recap.png') }
  expect(pageShare('season-recap', 'onShareAppMessage', state)).toEqual({ title: '我的 2026 蓉城赛季足迹', path: recapSharePath(2026), imageUrl: '/tmp/recap.png' })
  expect(pageShare('season-recap', 'onShareTimeline', state).query).toBe('season=2026&src=share&cid=recap2026')
})

test('standings poster sharing preserves its target, cover and any existing campaign', () => {
  for (const path of ['/pages/rankings/index?openPoster=1&table=standings', '/pages/rankings/index?openPoster=1&src=campaign&cid=existing']) {
    const payload = { title: '中超积分榜', path, imageUrl: '/tmp/standings.png' }
    const state = { rankingsContentRef: ref({ getStandingsPosterSharePayload: () => payload }) }
    expect(pageShare('rankings', 'onShareAppMessage', state)).toEqual({ ...payload, path: buildSharePath(path, 'rankings') })
  }
})
