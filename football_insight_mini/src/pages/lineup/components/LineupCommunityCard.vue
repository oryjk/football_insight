<template>
  <view class="lineup-card" @tap="emit('open')">
    <view class="card-row">
      <text v-if="rank" class="rank">#{{ rank }}</text>
      <image v-if="item.author.avatar_url" :src="item.author.avatar_url" class="avatar" mode="aspectFill" />
      <view v-else class="avatar avatar--fallback">{{ item.author.display_name.charAt(0) }}</view>
      <view class="identity"><text class="name">{{ item.author.display_name }}</text><text class="date">{{ createdLabel }}</text></view>
      <text class="formation">{{ item.formation }}</text>
    </view>
    <view class="card-row card-row--footer">
      <text class="caption">查看完整 11 人阵容 ›</text>
      <button class="like" :class="{ 'like--active': item.liked_by_me }" :disabled="pending" @tap.stop="emit('like')">{{ pending ? '处理中…' : item.liked_by_me ? '♥ 已赞' : '♡ 点赞' }} {{ item.likes_count }}</button>
    </view>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import type { PublicLineupSummary } from '../../../types/lineup'
import { lineupDateLabel } from '../lineupPresentation'
const props = defineProps<{ item: PublicLineupSummary; rank?: number; pending: boolean }>()
const emit = defineEmits<{ (e: 'open'): void; (e: 'like'): void }>()
const createdLabel = computed(() => lineupDateLabel(props.item.created_at))
</script>
<style scoped>
.lineup-card { padding: var(--fi-space-24); border: var(--fi-border-card); border-radius: var(--fi-radius-md); background: var(--fi-color-surface); box-shadow: var(--fi-shadow-soft); }
.card-row { display: flex; align-items: center; gap: var(--fi-space-16); }
.card-row--footer { justify-content: space-between; margin-top: var(--fi-space-24); }
.rank, .formation { color: var(--fi-color-primary); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }
.avatar { width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-round); flex-shrink: 0; }
.avatar--fallback { display: flex; align-items: center; justify-content: center; background: var(--fi-color-page-soft); color: var(--fi-color-primary); }
.identity { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: var(--fi-space-8); }
.name { color: var(--fi-color-text-strong); font-size: var(--fi-font-28); }
.date, .caption { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); }
.like { margin: 0; padding: var(--fi-space-12) var(--fi-space-20); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); line-height: var(--fi-leading-normal); }
.like::after { border: 0; }
.like--active { color: var(--fi-color-primary); }
</style>
