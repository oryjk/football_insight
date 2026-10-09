<template>
  <view class="board-header">
    <view class="hero-card">
      <view class="hero-card__icon">
        <view class="hero-card__icon-box">
          <image
            class="hero-card__icon-image"
            :src="boardIcon"
            mode="aspectFit"
          />
        </view>
      </view>
      <view class="hero-card__body">
        <text class="eyebrow">回流看板</text>
        <text class="hero-card__title">实时监控比赛余票动态</text>
        <text class="hero-card__summary"> 先看热区，再看价位和复盘结论 </text>
      </view>
      <view class="hero-card__action">
        <text class="hero-card__badge">
          <image class="inline-image-icon" :src="boardIcon" mode="aspectFit" />
          <text>回流看板</text>
        </text>
      </view>
    </view>
    <view class="team-switch">
      <button
        class="team-switch__item"
        :class="{
          'team-switch__item--active': selectedTeam === 'chengdurongcheng',
        }"
        @click="emit('team', 'chengdurongcheng')"
      >
        成都蓉城
      </button>
      <button
        class="team-switch__item"
        :class="{ 'team-switch__item--active': selectedTeam === 'yunnanyukun' }"
        @click="emit('team', 'yunnanyukun')"
      >
        云南玉昆
      </button>
    </view>
    <view class="tab-switch">
      <button
        class="tab-switch__item"
        :class="{ 'tab-switch__item--active': activeTab === 'current' }"
        @click="emit('tab', 'current')"
      >
        当前比赛
      </button>
      <button
        class="tab-switch__item"
        :class="{ 'tab-switch__item--active': activeTab === 'history' }"
        @click="emit('tab', 'history')"
      >
        历史比赛
      </button>
      <button
        class="tab-switch__item"
        :class="{ 'tab-switch__item--active': activeTab === 'history-stats' }"
        @click="emit('tab', 'history-stats')"
      >
        历史统计
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import boardIcon from '../../../static/ticket-watch/board.svg'
import type { TicketWatchTeam, TicketWatchTab } from '../types'
defineProps<{ selectedTeam: TicketWatchTeam; activeTab: TicketWatchTab }>()
const emit = defineEmits<{
  (e: 'team', team: TicketWatchTeam): void
  (e: 'tab', tab: TicketWatchTab): void
}>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.hero-card {
  animation: none;
  background: var(--fi-color-ticket-watch-metric-box-shadow);
  border-radius: var(--fi-radius-lg);
  padding: var(--fi-space-24);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-monitor-actions-button-ghost-border);
  box-shadow: var(--fi-shadow-ticket-hero-card);
}

.hero-card {
  display: flex;
  align-items: center;
  gap: var(--fi-space-18);
}

.hero-card__body {
  flex: 1;
  min-width: 0;
}

.hero-card__icon {
  flex-shrink: 0;
}

.hero-card__icon-box {
  width: 64rpx;
  height: 64rpx;
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-ticket-watch-metric-background-2);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-monitor-actions-button-ghost-border);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: var(--fi-shadow-ticket-hero-card-icon-box);
}

.hero-card__action {
  flex-shrink: 0;
}

.hero-card__icon-image {
  width: 34rpx;
  height: 34rpx;
}

.hero-card__title {
  display: block;
  margin-top: var(--fi-space-6);
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-ticket-1-18);
  font-weight: var(--fi-weight-normal);
}

.hero-card__summary {
  display: block;
  margin-top: var(--fi-space-6);
  color: var(--fi-color-ticket-hero-card-summary-color);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-ticket-1-45);
}

.hero-card__badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-12) var(--fi-space-20);
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-monitor-actions-button-ghost-border);
  background: var(--fi-color-ticket-watch-metric-background-2);
  color: var(--fi-color-ticket-hero-card-badge-color);
  font-size: var(--fi-font-22);
}

.team-switch {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--fi-space-12);
  padding: 0 0 var(--fi-space-16);
  background: transparent;
  border: 0;
}

.team-switch__item {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 64rpx;
  padding: 0 var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-team-switch-item-border);
  background: var(--fi-color-ticket-watch-metric-box-shadow);
  color: var(--fi-color-ticket-team-switch-item-color);
  font-size: var(--fi-font-23);
  font-weight: var(--fi-weight-bold);
  text-align: center;
  box-shadow: var(--fi-shadow-ticket-team-switch-item);
}

.team-switch__item--active {
  border-color: var(--fi-color-ticket-team-switch-item-active-border-color);
  background: var(--fi-color-ticket-team-switch-item-active-border-color);
  color: var(--fi-color-bg);
  box-shadow: var(--fi-shadow-ticket-team-switch-item-active);
}

.tab-switch {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--fi-space-12);
  padding: 0;
  background: transparent;
  border: 0;
}

.tab-switch__item {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 64rpx;
  padding: 0 var(--fi-space-14);
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-team-switch-item-border);
  background: var(--fi-color-ticket-watch-metric-box-shadow);
  color: var(--fi-color-ticket-team-switch-item-color);
  font-size: var(--fi-font-23);
  font-weight: var(--fi-weight-bold);
  text-align: center;
  box-shadow: var(--fi-shadow-ticket-team-switch-item);
}

.tab-switch__item--active {
  border-color: var(--fi-color-text-primary);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-bg);
  box-shadow: var(--fi-shadow-ticket-tab-switch-item-active);
}
.board-header {
  display: grid;
  gap: var(--fi-space-18);
}
</style>
