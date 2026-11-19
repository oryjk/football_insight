import { describe, expect, test } from 'bun:test'
import {
  buildDirectorShareText,
  computeDirectorStyleTag,
  createCustomPlanPlayer,
  deserializeDirectorDraft,
  filterCandidatesByLeague,
  serializeDirectorDraft,
  toUpsertPlayerInput,
  validateForeignQuota,
} from './helpers'
import {
  buildDirectorEditUrl,
  buildDirectorSharePath,
  buildDirectorViewUrl,
  parseDirectorEditQuery,
  parseDirectorViewCode,
} from '../../utils/directorLinks'

describe('director style tag calculation', () => {
  test('returns K联赛淘宝流 when signing K-league foreign player without full rebuild', () => {
    expect(
      computeDirectorStyleTag({
        foreignStaysCount: 3,
        foreignLeavesCount: 2,
        foreignIns: [{ league_tag: 'K1', position: 'FW' }],
        domesticIns: [],
      }),
    ).toBe('K联赛淘宝流')
  })

  test('prioritizes 推倒重建派 when K-league signing coincides with >=3 leaves and >=3 foreign ins', () => {
    expect(
      computeDirectorStyleTag({
        foreignStaysCount: 2,
        foreignLeavesCount: 3,
        foreignIns: [
          { league_tag: 'K1', position: 'FW' },
          { league_tag: 'J1', position: 'MF' },
          { league_tag: 'BRA', position: 'DF' },
        ],
        domesticIns: [],
      }),
    ).toBe('推倒重建派')
  })

  test('returns 推倒重建派 when foreignLeavesCount >= 3', () => {
    expect(
      computeDirectorStyleTag({
        foreignStaysCount: 2,
        foreignLeavesCount: 3,
        foreignIns: [{ league_tag: 'J1', position: 'MF' }],
        domesticIns: [],
      }),
    ).toBe('推倒重建派')
  })

  test('returns 稳健微调派 when >=4 stays and <=1 foreign signing', () => {
    expect(
      computeDirectorStyleTag({
        foreignStaysCount: 4,
        foreignLeavesCount: 1,
        foreignIns: [{ league_tag: 'CSL', position: 'MF' }],
        domesticIns: [],
      }),
    ).toBe('稳健微调派')
  })

  test('returns 锋线狂魔 when >=2 signings and >=50% FW', () => {
    expect(
      computeDirectorStyleTag({
        foreignStaysCount: 3,
        foreignLeavesCount: 2,
        foreignIns: [
          { league_tag: 'BRA', position: 'FW' },
          { league_tag: 'CSL', position: 'MF' },
        ],
        domesticIns: [],
      }),
    ).toBe('锋线狂魔')
  })

  test('returns 铁血防线派 when >=2 signings and >=50% DF/GK', () => {
    expect(
      computeDirectorStyleTag({
        foreignStaysCount: 3,
        foreignLeavesCount: 1,
        foreignIns: [{ league_tag: 'EUR', position: 'DF' }],
        domesticIns: [{ position: 'GK' }],
      }),
    ).toBe('铁血防线派')
  })

  test('falls back to 务实均衡派', () => {
    expect(
      computeDirectorStyleTag({
        foreignStaysCount: 3,
        foreignLeavesCount: 1,
        foreignIns: [{ league_tag: 'CSL', position: 'MF' }],
        domesticIns: [],
      }),
    ).toBe('务实均衡派')
  })
})

describe('foreign quota validation', () => {
  test('rejects 0 foreign players', () => {
    const res = validateForeignQuota(0, 0, 5)
    expect(res.valid).toBe(false)
    expect(res.overLimit).toBe(false)
    expect(res.remaining).toBe(5)
    expect(res.message.includes('至少')).toBe(true)
  })

  test('accepts 1..5 foreign players and computes remaining slots', () => {
    const res = validateForeignQuota(3, 1, 5)
    expect(res.valid).toBe(true)
    expect(res.total).toBe(4)
    expect(res.remaining).toBe(1)
    expect(res.progressPercent).toBe(80)
    expect(res.message).toBe('')
  })

  test('rejects >5 foreign players', () => {
    const res = validateForeignQuota(4, 2, 5)
    expect(res.valid).toBe(false)
    expect(res.overLimit).toBe(true)
    expect(res.remaining).toBe(0)
    expect(res.message.includes('超出上限')).toBe(true)
  })
})

describe('draft serialization & custom player helpers', () => {
  test('round-trips valid draft snapshots', () => {
    const custom = createCustomPlanPlayer({
      playerName: ' 金某某 ',
      position: 'MF',
      leagueTag: 'K1',
      currentClub: '蔚山HD',
      isForeign: true,
    })
    expect(custom.key).toBe('custom:金某某')
    expect(toUpsertPlayerInput(custom)).toEqual({
      source: 'custom',
      candidate_code: null,
      player_id: null,
      player_name: '金某某',
      position: 'MF',
      league_tag: 'K1',
      current_club: '蔚山HD',
    })

    const raw = serializeDirectorDraft({
      teamId: 77680,
      circleCode: 'AB12CD34',
      fromCode: 'AB12CD34',
      foreignStayIds: [1001, 1002],
      foreignLeaveIds: [1003],
      foreignIns: [custom],
      domesticIns: [],
      domesticOuts: [2001],
      manifesto: '冲亚冠！',
      savedAt: 1700000000000,
    })
    const parsed = deserializeDirectorDraft(raw)
    expect(parsed?.teamId).toBe(77680)
    expect(parsed?.foreignStayIds).toEqual([1001, 1002])
    expect(parsed?.foreignIns.length).toBe(1)
    expect(parsed?.manifesto).toBe('冲亚冠！')
  })

  test('returns null on malformed draft json', () => {
    expect(deserializeDirectorDraft('')).toBeNull()
    expect(deserializeDirectorDraft('{bad json')).toBeNull()
    expect(deserializeDirectorDraft('{"teamId": 0}')).toBeNull()
  })

  test('filters candidates by K league prefix', () => {
    const list = [
      { key: '1', source: 'pool' as const, player_id: null, candidate_code: 'a', player_name: 'A', position: 'FW' as const, country: '巴西', is_foreign: true, league_tag: 'K1', current_club: '蔚山', tags: [], avatar_url: null },
      { key: '2', source: 'pool' as const, player_id: null, candidate_code: 'b', player_name: 'B', position: 'MF' as const, country: '韩国', is_foreign: true, league_tag: 'K2', current_club: '水原', tags: [], avatar_url: null },
      { key: '3', source: 'pool' as const, player_id: null, candidate_code: 'c', player_name: 'C', position: 'DF' as const, country: '日本', is_foreign: true, league_tag: 'J1', current_club: '横滨', tags: [], avatar_url: null },
    ]
    expect(filterCandidatesByLeague(list, 'K').map(item => item.key)).toEqual(['1', '2'])
    expect(filterCandidatesByLeague(list, 'J1').map(item => item.key)).toEqual(['3'])
  })
})

describe('director share text & link helpers', () => {
  test('builds share text with and without manifesto', () => {
    const base = {
      team_name: '成都蓉城',
      style_tag: 'K联赛淘宝流',
      foreign_stays: [createCustomPlanPlayer({ playerName: 'A', position: 'FW', leagueTag: 'CSL', currentClub: '蓉城', isForeign: true })],
      foreign_ins: [createCustomPlanPlayer({ playerName: 'B', position: 'MF', leagueTag: 'K1', currentClub: '大邱', isForeign: true })],
      manifesto: '预算全砸K联赛！',
    }
    expect(buildDirectorShareText(base)).toBe('我给成都蓉城定了2外援（K联赛淘宝流）：「预算全砸K联赛！」不服来PK！')
    expect(buildDirectorShareText({ ...base, manifesto: '' })).toBe('我给成都蓉城定了2外援（K联赛淘宝流），不服来微信群PK！')
  })

  test('builds and parses edit/view URLs and share paths', () => {
    const editUrl = buildDirectorEditUrl({ teamId: 77680, circleCode: 'AB12CD34', fromCode: 'EF56GH78' })
    expect(editUrl).toBe('/pages/director/index?team_id=77680&circle_code=AB12CD34&from_code=EF56GH78')
    expect(parseDirectorEditQuery({ team_id: '77680', circle_code: 'AB12CD34', from_code: 'EF56GH78' })).toEqual({
      teamId: 77680,
      circleCode: 'AB12CD34',
      fromCode: 'EF56GH78',
    })

    const viewUrl = buildDirectorViewUrl('AB12CD34')
    expect(viewUrl).toBe('/pages/director/view?code=AB12CD34')
    expect(buildDirectorSharePath(viewUrl)).toBe('/pages/director/view?code=AB12CD34&src=share&cid=director')
    expect(parseDirectorViewCode({ code: 'AB12CD34' })).toBe('AB12CD34')
    expect(parseDirectorViewCode({ scene: encodeURIComponent('s=share&c=director&t=ZX98CV76') })).toBe('ZX98CV76')
  })
})
