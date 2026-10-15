<template>
  <view class="player-goals-table">
    <view class="ranking-caption">
      <text>近 10 次出场进球 · 由旧到新</text>
      <text>累计</text>
    </view>
    <view class="ranking-list">
      <view
        v-for="row in players"
        :key="row.player_id"
        class="ranking-row"
      >
        <text class="ranking-row__rank" :class="`ranking-row__rank--${row.rank_no}`">#{{ row.rank_no }}</text>
        <image
          :src="row.avatar_storage_url || ''"
          mode="aspectFill"
          class="ranking-row__avatar"
          :role="playerDetails ? 'button' : undefined"
          :aria-label="playerDetails ? `查看${row.player_name}的球员信息` : undefined"
          @click.stop="playerDetails && emit('open-player', row.player_id)"
        />
        <view class="ranking-row__body">
          <text class="ranking-row__name">{{ row.player_name }}</text>
          <text class="ranking-row__note">{{ row.team_name }}</text>
        </view>
        <view class="ranking-row__chart">
          <FiPlayerGoalBars
            :player-name="row.player_name"
            :appearances="row.recent_goals"
            :scale-max="scaleMax"
            @select="emit('select-match', row.player_name, $event)"
          />
        </view>
        <view class="ranking-row__metric">
          <text class="ranking-row__metric-value">{{ row.score_value }}</text>
          <text class="ranking-row__metric-note">累计进球</text>
        </view>
      </view>
    </view>
    <view class="ranking-legend">
      <text class="ranking-legend__goals">▮</text><text>进球</text>
      <text class="ranking-legend__zero">—</text><text>0 球</text>
      <text class="ranking-legend__hat">帽</text><text>帽子戏法</text>
      <text class="ranking-legend__hint">点击柱子看单场</text>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import FiPlayerGoalBars from './FiPlayerGoalBars.vue'
import type { PlayerMatchGoals } from '../types/insight'
import { goalChartScale, type PlayerGoalEntry } from '../utils/playerGoals'

const props = defineProps<{ players: PlayerGoalEntry[]; playerDetails?: boolean }>()
const scaleMax = computed(() => goalChartScale(props.players.map(row => row.recent_goals)))
const emit = defineEmits<{
  (event: 'open-player', playerId: number): void
  (event: 'select-match', playerName: string, match: PlayerMatchGoals): void
}>()
</script>

<style scoped lang="css">
.ranking-list {
  margin-top: var(--fi-space-12);
  display: grid;
  gap: var(--fi-space-10);
}

.ranking-caption {
  display: flex;
  justify-content: space-between;
  gap: var(--fi-space-8);
  margin-top: var(--fi-space-16);
  color: var(--fi-color-text-caption);
  font-size: var(--fi-font-20);
}

.ranking-row {
  display: grid;
  grid-template-columns: var(--fi-space-32) var(--fi-space-44) minmax(0, calc(var(--fi-font-28) * 4.5)) minmax(0, 1fr) max-content;
  align-items: center;
  column-gap: var(--fi-space-8);
  padding: var(--fi-space-8) 0;
}

.ranking-row__chart {
  min-width: 0;
}

.ranking-row__rank {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.ranking-row__rank--1 {
  color: var(--fi-color-primary);
}

.ranking-row__avatar {
  width: var(--fi-space-44);
  height: var(--fi-space-44);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
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

.ranking-legend {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--fi-space-6);
  margin-top: var(--fi-space-12);
  padding-top: var(--fi-space-12);
  border-top: var(--fi-border-width) solid var(--fi-color-divider-soft);
  font-size: var(--fi-font-20);
  color: var(--fi-color-text-caption);
}
.ranking-legend__goals { color: var(--fi-color-primary); }
.ranking-legend__zero { color: var(--fi-color-team-form-line); }
.ranking-legend__hat { color: var(--fi-color-hat-trick-text); }
.ranking-legend__hint { margin-left: auto; }
</style>
