<template>
  <view class="goal-match-dialog-mask" @tap="emit('close')" @touchmove.stop.prevent>
    <view class="goal-match-dialog" role="dialog" :aria-label="detail.title" aria-modal="true" @tap.stop @touchmove.stop>
      <text class="goal-match-dialog__title">{{ detail.title }}</text>
      <scroll-view scroll-y class="goal-match-dialog__body" @touchmove.stop>
        <text class="goal-match-dialog__detail">{{ detail.content }}</text>
      </scroll-view>
      <view class="goal-match-dialog__footer">
        <button class="goal-match-dialog__confirm" hover-class="goal-match-dialog__confirm--pressed" @tap="emit('close')">确定</button>
      </view>
    </view>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { PlayerMatchGoals } from '../../../types/insight'
import { buildGoalMatchDetail } from '../../../utils/playerGoals'

const props = defineProps<{ playerName: string; match: PlayerMatchGoals }>()
const emit = defineEmits<{ (event: 'close'): void }>()
const detail = computed(() => buildGoalMatchDetail(props.playerName, props.match))
</script>

<style scoped>
.goal-match-dialog-mask {
  position: fixed;
  inset: 0;
  z-index: 90;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--fi-color-overlay);
}

.goal-match-dialog {
  box-sizing: border-box;
  width: calc(100% - var(--fi-space-64) * 2);
  max-width: calc(var(--fi-space-64) * 10);
  padding: var(--fi-space-40);
  border-radius: var(--fi-radius-lg);
  background: var(--fi-color-surface);
  text-align: center;
}

.goal-match-dialog__title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-snug);
  font-weight: var(--fi-weight-bold);
  overflow-wrap: anywhere;
  max-height: 20vh;
  overflow-y: auto;
}

.goal-match-dialog__body {
  margin-top: var(--fi-space-24);
  max-height: 35vh;
}

.goal-match-dialog__detail {
  display: block;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-relaxed);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.goal-match-dialog__footer {
  margin-top: var(--fi-space-40);
}

.goal-match-dialog__confirm {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0;
  padding: var(--fi-space-24) var(--fi-space-20);
  border-radius: var(--fi-radius-sm);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-none);
  font-weight: var(--fi-weight-bold);
}

.goal-match-dialog__confirm::after {
  border: 0;
}

.goal-match-dialog__confirm--pressed {
  background: var(--fi-color-primary-deep);
}
</style>
