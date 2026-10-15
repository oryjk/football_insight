<template>
  <HomeSheetShell animated kicker="MATCH CENTER · 技术统计" @close="emit('close')">
    <view class="tech-stats-body">
      <view class="tech-board">
        <view class="tech-board__stripe" />
        <text class="tech-board__round">第 {{ match.round_number }} 轮 · 中超</text>
        <view class="tech-board__score-row">
          <text class="tech-board__team">{{ match.home_team_name }}</text>
          <view class="tech-board__score">
            <text class="tech-board__num">{{ match.home_score }}</text>
            <text class="tech-board__colon">:</text>
            <text class="tech-board__num">{{ match.away_score }}</text>
          </view>
          <text class="tech-board__team">{{ match.away_team_name }}</text>
        </view>
        <view class="tech-board__foot">
          <text class="tech-board__ft">FT</text>
          <text class="tech-board__meta">{{ match.match_date }} {{ match.match_time }}</text>
        </view>
      </view>

      <text v-if="!techStats.length" class="tech-stats-sheet__empty">这场比赛暂时还没有技术统计</text>

      <view v-else class="tech-stats-sheet__list">
        <view
          v-for="(stat, index) in techStats"
          :key="stat.key"
          class="tech-stat-row"
          :style="{ '--tech-stat-delay': `${120 + index * 70}ms` }"
        >
          <text class="tech-stat-row__label">{{ stat.label }}</text>
          <view class="tech-stat-row__line">
            <text class="tech-stat-row__value" :class="{ 'tech-stat-row__value--lead': stat.homeBarPercent > stat.awayBarPercent }">{{ stat.homeValue }}</text>
            <view class="tech-stat-row__track tech-stat-row__track--home">
              <view
                class="tech-stat-row__fill tech-stat-row__fill--home"
                :class="{ 'tech-stat-row__fill--dim': stat.homeBarPercent < stat.awayBarPercent }"
                :style="{ width: `${stat.homeBarPercent}%` }"
              />
            </view>
            <view class="tech-stat-row__track tech-stat-row__track--away">
              <view
                class="tech-stat-row__fill tech-stat-row__fill--away"
                :class="{ 'tech-stat-row__fill--dim': stat.awayBarPercent < stat.homeBarPercent }"
                :style="{ width: `${stat.awayBarPercent}%` }"
              />
            </view>
            <text class="tech-stat-row__value tech-stat-row__value--away" :class="{ 'tech-stat-row__value--lead': stat.awayBarPercent > stat.homeBarPercent }">{{ stat.awayValue }}</text>
          </view>
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

.tech-board {
  position: sticky;
  top: 0;
  z-index: 3;
  margin-top: var(--fi-space-20);
  overflow: hidden;
  border-radius: var(--fi-radius-lg);
  background: var(--fi-gradient-topic-hero);
  border: 1rpx solid var(--fi-color-match-card-hairline);
  box-shadow: var(--fi-shadow-match-card);
  color: var(--fi-color-topic-hero-text);
  text-align: center;
}

.tech-board__stripe {
  height: 6rpx;
  background: linear-gradient(90deg, var(--fi-color-primary-deep), var(--fi-color-primary), var(--fi-color-primary-deep));
}

.tech-board__round {
  display: block;
  margin-top: var(--fi-space-24);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-extrabold);
  letter-spacing: 4rpx;
}

.tech-board__score-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fi-space-16) var(--fi-space-24) 0;
}

.tech-board__team {
  flex: 1;
  min-width: 0;
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-snug);
  font-weight: var(--fi-weight-bold);
  overflow-wrap: anywhere;
}

.tech-board__score {
  flex: none;
  display: flex;
  align-items: center;
  padding: 0 var(--fi-space-16);
}

.tech-board__num {
  min-width: 64rpx;
  font-size: 88rpx;
  line-height: 1.1;
  font-weight: var(--fi-weight-black);
  font-style: italic;
}

.tech-board__colon {
  padding: 0 var(--fi-space-4);
  color: var(--fi-color-topic-hero-caption);
  font-size: 56rpx;
  font-weight: var(--fi-weight-bold);
}

.tech-board__foot {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-12);
  padding: var(--fi-space-8) 0 var(--fi-space-24);
}

.tech-board__ft {
  padding: 2rpx var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-topic-hero-control);
  color: var(--fi-color-topic-hero-muted);
  font-size: var(--fi-font-20);
  font-weight: var(--fi-weight-extrabold);
  letter-spacing: 2rpx;
}

.tech-board__meta {
  color: var(--fi-color-topic-hero-caption);
  font-size: var(--fi-font-24);
}

.tech-stats-sheet__empty {
  display: block;
  margin-top: var(--fi-space-32);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-26);
  text-align: center;
}

.tech-stats-sheet__list {
  margin-top: var(--fi-space-16);
}

.tech-stat-row {
  padding: var(--fi-space-20) var(--fi-space-8);
  border-top: 1rpx solid var(--fi-color-divider-soft);
  opacity: 0;
  transform: translateY(14rpx);
  animation: tech-stat-row-enter 320ms cubic-bezier(0.24, 0.88, 0.28, 1) both;
  animation-delay: var(--tech-stat-delay, 120ms);
}

.tech-stat-row:first-child {
  border-top: none;
}

.tech-stat-row__label {
  display: block;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 4rpx;
  text-align: center;
}

.tech-stat-row__line {
  display: flex;
  align-items: center;
  gap: var(--fi-space-8);
  margin-top: var(--fi-space-12);
}

.tech-stat-row__value {
  flex: none;
  width: 88rpx;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-bold);
  text-align: left;
}

.tech-stat-row__value--away {
  text-align: right;
}

.tech-stat-row__value--lead {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  font-weight: var(--fi-weight-black);
}

.tech-stat-row__track {
  flex: 1;
  height: 12rpx;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-divider-soft);
  overflow: hidden;
  display: flex;
  align-items: center;
}

.tech-stat-row__track--home {
  justify-content: flex-end;
}

.tech-stat-row__fill {
  height: 100%;
  border-radius: var(--fi-radius-round);
  transform: scaleX(0);
  animation: tech-stat-fill-grow 480ms cubic-bezier(0.22, 1, 0.36, 1) forwards;
  animation-delay: calc(var(--tech-stat-delay, 120ms) + 70ms);
}

.tech-stat-row__fill--home {
  background: var(--fi-color-primary);
  transform-origin: right center;
}

.tech-stat-row__fill--away {
  background: var(--fi-color-topic-hero-bg);
  transform-origin: left center;
}

.tech-stat-row__fill--dim {
  opacity: 0.35;
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
