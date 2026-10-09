<template>
<view class="schedule-dialog" @click.self="emit('close')">
  <view class="schedule-dialog__sheet">
    <view class="section-heading section-heading--compact">
      <view>
        <text class="section-kicker">Round Fixtures</text>
        <text class="section-title">第 {{ roundNumber }} 轮对阵</text>
      </view>
      <button class="schedule-dialog__close" @click="emit('close')">关闭</button>
    </view>
    <FiLoading v-if="loading" title="整轮对阵加载中" caption="这轮比赛马上整理好。" />
    <view v-else-if="errorMessage" class="state-card state-card--error"><text>{{ errorMessage }}</text></view>
    <view v-else class="match-stack">
      <view v-for="match in matches" :key="match.match_id">
        <MatchesMatchCard :match="match" :now-iso="nowIso" :show-status-accent="false" @open-tech-stats="emit('open-tech-stats', $event)" />
      </view>
    </view>
  </view>
</view>
</template>

<script setup lang="ts">
import type { MatchCard } from '../../../types/insight'
import FiLoading from '../../../components/FiLoading.vue'
import MatchesMatchCard from './MatchesMatchCard.vue'

defineProps<{ roundNumber: number; matches: MatchCard[]; loading: boolean; errorMessage: string; nowIso: string }>()
const emit = defineEmits<{
  (event: 'close'): void
  (event: 'open-tech-stats', match: MatchCard): void
}>()
</script>

<style scoped lang="css">
@import '../matches-shared.css';

.schedule-dialog__sheet {
  background: var(--fi-color-schedule-dialog-sheet-background);
  border-radius: var(--fi-radius-xl);
  padding: var(--fi-space-28);
  /* H5 底部栏占用窗口空间；小程序没有该变量时保留原始留白。 */
  padding-bottom: calc(var(--fi-space-28) + var(--window-bottom, 0rpx));
  border: var(--fi-component-matches-schedule-dialog-sheet-border);
  box-shadow: var(--fi-shadow-schedule-dialog-sheet-box-shadow);
}
.schedule-dialog {
  position: fixed;
  inset: 0;
  z-index: 20;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: var(--fi-space-28);
  background: var(--fi-color-schedule-dialog-background);
}
.schedule-dialog__sheet {
  width: 100%;
  max-height: 76vh;
  overflow-y: auto;
}
.schedule-dialog__close {
  display: inline-flex;
  margin: 0 0 0 auto;
  align-items: center;
  justify-content: center;
  white-space: nowrap;
  line-height: var(--fi-leading-none);
  padding: var(--fi-space-10) var(--fi-space-18);
  border-radius: var(--fi-radius-round);
  background: var(--fi-color-page);
  color: var(--fi-color-text-secondary);
  font-size: var(--fi-font-24);
}
</style>
