<template>
  <view class="panel watch-match-card watch-match-card--reference">
    <view class="watch-match-card__topline">
      <view class="watch-match-card__identity">
        <view class="watch-match-card__ball">
          <image
            class="watch-match-card__ball-image"
            :src="soccerBallIcon"
            mode="aspectFit"
          />
        </view>
        <view class="watch-match-card__title-wrap">
          <text class="section-title section-title--match"
            >{{ match.home_team_name }} VS {{ match.away_team_name }}</text
          >
        </view>
      </view>

      <view class="watch-match-card__membership-pill">
        <image
          class="watch-match-card__membership-icon"
          :src="crownIcon"
          mode="aspectFit"
        />
        <text>{{ membershipLabel }}</text>
      </view>
    </view>

    <view class="watch-match-card__meta watch-match-card__meta--reference">
      <text class="watch-match-card__round"
        >第 {{ match.round_number }} 轮</text
      >
      <text class="watch-match-card__date"
        >{{ match.match_date }} {{ match.match_time }}</text
      >
    </view>

    <view class="watch-metrics">
      <view class="watch-metric">
        <view class="watch-metric__head">
          <text class="watch-metric__label">命中区域</text>
          <image
            class="watch-metric__icon-image"
            :src="targetIcon"
            mode="aspectFit"
          />
        </view>
        <text class="watch-metric__value">{{ availableRegions }}</text>
      </view>
      <view class="watch-metric">
        <view class="watch-metric__head">
          <text class="watch-metric__label">累计回流</text>
          <image
            class="watch-metric__icon-image"
            :src="trendIcon"
            mode="aspectFit"
          />
        </view>
        <text class="watch-metric__value">{{ totalOccurrences }}</text>
      </view>
    </view>

    <view class="watch-monitor-actions">
      <button
        class="watch-monitor-actions__button"
        :class="{ 'watch-monitor-actions__button--ghost': monitoring }"
        @tap="emit('monitor')"
      >
        <image
          class="watch-monitor-actions__button-icon-image"
          :src="playCircleIcon"
          mode="aspectFit"
        />
        {{ monitoring ? '停止监控' : '开始监控' }}
      </button>
      <button
        class="watch-monitor-actions__button watch-monitor-actions__button--subscribe"
        :class="{ 'watch-monitor-actions__button--subscribed': subscribed }"
        @tap="emit('subscribe')"
      >
        {{ subscribed ? '已开通' : '订阅提醒' }}
      </button>
      <button class="watch-monitor-actions__button" @tap="emit('match-id')">
        获取比赛 ID
      </button>
    </view>

    <text class="watch-match-card__note watch-match-card__note--reference">
      <image
        class="watch-match-card__note-icon-image"
        :src="shieldIcon"
        mode="aspectFit"
      />
      <text>{{ message }}</text>
    </text>
  </view>
</template>

<script setup lang="ts">
import type { TicketWatchMatchSummary } from '../../../types/ticketWatch'

import soccerBallIcon from '../../../static/ticket-watch/soccer-ball.svg'
import crownIcon from '../../../static/ticket-watch/crown.svg'
import targetIcon from '../../../static/ticket-watch/target.svg'
import trendIcon from '../../../static/ticket-watch/trend.svg'
import playCircleIcon from '../../../static/ticket-watch/play-circle.svg'
import shieldIcon from '../../../static/ticket-watch/shield.svg'
defineProps<{
  match: TicketWatchMatchSummary
  availableRegions: number
  totalOccurrences: number
  membershipLabel: string
  monitoring: boolean
  subscribed: boolean
  message: string
}>()
const emit = defineEmits<{
  (e: 'monitor'): void
  (e: 'subscribe'): void
  (e: 'match-id'): void
}>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.watch-match-card {
  display: flex;
  flex-direction: column;
  gap: var(--fi-space-14);
}

.watch-match-card--reference {
  gap: var(--fi-space-20);
  padding: var(--fi-space-34) var(--fi-space-32) var(--fi-space-28);
  border-radius: var(--fi-radius-ticket-34);
  border-color: var(--fi-color-ticket-watch-match-card-reference-border-color);
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-watch-match-card-reference-background),
    var(--fi-color-ticket-watch-match-card-reference-background-2)
  );
  box-shadow: var(--fi-shadow-ticket-watch-match-card-reference);
}

.watch-match-card__topline {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-16);
}

.watch-match-card__identity {
  display: flex;
  align-items: center;
  gap: var(--fi-space-14);
  flex: 1;
  min-width: 0;
}

.watch-match-card__ball {
  width: 40rpx;
  height: 40rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.watch-match-card__ball-image {
  width: 40rpx;
  height: 40rpx;
}

.watch-match-card__title-wrap {
  flex: 1;
  min-width: 0;
  padding-top: 0;
}

.watch-match-card__membership-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-6);
  flex-shrink: 0;
  min-height: 40rpx;
  padding: 0 var(--fi-space-14) 0 var(--fi-space-12);
  border-radius: var(--fi-radius-round);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-match-card-membership-pill-border);
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-watch-match-card-membership-pill-background),
    var(--fi-color-ticket-watch-match-card-membership-pill-background-2)
  );
  color: var(--fi-color-primary-darker);
  font-size: var(--fi-font-18);
  font-weight: var(--fi-weight-normal);
  line-height: var(--fi-leading-none);
  box-shadow: var(--fi-shadow-ticket-watch-match-card-membership-pill);
}

.watch-match-card__membership-icon {
  width: 18rpx;
  height: 18rpx;
  flex-shrink: 0;
}

.watch-match-card__round {
  display: block;
  color: var(--fi-color-ticket-watch-match-card-round-color);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-medium);
  line-height: var(--fi-leading-none);
}

.watch-match-card__meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--fi-color-ticket-watch-match-card-meta-color);
  font-size: var(--fi-font-22);
}

.watch-match-card__meta--reference {
  color: var(--fi-color-ticket-watch-match-card-round-color);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
  padding-left: var(--fi-space-68);
  margin-top: -4rpx;
}

.watch-match-card__date {
  color: var(--fi-color-ticket-watch-match-card-date-color);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-medium);
}

.watch-metric__head {
  display: flex;
  align-items: center;
  justify-content: flex-start;
}

.watch-metric__icon-image {
  position: absolute;
  right: 18rpx;
  top: 50%;
  transform: translateY(-50%);
  width: 46rpx;
  height: 46rpx;
  flex-shrink: 0;
  opacity: 0.72;
}

.watch-match-card__note {
  display: block;
  color: var(--fi-color-ticket-watch-match-card-note-color);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-ticket-1-4);
  text-align: center;
}

.watch-match-card__note--reference {
  color: var(--fi-color-ticket-watch-match-card-note-reference-color);
  font-size: var(--fi-font-20);
  line-height: var(--fi-leading-snug);
}

.watch-match-card__note-icon-image {
  width: 20rpx;
  height: 20rpx;
  margin-right: var(--fi-space-8);
  vertical-align: middle;
  opacity: 0.9;
}

.watch-metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--fi-space-16);
}

.watch-monitor-actions {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--fi-space-12);
}

.watch-monitor-actions__button {
  width: 100%;
  min-height: 78rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--fi-space-10);
  border-radius: var(--fi-radius-ticket-22);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-bg);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-extrabold);
  text-align: center;
  box-shadow: var(--fi-shadow-ticket-watch-monitor-actions-button);
}

.watch-monitor-actions__button-icon-image {
  width: 30rpx;
  height: 30rpx;
  flex-shrink: 0;
}

.watch-monitor-actions__button::after {
  border: none;
}

.watch-monitor-actions__button--ghost {
  background: var(--fi-color-ticket-watch-metric-background-2);
  color: var(--fi-color-ticket-watch-monitor-actions-button-ghost-color);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-monitor-actions-button-ghost-border);
  box-shadow: none;
}

.watch-monitor-actions__button--subscribe {
  background: var(--fi-color-ticket-watch-metric-background-2);
  color: var(--fi-color-ticket-watch-monitor-actions-button-ghost-color);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-monitor-actions-button-ghost-border);
  box-shadow: none;
}

.watch-monitor-actions__button--subscribed {
  background: var(
    --fi-color-ticket-watch-monitor-actions-button-subscribed-background
  );
  color: var(--fi-color-ticket-watch-monitor-actions-button-subscribed-color);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-monitor-actions-button-subscribed-border);
  box-shadow: none;
}

.watch-metric {
  position: relative;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  min-height: 106rpx;
  border-radius: var(--fi-radius-ticket-22);
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-watch-metric-background),
    var(--fi-color-ticket-watch-metric-background-2)
  );
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-watch-metric-border);
  padding: var(--fi-space-16) var(--fi-space-78) var(--fi-space-18)
    var(--fi-space-18);
  box-shadow: var(--fi-shadow-ticket-watch-metric);
}

.watch-metric__label {
  display: block;
  color: var(--fi-color-ticket-watch-metric-label-color);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-none);
}

.watch-metric__value {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-ticket-watch-metric-value-color);
  font-size: var(--fi-font-44);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
}
</style>
