<template>
  <view class="player-sheet-mask" @click.self="emit('close')" @touchmove.stop.prevent>
    <view class="player-sheet" @click.stop @touchmove.stop>
      <view class="player-sheet__heading">
        <view>
          <text class="player-sheet__kicker">球员六维能力</text>
          <view class="player-sheet__title">
            <image :src="player.avatar_storage_url || ''" mode="aspectFill" class="player-sheet__avatar" />
            <view class="player-sheet__title-body">
              <text class="player-sheet__name">{{ player.player_name }}</text>
              <text class="player-sheet__team">{{ player.team_name }} · {{ categoryLabelText }}</text>
            </view>
          </view>
        </view>
        <button class="player-sheet__close" @click="emit('close')">关闭</button>
      </view>

      <view class="player-sheet__ability">
        <view class="player-sheet__ability-heading">
          <text class="player-sheet__ability-title">六维能力</text>
          <text class="player-sheet__ability-caption">按各榜单第一名折算，满分 100；未入榜维度按 0 计</text>
        </view>
        <view class="player-sheet__ability-body">
          <view class="player-sheet__ability-chart">
            <TeamAbilityHexagon :values="abilityValues" :size="220" show-labels />
          </view>
          <view class="player-sheet__legend">
            <view
              v-for="(axis, index) in abilityAxes"
              :key="axis"
              class="player-sheet__legend-item"
            >
              <text class="player-sheet__legend-dot" :class="`player-sheet__legend-dot--${index + 1}`"></text>
              <view class="player-sheet__legend-text">
                <text class="player-sheet__legend-label">{{ axis }}</text>
                <text class="player-sheet__legend-raw">{{ rawTexts[index] ?? '—' }}</text>
              </view>
              <text class="player-sheet__legend-value">{{ formatAbilityScore(abilityValues[index]) }}</text>
            </view>
          </view>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import TeamAbilityHexagon from './TeamAbilityHexagon.vue'
import type { PlayerRankingEntry } from '../../../types/insight'
import { TEAM_ABILITY_AXES } from '../ability'

defineProps<{
  player: PlayerRankingEntry
  categoryLabelText: string
  abilityValues: number[]
  rawTexts: string[]
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const abilityAxes = TEAM_ABILITY_AXES

function formatAbilityScore(value: number | undefined): string {
  return `${Math.round((value ?? 0) * 100)}`
}
</script>

<style scoped lang="css">
.player-sheet-mask {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: var(--fi-space-28);
  background: rgba(21, 22, 27, 0.32);
  animation: fi-overlay-fade-in 180ms ease both;
}

.player-sheet {
  width: 100%;
  max-height: 76vh;
  overflow-y: auto;
  padding: var(--fi-space-28);
  border-radius: 32rpx;
  background: var(--fi-primitive-white);
  animation: fi-sheet-up 240ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.player-sheet__heading {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.player-sheet__kicker {
  margin: 0;
  color: var(--fi-component-kicker-text);
  font-size: var(--fi-component-kicker-size);
  font-weight: var(--fi-weight-bold);
  letter-spacing: 3rpx;
}

.player-sheet__title {
  display: flex;
  align-items: center;
  gap: var(--fi-space-14);
  min-width: 0;
  margin-top: var(--fi-space-8);
}

.player-sheet__avatar {
  width: 88rpx;
  height: 88rpx;
  border-radius: 50%;
  flex: 0 0 auto;
  background: var(--fi-color-page-soft);
}

.player-sheet__title-body {
  min-width: 0;
  display: grid;
  gap: var(--fi-space-6);
}

.player-sheet__name {
  color: var(--fi-color-text-strong);
  font-size: 40rpx;
  line-height: 1.1;
  font-weight: var(--fi-weight-extrabold);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.player-sheet__team {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-tight);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.player-sheet__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-component-close-bg);
  font-size: var(--fi-component-close-size);
  color: var(--fi-component-close-text);
  margin: 0 0 0 auto;
}

.player-sheet__ability {
  margin-top: var(--fi-space-22);
  padding: var(--fi-space-22) var(--fi-space-24);
  border-radius: var(--fi-radius-md);
  border: var(--fi-primitive-border-width) solid var(--fi-color-border-chip);
  background: linear-gradient(160deg, rgba(250, 247, 240, 0.9), rgba(255, 255, 255, 0.98));
  display: grid;
  gap: var(--fi-space-18);
}

.player-sheet__ability-heading {
  display: grid;
  gap: var(--fi-space-6);
}

.player-sheet__ability-title {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}

.player-sheet__ability-caption {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-tight);
}

.player-sheet__ability-body {
  display: grid;
  gap: var(--fi-space-20);
  justify-items: center;
}

/* 给顶点标注预留空间，避免文字被裁切 */
.player-sheet__ability-chart {
  padding: 40rpx 64rpx;
}

.player-sheet__legend {
  width: 100%;
  display: grid;
  gap: var(--fi-space-12);
}

.player-sheet__legend-item {
  display: flex;
  align-items: center;
  gap: var(--fi-space-10);
  min-width: 0;
  padding: var(--fi-space-10) var(--fi-space-14);
  border-radius: var(--fi-radius-sm);
  background: rgba(246, 247, 251, 0.8);
}

.player-sheet__legend-dot {
  width: 12rpx;
  height: 12rpx;
  border-radius: 50%;
  flex-shrink: 0;
  background: rgba(var(--fi-primitive-red-rgb, 214, 48, 49), 0.85);
}

.player-sheet__legend-dot--2 { background: #e89b0c; }
.player-sheet__legend-dot--3 { background: #2563eb; }
.player-sheet__legend-dot--4 { background: #16a34a; }
.player-sheet__legend-dot--5 { background: #7c5cbf; }
.player-sheet__legend-dot--6 { background: #0e9aa7; }

.player-sheet__legend-text {
  min-width: 0;
  display: flex;
  align-items: baseline;
  gap: var(--fi-space-8);
}

.player-sheet__legend-label {
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}

.player-sheet__legend-raw {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-none);
  white-space: nowrap;
}

.player-sheet__legend-value {
  margin-left: auto;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}
</style>
