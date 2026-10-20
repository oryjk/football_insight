import { describe, expect, test } from 'bun:test'
import type { TransferCardView } from '../../types/transfer'
import { TRANSFER_POSTER, drawTransferPoster, transferPosterHeadline } from './transferPoster'

const view: TransferCardView = {
  team: { team_id: 77680, team_name: '成都蓉城', avatar_url: 'https://crest' },
  window: { code: '2026-winter', name: '2026 冬窗', opens_at: '', closes_at: '', open: true },
  can_vote: true,
  card: {
    share_code: 'ABCD1234', intent: 'out', returning: false, up: 128, down: 23, my_vote: 0, rank: 2, created_at: '',
    player: { player_id: 1, name: '某某', avatar_url: 'https://player', position: 'FW', shirt_number: 9, current_team: null, departed: false },
  },
}
function recorder() {
  const texts: string[] = [], images: string[] = []
  const noop = () => {}
  const context = { setFillStyle: noop, setFontSize: noop, fillText: (value: string) => texts.push(value), fillRect: noop, setStrokeStyle: noop, setLineWidth: noop, strokeRect: noop, beginPath: noop, arc: noop, save: noop, clip: noop, restore: noop, setTextAlign: noop, drawImage: (path: string) => images.push(path) } as unknown as UniApp.CanvasContext
  return { context, texts, images }
}

describe('transfer poster', () => {
  test('size and headline follow the card stance', () => {
    expect([TRANSFER_POSTER.width, TRANSFER_POSTER.height]).toEqual([750, 1200])
    expect(transferPosterHeadline(view)).toBe('成都蓉城球迷希望他离队')
    expect(transferPosterHeadline({ ...view, card: { ...view.card, intent: 'in', returning: true } })).toBe('成都蓉城球迷希望他回归')
  })

  test('poster_requires_a_mini_program_code', () => {
    const { context } = recorder()
    expect(() => drawTransferPoster(context, view, new Map(), '', { display_name: '球迷', avatar_url: null })).toThrow('小程序码')
  })

  test('draws player, stance, tally, rank, sharer and code', () => {
    const { context, texts, images } = recorder()
    const sharer = { display_name: '转发球迷', avatar_url: 'https://sharer' }
    drawTransferPoster(context, view, new Map([['https://crest', '/crest.png'], ['https://player', '/player.png'], ['https://sharer', '/sharer.png']]), '/code.png', sharer)
    expect(images).toEqual(['/crest.png', '/player.png', '/sharer.png', '/code.png'])
    for (const expected of ['某某', '成都蓉城球迷希望他离队', '赞成 128', '反对 23', '最想送走第 2 名', '转发球迷', '邀你表态', '扫码投一票']) {
      expect(texts.some(value => value.includes(expected))).toBe(true)
    }
  })

  test('poster_draws_without_any_remote_images', () => {
    const { context, texts, images } = recorder()
    drawTransferPoster(context, { ...view, card: { ...view.card, rank: null } }, new Map(), '/code.png', { display_name: '', avatar_url: null })
    expect(images).toEqual(['/code.png'])
    // 球员头像缺失用名字首字；分享人昵称为空时显示"球迷"
    expect(texts.includes('某')).toBe(true)
    expect(texts.includes('球迷')).toBe(true)
    expect(texts.some(value => value.includes('最想送走第'))).toBe(false)
  })
})
