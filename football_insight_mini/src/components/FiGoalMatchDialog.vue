<template>
  <view class="goal-match-dialog-mask" @tap="emit('close')" @touchmove.stop.prevent>
    <view class="goal-match-dialog" role="dialog" :aria-label="detail.title" aria-modal="true" @tap.stop @touchmove.stop>
      <button class="goal-match-dialog__close" aria-label="关闭单场进球详情" hover-class="goal-match-dialog__close--pressed" @tap="emit('close')">
        <text aria-hidden="true">×</text>
      </button>
      <text class="goal-match-dialog__title">{{ detail.title }}</text>
      <scroll-view scroll-y class="goal-match-dialog__body" @touchmove.stop>
        <text class="goal-match-dialog__detail">{{ detail.content }}</text>
      </scroll-view>
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
  position: relative;
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
  padding: 0 var(--fi-space-40);
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

.goal-match-dialog__close {
  position: absolute;
  top: var(--fi-space-12);
  right: var(--fi-space-12);
  width: var(--fi-space-64);
  height: var(--fi-space-64);
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0;
  padding: 0;
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-primary);
  font-size: var(--fi-font-32);
  line-height: var(--fi-leading-none);
  font-weight: var(--fi-weight-bold);
}

.goal-match-dialog__close::after {
  border: 0;
}

.goal-match-dialog__close--pressed {
  background: var(--fi-color-divider-soft);
}
</style>
