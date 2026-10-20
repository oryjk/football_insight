<template>
  <view class="lineup-card" :class="{ 'lineup-card--compact': compact }" @tap="emit('open')">
    <view class="card-row">
      <text v-if="rank && !compact" class="rank">#{{ rank }}</text>
      <image v-if="item.author.avatar_url" :src="item.author.avatar_url" class="avatar" mode="aspectFill" />
      <view v-else class="avatar avatar--fallback">{{ item.author.display_name.charAt(0) }}</view>
      <view class="identity"><text class="name">{{ item.author.display_name }}</text><text class="date">{{ createdLabel }}</text></view>
      <text class="formation">{{ item.formation }}</text>
      <view v-if="compact" class="reaction-slot"><FiReactionBar :count="item.likes_count" :liked="item.liked_by_me" :disliked="item.disliked_by_me" :pending="pending" @like="emit('like')" @dislike="emit('dislike')" /></view>
    </view>
    <view v-if="!compact" class="card-row card-row--footer">
      <text class="caption">查看完整 11 人阵容 ›</text>
      <view class="reaction-slot"><FiReactionBar :count="item.likes_count" :liked="item.liked_by_me" :disliked="item.disliked_by_me" :pending="pending" @like="emit('like')" @dislike="emit('dislike')" /></view>
    </view>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import FiReactionBar from './FiReactionBar.vue'
import type { PublicLineupSummary } from '../types/lineup'
import { lineupDateLabel } from '../utils/lineupPresentation'
const props = withDefaults(defineProps<{ item: PublicLineupSummary; rank?: number; pending: boolean; compact?: boolean }>(), { compact: false })
const emit = defineEmits<{ (e: 'open' | 'like' | 'dislike'): void }>()
const createdLabel = computed(() => lineupDateLabel(props.item.created_at))
</script>
<style scoped>
.lineup-card { padding: var(--fi-space-24); border: var(--fi-border-card); border-radius: var(--fi-radius-md); background: var(--fi-color-surface); box-shadow: var(--fi-shadow-soft); }
.card-row { display: flex; align-items: center; gap: var(--fi-space-16); }
.reaction-slot { flex-shrink: 0; }
.card-row--footer { justify-content: space-between; margin-top: var(--fi-space-24); }
.rank, .formation { color: var(--fi-color-primary); font-size: var(--fi-font-28); font-weight: var(--fi-weight-bold); }
.avatar { width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-round); flex-shrink: 0; }
.avatar--fallback { display: flex; align-items: center; justify-content: center; background: var(--fi-color-page-soft); color: var(--fi-color-primary); }
.identity { display: flex; flex: 1; min-width: 0; flex-direction: column; gap: var(--fi-space-8); }
.name { color: var(--fi-color-text-strong); font-size: var(--fi-font-28); }
.date, .caption { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); }
.lineup-card--compact { padding: var(--fi-space-16) 0; border: 0; border-radius: 0; background: transparent; box-shadow: none; }
.lineup-card--compact .avatar { width: var(--fi-space-48); height: var(--fi-space-48); }
.lineup-card--compact .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: var(--fi-font-26); }
.lineup-card--compact .date { font-size: var(--fi-font-20); }
.lineup-card--compact .identity { gap: var(--fi-space-4); }
.lineup-card--compact .formation { flex-shrink: 0; font-size: var(--fi-font-26); }
</style>
