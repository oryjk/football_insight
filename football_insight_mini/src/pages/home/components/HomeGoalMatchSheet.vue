<template>
  <view class="goal-match-sheet" @touchmove.stop.prevent>
    <HomeSheetShell kicker="单场进球" @close="emit('close')">
      <template #title>
        <text class="goal-match-sheet__title">{{ detail.title }}</text>
      </template>

      <view class="goal-match-sheet__body" @touchmove.stop>
        <view class="goal-match-sheet__summary">
          <text class="goal-match-sheet__detail">{{ detail.content }}</text>
        </view>
        <button class="goal-match-sheet__confirm" @click="emit('close')">知道了</button>
      </view>
    </HomeSheetShell>
  </view>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import HomeSheetShell from './HomeSheetShell.vue'
import type { PlayerMatchGoals } from '../../../types/insight'
import { buildGoalMatchDetail } from '../../../utils/playerGoals'

const props = defineProps<{ playerName: string; match: PlayerMatchGoals }>()
const emit = defineEmits<{ (event: 'close'): void }>()
const detail = computed(() => buildGoalMatchDetail(props.playerName, props.match))
</script>

<style scoped>
.goal-match-sheet {
  position: fixed;
  inset: 0;
  z-index: 90;
}

.goal-match-sheet__title {
  display: block;
  color: var(--fi-color-text-strong);
  font-size: var(--fi-font-36);
  line-height: var(--fi-leading-snug);
  font-weight: var(--fi-weight-extrabold);
  overflow-wrap: anywhere;
}

.goal-match-sheet__body {
  padding-bottom: calc(var(--fi-space-64) + var(--fi-space-40) + env(safe-area-inset-bottom));
}

.goal-match-sheet__summary {
  margin-top: var(--fi-space-24);
  padding: var(--fi-space-24);
  border-radius: var(--fi-radius-lg);
  border: var(--fi-border-card);
  background: var(--fi-color-surface);
}

.goal-match-sheet__detail {
  display: block;
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-relaxed);
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}

.goal-match-sheet__confirm {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: var(--fi-space-24);
  padding: var(--fi-space-20) var(--fi-space-30);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-primary);
  color: var(--fi-color-surface);
  font-size: var(--fi-font-28);
  line-height: var(--fi-leading-none);
  font-weight: var(--fi-weight-bold);
}
</style>
