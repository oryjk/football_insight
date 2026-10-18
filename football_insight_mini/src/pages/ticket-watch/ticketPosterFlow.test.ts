import { expect, test } from 'bun:test'
import { effectScope, nextTick } from 'vue'
import { createTicketPosterFlow } from './ticketPosterFlow'
import { buildTicketPosterContent } from './sharePoster'
import { useTicketWatchSharePoster } from './useTicketWatchSharePoster'
import { useTicketWatchState } from './useTicketWatchState'
import { posterMatch, posterSections } from './sharePoster.test'
import type { CurrentUser } from '../../types/auth'
import type { TicketPosterDependencies } from './ticketPosterFlow'

const content = buildTicketPosterContent({ mode: 'history', match: posterMatch, sections: posterSections })!
const code = { image_base64: 'code', mime_type: 'image/png' as const }
function deferred<T>() { let resolve!: (value: T) => void; const promise = new Promise<T>(done => { resolve = done }); return { promise, resolve } }

test('inflight code from an old selection cannot publish or clear new generation', async () => {
  const old = deferred<typeof code>()
  let codes = 0
  const flow = createTicketPosterFlow({ getCode: () => ++codes === 1 ? old.promise : Promise.resolve(code), render: async value => `${value.teams}.png` })
  const first = flow.generate(content)
  flow.reset()
  const second = flow.generate({ ...content, teams: '另一场比赛' })
  old.resolve(code)
  await Promise.all([first, second])
  expect(flow.imagePath.value).toBe('另一场比赛.png')
  expect(flow.generating.value).toBe(false)
})

test('one canvas serializes drawing and ignores an export completed after reset', async () => {
  const exported = deferred<string>()
  let renders = 0
  const flow = createTicketPosterFlow({ getCode: async () => code, render: async () => ++renders === 1 ? exported.promise : 'new.png' })
  const first = flow.generate(content)
  await Promise.resolve(); await Promise.resolve()
  expect(renders).toBe(1)
  flow.reset()
  const second = flow.generate(content)
  await Promise.resolve(); await Promise.resolve()
  expect(renders).toBe(1)
  exported.resolve('old.png')
  await Promise.all([first, second])
  expect(flow.imagePath.value).toBe('new.png')
})

test('code failure is visible, stops loading and does not render a misleading code-free image', async () => {
  let rendered = false
  const flow = createTicketPosterFlow({ getCode: async () => { throw new Error('unavailable') }, render: async () => { rendered = true; return 'image.png' } })
  await flow.generate(content)
  expect(flow.error.value).toBe('二维码暂时获取失败，请重试')
  expect(flow.generating.value).toBe(false)
  expect(flow.imagePath.value).toBe('')
  expect(rendered).toBe(false)
})

test('repeated generate during loading requests a single code', async () => {
  const pending = deferred<typeof code>()
  let calls = 0
  const flow = createTicketPosterFlow({ getCode: () => { calls++; return pending.promise }, render: async () => 'image.png' })
  const first = flow.generate(content)
  await flow.generate(content)
  expect(calls).toBe(1)
  pending.resolve(code); await first
})

test('actual page selection/tab/review invalidates images and native preview preserves a completed export', async () => {
  const scope = effectScope()
  const state = useTicketWatchState()
  state.isPageVisible.value = true
  state.activeTab.value = 'history'
  state.historyMatches.value = [posterMatch]
  state.historyTrendTotals.value = { [posterMatch.match_id]: 30 }
  state.displayedHistoryMatchId.value = posterMatch.match_id
  state.selectedHistoryMatchId.value = posterMatch.match_id
  state.historySections.value = posterSections
  state.historyLoading.value = false
  state.currentUser.value = { id: 'user', membership_tier: 'V9' } as CurrentUser
  const poster = scope.run(() => useTicketWatchSharePoster(state, () => {}, { getCode: async () => code, render: async () => 'image.png' }))!
  await poster.generate()
  expect(poster.imagePath.value).toBe('image.png')
  state.selectedHistoryMatchId.value = 100
  expect(poster.imagePath.value).toBe('')
  expect(poster.visible.value).toBe(false)
  state.selectedHistoryMatchId.value = posterMatch.match_id
  await poster.generate()
  state.activeTab.value = 'history-stats'
  expect(poster.visible.value).toBe(false)
  expect(poster.imagePath.value).toBe('')
  state.activeTab.value = 'history'
  await poster.generate()
  state.systemConfigUnderReview.value = true
  expect(poster.visible.value).toBe(false)
  expect(poster.imagePath.value).toBe('')
  state.systemConfigUnderReview.value = false
  await poster.generate()
  state.isPageVisible.value = false
  await nextTick()
  expect(poster.imagePath.value).toBe('image.png')
  scope.stop()
  expect(poster.imagePath.value).toBe('')
})

for (const mode of ['current', 'history'] as const) {
  test(`${mode} same-match refresh preserves an in-flight frozen snapshot and completed preview`, async () => {
    const scope = effectScope()
    const state = useTicketWatchState()
    state.isPageVisible.value = true
    state.activeTab.value = mode
    state.currentLoading.value = false
    state.historyLoading.value = false
    state.currentUser.value = { id: 'user', membership_tier: 'V9' } as CurrentUser
    state.currentMatch.value = posterMatch
    state.currentSections.value = posterSections
    state.currentBoardLoadedAtMs.value = Date.parse('2026-10-19T10:13:24+08:00')
    state.historyMatches.value = [posterMatch]
    state.historyTrendTotals.value = { [posterMatch.match_id]: 30 }
    state.displayedHistoryMatchId.value = posterMatch.match_id
    state.selectedHistoryMatchId.value = posterMatch.match_id
    state.historySections.value = posterSections
    const pending = deferred<typeof code>()
    let renderedCutoff: string | null = null, renderedTotal = 0
    const dependencies: TicketPosterDependencies = {
      getCode: () => pending.promise,
      render: async snapshot => { renderedCutoff = snapshot.cutoffLabel; renderedTotal = snapshot.totalOccurrences; return 'frozen.png' },
    }
    const poster = scope.run(() => useTicketWatchSharePoster(state, () => {}, dependencies))!
    const generating = poster.generate()
    const newSections = posterSections.map(section => ({ ...section, items: section.items.map(item => ({ ...item, occurrences: item.occurrences + 5 })) }))
    state.currentBoardLoadedAtMs.value += 60_000
    if (mode === 'current') state.currentSections.value = newSections
    else state.historySections.value = newSections
    pending.resolve(code)
    await generating
    expect(poster.imagePath.value).toBe('frozen.png')
    expect(renderedTotal).toBe(30)
    expect(renderedCutoff).toBe(mode === 'current' ? '2026-10-19 10:13:24' : null)
    // Simulate returning from system preview and onShow refreshing both boards.
    state.isPageVisible.value = false
    state.isPageVisible.value = true
    state.currentBoardLoadedAtMs.value += 60_000
    state.currentSections.value = [...newSections]
    state.historySections.value = [...newSections]
    await nextTick()
    expect(poster.imagePath.value).toBe('frozen.png')
    if (mode === 'current') state.currentMatch.value = { ...posterMatch, match_id: 100 }
    else state.selectedHistoryMatchId.value = 100
    expect(poster.imagePath.value).toBe('')
    scope.stop()
  })
}
