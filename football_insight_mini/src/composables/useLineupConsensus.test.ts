import { describe, expect, test } from 'bun:test'
import type { LineupConsensus } from '../types/lineup'
import { createLineupConsensus } from './useLineupConsensus'

const sample = (teamId: number, total = 3): LineupConsensus => ({
  team: { team_id: teamId, team_name: `T${teamId}`, avatar_url: null },
  total, threshold: 10, unlocked: false, formation: null, formation_share: 0, players: [],
})

describe('lineup consensus loader', () => {
  test('stale_team_responses_are_dropped', async () => {
    let releaseFirst: (value: LineupConsensus) => void = () => {}
    const loader = createLineupConsensus(id => id === 1 ? new Promise(resolve => { releaseFirst = resolve }) : Promise.resolve(sample(2)))
    const first = loader.load(1)
    await loader.load(2)
    releaseFirst(sample(1)); await first
    expect(loader.consensus.value?.team.team_id).toBe(2)
  })

  test('a failed refresh keeps the same team result; switching teams clears it', async () => {
    let fail = false
    const loader = createLineupConsensus(async id => { if (fail) throw new Error('offline'); return sample(id, 5) })
    await loader.load(1)
    fail = true
    await loader.load(1)
    expect(loader.consensus.value?.total).toBe(5)
    await loader.load(2)
    expect(loader.consensus.value).toBe(null)
    loader.reset()
    expect(loader.consensus.value).toBe(null)
  })
})
