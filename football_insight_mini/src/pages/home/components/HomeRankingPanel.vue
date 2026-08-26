<template>
  <HomePanelCard :kicker="kicker" :title="title">
    <view class="ranking-list">
      <view
        v-for="row in rows"
        :key="row.key"
        class="ranking-row"
        :class="{ 'ranking-row--interactive': row.interactive }"
        :hover-class="row.interactive ? 'ranking-row--pressed' : undefined"
        hover-stay-time="100"
        @click="row.interactive && emit('select', row)"
      >
        <text class="ranking-row__rank" :class="`ranking-row__rank--${row.rankNo}`">#{{ row.rankNo }}</text>
        <image
          :src="row.avatar || ''"
          mode="aspectFit"
          class="ranking-row__avatar"
          :class="{ 'ranking-row__avatar--player': row.avatarMode === 'fill' }"
        />
        <view class="ranking-row__body">
          <text class="ranking-row__name">{{ row.name }}</text>
          <text class="ranking-row__note">{{ row.note }}</text>
        </view>
        <view class="ranking-row__metric">
          <text class="ranking-row__metric-value">{{ row.metricValue }}</text>
          <text class="ranking-row__metric-note">{{ row.metricNote }}</text>
        </view>
      </view>
    </view>
  </HomePanelCard>
</template>

<script setup lang="ts">
import HomePanelCard from './HomePanelCard.vue'

interface HomeRankingRow {
  key: string | number
  rankNo: number
  avatar: string | null
  name: string
  note: string
  metricValue: string | number
  metricNote: string
  interactive?: boolean
  avatarMode?: 'fit' | 'fill'
}

defineProps<{
  kicker: string
  title: string
  rows: HomeRankingRow[]
}>()

const emit = defineEmits<{
  (e: 'select', row: HomeRankingRow): void
}>()
</script>

<style scoped lang="css">
.ranking-list {
  margin-top: 12rpx;
  display: grid;
  gap: 10rpx;
}

.ranking-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16rpx;
}

.ranking-row--interactive {
  padding: 10rpx 8rpx;
  border-radius: 24rpx;
  transition: transform 180ms ease, background-color 180ms ease;
}

.ranking-row--pressed {
  transform: scale(0.992);
  background: rgba(var(--fi-primitive-red-rgb), 0.08);
}

.ranking-row__rank {
  width: 72rpx;
  color: var(--fi-color-text-muted);
  font-size: 24rpx;
  font-weight: 700;
}

.ranking-row__rank--1 {
  color: var(--fi-color-primary);
}

.ranking-row__rank--2 {
  color: #2563eb;
}

.ranking-row__rank--3 {
  color: #16a34a;
}

.ranking-row__avatar {
  width: 68rpx;
  height: 68rpx;
  border-radius: 999rpx;
  background: #f5f6fa;
}

.ranking-row__avatar--player {
  object-fit: cover;
}

.ranking-row__body,
.ranking-row__metric {
  display: grid;
}

.ranking-row__body {
  flex: 1;
  min-width: 0;
}

.ranking-row__metric {
  justify-items: end;
}

.ranking-row__name {
  color: var(--fi-color-text-strong);
  font-size: 30rpx;
  font-weight: 700;
}

.ranking-row__note,
.ranking-row__metric-note {
  color: var(--fi-color-text-muted);
  font-size: 22rpx;
}

.ranking-row__metric-value {
  color: var(--fi-color-text-strong);
  font-size: 36rpx;
  font-weight: 800;
}
</style>
