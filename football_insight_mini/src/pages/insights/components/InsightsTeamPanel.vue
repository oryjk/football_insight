<template>
  <FiPanel>
    <InsightsSummaryPanel :insight="insight" @open-board="emit('open-board')" />
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import type { TeamInsight } from '../../../types/insight'
import InsightsSummaryPanel from './InsightsSummaryPanel.vue'

defineProps<{
  insight: TeamInsight
}>()

const emit = defineEmits<{
  (e: 'open-board'): void
}>()
</script>
