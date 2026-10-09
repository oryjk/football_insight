<template>
  <view class="subscription-mask" @tap="emit('close')">
    <view class="subscription-dialog" @tap.stop>
      <view class="subscription-dialog__head">
        <text class="subscription-dialog__title">订阅回流提醒</text>
        <button class="subscription-dialog__close" @tap="emit('close')">
          ×
        </button>
      </view>
      <text class="subscription-dialog__match">
        {{ match.home_team_name }} VS {{ match.away_team_name }}
      </text>
      <view v-if="subscribed" class="subscription-opened-card">
        <text class="subscription-opened-card__title">已开通回流提醒</text>
        <text class="subscription-opened-card__desc"
          >有回流命中时会通知到以下邮箱</text
        >
        <text class="subscription-opened-card__email">{{
          email || '暂无邮箱记录'
        }}</text>
      </view>
      <input
        v-else
        :value="email"
        @input="handleEmailInput"
        class="subscription-dialog__input"
        type="text"
        placeholder="填写接收提醒的邮箱"
      />
      <view v-if="!subscribed" class="subscription-plan-list">
        <button
          v-for="plan in plans"
          :key="plan.code"
          class="subscription-plan"
          :class="{
            'subscription-plan--active': selectedPlanCode === plan.code,
          }"
          @tap="emit('plan', plan.code)"
        >
          <view>
            <text class="subscription-plan__title">{{ plan.title }}</text>
            <text class="subscription-plan__desc">{{ plan.description }}</text>
          </view>
          <text class="subscription-plan__price">{{
            formatRefluxSubscriptionPrice(plan.price_cents)
          }}</text>
        </button>
      </view>
      <text v-if="subscribed" class="subscription-dialog__status"
        >当前比赛已开通提醒</text
      >
      <button
        v-if="!subscribed"
        class="subscription-dialog__action"
        :disabled="submitting || !selectedPlanCode"
        @tap="emit('submit')"
      >
        {{ submitting ? '处理中...' : '微信支付并开通' }}
      </button>
    </view>
  </view>
</template>

<script setup lang="ts">
import type {
  TicketWatchMatchSummary,
  RefluxSubscriptionPlan,
} from '../../../types/ticketWatch'

import { formatRefluxSubscriptionPrice } from '../helpers'
defineProps<{
  match: TicketWatchMatchSummary
  subscribed: boolean
  email: string
  plans: RefluxSubscriptionPlan[]
  selectedPlanCode: string
  submitting: boolean
}>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'submit'): void
  (e: 'email', email: string): void
  (e: 'plan', code: string): void
}>()
function handleEmailInput(event: unknown): void {
  const input = event as { detail: { value: string } }
  emit('email', input.detail.value)
}
</script>

<style scoped lang="css">
@import '../ticket-watch-shared.css';

.subscription-mask {
  position: fixed;
  inset: 0;
  z-index: 21;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 0;
  background: var(--fi-color-ticket-subscription-mask-background);
  animation: subscription-mask-fade-in 180ms ease-out both;
}

.subscription-dialog {
  width: 100%;
  display: grid;
  gap: var(--fi-space-20);
  box-sizing: border-box;
  padding: var(--fi-space-34) var(--fi-space-28)
    calc(var(--fi-space-24) + env(safe-area-inset-bottom));
  border-radius: var(--fi-radius-ticket-32) var(--fi-radius-ticket-32) 0 0;
  background: var(--fi-color-bg);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-subscription-dialog-border);
  border-bottom: 0;
  box-shadow: var(--fi-shadow-ticket-subscription-dialog);
  animation: subscription-dialog-slide-up 240ms cubic-bezier(0.22, 1, 0.36, 1)
    both;
}

.subscription-dialog__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-16);
}

.subscription-dialog__title {
  color: var(--fi-color-ticket-history-trend-summary-value-color);
  font-size: var(--fi-font-34);
  font-weight: var(--fi-weight-extrabold);
}

.subscription-dialog__close {
  margin: 0 0 0 auto;
  width: 56rpx;
  height: 56rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-ticket-watch-metric-background-2);
  color: var(--fi-color-ticket-subscription-dialog-close-color);
  font-size: var(--fi-font-34);
  line-height: var(--fi-leading-none);
}

.subscription-dialog__close::after {
  border: none;
}

.subscription-dialog__match,
.subscription-dialog__status {
  color: var(--fi-color-ticket-subscription-dialog-match-color);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-normal);
}

.subscription-dialog__status {
  color: var(--fi-color-ticket-subscription-dialog-status-color);
  font-weight: var(--fi-weight-bold);
}

.subscription-dialog__input {
  min-height: 78rpx;
  padding: 0 var(--fi-space-24);
  border-radius: var(--fi-radius-ticket-20);
  background: var(--fi-color-bg);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-subscription-dialog-border);
  color: var(--fi-color-ticket-history-trend-summary-value-color);
  font-size: var(--fi-font-28);
}

.subscription-opened-card {
  display: grid;
  gap: var(--fi-space-10);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-ticket-22);
  background: var(--fi-color-ticket-subscription-opened-card-background);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-subscription-opened-card-border);
}

.subscription-opened-card__title,
.subscription-opened-card__desc,
.subscription-opened-card__email {
  display: block;
}

.subscription-opened-card__title {
  color: var(--fi-color-ticket-watch-monitor-actions-button-subscribed-color);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-black);
}

.subscription-opened-card__desc {
  color: var(--fi-color-ticket-subscription-opened-card-desc-color);
  font-size: var(--fi-font-24);
  line-height: var(--fi-leading-ticket-1-45);
}

.subscription-opened-card__email {
  color: var(--fi-color-ticket-history-trend-summary-value-color);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
  word-break: break-all;
}

.subscription-plan-list {
  display: grid;
  gap: var(--fi-space-14);
}

.subscription-plan {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-18);
  padding: var(--fi-space-22);
  border-radius: var(--fi-radius-ticket-20);
  background: var(--fi-color-bg);
  border: var(--fi-border-width) solid
    var(--fi-color-ticket-subscription-plan-border);
  text-align: left;
}

.subscription-plan::after {
  border: none;
}

.subscription-plan--active {
  border-color: var(--fi-color-ticket-history-trend-summary-value-color);
  box-shadow: var(--fi-shadow-ticket-subscription-plan-active);
}

.subscription-plan__title,
.subscription-plan__desc {
  display: block;
}

.subscription-plan__title {
  color: var(--fi-color-ticket-history-trend-summary-value-color);
  font-size: var(--fi-font-28);
  font-weight: var(--fi-weight-extrabold);
}

.subscription-plan__desc {
  margin-top: var(--fi-space-8);
  color: var(--fi-color-ticket-interest-confirm-dialog-button-ghost-color);
  font-size: var(--fi-font-22);
  line-height: var(--fi-leading-ticket-1-45);
}

.subscription-plan__price {
  color: var(--fi-color-ticket-history-trend-summary-value-color);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-black);
  flex-shrink: 0;
}

.subscription-dialog__action {
  height: 82rpx;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border-radius: var(--fi-radius-ticket-22);
  background: var(--fi-color-ticket-history-trend-summary-value-color);
  color: var(--fi-color-bg);
  font-size: var(--fi-font-30);
  font-weight: var(--fi-weight-extrabold);
  line-height: var(--fi-leading-none);
  text-align: center;
}

.subscription-dialog__action::after {
  border: none;
}

@keyframes subscription-mask-fade-in {
  from {
    background: var(--fi-color-ticket-from-background);
  }
  to {
    background: var(--fi-color-ticket-subscription-mask-background);
  }
}

@keyframes subscription-dialog-slide-up {
  from {
    opacity: 0;
    transform: translateY(72rpx);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
