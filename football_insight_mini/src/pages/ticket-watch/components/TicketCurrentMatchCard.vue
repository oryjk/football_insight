<template>
  <view class="watch-match-card-shell">
    <FiPanel>
    <view class="watch-match-card">
      <view class="watch-match-card__topline">
        <view class="watch-match-card__identity">
          <image class="watch-match-card__ball" :src="soccerBallIcon" mode="aspectFit" />
          <text class="watch-match-card__title">{{ match.home_team_name }} VS {{ match.away_team_name }}</text>
        </view>
        <text class="watch-match-card__meta">第 {{ match.round_number }} 轮 · {{ match.match_date }} {{ match.match_time }}</text>
      </view>

      <TicketRefreshStatus
        :monitoring="monitoring"
        :refreshing="refreshing"
        :poll-failing="pollFailing"
        :interval-seconds="intervalSeconds"
        :last-updated-at-ms="lastUpdatedAtMs"
        :ticking="ticking"
      />

      <view class="watch-metrics">
        <view class="watch-metric">
          <view class="watch-metric__head">
            <text class="watch-metric__label">命中区域</text>
            <image class="watch-metric__icon" :src="targetIcon" mode="aspectFit" />
          </view>
          <text class="watch-metric__value">{{ availableRegions }}</text>
        </view>
        <view class="watch-metric">
          <view class="watch-metric__head">
            <text class="watch-metric__label">累计回流</text>
            <image class="watch-metric__icon" :src="trendIcon" mode="aspectFit" />
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
          <image class="watch-monitor-actions__icon" :src="playCircleIcon" mode="aspectFit" />
          {{ monitoring ? '停止监控' : '开始监控' }}
        </button>
        <view class="watch-monitor-actions__secondary">
          <button
            class="watch-monitor-actions__button watch-monitor-actions__button--outline"
            :class="{ 'watch-monitor-actions__button--subscribed': subscribed }"
            @tap="emit('subscribe')"
          >{{ subscribed ? '已开通' : '订阅提醒' }}</button>
          <button
            class="watch-monitor-actions__button watch-monitor-actions__button--outline"
            @tap="emit('match-id')"
          >获取比赛 ID</button>
        </view>
      </view>

      <view class="watch-match-card__note">
        <image class="watch-match-card__note-icon" :src="shieldIcon" mode="aspectFit" />
        <text>{{ message }}</text>
      </view>
    </view>
    </FiPanel>
  </view>
</template>

<script setup lang="ts">
import FiPanel from '../../../components/FiPanel.vue'
import TicketRefreshStatus from './TicketRefreshStatus.vue'
import type { TicketWatchMatchSummary } from '../../../types/ticketWatch'
import soccerBallIcon from '../../../static/ticket-watch/soccer-ball.svg'
import targetIcon from '../../../static/ticket-watch/target.svg'
import trendIcon from '../../../static/ticket-watch/trend.svg'
import playCircleIcon from '../../../static/ticket-watch/play-circle.svg'
import shieldIcon from '../../../static/ticket-watch/shield.svg'

defineProps<{
  match: TicketWatchMatchSummary
  availableRegions: number
  totalOccurrences: number
  monitoring: boolean
  refreshing: boolean
  pollFailing: boolean
  intervalSeconds: number
  lastUpdatedAtMs: number | null
  ticking: boolean
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
.watch-match-card-shell { display: flex; flex-direction: column; align-items: stretch; }
.watch-match-card { display: flex; flex-direction: column; gap: var(--fi-space-20); }
.watch-match-card__topline { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-16); }
.watch-match-card__identity { display: flex; align-items: center; gap: var(--fi-space-14); flex: 1; min-width: 0; }
.watch-match-card__ball { width: var(--fi-icon-md); height: var(--fi-icon-md); flex-shrink: 0; }
.watch-match-card__title { color: var(--fi-color-text-strong); font-size: var(--fi-font-32); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-snug); }
.watch-match-card__meta { flex-shrink: 0; color: var(--fi-color-text-muted); font-size: var(--fi-font-20); font-weight: var(--fi-weight-medium); line-height: var(--fi-leading-normal); white-space: nowrap; }
.watch-metrics { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--fi-space-16); }
.watch-metric { display: flex; flex-direction: column; gap: var(--fi-space-10); padding: var(--fi-space-16) var(--fi-space-18); border-radius: var(--fi-radius-md); background: var(--fi-color-page); }
.watch-metric__head { display: flex; align-items: center; justify-content: space-between; gap: var(--fi-space-8); }
.watch-metric__icon { width: var(--fi-icon-md); height: var(--fi-icon-md); flex-shrink: 0; }
.watch-metric__label { color: var(--fi-color-text-muted); font-size: var(--fi-font-22); font-weight: var(--fi-weight-medium); line-height: var(--fi-leading-normal); }
.watch-metric__value { color: var(--fi-color-text-strong); font-size: var(--fi-font-44); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-none); }
.watch-monitor-actions { display: grid; gap: var(--fi-space-12); }
.watch-monitor-actions__secondary { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--fi-space-12); }
.watch-monitor-actions__button { box-sizing: border-box; width: 100%; min-height: var(--fi-space-78); margin: 0; display: flex; align-items: center; justify-content: center; gap: var(--fi-space-10); padding: var(--fi-space-16) var(--fi-space-12); border: var(--fi-border-width) solid var(--fi-color-text-primary); border-radius: var(--fi-radius-md); background: var(--fi-color-text-primary); color: var(--fi-color-surface); font-size: var(--fi-font-26); font-weight: var(--fi-weight-extrabold); line-height: var(--fi-leading-snug); text-align: center; box-shadow: var(--fi-shadow-soft); }
.watch-monitor-actions__icon { width: var(--fi-icon-sm); height: var(--fi-icon-sm); flex-shrink: 0; }
.watch-monitor-actions__button::after { border: none; }
.watch-monitor-actions__button--ghost { background: var(--fi-color-page); color: var(--fi-color-text-secondary); border-color: var(--fi-color-border-chip); box-shadow: none; }
.watch-monitor-actions__button--outline { background: var(--fi-color-surface); color: var(--fi-color-text-secondary); border-color: var(--fi-color-border-chip); box-shadow: none; }
.watch-monitor-actions__button--subscribed { background: var(--fi-color-ticket-watch-monitor-actions-button-subscribed-background); color: var(--fi-color-ticket-watch-monitor-actions-button-subscribed-color); border-color: var(--fi-color-ticket-watch-monitor-actions-button-subscribed-border); }
.watch-match-card__note { display: flex; align-items: flex-start; justify-content: center; gap: var(--fi-space-8); color: var(--fi-color-text-caption); font-size: var(--fi-font-20); font-weight: var(--fi-weight-medium); line-height: var(--fi-leading-normal); }
.watch-match-card__note-icon { width: var(--fi-icon-sm); height: var(--fi-icon-sm); flex-shrink: 0; }
</style>
