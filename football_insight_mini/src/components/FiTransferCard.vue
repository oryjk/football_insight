<template>
  <!-- 一行转会提名：名次、头像、名字与位置/现属球队、标签、赞踩（显示赞数与踩数）。洞察页与转会分包共用。 -->
  <view class="transfer-card" :class="`transfer-card--${size}`" hover-class="transfer-card--pressed" @tap="emit('open')">
    <text v-if="showRank && card.rank !== null" class="transfer-card__rank">{{ card.rank }}</text>
    <image v-if="card.player.avatar_url" class="transfer-card__avatar" :src="card.player.avatar_url" mode="aspectFill" lazy-load />
    <view v-else class="transfer-card__avatar transfer-card__avatar--fallback"><text>{{ card.player.name.slice(0, 1) }}</text></view>
    <view class="transfer-card__body">
      <view class="transfer-card__title">
        <text class="transfer-card__name">{{ card.player.name }}</text>
        <text v-if="card.returning" class="transfer-card__tag transfer-card__tag--accent">希望回归</text>
        <text v-if="card.player.departed" class="transfer-card__tag">已离队</text>
      </view>
      <text class="transfer-card__meta">{{ meta }}</text>
    </view>
    <!-- 胶囊外围一圈安全区：点在胶囊附近的空白处不会误开单卡页。 -->
    <view class="transfer-card__reaction" @tap.stop>
      <FiReactionBar :size="size" :count="card.up" :dislike-count="card.down" :liked="card.my_vote === 1" :disliked="card.my_vote === -1"
        :pending="pending" @like="emit('vote', 1)" @dislike="emit('vote', -1)" />
    </view>
  </view>
</template>
<script setup lang="ts">
import { computed } from 'vue'
import FiReactionBar from './FiReactionBar.vue'
import type { TransferCard } from '../types/transfer'
import { positionLabel } from '../utils/transferLinks'
const props = withDefaults(defineProps<{ card: TransferCard; pending: boolean; showRank?: boolean; size?: 'sm' | 'md' }>(), { showRank: true, size: 'sm' })
const emit = defineEmits<{ (e: 'open'): void; (e: 'vote', value: 1 | -1): void }>()
const meta = computed(() => [positionLabel(props.card.player.position), props.card.player.current_team?.team_name ?? '暂无球队'].filter(Boolean).join(' · '))
</script>
<style scoped>
.transfer-card { display: flex; align-items: center; gap: var(--fi-space-16); padding: var(--fi-space-16) 0; }
.transfer-card--pressed { opacity: var(--fi-opacity-pressed); }
.transfer-card__rank { flex-shrink: 0; width: var(--fi-space-32); color: var(--fi-color-primary); font-size: var(--fi-font-26); font-weight: var(--fi-weight-extrabold); text-align: center; }
.transfer-card__avatar { flex-shrink: 0; width: var(--fi-space-64); height: var(--fi-space-64); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); }
.transfer-card--sm .transfer-card__avatar { width: var(--fi-space-48); height: var(--fi-space-48); }
.transfer-card__avatar--fallback { display: flex; align-items: center; justify-content: center; color: var(--fi-color-text-secondary); font-size: var(--fi-font-24); font-weight: var(--fi-weight-bold); }
.transfer-card__body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: var(--fi-space-4); }
.transfer-card__title { display: flex; align-items: center; gap: var(--fi-space-8); min-width: 0; }
.transfer-card__name { min-width: 0; overflow: hidden; color: var(--fi-color-text-strong); font-size: var(--fi-font-26); font-weight: var(--fi-weight-bold); text-overflow: ellipsis; white-space: nowrap; }
.transfer-card--md .transfer-card__name { font-size: var(--fi-font-30); }
.transfer-card__tag { flex-shrink: 0; padding: var(--fi-space-2) var(--fi-space-10); border-radius: var(--fi-radius-round); background: var(--fi-color-page-soft); color: var(--fi-color-text-secondary); font-size: var(--fi-font-20); font-weight: var(--fi-weight-bold); }
.transfer-card__tag--accent { background: var(--fi-color-reaction-like-bg); color: var(--fi-color-primary); }
.transfer-card__meta { overflow: hidden; color: var(--fi-color-text-muted); font-size: var(--fi-font-22); text-overflow: ellipsis; white-space: nowrap; }
.transfer-card__reaction { flex-shrink: 0; align-self: stretch; display: flex; align-items: center; padding-left: var(--fi-space-16); }
</style>
