<template>
  <HomeSheetShell animated kicker="比赛技术统计" title="技术统计" @close="emit('close')">
    <view class="tech-stats-body">
      <view class="tech-stats-sheet__summary">
        <text class="tech-stats-sheet__teams">
          {{ match.home_team_name }} {{ match.home_score }} : {{ match.away_score }} {{ match.away_team_name }}
        </text>
        <text class="tech-stats-sheet__meta">
          第 {{ match.round_number }} 轮 · {{ match.match_date }} {{ match.match_time }}
        </text>
      </view>

      <view class="tech-stats-sheet__list">
        <view
          v-for="(stat, index) in techStats"
          :key="stat.key"
          class="tech-stat-row"
          :style="{ '--tech-stat-delay': `${120 + index * 70}ms` }"
        >
          <text class="tech-stat-row__value">{{ stat.homeValue }}</text>
          <view class="tech-stat-row__track tech-stat-row__track--home">
            <view class="tech-stat-row__fill tech-stat-row__fill--home" :style="{ width: `${stat.homeBarPercent}%` }" />
          </view>
          <text class="tech-stat-row__label">{{ stat.label }}</text>
          <view class="tech-stat-row__track tech-stat-row__track--away">
            <view class="tech-stat-row__fill tech-stat-row__fill--away" :style="{ width: `${stat.awayBarPercent}%` }" />
          </view>
          <text class="tech-stat-row__value tech-stat-row__value--away">{{ stat.awayValue }}</text>
        </view>
      </view>
    </view>
  </HomeSheetShell>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import HomeSheetShell from './HomeSheetShell.vue'
import { resolveHomePulseTechStats, type HomePulseLeadMatch } from '../helpers'

const props = defineProps<{
  match: HomePulseLeadMatch
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const techStats = computed(() => resolveHomePulseTechStats(props.match))
</script>

<style scoped lang="css">
.tech-stats-body {
  max-height: 72vh;
  padding-bottom: calc(40rpx + env(safe-area-inset-bottom) + 100rpx);
}

.tech-stats-sheet__summary {
  position: sticky;
  top: 0;
  z-index: 3;
  margin-top: 18rpx;
  padding: 22rpx 24rpx;
  border-radius: 24rpx;
  border: 2rpx solid rgba(232, 233, 238, 0.95);
  background: var(--fi-primitive-white);
  box-shadow: 0 12rpx 28rpx rgba(18, 20, 28, 0.06);
}

.tech-stats-sheet__teams {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: 30rpx;
  line-height: 1.35;
  font-weight: 800;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tech-stats-sheet__meta {
  display: block;
  margin-top: 10rpx;
  color: var(--fi-color-text-muted);
  font-size: 24rpx;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tech-stats-sheet__list {
  margin-top: 24rpx;
  display: grid;
  gap: 18rpx;
}

.tech-stat-row {
  display: grid;
  grid-template-columns: 44rpx minmax(0, 1fr) auto minmax(0, 1fr) 44rpx;
  align-items: center;
  gap: 16rpx;
  padding: 18rpx 0;
  border-top: 2rpx solid #f0f1f5;
  opacity: 0;
  transform: translateY(14rpx);
  animation: tech-stat-row-enter 320ms cubic-bezier(0.24, 0.88, 0.28, 1) both;
  animation-delay: var(--tech-stat-delay, 120ms);
}

.tech-stat-row:first-child {
  border-top: none;
}

.tech-stat-row__value {
  color: var(--fi-color-text-strong);
  font-size: 28rpx;
  font-weight: 800;
  text-align: left;
}

.tech-stat-row__value--away {
  text-align: right;
}

.tech-stat-row__label {
  min-width: 84rpx;
  color: #2a2c31;
  font-size: 30rpx;
  font-weight: 800;
  text-align: center;
}

.tech-stat-row__track {
  height: 14rpx;
  border-radius: 999rpx;
  background: #eceef3;
  overflow: hidden;
  display: flex;
  align-items: center;
}

.tech-stat-row__track--home {
  justify-content: flex-end;
}

.tech-stat-row__fill {
  height: 100%;
  border-radius: 999rpx;
  background: var(--fi-primitive-ink);
  transform: scaleX(0);
  animation: tech-stat-fill-grow 480ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
  animation-delay: calc(var(--tech-stat-delay, 120ms) + 70ms);
}

.tech-stat-row__fill--home {
  background: var(--fi-color-primary);
  transform-origin: right center;
}

.tech-stat-row__fill--away {
  background: var(--fi-color-primary);
  transform-origin: left center;
}

@keyframes tech-stat-row-enter {
  from {
    opacity: 0;
    transform: translateY(14rpx);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes tech-stat-fill-grow {
  from {
    transform: scaleX(0);
  }
  to {
    transform: scaleX(1);
  }
}
</style>
