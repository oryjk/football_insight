import { expect, test } from 'bun:test'
import { miniPitchPosition, myTacticsSummary } from './tacticsPresentation'
const draft = { formation: '4-3-3', slots: [{ slot: 0, player_id: 7, x: 0.5, y: 0.9 }], updated_at: '2026-10-19' }
test('cloud draft stays distinct from published snapshot regardless of timestamp or slot order', () => {
  expect(myTacticsSummary(draft, null).status).toBe('草稿已保存')
  const published = { ...draft, share_code: 'Ab12Cd34' }
  expect(myTacticsSummary({ ...draft, updated_at: 'later' }, published).status).toBe('已发布')
  expect(myTacticsSummary({ ...draft, formation: '4-4-2' }, published).status).toBe('有未发布的修改')
  expect(myTacticsSummary({ ...draft, slots: [{ ...draft.slots[0]!, player_id: null }] }, published).filled).toBe(0)
  expect(myTacticsSummary(null, published).snapshot).toEqual(published)
  expect(myTacticsSummary(null, null).action).toBe('排我的阵容')
})
test('the call to action invites the fan according to where their lineup stands', () => {
  const published = { ...draft, share_code: 'Ab12Cd34' }
  expect(myTacticsSummary(null, null).action).toBe('排我的阵容')
  expect(myTacticsSummary(draft, null).action).toBe('继续排阵')
  expect(myTacticsSummary({ ...draft, formation: '4-4-2' }, published).action).toBe('继续排阵')
  expect(myTacticsSummary({ ...draft, updated_at: 'later' }, published).action).toBe('改我的阵容')
  expect(myTacticsSummary(null, published).action).toBe('改我的阵容')
})
test('thumbnail positions remain in pitch bounds even with malformed coordinates', () => {
  expect(miniPitchPosition(0.5, 0.9)).toBe('left:50%;top:90%')
  expect(miniPitchPosition(-1, 2)).toBe('left:6%;top:94%')
  expect(miniPitchPosition(NaN, Infinity)).toBe('left:50%;top:50%')
})
