import { ref } from 'vue'
import { getLineupConsensus } from '../api/lineup'
import type { LineupConsensus } from '../types/lineup'

/** 按球队加载共识阵容：换队后迟到的旧响应丢弃；同队刷新失败保留上次结果（共识是增强信息，不报错）。 */
export function createLineupConsensus(read: (teamId: number) => Promise<LineupConsensus> = getLineupConsensus) {
  const consensus = ref<LineupConsensus | null>(null)
  let generation = 0, current = 0
  async function load(teamId: number): Promise<void> {
    const owner = ++generation
    if (teamId !== current) consensus.value = null
    current = teamId
    if (!teamId) return
    try {
      const result = await read(teamId)
      if (owner === generation) consensus.value = result
    } catch {
      /* 保留同队上次结果 */
    }
  }
  function reset(): void { generation++; current = 0; consensus.value = null }
  return { consensus, load, reset }
}
