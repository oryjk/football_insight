<template>
  <view class="topic-tags">
    <button v-for="tag in tags" :key="tag.teamId" class="topic-tag"
      :class="{ 'topic-tag--selected': tag.teamId === selectedTeamId }" @tap="$emit('select', tag.teamId)">{{ tag.name }}</button>
  </view>
</template>
<script setup lang="ts">
import type { DailyTopicTag } from '../../types/dailyTopic'
defineProps<{ tags: DailyTopicTag[]; selectedTeamId?: number | null }>()
defineEmits<{ (event: 'select', teamId: number): void }>()
</script>
<style scoped>
.topic-tags { display: flex; flex-wrap: wrap; gap: var(--fi-space-12); }
.topic-tag { display: inline-flex; margin: 0; padding: var(--fi-space-8) var(--fi-space-16); background: var(--fi-color-bg); color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); border-radius: var(--fi-radius-round); border: var(--fi-border-divider); }
.topic-tag::after { border: 0; }
.topic-tag--selected { color: var(--fi-color-primary); border-color: var(--fi-color-primary); background: var(--fi-color-topic-audio-surface); }
</style>
