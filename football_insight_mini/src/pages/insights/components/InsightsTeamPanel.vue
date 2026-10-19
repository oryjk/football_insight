<template>
  <FiPanel>
    <InsightsSummaryPanel :insight="insight" />
  </FiPanel>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import type { TeamInsight } from '../../../types/insight'
import InsightsSummaryPanel from './InsightsSummaryPanel.vue'

defineProps<{
  insight: TeamInsight
}>()

</script>
