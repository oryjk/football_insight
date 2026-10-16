<template>
  <FiPanel>
    <view class="team-panel">
      <InsightsTeamSelector
        :teams="teams"
        :selected-team-id="selectedTeamId"
        @update:selected-team-id="emit('update:selectedTeamId', $event)"
      />
      <InsightsSummaryPanel :insight="insight" @open-board="emit('open-board')" />
    </view>
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import type { TeamInsight, TeamInsightTeam } from '../../../types/insight'
import InsightsTeamSelector from './InsightsTeamSelector.vue'
import InsightsSummaryPanel from './InsightsSummaryPanel.vue'

defineProps<{
  teams: TeamInsightTeam[]
  selectedTeamId: number | null
  insight: TeamInsight
}>()

const emit = defineEmits<{
  (e: 'update:selectedTeamId', value: number): void
  (e: 'open-board'): void
}>()
</script>

<style scoped lang="css">
.team-panel {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: var(--fi-space-22);
  min-width: 0;
}
</style>
