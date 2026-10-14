<template>
  <FiPanel>
    <FiSectionHeading kicker="积分榜头部" title="先看联赛头部格局" />
    <FiStandingsTable :teams="teams" @select="emit('select', $event)" />
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import FiSectionHeading from '../../../components/FiSectionHeading.vue'
import FiStandingsTable from '../../../components/FiStandingsTable.vue'
import type { OverviewStanding } from '../../../types/insight'

defineProps<{ teams: OverviewStanding[] }>()
const emit = defineEmits<{ (event: 'select', row: { key: number }): void }>()
</script>
