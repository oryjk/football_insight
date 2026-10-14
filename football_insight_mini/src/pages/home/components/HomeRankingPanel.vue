<template>
  <HomePanelCard :kicker="kicker" :title="title">
    <FiPlayerGoalsTable :players="players" @select-match="(name, match) => emit('select-match', name, match)" />
  </HomePanelCard>
</template>

<script setup lang="ts">
import HomePanelCard from './HomePanelCard.vue'
import FiPlayerGoalsTable from '../../../components/FiPlayerGoalsTable.vue'
import type { PlayerGoalEntry } from '../../../utils/playerGoals'
import type { PlayerMatchGoals } from '../../../types/insight'

defineProps<{ kicker: string; title: string; players: PlayerGoalEntry[] }>()
const emit = defineEmits<{
  (event: 'select-match', playerName: string, match: PlayerMatchGoals): void
}>()
</script>
