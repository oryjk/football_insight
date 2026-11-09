import { describe, expect, test } from 'bun:test'
import type { ClubPlayer, PublicLineup, SquadPlayer, SquadResponse } from '../../types/lineup'
import { assignPlayer, emptyLineup } from './helpers'
import { buildPitchPlayers, buildSearchGroups, buildSheetGroups, buildSheetTabs, positionIndex, resolveSheetTab, teamShortName } from './lineupViewModel'
import { publicPitch } from './lineupPoster'

function player(id: number, position: SquadPlayer['position'], extra: Partial<SquadPlayer> = {}): SquadPlayer {
  return { player_id: id, name: `P${id}`, shirt_number: id, position, avatar_url: null, last_season: 2026, ...extra }
}

const squad: SquadResponse = {
  team: { team_id: 1, team_name: '成都蓉城', avatar_url: null },
  active: [player(1, 'GK'), player(2, 'DF'), player(3, 'MF'), player(4, 'FW')],
  former: [player(20, 'DF', { last_season: 2025 }), player(21, 'FW', { last_season: 2024 })],
}

describe('buildPitchPlayers', () => {
  test('joins slots with squad data and marks former players', () => {
    let slots = assignPlayer(emptyLineup('4-3-3'), 0, 1)
    slots = assignPlayer(slots, 1, 20)
    const pitch = buildPitchPlayers(slots, '4-3-3', squad)
    expect(pitch.length).toBe(11)
    expect(pitch[0].player?.player_id).toBe(1)
    expect(pitch[0].player?.former).toBe(false)
    expect(pitch[1].player?.player_id).toBe(20)
    expect(pitch[1].player?.former).toBe(true)
    expect(pitch[1].player?.last_season).toBe(2025)
    expect(pitch[2].player).toBeNull()
    expect(pitch[0].label).toBe('门将')
  })

  test('a former player in a saved lineup stays visible and keeps the former flag', () => {
    const slots = assignPlayer(emptyLineup('4-3-3'), 3, 21)
    const entry = buildPitchPlayers(slots, '4-3-3', squad)[3]
    expect(entry.player?.player_id).toBe(21)
    expect(entry.player?.former).toBe(true)
  })

  test('a player id the squad no longer contains renders as an empty slot', () => {
    const slots = assignPlayer(emptyLineup('4-3-3'), 3, 9999)
    expect(buildPitchPlayers(slots, '4-3-3', squad)[3].player).toBeNull()
  })

  test('works before the squad has loaded', () => {
    expect(buildPitchPlayers(emptyLineup('4-3-3'), '4-3-3', null).every((p) => p.player === null)).toBe(true)
  })

  test('carries the slot coordinates from the lineup, not the formation defaults', () => {
    const moved = emptyLineup('4-3-3').map((s) => (s.slot === 2 ? { ...s, x: 0.31, y: 0.62 } : s))
    const entry = buildPitchPlayers(moved, '4-3-3', squad)[2]
    expect([entry.x, entry.y]).toEqual([0.31, 0.62])
  })
})

describe('buildSheetGroups', () => {
  test('active group first, former group second; preferred position first; on-pitch players flagged', () => {
    const slots = assignPlayer(emptyLineup('4-3-3'), 1, 2)
    const groups = buildSheetGroups(squad, slots, 'DF')
    expect(groups.map((g) => g.key)).toEqual(['active', 'former'])
    const active = groups[0].players
    expect(active[0].player_id).toBe(2)
    expect(active.find((p) => p.player_id === 2)!.onPitch).toBe(true)
    expect(active.find((p) => p.player_id === 3)!.onPitch).toBe(false)
    expect(active.find((p) => p.player_id === 1)!.preferred).toBe(false)
    expect(active.find((p) => p.player_id === 2)!.preferred).toBe(true)
    expect(groups[1].players[0].player_id).toBe(20)
    expect(groups[1].players.every((p) => p.former)).toBe(true)
  })

  test('omits the former group when there are no former players', () => {
    expect(buildSheetGroups({ ...squad, former: [] }, emptyLineup('4-3-3'), 'MF').map((g) => g.key)).toEqual(['active'])
  })

  test('keeps the original squad order within preferred and non-preferred players', () => {
    const many: SquadResponse = {
      ...squad,
      active: [player(5, 'MF'), player(6, 'DF'), player(7, 'MF'), player(8, 'DF')],
      former: [],
    }
    const ids = buildSheetGroups(many, emptyLineup('4-3-3'), 'DF')[0].players.map((p) => p.player_id)
    expect(ids).toEqual([6, 8, 5, 7])
  })
})

describe('positionIndex', () => {
  test('looks up positions of both current and former players', () => {
    const of = positionIndex(squad)
    expect(of(2)).toBe('DF')
    expect(of(21)).toBe('FW')
    expect(of(999)).toBeNull()
    expect(positionIndex(null)(1)).toBeNull()
  })
})

describe('sheet tabs (现役 / 前队员)', () => {
  const groups = buildSheetGroups(squad, emptyLineup('4-3-3'), 'DF')

  test('builds one tab per group with its head count', () => {
    expect(buildSheetTabs(groups)).toEqual([
      { key: 'active', label: '现役 4' },
      { key: 'former', label: '前队员 2' },
    ])
  })

  test('has no tab switcher when there is only one group', () => {
    const onlyActive = buildSheetGroups({ ...squad, former: [] }, emptyLineup('4-3-3'), 'DF')
    expect(buildSheetTabs(onlyActive)).toEqual([])
  })

  test('resolves the visible group: the chosen tab when it exists, otherwise the active group', () => {
    expect(resolveSheetTab(groups, 'former')?.key).toBe('former')
    expect(resolveSheetTab(groups, 'active')?.key).toBe('active')
    const onlyActive = buildSheetGroups({ ...squad, former: [] }, emptyLineup('4-3-3'), 'DF')
    expect(resolveSheetTab(onlyActive, 'former')?.key).toBe('active') // 没有前队员时不会停在不存在的那一页
  })

  test('no groups at all resolves to nothing', () => {
    expect(resolveSheetTab([], 'active')).toBeNull()
  })
})


describe('players from other teams', () => {
  const zeca: ClubPlayer = { ...player(900, 'FW'), name: '泽卡', team: { team_id: 136, team_name: '山东泰山' } }

  test('short team names drop the city or the trailing 队', () => {
    expect(['山东泰山', '天津津门虎', '浙江队', '深圳新鹏城', '成都'].map(teamShortName)).toEqual(['泰山', '津门虎', '浙江', '新鹏城', '成都'])
  })

  test('outsiders on the pitch carry their club badge; own players do not', () => {
    let slots = assignPlayer(emptyLineup('4-3-3'), 0, 1)
    slots = assignPlayer(slots, 10, 900)
    const pitch = buildPitchPlayers(slots, '4-3-3', squad, [zeca])
    expect(pitch[0].player?.club).toBe(null)
    expect(pitch[10].player).toMatchObject({ player_id: 900, name: '泽卡', club: '泰山', former: false })
    expect(positionIndex(squad, [zeca])(900)).toBe('FW')
  })

  test('search lists own matches first, then other teams without duplicating own players', () => {
    const local: SquadResponse = { ...squad, active: [...squad.active, { ...player(5, 'FW'), name: '王子铭' }], former: [{ ...player(20, 'DF', { last_season: 2025 }), name: '王老将' }] }
    const remote: ClubPlayer[] = [
      { ...player(5, 'FW'), name: '王子铭', team: { team_id: 1, team_name: '成都蓉城' } },
      { ...player(800, 'DF'), name: '王燊超', team: { team_id: 2, team_name: '上海海港' } },
      { ...player(801, 'FW'), name: '王钰栋', team: { team_id: 3, team_name: '浙江队' } },
    ]
    const groups = buildSearchGroups('王', local, emptyLineup('4-3-3'), 'FW', remote)
    expect(groups.map(g => [g.key, g.players.map(p => p.name)])).toEqual([
      ['team', ['王子铭', '王老将']],
      ['others', ['王钰栋', '王燊超']],
    ])
    expect(groups[1].players[0].club).toBe('浙江队')
    expect(buildSearchGroups('  ', local, emptyLineup('4-3-3'), 'FW', remote)).toEqual([])
  })

  test('shared lineups show outsiders with their badge', () => {
    const lineup = {
      formation: '4-3-3', team: squad.team,
      slots: assignPlayer(assignPlayer(emptyLineup('4-3-3'), 0, 1), 10, 900),
      players: [{ ...player(1, 'GK'), former: false }, { ...player(900, 'FW'), name: '泽卡', former: false, team: { team_id: 136, team_name: '山东泰山' } }],
    } as unknown as PublicLineup
    const pitch = publicPitch(lineup)
    expect(pitch[10].player?.club).toBe('泰山')
    expect(pitch[0].player?.club).toBe(null)
  })
})
