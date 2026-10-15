<template>
  <view class="goal-card-mask" @tap="emit('close')" @touchmove.stop.prevent>
    <view class="goal-card" role="dialog" :aria-label="detail.title" aria-modal="true" @tap.stop @touchmove.stop>
      <view class="goal-card__stripe" />
      <button class="goal-card__close" aria-label="关闭单场进球详情" hover-class="goal-card__close--pressed" @tap="emit('close')">
        <text aria-hidden="true">×</text>
      </button>

      <view class="goal-card__head">
        <text class="goal-card__kicker">第 {{ match.round_number }} 轮 · 单场进球</text>
        <view class="goal-card__score">
          <text class="goal-card__goals">{{ match.goals }}</text>
          <text class="goal-card__unit">球</text>
          <text v-if="isHatTrick" class="goal-card__badge">帽子戏法</text>
        </view>
        <text class="goal-card__player">{{ playerName }}</text>
      </view>

      <view class="goal-card__tear">
        <view class="goal-card__notch goal-card__notch--left" />
        <view class="goal-card__dash" />
        <view class="goal-card__notch goal-card__notch--right" />
      </view>

      <view class="goal-card__fixture">
        <text class="goal-card__team">{{ match.home_team_name }}</text>
        <text class="goal-card__vs">VS</text>
        <text class="goal-card__team">{{ match.away_team_name }}</text>
      </view>

      <view class="goal-card__meta">
        <view class="goal-card__meta-item">
          <text class="goal-card__meta-label">比赛时间</text>
          <text class="goal-card__meta-value">{{ match.match_date }} {{ match.match_time }}</text>
        </view>
        <view class="goal-card__meta-item goal-card__meta-item--end">
          <text class="goal-card__meta-label">出场</text>
          <text class="goal-card__meta-value">{{ match.minutes_played }}′</text>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { PlayerMatchGoals } from '../types/insight'
import { buildGoalMatchDetail } from '../utils/playerGoals'

const props = defineProps<{ playerName: string; match: PlayerMatchGoals }>()
const emit = defineEmits<{ (event: 'close'): void }>()
const detail = computed(() => buildGoalMatchDetail(props.playerName, props.match))
const isHatTrick = computed(() => props.match.goals === 3)
</script>

<style scoped>
.goal-card-mask {
  position: fixed;
  inset: 0;
  z-index: 90;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--fi-color-match-card-overlay);
}

.goal-card {
  position: relative;
  box-sizing: border-box;
  width: calc(100% - var(--fi-space-64) * 2);
  max-width: calc(var(--fi-space-64) * 10);
  overflow: hidden;
  border-radius: var(--fi-radius-topic-hero);
  background: var(--fi-gradient-topic-hero);
  box-shadow: var(--fi-shadow-match-card);
  border: 1rpx solid var(--fi-color-match-card-hairline);
  color: var(--fi-color-topic-hero-text);
}

.goal-card__stripe {
  height: 8rpx;
  background: linear-gradient(90deg, var(--fi-color-primary-deep), var(--fi-color-primary), var(--fi-color-primary-deep));
}

.goal-card__close {
  position: absolute;
  top: var(--fi-space-24);
  right: var(--fi-space-24);
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0;
  padding: 0;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-topic-hero-control);
  color: var(--fi-color-topic-hero-muted);
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-none);
}

.goal-card__close::after {
  border: 0;
}

.goal-card__close--pressed {
  background: var(--fi-color-topic-hero-track);
}

.goal-card__head {
  padding: var(--fi-space-40) var(--fi-space-40) var(--fi-space-32);
}

.goal-card__kicker {
  display: block;
  padding-right: var(--fi-space-64);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
  font-weight: var(--fi-weight-extrabold);
  letter-spacing: 4rpx;
}

.goal-card__score {
  display: flex;
  align-items: baseline;
  margin-top: var(--fi-space-16);
}

.goal-card__goals {
  font-size: 128rpx;
  line-height: 1;
  font-weight: var(--fi-weight-black);
  font-style: italic;
  letter-spacing: -4rpx;
}

.goal-card__unit {
  margin-left: var(--fi-space-8);
  color: var(--fi-color-topic-hero-muted);
  font-size: var(--fi-font-32);
  font-weight: var(--fi-weight-bold);
}

.goal-card__badge {
  margin-left: var(--fi-space-20);
  padding: 4rpx var(--fi-space-16);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-match-card-gold-soft);
  border: 1rpx solid var(--fi-color-match-card-gold);
  color: var(--fi-color-match-card-gold);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
  font-weight: var(--fi-weight-bold);
  align-self: center;
}

.goal-card__player {
  display: block;
  margin-top: var(--fi-space-12);
  font-size: var(--fi-font-36);
  line-height: var(--fi-leading-snug);
  font-weight: var(--fi-weight-bold);
  overflow-wrap: anywhere;
}

.goal-card__tear {
  position: relative;
  display: flex;
  align-items: center;
  height: var(--fi-space-32);
}

.goal-card__dash {
  flex: 1;
  margin: 0 var(--fi-space-32);
  border-top: 2rpx dashed var(--fi-color-topic-hero-track);
}

.goal-card__notch {
  position: absolute;
  top: 0;
  width: var(--fi-space-32);
  height: var(--fi-space-32);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-match-card-overlay);
}

.goal-card__notch--left {
  left: calc(var(--fi-space-16) * -1);
}

.goal-card__notch--right {
  right: calc(var(--fi-space-16) * -1);
}

.goal-card__fixture {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--fi-space-24) var(--fi-space-40) var(--fi-space-8);
}

.goal-card__team {
  flex: 1;
  min-width: 0;
  font-size: var(--fi-font-30);
  line-height: var(--fi-leading-snug);
  font-weight: var(--fi-weight-bold);
  text-align: center;
  overflow-wrap: anywhere;
}

.goal-card__vs {
  flex: none;
  padding: 0 var(--fi-space-16);
  color: var(--fi-color-topic-hero-caption);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-black);
  font-style: italic;
  letter-spacing: 2rpx;
}

.goal-card__meta {
  display: flex;
  justify-content: space-between;
  margin-top: var(--fi-space-24);
  padding: var(--fi-space-24) var(--fi-space-40) var(--fi-space-40);
  border-top: 1rpx solid var(--fi-color-match-card-hairline);
}

.goal-card__meta-item {
  display: flex;
  flex-direction: column;
}

.goal-card__meta-item--end {
  align-items: flex-end;
}

.goal-card__meta-label {
  color: var(--fi-color-topic-hero-caption);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}

.goal-card__meta-value {
  margin-top: var(--fi-space-4);
  color: var(--fi-color-topic-hero-muted);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-normal);
  font-weight: var(--fi-weight-bold);
}
</style>
