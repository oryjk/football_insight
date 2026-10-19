import { describe, expect, test } from 'bun:test'
import type { PublicLineup, SavedLineup } from '../../types/lineup'
import { emptyLineup } from './helpers'
import { canShareLineup, drawLineupPoster, lineupSharePayload, posterPlayerPoint, LINEUP_CARD, LINEUP_POSTER } from './lineupPoster'
import { prepareEditorShare, type EditorShareSource } from './lineupEditorShare'

const slots = emptyLineup('4-3-3').map((slot, index) => ({ ...slot, player_id: index + 1 }))
const saved: SavedLineup = { slots, formation: '4-3-3', share_code: 'Ab12Cd34', updated_at: '2026-10-19T00:00:00Z' }
const lineup: PublicLineup = { ...saved, team: { team_id: 1, team_name: '成都蓉城', avatar_url: null }, author: { display_name: '球迷', avatar_url: null }, created_at: saved.updated_at, likes_count: 0, liked_by_me: false, players: slots.map(slot => ({ player_id: slot.player_id, name: `球员${slot.player_id}`, shirt_number: slot.player_id, avatar_url: null, position: 'MF', last_season: 2026, former: false })) }
function source(overrides: Partial<EditorShareSource> = {}): EditorShareSource { return { context: () => 'team1:revision1', slots: () => slots, save: async () => 'saved', saved: () => saved, dirty: () => false, detail: async () => lineup, ...overrides } }

describe('editor sharing gates', () => {
  test('requires exactly eleven distinct players before any save or public request', async () => {
    let calls = 0
    const outcome = await prepareEditorShare(source({ slots: () => slots.slice(0, 10), save: async () => { calls++; return 'saved' } }))
    expect(outcome.result).toBe('incomplete'); expect(calls).toBe(0)
    expect(canShareLineup(slots)).toBe(true)
    expect(canShareLineup(slots.map(slot => ({ ...slot, player_id: 1 })))).toBe(false)
  })
  test('saves current edits first and shares the code returned by that save', async () => {
    const calls: string[] = []; let metadata: SavedLineup | null = null
    const result = await prepareEditorShare(source({ save: async () => { calls.push('save'); metadata = saved; return 'saved' }, saved: () => metadata, detail: async code => { calls.push(code); return lineup } }))
    expect(calls).toEqual(['save', 'Ab12Cd34']); expect(result.lineup?.share_code).toBe(saved.share_code)
  })
  test('login and failed save never request a public object', async () => {
    for (const result of ['need-login', 'failed'] as const) {
      let calls = 0
      expect((await prepareEditorShare(source({ save: async () => result, detail: async () => { calls++; return lineup } }))).result).toBe(result)
      expect(calls).toBe(0)
    }
  })
  test('edits during save or a team switch during detail cannot publish an obsolete lineup', async () => {
    let context = 'A'
    const edited = await prepareEditorShare(source({ context: () => context, save: async () => { context = 'B'; return 'saved' } }))
    expect(edited.result).toBe('cancelled')
    context = 'A'
    const switched = await prepareEditorShare(source({ context: () => context, detail: async () => { context = 'B'; return lineup } }))
    expect(switched.result).toBe('cancelled'); expect(switched.lineup).toBe(undefined)
  })
  test('a save of an earlier revision cannot share while current edits are still dirty', async () => {
    expect((await prepareEditorShare(source({ dirty: () => true }))).result).toBe('cancelled')
  })
})

describe('poster content and geometry', () => {
  test('export sizes and native readonly destination are explicit', () => {
    expect([LINEUP_POSTER.width, LINEUP_POSTER.height]).toEqual([750, 1200])
    expect(LINEUP_CARD.width / LINEUP_CARD.height).toBe(5 / 4)
    expect(lineupSharePayload(lineup).path).toBe('/pages/lineup/view?code=Ab12Cd34&src=share&cid=lineup')
    expect(posterPlayerPoint(-1, 2)).toEqual({ x: 90, y: 870 })
    expect(posterPlayerPoint(.5, .9).y > posterPlayerPoint(.5, .2).y).toBe(true)
  })
  test('failed avatars retain numbers/names; qrcode is mandatory for exported poster', () => {
    const texts: string[] = [], images: string[] = []
    const noop = () => {}
    const context = { setFillStyle: noop, setFontSize: noop, fillText: (value: string) => texts.push(value), fillRect: noop, setStrokeStyle: noop, setLineWidth: noop, strokeRect: noop, beginPath: noop, moveTo: noop, lineTo: noop, stroke: noop, arc: noop, save: noop, clip: noop, restore: noop, setTextAlign: noop, drawImage: (path: string) => images.push(path) } as unknown as UniApp.CanvasContext
    expect(() => drawLineupPoster(context, lineup, new Map(), '')).toThrow('小程序码')
    drawLineupPoster(context, lineup, new Map(), '/code.png')
    expect(images).toEqual(['/code.png']); expect(texts.includes('11')).toBe(true); expect(texts.includes('球员11')).toBe(true); expect(texts.includes('成都蓉城')).toBe(true)
    texts.length = 0; images.length = 0
    const withAvatar = { ...lineup, team: { ...lineup.team, avatar_url: 'https://crest' }, author: { ...lineup.author, display_name: '超长昵称'.repeat(8) }, players: lineup.players.map((player, index) => ({ ...player, former: index === 0, avatar_url: index === 0 ? 'https://avatar' : null })) }
    drawLineupPoster(context, withAvatar, new Map([['https://avatar', '/avatar.png'], ['https://crest', '/crest.png']]), '', true)
    expect(images).toEqual(['/crest.png', '/avatar.png']); expect(texts.includes('球员1')).toBe(true)
    expect(texts.includes('前')).toBe(true); expect(texts.includes('1')).toBe(true)
    expect(texts.some(value => value.includes('…的战术板') && Array.from(value).length <= 21)).toBe(true)
  })
})
