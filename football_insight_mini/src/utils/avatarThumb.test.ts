import { describe, expect, test } from 'bun:test'
import { avatarCandidates, avatarThumbUrl } from './avatarThumb'

const BASE = 'https://oryjk.cn:82/minio/football-insight/summary/players'

describe('avatar thumbnails', () => {
  test('only_minio_player_avatars_get_thumbnails', () => {
    expect(avatarThumbUrl(`${BASE}/475933.png`, 96)).toBe(`${BASE}/thumbs/96/475933.png.jpg`)
    expect(avatarThumbUrl(`${BASE}/113121-team143-20260304.jpg`, 240)).toBe(`${BASE}/thumbs/240/113121-team143-20260304.jpg.jpg`)
    for (const url of ['https://oryjk.cn:82/minio/football-insight/summary/teams/1.png', 'https://thirdwx.qlogo.cn/abc/132', 'https://example.com/players/1.png']) {
      expect(avatarThumbUrl(url, 96)).toBe(url)
    }
    expect(avatarThumbUrl(null, 96)).toBe('')
    expect(avatarThumbUrl('', 96)).toBe('')
  })

  test('avatar_candidates_fall_back_to_the_original', () => {
    expect(avatarCandidates(`${BASE}/1.png`, 96)).toEqual([`${BASE}/thumbs/96/1.png.jpg`, `${BASE}/1.png`])
    expect(avatarCandidates('https://thirdwx.qlogo.cn/abc/132', 96)).toEqual(['https://thirdwx.qlogo.cn/abc/132'])
    expect(avatarCandidates(null, 96)).toEqual([])
  })
})
