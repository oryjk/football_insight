<template>
  <view class="topic-heat" :class="{ 'topic-heat--hero': variant === 'hero' }">
    <view v-if="variant === 'hero'" class="topic-heat__track" aria-hidden="true">
      <view class="topic-heat__fill" :style="{ width: fillWidth }" />
    </view>
    <image v-else class="topic-heat__icon" :src="fireIcon" mode="aspectFit" />
    <text class="topic-heat__label">热度 {{ formatTopicHeat(value) }}</text>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import fireIcon from '../../static/ticket-watch/fire.svg'
import { formatTopicHeat } from '../../utils/dailyTopics'
const props = withDefaults(defineProps<{ value: number; variant?: 'default' | 'hero' }>(), { variant: 'default' })
// 仅表示热度强弱，读数仍使用原始计数；超过 100 时填满装饰条。
const fillWidth = computed(() => `${Math.min(100, Number.isSafeInteger(props.value) && props.value >= 0 ? props.value : 0)}%`)
</script>
<style scoped>
.topic-heat { display: inline-flex; align-items: center; gap: var(--fi-space-6); color: var(--fi-color-text-secondary); font-size: var(--fi-font-22); line-height: var(--fi-leading-normal); white-space: nowrap; }
.topic-heat__icon { flex-shrink: 0; width: var(--fi-space-24); height: var(--fi-space-24); }
.topic-heat--hero { display: flex; width: 100%; min-width: 0; gap: var(--fi-space-16); color: var(--fi-color-topic-hero-muted); }
.topic-heat__track { flex: 1; min-width: var(--fi-space-16); height: var(--fi-space-8); border-radius: var(--fi-radius-round); background: var(--fi-color-topic-hero-track); overflow: hidden; }
.topic-heat__fill { height: 100%; border-radius: inherit; background: var(--fi-gradient-topic-heat); }
.topic-heat__label { flex-shrink: 0; }
</style>
