<template>
  <view v-if="visible" class="match-id-mask" @touchmove.stop.prevent @tap="close">
    <view class="match-id-card" @touchmove.stop @tap.stop>
      <view class="match-id-card__head">
        <view>
          <text class="match-id-card__kicker">比赛 ID</text>
          <text class="match-id-card__title">{{
            state === 'unlocked' ? '已解锁' : '获取比赛 ID'
          }}</text>
        </view>
        <button class="match-id-card__close" :disabled="paying" @tap="close">
          ×
        </button>
      </view>

      <text class="match-id-card__match">{{ matchLabel }}</text>

      <view v-if="state === 'loading'" class="match-id-card__body match-id-card__body--loading">
        <text class="match-id-card__copy">正在查询解锁状态…</text>
      </view>
      <view v-else-if="state === 'unlocked'" class="match-id-card__body">
        <view class="match-id-value-panel">
          <text v-if="matchId !== null" class="match-id-value-panel__value">{{ matchId }}</text>
          <text v-else class="match-id-card__copy">业务 ID 暂不可用</text>
          <button class="match-id-value-panel__copy" :disabled="matchId === null" @tap="copy">复制</button>
        </view>
        <text class="match-id-card__source">{{ sourceLabel }}</text>
      </view>

      <view v-else class="match-id-card__body">
        <text class="match-id-card__copy"
          >V6 及以上会员可免费查看，或支付 ¥5 解锁本场比赛 ID。</text
        >
        <view class="match-id-actions">
          <button
            class="match-id-actions__button match-id-actions__button--ghost"
            :disabled="paying"
            @tap="upgrade"
          >
            升级到 V6
          </button>
          <button
            class="match-id-actions__button match-id-actions__button--pay"
            :disabled="paying"
            @tap="pay"
          >
            {{ paying ? '处理中...' : '¥5 解锁本场' }}
          </button>
        </view>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { buildMatchIdSourceLabel } from '../helpers'

const props = defineProps<{
  visible: boolean
  matchId: string | null
  matchLabel: string
  state: 'loading' | 'locked' | 'unlocked' | 'paying'
  via: 'membership' | 'purchase' | null
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'pay'): void
  (e: 'upgrade'): void
}>()

const paying = computed(() => props.state === 'paying')

const sourceLabel = computed(() => buildMatchIdSourceLabel(props.via))

function close(): void {
  if (paying.value) {
    return
  }

  emit('close')
}

function pay(): void {
  if (paying.value) {
    return
  }

  emit('pay')
}

function upgrade(): void {
  if (paying.value) {
    return
  }

  emit('upgrade')
}

function copy(): void {
  if (props.matchId === null) {
    return
  }

  uni.setClipboardData({
    data: String(props.matchId),
    success: () => {
      uni.showToast({ title: '已复制', icon: 'none' })
    },
  })
}
</script>

<style scoped lang="css">
/* 动画 keyframes 定义在组件内：user 页的 fi-fade-in-up 是页面局部样式，跨页引用不生效。 */
@keyframes fi-match-id-overlay-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.match-id-mask {
  position: fixed;
  inset: 0;
  bottom: var(--window-bottom, 0px);
  z-index: 90;
  background: var(--fi-color-overlay);
  backdrop-filter: blur(var(--fi-space-8));
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 var(--fi-space-32);
  animation: fi-match-id-overlay-in 180ms ease both;
}

.match-id-card {
  width: 100%;
  max-width: calc(var(--fi-space-64) * 10);
  box-sizing: border-box;
  border-radius: var(--fi-radius-lg);
  background: var(--fi-color-ticket-history-trend-summary-item-background-2);
  padding: var(--fi-space-28) var(--fi-space-24) var(--fi-space-32);
  box-shadow: var(--fi-shadow-card-strong);
}

.match-id-card__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--fi-space-12);
}

.match-id-card__kicker {
  display: block;
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-22);
  font-weight: var(--fi-weight-bold);
  letter-spacing: var(--fi-space-3);
}

.match-id-card__title {
  display: block;
  margin-top: var(--fi-space-10);
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-44);
  line-height: var(--fi-leading-tight);
  font-weight: var(--fi-weight-extrabold);
}

.match-id-card__close {
  padding: 0;
  width: var(--fi-space-56);
  height: var(--fi-space-56);
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: var(--fi-leading-none);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-32);
}

.match-id-card__close::after {
  border: none;
}

.match-id-card__match {
  display: block;
  margin-top: var(--fi-space-14);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-normal);
}

.match-id-card__body {
  margin-top: var(--fi-space-22);
  /* Reserve enough room for the longest (locked) content before the request resolves. */
  min-height: calc(var(--fi-space-84) + var(--fi-space-18) + var(--fi-font-26) * var(--fi-leading-relaxed) * 3);
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: var(--fi-space-18);
}

.match-id-card__body--loading {
  align-items: center;
}

.match-id-value-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--fi-space-18);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-md);
  border: var(--fi-border-card);
  background: var(--fi-color-page-soft);
}

.match-id-value-panel__value {
  color: var(--fi-color-text-primary);
  font-size: var(--fi-font-48);
  font-weight: var(--fi-weight-extrabold);
  letter-spacing: var(--fi-space-2);
  line-height: var(--fi-leading-none);
  word-break: break-all;
}

.match-id-value-panel__copy {
  flex-shrink: 0;
  margin: 0;
  min-width: calc(var(--fi-space-56) * 2);
  height: var(--fi-space-64);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 var(--fi-space-24);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-24);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.match-id-value-panel__copy::after {
  border: none;
}

.match-id-card__source {
  color: var(--fi-color-text-muted);
  font-size: var(--fi-font-24);
}

.match-id-card__copy {
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-26);
  line-height: var(--fi-leading-relaxed);
}

.match-id-actions {
  display: flex;
  gap: var(--fi-space-16);
}

.match-id-actions__button {
  flex: 1;
  margin: 0;
  height: var(--fi-space-84);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--fi-radius-round);
  font-size: var(--fi-font-26);
  font-weight: var(--fi-weight-bold);
  line-height: var(--fi-leading-none);
}

.match-id-actions__button::after {
  border: none;
}

.match-id-actions__button--ghost {
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
}

.match-id-actions__button--pay {
  background: var(--fi-color-text-primary);
  color: var(--fi-color-surface);
}

.match-id-actions__button[disabled] {
  opacity: 0.55;
}
</style>
