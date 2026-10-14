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
  margin-top: var(--fi-space-12);
  display: grid;
  gap: var(--fi-space-10);
}

.ranking-row {
  display: grid;
  grid-template-columns: var(--fi-space-32) var(--fi-space-44) minmax(0, 1fr) max-content;
  align-items: center;
  column-gap: var(--fi-space-8);
  padding: var(--fi-space-8) 0;
}

.ranking-row--interactive {
  border-radius: var(--fi-radius-md);
  transition: transform 180ms ease, background-color 180ms ease;
}

.ranking-row--pressed {
  transform: scale(0.992);
  background: var(--fi-color-ranking-pressed);
}

.ranking-row__rank {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.ranking-row__rank--1 {
  color: var(--fi-color-primary);
}

.ranking-row__rank--2 {
  color: var(--fi-color-ranking-second);
}

.ranking-row__rank--3 {
  color: var(--fi-color-ranking-third);
}

.ranking-row__avatar {
  width: var(--fi-space-44);
  height: var(--fi-space-44);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
}

.ranking-row__avatar--player {
  object-fit: cover;
}

.ranking-row__body,
.ranking-row__metric {
  display: grid;
}

.ranking-row__body {
  min-width: 0;
}

.ranking-row__metric {
  justify-items: end;
}

.ranking-row__name {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-bold);
}

.ranking-row__note,
.ranking-row__metric-note {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
}

.ranking-row__name,
.ranking-row__note {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ranking-row__metric-value {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-extrabold);
}
</style>
