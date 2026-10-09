<template>
<view
        class="sheet-mask sheet-mask--tech-stats"
        @click.self="emit('close')"
      >
        <view class="sheet-card tech-stats-sheet">
          <view class="section-heading section-heading--compact">
            <view>
              <text class="section-kicker">比赛技术统计</text>
              <text class="section-title">技术统计</text>
            </view>
            <button class="tech-stats-sheet__close" @click="emit('close')">关闭</button>
          </view>

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
              v-for="(stat, index) in rows"
              :key="stat.key"
              class="tech-stat-row"
              :style="getTechStatRowStyle(index)"
            >
              <text class="tech-stat-row__value">{{ stat.homeValue }}</text>
              <view class="tech-stat-row__track tech-stat-row__track--home">
                <view
                  class="tech-stat-row__fill tech-stat-row__fill--home"
                  :style="{ width: `${stat.homeBarPercent}%` }"
                />
              </view>
              <text class="tech-stat-row__label">{{ stat.label }}</text>
              <view class="tech-stat-row__track tech-stat-row__track--away">
                <view
                  class="tech-stat-row__fill tech-stat-row__fill--away"
                  :style="{ width: `${stat.awayBarPercent}%` }"
                />
              </view>
              <text class="tech-stat-row__value tech-stat-row__value--away">{{ stat.awayValue }}</text>
            </view>
          </view>

        </view>
      </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { MatchCard } from '../../../types/insight'
import { resolveMatchTechStats } from '../helpers'

const props = defineProps<{ match: MatchCard }>()
const emit = defineEmits<{ (event: 'close'): void }>()
const rows = computed(() => resolveMatchTechStats(props.match))
function getTechStatRowStyle(index: number) {
  return { '--tech-stat-delay': `${120 + index * 70}ms` }
}
</script>

<style scoped lang="css">
@import '../matches-shared.css';

.sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 30;
  background: var(--fi-color-sheet-mask-background);
  backdrop-filter: blur(var(--fi-space-8));
  display: flex;
  align-items: flex-end;
}
.sheet-mask--tech-stats {
  animation: tech-stats-mask-fade 220ms ease-out both;
}
.sheet-card {
  width: 100%;
  max-height: 78vh;
  border-radius: var(--fi-radius-xl) var(--fi-radius-xl) 0 0;
  background: var(--fi-color-ticket-history-trend-summary-item-background-2);
  padding: var(--fi-space-28) var(--fi-space-24) var(--fi-space-40);
  box-shadow: var(--fi-shadow-sheet-card-box-shadow);
  overflow-y: auto;
}
.tech-stats-sheet {
  max-height: 72vh;
  transform-origin: center bottom;
  animation: tech-stats-sheet-enter 280ms cubic-bezier(0.2, 0.9, 0.22, 1) both;
  padding-bottom: calc(var(--fi-space-40) + env(safe-area-inset-bottom) + var(--fi-space-100));
}
.tech-stats-sheet__close {
  display: inline-flex;
  margin: 0 0 0 auto;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-12) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}
.tech-stats-sheet__summary {
  position: sticky;
  top: 0;
  z-index: 3;
  margin-top: var(--fi-space-18);
  padding: var(--fi-space-22) var(--fi-space-24);
  border-radius: var(--fi-radius-md);
  border: var(--fi-component-matches-tech-stats-sheet-summary-border);
  background: var(--fi-color-bg);
  box-shadow: var(--fi-shadow-tech-stats-sheet-summary-box-shadow);
}
.tech-stats-sheet__teams {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-30);
  line-height: var(--fi-leading-tech-stats-sheet-teams-line-height);
  font-weight: var(--fi-weight-extrabold);
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tech-stats-sheet__meta {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.tech-stats-sheet__list {
  margin-top: var(--fi-space-24);
  display: grid;
  gap: var(--fi-space-18);
}
.tech-stat-row {
  display: grid;
  grid-template-columns: var(--fi-space-44) minmax(0, 1fr) auto minmax(0, 1fr) var(--fi-space-44);
  align-items: center;
  gap: var(--fi-space-16);
  padding: var(--fi-space-18) 0;
  border-top: var(--fi-component-matches-tech-stat-row-border-top);
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
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
  text-align: left;
}
.tech-stat-row__value--away {
  text-align: right;
}
.tech-stat-row__label {
  min-width: var(--fi-space-84);
  color: var(--fi-color-tech-stat-row-label-color);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  text-align: center;
}
.tech-stat-row__track {
  height: var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-tech-stat-row-track-background);
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
  background: var(--fi-color-text-primary);
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
@keyframes tech-stats-mask-fade {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes tech-stats-sheet-enter {
  from {
    opacity: 0;
    transform: translateY(32rpx) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
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
