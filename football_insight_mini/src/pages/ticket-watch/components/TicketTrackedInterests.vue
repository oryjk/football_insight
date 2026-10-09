<template>
  <view class="panel tracked-interest-panel">
    <view class="section-heading section-heading--compact">
      <view>
        <text class="section-kicker">My Tracking</text>
        <text class="section-title">我的钓区追踪</text>
      </view>
      <view class="section-heading__actions">
        <text class="meta-pill">
          {{ summary.hitCount }}/{{ summary.total }} 已等到
        </text>
        <button class="collapse-pill" @tap="emit('toggle')">
          {{ collapsed ? '展开' : '收起' }}
        </button>
      </view>
    </view>

    <template v-if="!collapsed">
      <text v-if="summary.total" class="tracked-interest-summary">
        你当前钓了 {{ summary.total }} 个区，其中
        {{ summary.hitCount }} 个区已经等到回流，{{
          summary.pendingCount
        }}
        个区还在继续等。
      </text>
      <text
        v-else
        class="tracked-interest-summary tracked-interest-summary--empty"
      >
        你还没标记自己的钓区，点下面分区卡片就会开始记录等待时间。
      </text>

      <view v-if="interests.length" class="tracked-interest-list">
        <view
          v-for="interest in interests"
          :key="`tracked-interest-${interest.block_name}`"
          class="tracked-interest-item"
        >
          <view class="tracked-interest-item__head">
            <text class="tracked-interest-item__name">{{
              interest.block_name
            }}</text>
            <text
              class="tracked-interest-item__status"
              :class="{
                'tracked-interest-item__status--hit':
                  interest.first_inventory_at,
              }"
            >
              {{
                formatTrackedInterestWaitLabel(
                  interest.started_at,
                  interest.first_inventory_at,
                )
              }}
            </text>
          </view>
          <view class="tracked-interest-item__meta">
            <text
              >开始钓：{{
                formatTrackedInterestTime(interest.started_at)
              }}</text
            >
            <text>
              首次回流：{{
                interest.first_inventory_at
                  ? formatTrackedInterestTime(interest.first_inventory_at)
                  : '暂未等到'
              }}
            </text>
          </view>
        </view>
      </view>
    </template>
  </view>
</template>

<script setup lang="ts">
import type { TicketWatchTrackedInterest } from '../../../types/ticketWatch'
import type { TicketWatchTrackedInterestSummary } from '../helpers'
import {
  formatTrackedInterestTime,
  formatTrackedInterestWaitLabel,
} from '../helpers'
defineProps<{
  interests: TicketWatchTrackedInterest[]
  summary: TicketWatchTrackedInterestSummary
  collapsed: boolean
}>()
const emit = defineEmits<{ (e: 'toggle'): void }>()
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.tracked-interest-panel {
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-tracked-interest-panel-background),
    var(--fi-color-ticket-history-trend-summary-item-background-2)
  );
}

.tracked-interest-summary {
  display: block;
  color: var(--fi-color-ticket-insight-copy-color);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-ticket-1-6);
}

.tracked-interest-summary--empty {
  color: var(--fi-color-ticket-tracked-interest-summary-empty-color);
}

.tracked-interest-list {
  display: grid;
  gap: var(--fi-space-10);
  margin-top: var(--fi-space-16);
}

.tracked-interest-item {
  border-radius: var(--fi-radius-sm);
  padding: var(--fi-space-18) var(--fi-space-20);
  background: linear-gradient(
    180deg,
    var(--fi-color-ticket-watch-metric-box-shadow),
    var(--fi-color-ticket-tracked-interest-item-background)
  );
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-tracked-interest-item-border);
}

.tracked-interest-item__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-20);
}

.tracked-interest-item__name {
  color: var(--fi-color-ticket-focus-panel-headline-title-color);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-normal);
}

.tracked-interest-item__status {
  color: var(--fi-color-ticket-tracked-interest-summary-empty-color);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
}

.tracked-interest-item__status--hit {
  color: var(--fi-color-ticket-tracked-interest-item-status-hit-color);
}

.tracked-interest-item__meta {
  display: grid;
  gap: var(--fi-space-8);
  margin-top: var(--fi-space-10);
  color: var(--fi-color-ticket-tracked-interest-item-meta-color);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-normal);
}
</style>
