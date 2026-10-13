<template>
  <view class="topic-heat"><image class="topic-heat__icon" :src="fireIcon" mode="aspectFit" /><text>热度 {{ formatTopicHeat(value) }}</text></view>
</template>
<script setup lang="ts">
import fireIcon from '../../static/ticket-watch/fire.svg'
import { formatTopicHeat } from '../../utils/dailyTopics'
defineProps<{ value: number }>()
</script>
<style scoped>
.topic-heat { display: inline-flex; align-items: center; gap: var(--fi-space-6); color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); white-space: nowrap; }
.topic-heat__icon { flex-shrink: 0; width: var(--fi-space-24); height: var(--fi-space-24); }
</style>
